use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display)]
pub enum MetricCategory {
    Metadata,
    Stability,
    Usage,
    Community,
    Activity,
    Documentation,
    Trustworthiness,
    Codebase,
    Advisories,
}

impl MetricCategory {
    #[must_use]
    pub const fn as_uppercase_str(self) -> &'static str {
        match self {
            Self::Metadata => "METADATA",
            Self::Stability => "STABILITY",
            Self::Usage => "USAGE",
            Self::Community => "COMMUNITY",
            Self::Activity => "ACTIVITY",
            Self::Documentation => "DOCUMENTATION",
            Self::Trustworthiness => "TRUSTWORTHINESS",
            Self::Codebase => "CODEBASE",
            Self::Advisories => "ADVISORIES",
        }
    }
}
