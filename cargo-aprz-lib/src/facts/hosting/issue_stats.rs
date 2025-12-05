use super::AgeStats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStats {
    pub open_count: u64,
    pub closed_count: u64,
    pub open_age: AgeStats,
    pub closed_age: AgeStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_access() {
        let open_age = AgeStats {
            avg: 10,
            p50: 8,
            p75: 15,
            p90: 25,
            p95: 30,
        };

        let closed_age = AgeStats {
            avg: 5,
            p50: 4,
            p75: 7,
            p90: 12,
            p95: 15,
        };

        let stats = IssueStats {
            open_count: 42,
            closed_count: 100,
            open_age,
            closed_age,
        };

        assert_eq!(stats.open_count, 42);
        assert_eq!(stats.closed_count, 100);
        assert_eq!(stats.open_age.avg, 10);
        assert_eq!(stats.closed_age.avg, 5);
    }

    #[test]
    fn test_clone() {
        let stats1 = IssueStats {
            open_count: 42,
            closed_count: 100,
            open_age: AgeStats {
                avg: 10,
                p50: 8,
                p75: 15,
                p90: 25,
                p95: 30,
            },
            closed_age: AgeStats::default(),
        };

        let stats2 = stats1.clone();

        assert_eq!(stats1.open_count, stats2.open_count);
        assert_eq!(stats1.closed_count, stats2.closed_count);
        assert_eq!(stats1.open_age.avg, stats2.open_age.avg);
        assert_eq!(stats1.closed_age.avg, stats2.closed_age.avg);
    }

    #[test]
    fn test_serialize_deserialize() {
        let stats = IssueStats {
            open_count: 42,
            closed_count: 100,
            open_age: AgeStats {
                avg: 10,
                p50: 8,
                p75: 15,
                p90: 25,
                p95: 30,
            },
            closed_age: AgeStats {
                avg: 5,
                p50: 4,
                p75: 7,
                p90: 12,
                p95: 15,
            },
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: IssueStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.open_count, deserialized.open_count);
        assert_eq!(stats.closed_count, deserialized.closed_count);
        assert_eq!(stats.open_age.avg, deserialized.open_age.avg);
        assert_eq!(stats.closed_age.p95, deserialized.closed_age.p95);
    }

    #[test]
    fn test_json_format() {
        let stats = IssueStats {
            open_count: 42,
            closed_count: 100,
            open_age: AgeStats::default(),
            closed_age: AgeStats::default(),
        };

        let json = serde_json::to_string(&stats).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"open_count\":42"));
        assert!(json.contains("\"closed_count\":100"));
        assert!(json.contains("\"open_age\""));
        assert!(json.contains("\"closed_age\""));
    }

    #[test]
    fn test_with_default_age_stats() {
        let stats = IssueStats {
            open_count: 10,
            closed_count: 20,
            open_age: AgeStats::default(),
            closed_age: AgeStats::default(),
        };

        assert_eq!(stats.open_age.avg, 0);
        assert_eq!(stats.closed_age.p95, 0);
    }
}
