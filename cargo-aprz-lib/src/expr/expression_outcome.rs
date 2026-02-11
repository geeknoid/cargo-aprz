use std::sync::Arc;

/// The outcome of evaluating a single expression.
#[derive(Debug, Clone)]
pub struct ExpressionOutcome {
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub result: bool,
}

impl ExpressionOutcome {
    #[must_use]
    #[expect(clippy::missing_const_for_fn, reason = "Arc<str> parameters prevent const")]
    pub fn new(name: Arc<str>, description: Arc<str>, result: bool) -> Self {
        Self {
            name,
            description,
            result,
        }
    }
}

