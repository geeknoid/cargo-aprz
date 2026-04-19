use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Notify, Semaphore};

/// Limits concurrency and supports temporary pausing of work dispatch.
///
/// Wrap in an `Arc` via [`Throttler::new`], then call [`Throttler::acquire`] before
/// each unit of work. At most `max_concurrent` tasks will run simultaneously.
/// Any task can call [`Throttler::pause_for`] to temporarily halt new work dispatch
/// (e.g. in response to downstream backpressure).
///
/// When multiple tasks call [`Throttler::pause_for`] concurrently, the longest
/// pause wins — shorter pauses are ignored if a longer one is already active.
#[derive(Debug)]
pub struct Throttler {
    semaphore: Arc<Semaphore>,
    paused: AtomicBool,
    resume: Notify,
    /// Tracks when the current pause should expire. Used to ensure the longest
    /// pause wins when multiple `pause_for` calls overlap.
    resume_at: std::sync::Mutex<Option<Instant>>,
}

impl Throttler {
    /// Create a new throttler that allows at most `max_concurrent` tasks at a time.
    pub fn new(max_concurrent: usize) -> Arc<Self> {
        Arc::new(Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            paused: AtomicBool::new(false),
            resume: Notify::new(),
            resume_at: std::sync::Mutex::new(None),
        })
    }

    /// Wait until unpaused, then acquire a concurrency slot.
    ///
    /// The returned permit must be held for the duration of the work. When it
    /// is dropped, the slot becomes available for another task.
    pub async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
        loop {
            // Register the notification future *before* checking the condition
            // to avoid lost wakeups (the standard Notify pattern).
            let notified = self.resume.notified();

            if self.paused.load(Ordering::Acquire) {
                notified.await;
                continue;
            }

            let permit = Arc::clone(&self.semaphore)
                .acquire_owned()
                .await
                .expect("semaphore is never closed");

            // Double-check: if a pause started while we were waiting for the
            // semaphore, release the permit and wait for the pause to lift.
            if self.paused.load(Ordering::Acquire) {
                drop(permit);
                continue;
            }

            return permit;
        }
    }

    /// Returns whether the throttler is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }

    /// Minimum extension required for a new pause to override an active one.
    /// Prevents near-simultaneous callers (e.g. concurrent tasks that all
    /// discovered the same rate-limit reset time) from each "winning" the pause
    /// due to tiny `Instant::now()` drift between calls.
    const MIN_PAUSE_EXTENSION: Duration = Duration::from_secs(1);

    /// Pause dispatching for `duration`, then automatically resume.
    ///
    /// Tasks already running are not interrupted. Tasks waiting in [`acquire`](Self::acquire)
    /// will remain parked until the duration elapses. If a pause with a similar
    /// or longer duration is already active, this call is a no-op and returns `false`.
    /// Returns `true` only when a new pause is actually established.
    #[expect(clippy::significant_drop_tightening, reason = "paused must be set inside the lock for correctness")]
    pub fn pause_for(self: &Arc<Self>, duration: Duration) -> bool {
        let new_resume_at = Instant::now() + duration;

        {
            let mut guard = self.resume_at.lock().expect("lock not poisoned");
            if guard.is_some_and(|existing| existing + Self::MIN_PAUSE_EXTENSION >= new_resume_at) {
                return false; // an equivalent or longer pause is already active
            }
            *guard = Some(new_resume_at);
            // Set paused inside the lock so that resume_at and paused are always
            // consistent when observed together.
            self.paused.store(true, Ordering::Release);
        }
        let this = Arc::clone(self);
        drop(tokio::spawn(async move {
            tokio::time::sleep(duration).await;

            let should_resume = Self::try_resume(&this);
            if should_resume {
                this.resume.notify_waiters();
            }
        }));

        true
    }

    /// Attempt to resume after a pause expires. Returns `true` if the pause was
    /// lifted, `false` if a longer pause was scheduled after us.
    #[expect(clippy::significant_drop_tightening, reason = "paused must be cleared inside the lock for correctness")]
    fn try_resume(this: &Arc<Self>) -> bool {
        let mut guard = this.resume_at.lock().expect("lock not poisoned");
        if guard.is_some_and(|t| Instant::now() >= t) {
            *guard = None;
            // Clear paused inside the lock so that resume_at and paused
            // are always consistent when observed together.
            this.paused.store(false, Ordering::Release);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::AtomicUsize;

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri cannot call CreateIoCompletionPort on Windows")]
    async fn limits_concurrency() {
        let throttler = Throttler::new(2);
        let active = Arc::new(AtomicUsize::new(0));
        let max_seen = Arc::new(AtomicUsize::new(0));

        let tasks: Vec<_> = (0..10)
            .map(|_| {
                let throttler = Arc::clone(&throttler);
                let active = Arc::clone(&active);
                let max_seen = Arc::clone(&max_seen);
                tokio::spawn(async move {
                    let _permit = throttler.acquire().await;
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    _ = max_seen.fetch_max(current, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    _ = active.fetch_sub(1, Ordering::SeqCst);
                })
            })
            .collect();

        _ = futures_util::future::join_all(tasks).await;

        assert!(max_seen.load(Ordering::SeqCst) <= 2);
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri cannot call CreateIoCompletionPort on Windows")]
    async fn pause_blocks_new_work() {
        let throttler = Throttler::new(5);

        // Pause for 200ms
        let _ = throttler.pause_for(Duration::from_millis(200));

        let start = tokio::time::Instant::now();
        let _permit = throttler.acquire().await;
        let elapsed = start.elapsed();

        // Should have waited at least ~200ms
        assert!(elapsed >= Duration::from_millis(150));
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri cannot call CreateIoCompletionPort on Windows")]
    async fn acquire_unblocks_after_short_pause() {
        // Verifies that acquire() does not deadlock when a pause lifts
        // (the register-before-check Notify pattern prevents lost wakeups).
        let throttler = Throttler::new(1);

        let _ = throttler.pause_for(Duration::from_millis(50));

        let start = tokio::time::Instant::now();
        let _permit = throttler.acquire().await;
        let elapsed = start.elapsed();

        // Must have waited for the pause, but not much longer
        assert!(elapsed >= Duration::from_millis(30));
        assert!(elapsed < Duration::from_secs(2), "acquire() should not deadlock");
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri cannot call CreateIoCompletionPort on Windows")]
    async fn no_work_dispatched_during_pause() {
        // Verifies that acquire() does not return a permit until the pause has
        // mostly elapsed.
        let throttler = Throttler::new(5);

        let _ = throttler.pause_for(Duration::from_millis(200));

        let t = Arc::clone(&throttler);
        let task = tokio::spawn(async move {
            let start = tokio::time::Instant::now();
            let _permit = t.acquire().await;
            start.elapsed()
        });

        let elapsed = task.await.unwrap();
        assert!(elapsed >= Duration::from_millis(150), "work should not run while paused");
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri cannot call CreateIoCompletionPort on Windows")]
    async fn longer_pause_wins() {
        let throttler = Throttler::new(5);

        // First pause: 100ms
        assert!(throttler.pause_for(Duration::from_millis(100)));
        // Second pause: 2000ms — must exceed first + MIN_PAUSE_EXTENSION (1s)
        assert!(throttler.pause_for(Duration::from_millis(2000)));

        let start = tokio::time::Instant::now();
        let _permit = throttler.acquire().await;
        let elapsed = start.elapsed();

        // Should wait for the longer pause
        assert!(elapsed >= Duration::from_millis(1500));
    }
}
