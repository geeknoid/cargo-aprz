//! `RustSec`vulnerability advisory database fact provider.

mod advisory_data;
mod provider;

#[cfg(any(test, debug_assertions))]
pub use advisory_data::AdvisoryCounts;
pub use advisory_data::AdvisoryData;
pub use provider::Provider;
