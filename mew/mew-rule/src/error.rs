//! Rule error types.

use thiserror::Error;

/// Result type for rule operations.
pub type RuleResult<T> = Result<T, RuleError>;

/// Errors that can occur during rule execution.
#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Maximum recursion depth ({depth}) exceeded")]
    MaxDepthExceeded { depth: usize },

    #[error("Maximum actions ({count}) exceeded")]
    MaxActionsExceeded { count: usize },

    #[error("Rule execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("Pattern error: {message}")]
    PatternError { message: String },

    #[error("Unknown rule: {name}")]
    UnknownRule { name: String },
}

impl RuleError {
    pub fn max_depth_exceeded(depth: usize) -> Self {
        Self::MaxDepthExceeded { depth }
    }

    pub fn max_actions_exceeded(count: usize) -> Self {
        Self::MaxActionsExceeded { count }
    }

    pub fn execution_failed(message: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            message: message.into(),
        }
    }

    pub fn pattern_error(message: impl Into<String>) -> Self {
        Self::PatternError {
            message: message.into(),
        }
    }

    pub fn unknown_rule(name: impl Into<String>) -> Self {
        Self::UnknownRule { name: name.into() }
    }
}

impl From<mew_pattern::PatternError> for RuleError {
    fn from(e: mew_pattern::PatternError) -> Self {
        Self::PatternError {
            message: e.to_string(),
        }
    }
}
