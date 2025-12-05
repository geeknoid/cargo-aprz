use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CodebaseData {
    pub timestamp: DateTime<Utc>,
    pub source_files_analyzed: u64,
    pub source_files_with_errors: u64,
    pub production_lines: u64,
    pub test_lines: u64,
    pub comment_lines: u64,
    pub unsafe_count: u64,
    pub example_count: u64,
    pub transitive_dependencies: u64,
    pub workflows_detected: bool,
    pub miri_detected: bool,
    pub clippy_detected: bool,
    #[serde(default)]
    pub contributors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_data() -> CodebaseData {
        CodebaseData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            source_files_analyzed: 100,
            source_files_with_errors: 2,
            production_lines: 5000,
            test_lines: 2000,
            comment_lines: 500,
            unsafe_count: 5,
            example_count: 3,
            transitive_dependencies: 50,
            workflows_detected: true,
            miri_detected: true,
            clippy_detected: true,
            contributors: 42,
        }
    }

    #[test]
    fn test_field_access() {
        let data = create_test_data();

        assert_eq!(data.source_files_analyzed, 100);
        assert_eq!(data.source_files_with_errors, 2);
        assert_eq!(data.production_lines, 5000);
        assert_eq!(data.test_lines, 2000);
        assert_eq!(data.comment_lines, 500);
        assert_eq!(data.unsafe_count, 5);
        assert_eq!(data.example_count, 3);
        assert_eq!(data.transitive_dependencies, 50);
        assert!(data.workflows_detected);
        assert!(data.miri_detected);
        assert!(data.clippy_detected);
        assert_eq!(data.contributors, 42);
    }

    #[test]
    fn test_clone() {
        let data1 = create_test_data();
        let data2 = data1.clone();

        assert_eq!(data1.source_files_analyzed, data2.source_files_analyzed);
        assert_eq!(data1.production_lines, data2.production_lines);
        assert_eq!(data1.workflows_detected, data2.workflows_detected);
        assert_eq!(data1.contributors, data2.contributors);
    }

    #[test]
    fn test_serialize_deserialize() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: CodebaseData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.source_files_analyzed, deserialized.source_files_analyzed);
        assert_eq!(data.production_lines, deserialized.production_lines);
        assert_eq!(data.unsafe_count, deserialized.unsafe_count);
        assert_eq!(data.workflows_detected, deserialized.workflows_detected);
        assert_eq!(data.contributors, deserialized.contributors);
    }

    #[test]
    fn test_json_format() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"source_files_analyzed\":100"));
        assert!(json.contains("\"production_lines\":5000"));
        assert!(json.contains("\"test_lines\":2000"));
        assert!(json.contains("\"unsafe_count\":5"));
        assert!(json.contains("\"workflows_detected\":true"));
        assert!(json.contains("\"contributors\":42"));
    }

    #[test]
    fn test_contributor_count_default() {
        // Test that contributor_count defaults to 0 if missing in JSON
        let json = r#"{
            "timestamp": "2024-01-01T00:00:00Z",
            "source_files_analyzed": 100,
            "source_files_with_errors": 2,
            "production_lines": 5000,
            "test_lines": 2000,
            "comment_lines": 500,
            "unsafe_count": 5,
            "example_count": 3,
            "transitive_dependencies": 50,
            "workflows_detected": true,
            "miri_detected": true,
            "clippy_detected": true
        }"#;

        let data: CodebaseData = serde_json::from_str(json).unwrap();

        // contributor_count should default to 0 when not present
        assert_eq!(data.contributors, 0);
    }

    #[test]
    fn test_with_zero_values() {
        use chrono::TimeZone;
        let data = CodebaseData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            source_files_analyzed: 0,
            source_files_with_errors: 0,
            production_lines: 0,
            test_lines: 0,
            comment_lines: 0,
            unsafe_count: 0,
            example_count: 0,
            transitive_dependencies: 0,
            workflows_detected: false,
            miri_detected: false,
            clippy_detected: false,
            contributors: 0,
        };

        assert_eq!(data.source_files_analyzed, 0);
        assert_eq!(data.unsafe_count, 0);
        assert!(!data.workflows_detected);
        assert_eq!(data.contributors, 0);
    }

    #[test]
    fn test_timestamp_roundtrip() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: CodebaseData = serde_json::from_str(&json).unwrap();

        // Timestamp should be preserved through serialization
        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());
    }
}
