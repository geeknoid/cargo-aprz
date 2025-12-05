use super::issue_stats::IssueStats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostingData {
    pub timestamp: DateTime<Utc>,
    pub stars: u64,
    pub forks: u64,
    pub subscribers: u64,
    pub commits_last_90_days: u64,
    pub issues: IssueStats,
    pub pulls: IssueStats,
}

#[cfg(test)]
mod tests {
    use super::super::AgeStats;
    use super::*;
    use chrono::TimeZone;

    fn create_test_issue_stats() -> IssueStats {
        IssueStats {
            open_count: 10,
            closed_count: 20,
            open_age: AgeStats {
                avg: 5,
                p50: 4,
                p75: 7,
                p90: 12,
                p95: 15,
            },
            closed_age: AgeStats::default(),
        }
    }

    #[test]
    fn test_field_access() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let data = HostingData {
            timestamp,
            stars: 1000,
            forks: 200,
            subscribers: 50,
            commits_last_90_days: 150,
            issues: create_test_issue_stats(),
            pulls: create_test_issue_stats(),
        };

        assert_eq!(data.stars, 1000);
        assert_eq!(data.forks, 200);
        assert_eq!(data.subscribers, 50);
        assert_eq!(data.commits_last_90_days, 150);
        assert_eq!(data.issues.open_count, 10);
        assert_eq!(data.pulls.closed_count, 20);
    }

    #[test]
    fn test_clone() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let data1 = HostingData {
            timestamp,
            stars: 1000,
            forks: 200,
            subscribers: 50,
            commits_last_90_days: 150,
            issues: create_test_issue_stats(),
            pulls: create_test_issue_stats(),
        };

        let data2 = data1.clone();

        assert_eq!(data1.stars, data2.stars);
        assert_eq!(data1.forks, data2.forks);
        assert_eq!(data1.timestamp, data2.timestamp);
        assert_eq!(data1.issues.open_count, data2.issues.open_count);
    }

    #[test]
    fn test_serialize_deserialize() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let data = HostingData {
            timestamp,
            stars: 1000,
            forks: 200,
            subscribers: 50,
            commits_last_90_days: 150,
            issues: create_test_issue_stats(),
            pulls: create_test_issue_stats(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: HostingData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.stars, deserialized.stars);
        assert_eq!(data.forks, deserialized.forks);
        assert_eq!(data.subscribers, deserialized.subscribers);
        assert_eq!(data.commits_last_90_days, deserialized.commits_last_90_days);
        assert_eq!(data.timestamp, deserialized.timestamp);
        assert_eq!(data.issues.open_count, deserialized.issues.open_count);
        assert_eq!(data.pulls.closed_count, deserialized.pulls.closed_count);
    }

    #[test]
    fn test_json_format() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let data = HostingData {
            timestamp,
            stars: 1000,
            forks: 200,
            subscribers: 50,
            commits_last_90_days: 150,
            issues: create_test_issue_stats(),
            pulls: create_test_issue_stats(),
        };

        let json = serde_json::to_string(&data).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"stars\":1000"));
        assert!(json.contains("\"forks\":200"));
        assert!(json.contains("\"subscribers\":50"));
        assert!(json.contains("\"commits_last_90_days\":150"));
        assert!(json.contains("\"issues\""));
        assert!(json.contains("\"pulls\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_with_zero_values() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let data = HostingData {
            timestamp,
            stars: 0,
            forks: 0,
            subscribers: 0,
            commits_last_90_days: 0,
            issues: IssueStats {
                open_count: 0,
                closed_count: 0,
                open_age: AgeStats::default(),
                closed_age: AgeStats::default(),
            },
            pulls: IssueStats {
                open_count: 0,
                closed_count: 0,
                open_age: AgeStats::default(),
                closed_age: AgeStats::default(),
            },
        };

        assert_eq!(data.stars, 0);
        assert_eq!(data.issues.open_count, 0);
        assert_eq!(data.pulls.closed_count, 0);
    }

    #[test]
    fn test_timestamp_roundtrip() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let data = HostingData {
            timestamp,
            stars: 100,
            forks: 10,
            subscribers: 5,
            commits_last_90_days: 25,
            issues: create_test_issue_stats(),
            pulls: create_test_issue_stats(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: HostingData = serde_json::from_str(&json).unwrap();

        // Timestamp should be preserved through serialization
        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());
    }
}
