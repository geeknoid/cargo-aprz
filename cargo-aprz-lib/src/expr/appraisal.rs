use super::{ExpressionOutcome, Risk};

/// The outcome of evaluating a crate against policy expressions.
#[derive(Debug, Clone)]
pub struct Appraisal {
    pub risk: Risk,
    pub expression_outcomes: Vec<ExpressionOutcome>,
}

impl Appraisal {
    #[must_use]
    pub const fn new(risk: Risk, expression_outcomes: Vec<ExpressionOutcome>) -> Self {
        Self {
            risk,
            expression_outcomes,
        }
    }
}
