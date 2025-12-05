//! Policy evaluation result type

/// Result of evaluating a policy
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyOutcome {
    /// Policy evaluation matched with the given points and information about the matching policy
    Match(f64, String),

    /// Policy evaluation didn't match with the given reason
    NoMatch(String),
}
