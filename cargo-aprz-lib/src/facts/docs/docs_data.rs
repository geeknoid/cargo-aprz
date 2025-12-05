use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocsData {
    pub timestamp: DateTime<Utc>,
    pub metrics: DocMetricState,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DocMetricState {
    Found(DocsMetrics),
    UnknownFormatVersion(u64),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocsMetrics {
    pub doc_coverage_percentage: f64,
    pub public_api_elements: u64,
    pub undocumented_elements: u64,
    pub examples_in_docs: u64,
    pub has_crate_level_docs: bool,
    pub broken_doc_links: u64,
}

#[cfg(test)]
#[expect(clippy::float_cmp, reason = "exact float comparison acceptable in tests for known values")]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_metrics() -> DocsMetrics {
        DocsMetrics {
            doc_coverage_percentage: 85.0,
            public_api_elements: 100,
            undocumented_elements: 15,
            examples_in_docs: 25,
            has_crate_level_docs: true,
            broken_doc_links: 2,
        }
    }

    fn create_test_docs_data() -> DocsData {
        DocsData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            metrics: DocMetricState::Found(create_test_metrics()),
        }
    }

    #[test]
    fn test_docs_metrics_field_access() {
        let metrics = create_test_metrics();

        assert_eq!(metrics.doc_coverage_percentage, 85.0);
        assert_eq!(metrics.public_api_elements, 100);
        assert_eq!(metrics.undocumented_elements, 15);
        assert_eq!(metrics.examples_in_docs, 25);
        assert!(metrics.has_crate_level_docs);
        assert_eq!(metrics.broken_doc_links, 2);
    }

    #[test]
    fn test_docs_metrics_clone() {
        let metrics1 = create_test_metrics();
        let metrics2 = metrics1.clone();

        assert_eq!(metrics1.doc_coverage_percentage, metrics2.doc_coverage_percentage);
        assert_eq!(metrics1.public_api_elements, metrics2.public_api_elements);
        assert_eq!(metrics1.has_crate_level_docs, metrics2.has_crate_level_docs);
    }

    #[test]
    fn test_docs_metrics_serialize_deserialize() {
        let metrics = create_test_metrics();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: DocsMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.doc_coverage_percentage, deserialized.doc_coverage_percentage);
        assert_eq!(metrics.public_api_elements, deserialized.public_api_elements);
        assert_eq!(metrics.undocumented_elements, deserialized.undocumented_elements);
        assert_eq!(metrics.examples_in_docs, deserialized.examples_in_docs);
        assert_eq!(metrics.has_crate_level_docs, deserialized.has_crate_level_docs);
        assert_eq!(metrics.broken_doc_links, deserialized.broken_doc_links);
    }

    #[test]
    fn test_doc_metric_state_found() {
        let state = DocMetricState::Found(create_test_metrics());

        match state {
            DocMetricState::Found(metrics) => {
                assert_eq!(metrics.doc_coverage_percentage, 85.0);
            }
            DocMetricState::UnknownFormatVersion(_) => panic!("Expected Found variant"),
        }
    }

    #[test]
    fn test_doc_metric_state_unknown_format() {
        let state = DocMetricState::UnknownFormatVersion(42);

        match state {
            DocMetricState::Found(_) => panic!("Expected UnknownFormatVersion variant"),
            DocMetricState::UnknownFormatVersion(version) => {
                assert_eq!(version, 42);
            }
        }
    }

    #[test]
    fn test_doc_metric_state_serialize_deserialize_found() {
        let state = DocMetricState::Found(create_test_metrics());

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: DocMetricState = serde_json::from_str(&json).unwrap();

        match (state, deserialized) {
            (DocMetricState::Found(m1), DocMetricState::Found(m2)) => {
                assert_eq!(m1.doc_coverage_percentage, m2.doc_coverage_percentage);
            }
            _ => panic!("Deserialization failed to preserve variant"),
        }
    }

    #[test]
    fn test_doc_metric_state_serialize_deserialize_unknown() {
        let state = DocMetricState::UnknownFormatVersion(99);

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: DocMetricState = serde_json::from_str(&json).unwrap();

        match (state, deserialized) {
            (DocMetricState::UnknownFormatVersion(v1), DocMetricState::UnknownFormatVersion(v2)) => {
                assert_eq!(v1, v2);
            }
            _ => panic!("Deserialization failed to preserve variant"),
        }
    }

    #[test]
    fn test_docs_data_with_found_metrics() {
        let data = create_test_docs_data();

        match &data.metrics {
            DocMetricState::Found(metrics) => {
                assert_eq!(metrics.doc_coverage_percentage, 85.0);
            }
            DocMetricState::UnknownFormatVersion(_) => panic!("Expected Found variant"),
        }
    }

    #[test]
    fn test_docs_data_with_unknown_format() {
        use chrono::TimeZone;
        let data = DocsData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            metrics: DocMetricState::UnknownFormatVersion(100),
        };

        match data.metrics {
            DocMetricState::Found(_) => panic!("Expected UnknownFormatVersion variant"),
            DocMetricState::UnknownFormatVersion(version) => {
                assert_eq!(version, 100);
            }
        }
    }

    #[test]
    fn test_docs_data_serialize_deserialize() {
        let data = create_test_docs_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: DocsData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());

        match (&data.metrics, &deserialized.metrics) {
            (DocMetricState::Found(m1), DocMetricState::Found(m2)) => {
                assert_eq!(m1.doc_coverage_percentage, m2.doc_coverage_percentage);
            }
            _ => panic!("Deserialization failed"),
        }
    }

    #[test]
    fn test_docs_data_clone() {
        let data1 = create_test_docs_data();
        let data2 = data1.clone();

        assert_eq!(data1.timestamp, data2.timestamp);

        match (&data1.metrics, &data2.metrics) {
            (DocMetricState::Found(m1), DocMetricState::Found(m2)) => {
                assert_eq!(m1.doc_coverage_percentage, m2.doc_coverage_percentage);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_docs_metrics_with_100_coverage() {
        let metrics = DocsMetrics {
            doc_coverage_percentage: 100.0,
            public_api_elements: 50,
            undocumented_elements: 0,
            examples_in_docs: 30,
            has_crate_level_docs: true,
            broken_doc_links: 0,
        };

        assert_eq!(metrics.doc_coverage_percentage, 100.0);
        assert_eq!(metrics.undocumented_elements, 0);
        assert_eq!(metrics.broken_doc_links, 0);
    }

    #[test]
    fn test_docs_metrics_with_zero_coverage() {
        let metrics = DocsMetrics {
            doc_coverage_percentage: 0.0,
            public_api_elements: 100,
            undocumented_elements: 100,
            examples_in_docs: 0,
            has_crate_level_docs: false,
            broken_doc_links: 5,
        };

        assert_eq!(metrics.doc_coverage_percentage, 0.0);
        assert_eq!(metrics.undocumented_elements, 100);
        assert!(!metrics.has_crate_level_docs);
    }
}
