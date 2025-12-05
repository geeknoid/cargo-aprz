use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocsData {
    pub timestamp: DateTime<Utc>,
    pub state: MetricState,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MetricState {
    Found(DocsMetrics),
    UnknownFormatVersion(u64),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocsMetrics {
    pub doc_coverage_percentage: u8,
    pub number_of_public_api_elements: u64,
    pub number_of_undocumented_elements: u64,
    pub number_of_examples_in_docs: u64,
    pub has_crate_level_docs: bool,
    pub broken_doc_links: u64,
}
