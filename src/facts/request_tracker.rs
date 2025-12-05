//! Request tracking for monitoring outstanding HTTP requests.

use crate::facts::progress_reporter::ProgressReporter;
use core::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Counter for a specific named request type.
#[derive(Debug, Default)]
struct RequestCounter {
    issued: AtomicU64,
    completed: AtomicU64,
}

/// Tracks outstanding requests and updates progress reporting.
///
/// This is used to monitor requests to external services like GitHub, docs.rs,
/// and codecov.io, providing visibility into the query phase of crate analysis.
///
/// Requests are tracked by name, allowing separate counters for different request types
/// (e.g., "GitHub", "docs.rs", "codecov.io").
#[derive(Debug, Clone)]
pub struct RequestTracker {
    counters: Arc<Mutex<HashMap<String, Arc<RequestCounter>>>>,
    progress: ProgressReporter,
}

impl RequestTracker {
    /// Create a new request tracker with the given progress reporter.
    #[must_use]
    pub fn new(progress: ProgressReporter) -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            progress,
        }
    }

    /// Get or create a counter for the given name.
    fn get_counter(&self, name: &str) -> Arc<RequestCounter> {
        let mut counters = self.counters.lock().expect("lock poisoned");
        Arc::clone(
            counters
                .entry(name.to_string())
                .or_insert_with(|| Arc::new(RequestCounter::default())),
        )
    }

    /// Mark that a new request has been issued for the given named category.
    pub fn add_request(&self, name: &str) {
        let counter = self.get_counter(name);
        let _ = counter.issued.fetch_add(1, Ordering::Relaxed);
        self.update_progress();
    }

    /// Mark that multiple new requests have been issued for the given named category.
    pub fn add_many_requests(&self, name: &str, count: u64) {
        if count == 0 {
            return;
        }
        let counter = self.get_counter(name);
        let _ = counter.issued.fetch_add(count, Ordering::Relaxed);
        self.update_progress();
    }

    /// Mark that a request has completed for the given named category.
    pub fn complete_request(&self, name: &str) {
        let counter = self.get_counter(name);
        let _ = counter.completed.fetch_add(1, Ordering::Relaxed);
        self.update_progress();
    }

    /// Update progress reporter with current request counts across all categories.
    fn update_progress(&self) {
        let counters = self.counters.lock().expect("lock poisoned");

        // Calculate totals
        let mut total_issued = 0u64;
        let mut total_completed = 0u64;
        let mut parts = Vec::new();

        // Collect stats for each named category, sorted by name for consistent ordering
        let mut names: Vec<_> = counters.keys().collect();
        names.sort();

        for name in names {
            if let Some(counter) = counters.get(name.as_str()) {
                let issued = counter.issued.load(Ordering::Relaxed);
                let completed = counter.completed.load(Ordering::Relaxed);

                if issued > 0 {
                    total_issued += issued;
                    total_completed += completed;
                    parts.push(format!("{completed}/{issued} {name}"));
                }
            }
        }

        // Update progress bar
        if total_issued > 0 {
            self.progress.enable_determinate_mode(total_issued);
            self.progress.set_position(total_completed);

            // Format message as "X/Y name1, X/Y name2, ..."
            let message = if parts.is_empty() {
                "No requests".to_string()
            } else {
                parts.join(", ")
            };
            self.progress.set_message(message);
        }
    }
}
