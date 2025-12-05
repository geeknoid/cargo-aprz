use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgeStats {
    pub avg: u32,
    pub p50: u32,
    pub p75: u32,
    pub p90: u32,
    pub p95: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let stats = AgeStats::default();
        assert_eq!(stats.avg, 0);
        assert_eq!(stats.p50, 0);
        assert_eq!(stats.p75, 0);
        assert_eq!(stats.p90, 0);
        assert_eq!(stats.p95, 0);
    }

    #[test]
    fn test_field_access() {
        let stats = AgeStats {
            avg: 10,
            p50: 8,
            p75: 15,
            p90: 25,
            p95: 30,
        };

        assert_eq!(stats.avg, 10);
        assert_eq!(stats.p50, 8);
        assert_eq!(stats.p75, 15);
        assert_eq!(stats.p90, 25);
        assert_eq!(stats.p95, 30);
    }

    #[test]
    fn test_clone() {
        let stats1 = AgeStats {
            avg: 10,
            p50: 8,
            p75: 15,
            p90: 25,
            p95: 30,
        };

        let stats2 = stats1.clone();

        assert_eq!(stats1.avg, stats2.avg);
        assert_eq!(stats1.p50, stats2.p50);
        assert_eq!(stats1.p75, stats2.p75);
        assert_eq!(stats1.p90, stats2.p90);
        assert_eq!(stats1.p95, stats2.p95);
    }

    #[test]
    fn test_serialize_deserialize() {
        let stats = AgeStats {
            avg: 10,
            p50: 8,
            p75: 15,
            p90: 25,
            p95: 30,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: AgeStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.avg, deserialized.avg);
        assert_eq!(stats.p50, deserialized.p50);
        assert_eq!(stats.p75, deserialized.p75);
        assert_eq!(stats.p90, deserialized.p90);
        assert_eq!(stats.p95, deserialized.p95);
    }

    #[test]
    fn test_json_format() {
        let stats = AgeStats {
            avg: 10,
            p50: 8,
            p75: 15,
            p90: 25,
            p95: 30,
        };

        let json = serde_json::to_string(&stats).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"avg\":10"));
        assert!(json.contains("\"p50\":8"));
        assert!(json.contains("\"p75\":15"));
        assert!(json.contains("\"p90\":25"));
        assert!(json.contains("\"p95\":30"));
    }
}
