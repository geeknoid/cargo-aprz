use core::fmt;
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

    /// Returns the pass/fail icon for this outcome.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        if self.result { "✔" } else { "✗" }
    }

    /// Returns a displayable `icon + name` value (no allocation until formatted).
    #[must_use]
    pub const fn icon_name(&self) -> IconName<'_> {
        IconName(self)
    }
}

/// A zero-allocation wrapper that displays `icon + name` for an [`ExpressionOutcome`].
#[derive(Debug)]
pub struct IconName<'a>(&'a ExpressionOutcome);

impl fmt::Display for IconName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0.icon(), self.0.name)
    }
}

impl fmt::Display for ExpressionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.result {
            write!(f, "{}: {}", self.name, self.description)
        } else {
            write!(f, "{} (failed): {}", self.name, self.description)
        }
    }
}
