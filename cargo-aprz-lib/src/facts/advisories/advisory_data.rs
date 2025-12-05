use chrono::{DateTime, Utc};
use rustsec::advisory::{Informational, Severity};
use serde::{Deserialize, Serialize};

/// Advisory counts for vulnerabilities and warnings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[expect(clippy::struct_field_names, reason = "all fields represent counts, suffix improves clarity")]
pub struct AdvisoryCounts {
    pub low_vulnerability_count: u64,
    pub medium_vulnerability_count: u64,
    pub high_vulnerability_count: u64,
    pub critical_vulnerability_count: u64,

    pub notice_warning_count: u64,
    pub unmaintained_warning_count: u64,
    pub unsound_warning_count: u64,
    pub yanked_warning_count: u64,
}

/// Security advisory data for a crate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvisoryData {
    pub timestamp: DateTime<Utc>,

    /// Advisory counts for the specific version being queried.
    pub per_version: AdvisoryCounts,

    /// Advisory counts across all versions of the crate (historical).
    pub total: AdvisoryCounts,
}

impl AdvisoryCounts {
    /// Apply an advisory to the counts.
    fn count_advisory(&mut self, advisory: &rustsec::Advisory) {
        if let Some(informational) = &advisory.metadata.informational {
            match informational {
                Informational::Notice => self.notice_warning_count += 1,
                Informational::Unmaintained => self.unmaintained_warning_count += 1,
                Informational::Unsound => self.unsound_warning_count += 1,
                // Note: yanked_warning_count is not used as rustsec doesn't provide yanked as an Informational type
                _ => {}
            }
            return;
        }

        if let Some(cvss) = &advisory.metadata.cvss {
            match cvss.severity() {
                Severity::None => {}
                Severity::Low => self.low_vulnerability_count += 1,
                Severity::Medium => self.medium_vulnerability_count += 1,
                Severity::High => self.high_vulnerability_count += 1,
                Severity::Critical => self.critical_vulnerability_count += 1,
            }
        }
    }
}

impl AdvisoryData {
    /// Count an advisory affecting the specific version being queried.
    pub(super) fn count_advisory_for_version(&mut self, advisory: &rustsec::Advisory) {
        self.per_version.count_advisory(advisory);
    }

    /// Count an advisory across all versions (historical).
    pub(super) fn count_advisory_historical(&mut self, advisory: &rustsec::Advisory) {
        self.total.count_advisory(advisory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_data() -> AdvisoryData {
        AdvisoryData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            per_version: AdvisoryCounts {
                low_vulnerability_count: 1,
                medium_vulnerability_count: 2,
                high_vulnerability_count: 1,
                critical_vulnerability_count: 1,
                notice_warning_count: 1,
                unmaintained_warning_count: 1,
                unsound_warning_count: 1,
                yanked_warning_count: 0,
            },
            total: AdvisoryCounts {
                low_vulnerability_count: 3,
                medium_vulnerability_count: 4,
                high_vulnerability_count: 2,
                critical_vulnerability_count: 1,
                notice_warning_count: 2,
                unmaintained_warning_count: 2,
                unsound_warning_count: 1,
                yanked_warning_count: 0,
            },
        }
    }

    #[test]
    fn test_default() {
        let data = AdvisoryData::default();

        assert_eq!(data.per_version.low_vulnerability_count, 0);
        assert_eq!(data.per_version.notice_warning_count, 0);
        assert_eq!(data.total.low_vulnerability_count, 0);
        assert_eq!(data.total.notice_warning_count, 0);
    }

    #[test]
    fn test_field_access() {
        let data = create_test_data();

        assert_eq!(data.per_version.low_vulnerability_count, 1);
        assert_eq!(data.per_version.medium_vulnerability_count, 2);
        assert_eq!(data.per_version.high_vulnerability_count, 1);
        assert_eq!(data.per_version.critical_vulnerability_count, 1);

        assert_eq!(data.per_version.notice_warning_count, 1);
        assert_eq!(data.per_version.unmaintained_warning_count, 1);
        assert_eq!(data.per_version.unsound_warning_count, 1);
        assert_eq!(data.per_version.yanked_warning_count, 0);

        assert_eq!(data.total.low_vulnerability_count, 3);
        assert_eq!(data.total.notice_warning_count, 2);
    }

    #[test]
    fn test_clone() {
        let data1 = create_test_data();
        let data2 = data1.clone();

        assert_eq!(data1.per_version.low_vulnerability_count, data2.per_version.low_vulnerability_count);
        assert_eq!(data1.per_version.notice_warning_count, data2.per_version.notice_warning_count);
        assert_eq!(data1.total.low_vulnerability_count, data2.total.low_vulnerability_count);
        assert_eq!(data1.timestamp, data2.timestamp);
    }

    #[test]
    fn test_serialize_deserialize() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: AdvisoryData = serde_json::from_str(&json).unwrap();

        assert_eq!(
            data.per_version.low_vulnerability_count,
            deserialized.per_version.low_vulnerability_count
        );
        assert_eq!(data.per_version.notice_warning_count, deserialized.per_version.notice_warning_count);
        assert_eq!(data.total.low_vulnerability_count, deserialized.total.low_vulnerability_count);
        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());
    }

    #[test]
    fn test_json_format() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"per_version\""));
        assert!(json.contains("\"total\""));
        assert!(json.contains("\"low_vulnerability_count\":1"));
        assert!(json.contains("\"notice_warning_count\":1"));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_with_no_issues() {
        let data = AdvisoryData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            ..Default::default()
        };

        assert_eq!(data.per_version.low_vulnerability_count, 0);
        assert_eq!(data.per_version.notice_warning_count, 0);
        assert_eq!(data.total.low_vulnerability_count, 0);
        assert_eq!(data.total.notice_warning_count, 0);
    }

    #[test]
    fn test_with_only_current_vulnerabilities() {
        let data = AdvisoryData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            per_version: AdvisoryCounts {
                critical_vulnerability_count: 3,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(data.per_version.critical_vulnerability_count, 3);
        assert_eq!(data.total.low_vulnerability_count, 0);
    }

    #[test]
    fn test_with_only_historical_vulnerabilities() {
        let data = AdvisoryData {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            total: AdvisoryCounts {
                low_vulnerability_count: 5,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(data.per_version.low_vulnerability_count, 0);
        assert_eq!(data.total.low_vulnerability_count, 5);
    }

    #[test]
    fn test_timestamp_roundtrip() {
        let data = create_test_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: AdvisoryData = serde_json::from_str(&json).unwrap();

        // Timestamp should be preserved through serialization
        assert_eq!(data.timestamp.timestamp(), deserialized.timestamp.timestamp());
    }
}
