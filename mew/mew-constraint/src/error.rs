//! Constraint error types.

use thiserror::Error;

/// Result type for constraint operations.
pub type ConstraintResult<T> = Result<T, ConstraintError>;

/// Errors that can occur during constraint checking.
#[derive(Debug, Error)]
pub enum ConstraintError {
    #[error("Constraint violated: {name}")]
    Violated { name: String },

    #[error("Unknown constraint: {name}")]
    UnknownConstraint { name: String },

    #[error("Pattern error: {message}")]
    PatternError { message: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl ConstraintError {
    pub fn violated(name: impl Into<String>) -> Self {
        Self::Violated { name: name.into() }
    }

    pub fn unknown_constraint(name: impl Into<String>) -> Self {
        Self::UnknownConstraint { name: name.into() }
    }

    pub fn pattern_error(message: impl Into<String>) -> Self {
        Self::PatternError {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}

impl From<mew_pattern::PatternError> for ConstraintError {
    fn from(e: mew_pattern::PatternError) -> Self {
        Self::PatternError {
            message: e.to_string(),
        }
    }
}
