//! Progress reporting for long-running operations.

use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::sync::Arc;
use std::time::Instant;

/// Shared state for delayed progress reporting.
#[derive(Debug)]
struct DelayedProgressState {
    start_time: Instant,
    delay: Duration,
    visible: AtomicBool,
    has_content: AtomicBool,
    is_indeterminate: AtomicBool,
}

/// A progress reporter that delays showing the progress bar until a threshold is reached.
///
/// This prevents brief flashes of progress bars for operations that complete quickly.
/// The progress bar is only shown if an operation takes longer than the delay threshold
/// AND has meaningful content (length or message set).
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    bar: ProgressBar,
    state: Arc<DelayedProgressState>,
}

impl ProgressReporter {
    /// Create a new progress reporter.
    ///
    /// The progress bar will only become visible if operations continue beyond the delay threshold.
    #[must_use]
    pub fn new(delay: Duration) -> Self {
        // Create the progress bar with the standard style
        let bar = ProgressBar::hidden();
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:>12.bold.cyan} [{bar:25}] {msg}")
                .expect("Failed to create progress style")
                .progress_chars("=> "),
        );
        bar.set_length(0);
        bar.set_draw_target(ProgressDrawTarget::hidden());

        Self {
            bar,
            state: Arc::new(DelayedProgressState {
                start_time: Instant::now(),
                delay,
                visible: AtomicBool::new(false),
                has_content: AtomicBool::new(false),
                is_indeterminate: AtomicBool::new(false),
            }),
        }
    }

    /// Check if enough time has elapsed and we have content, then make the progress bar visible if needed.
    fn ensure_visible(&self) {
        if !self.state.visible.load(Ordering::Relaxed)
            && self.state.has_content.load(Ordering::Relaxed)
            && self.state.start_time.elapsed() >= self.state.delay
        {
            self.state.visible.store(true, Ordering::Relaxed);
            // Make the progress bar visible by setting draw target to stderr
            // Use stderr_with_hz to ensure proper cursor management
            self.bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
        }
    }

    /// Force the progress bar to become visible immediately, bypassing the delay.
    ///
    /// This is useful when you know an operation will take significant time and want
    /// to show progress immediately rather than waiting for the delay threshold.
    pub fn force_visible(&self) {
        if !self.state.visible.load(Ordering::Relaxed) && self.state.has_content.load(Ordering::Relaxed) {
            self.state.visible.store(true, Ordering::Relaxed);
            self.bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
        }
    }

    /// Restore determinate (progress bar) mode after being in indeterminate (spinner) mode.
    ///
    /// This switches back to the standard progress bar style and disables the spinner animation.
    fn restore_determinate(&self) {
        if self.state.is_indeterminate.load(Ordering::Relaxed) {
            self.state.is_indeterminate.store(false, Ordering::Relaxed);

            // Disable steady tick animation
            self.bar.disable_steady_tick();

            // Restore the determinate progress bar style
            self.bar.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:>12.bold.cyan} [{bar:25}] {msg}")
                    .expect("Failed to create progress style")
                    .progress_chars("=> "),
            );
        }
    }

    /// Set the total length/size for the progress indicator.
    ///
    /// If the progress bar is currently in indeterminate mode, this will automatically
    /// switch it back to determinate (bar) mode.
    pub fn enable_determinate_mode(&self, len: u64) {
        // Restore determinate mode if we were in indeterminate mode
        self.restore_determinate();

        // Mark that we have content
        if len > 0 {
            self.state.has_content.store(true, Ordering::Relaxed);
        }
        self.ensure_visible();
        // Always forward to inner bar so it has correct state when it becomes visible
        self.bar.set_length(len);
    }

    /// Set the current position in the progress.
    ///
    /// If the progress bar is currently in indeterminate mode, this will automatically
    /// switch it back to determinate (bar) mode.
    pub fn set_position(&self, pos: u64) {
        // Restore determinate mode if we were in indeterminate mode
        self.restore_determinate();

        self.ensure_visible();
        // Always forward to inner bar so it has correct state when it becomes visible
        self.bar.set_position(pos);
    }

    /// Set a message to display with the progress.
    pub fn set_message(&self, msg: impl AsRef<str>) {
        let msg = msg.as_ref();

        // Mark that we have content
        if !msg.is_empty() {
            self.state.has_content.store(true, Ordering::Relaxed);
        }
        self.ensure_visible();
        self.bar.set_message(msg.to_string());
    }

    /// Set the prefix label for the progress bar (e.g., "Preparing", "Collecting").
    pub fn set_prefix(&self, prefix: &str) {
        // Prefix changes don't affect visibility, just forward to inner bar
        self.bar.set_prefix(prefix.to_string());
    }

    /// Periodically check visibility - this is called by a background task to ensure
    /// the bar becomes visible once the delay has elapsed, even during long awaits.
    pub fn tick_visibility(&self) {
        // Only check if we're not already visible
        if !self.state.visible.load(Ordering::Relaxed) {
            self.ensure_visible();
        }
    }

    /// Enable indeterminate/pulse mode for operations with unknown duration or size.
    ///
    /// This switches the progress indicator to show ongoing activity without a specific
    /// completion target. Useful for git operations, streaming data of unknown size, etc.
    ///
    /// To switch back to determinate mode, simply call `set_length()` or `set_position()`.
    pub fn enable_indeterminate_mode(&self) {
        // Mark that we're in indeterminate mode
        self.state.is_indeterminate.store(true, Ordering::Relaxed);

        // Mark that we have content (indeterminate mode indicates ongoing work)
        self.state.has_content.store(true, Ordering::Relaxed);
        self.ensure_visible();

        // Switch to spinner mode for indeterminate progress
        // Using square brackets to match the progress bar visual style
        // Pad spinner to 25 chars to align with progress bar width
        let template = "  {prefix:>8.bold.cyan} [{spinner}] {msg}";
        self.bar.set_style(
            ProgressStyle::default_spinner()
                .template(template)
                .expect("Failed to create spinner style")
                .tick_strings(&[
                    "=                        ", // 12 spaces, char, 12 spaces = 25 total
                    "==                       ",
                    "===                      ",
                    " ===                     ",
                    "  ===                    ",
                    "   ===                   ",
                    "    ===                  ",
                    "     ===                 ",
                    "      ===                ",
                    "       ===               ",
                    "        ===              ",
                    "         ===             ",
                    "          ===            ",
                    "           ===           ",
                    "            ===          ",
                    "             ===         ",
                    "              ===        ",
                    "               ===       ",
                    "                ===      ",
                    "                 ===     ",
                    "                  ===    ",
                    "                   ===   ",
                    "                    ===  ",
                    "                     === ",
                    "                      ===",
                    "                       ==",
                    "                        =",
                    "                       ==",
                    "                      ===",
                    "                     === ",
                    "                    ===  ",
                    "                   ===   ",
                    "                  ===    ",
                    "                 ===     ",
                    "                ===      ",
                    "               ===       ",
                    "              ===        ",
                    "             ===         ",
                    "            ===          ",
                    "           ===           ",
                    "          ===            ",
                    "         ===             ",
                    "        ===              ",
                    "       ===               ",
                    "      ===                ",
                    "     ===                 ",
                    "    ===                  ",
                    "   ===                   ",
                    "  ===                    ",
                    " ===                     ",
                    "===                      ",
                    "==                       ",
                ]),
        );
        // Enable auto-ticking animation at 100ms intervals
        self.bar.enable_steady_tick(Duration::from_millis(100));
    }

    /// Finish and clear the progress indicator.
    pub fn finish_and_clear(&self) {
        // Only finish and clear if the bar was actually made visible
        if self.state.visible.load(Ordering::Relaxed) {
            self.bar.finish_and_clear();
        }
    }

    /// Start a background task that periodically checks if the progress bar should become visible.
    ///
    /// Returns a guard that will abort the task when dropped.
    #[must_use]
    pub fn start_visibility_checking(&self) -> VisibilityTaskGuard {
        let progress_clone = self.clone();
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(250));
            loop {
                let _ = interval.tick().await;
                progress_clone.tick_visibility();
            }
        });
        VisibilityTaskGuard(task)
    }
}

/// Guard that aborts the visibility checking task when dropped.
#[derive(Debug)]
pub struct VisibilityTaskGuard(tokio::task::JoinHandle<()>);

impl Drop for VisibilityTaskGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}
