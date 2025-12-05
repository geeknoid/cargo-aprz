use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoverageData {
    pub timestamp: DateTime<Utc>,
    pub code_coverage_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone};

    fn create_test_data() -> CoverageData {
        CoverageData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            code_coverage_percentage: 85.5,
        }
    }

    #[test]
    fn test_field_access() {
        let data = create_test_data();

        assert!((data.code_coverage_percentage - 85.5).abs() < f64::EPSILON);
        assert_eq!(data.timestamp.year(), 2024);
    }

    #[test]
    fn test_clone() {
        let data1 = create_test_data();
        let data2 = data1.clone();

        assert!((data1.code_coverage_percentage - data2.code_coverage_percentage).abs() < f64::EPSILON);
        assert_eq!(data1.timestamp, data2.timestamp);
    }

    #[test]
    fn test_serialize_deserialize() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: CoverageData = serde_json::from_str(&json).unwrap();

        assert!((data.code_coverage_percentage - deserialized.code_coverage_percentage).abs() < f64::EPSILON);
        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());
    }

    #[test]
    fn test_json_format() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"code_coverage_percentage\":85.5"));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_with_zero_coverage() {
        use chrono::TimeZone;
        let data = CoverageData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            code_coverage_percentage: 0.0,
        };

        assert!(data.code_coverage_percentage.abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_full_coverage() {
        use chrono::TimeZone;
        let data = CoverageData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            code_coverage_percentage: 100.0,
        };

        assert!((data.code_coverage_percentage - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_fractional_coverage() {
        use chrono::TimeZone;
        let data = CoverageData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            code_coverage_percentage: 75.234,
        };

        assert!((data.code_coverage_percentage - 75.234).abs() < 0.001);
    }

    #[test]
    fn test_timestamp_roundtrip() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: CoverageData = serde_json::from_str(&json).unwrap();

        // Timestamp should be preserved through serialization
        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());
    }
}
