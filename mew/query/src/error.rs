//! Query error types.

use thiserror::Error;

/// Result type for query operations.
pub type QueryResult<T> = Result<T, QueryError>;

/// Errors that can occur during query execution.
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Unknown type: {name}")]
    UnknownType { name: String },

    #[error("Unknown attribute: {attr} on type {type_name}")]
    UnknownAttribute { type_name: String, attr: String },

    #[error("Pattern matching failed: {message}")]
    PatternError { message: String },

    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Aggregate error: {message}")]
    AggregateError { message: String },

    #[error("Query aborted: {reason}")]
    Aborted { reason: String },
}

impl QueryError {
    pub fn unknown_type(name: impl Into<String>) -> Self {
        Self::UnknownType { name: name.into() }
    }

    pub fn unknown_attribute(type_name: impl Into<String>, attr: impl Into<String>) -> Self {
        Self::UnknownAttribute {
            type_name: type_name.into(),
            attr: attr.into(),
        }
    }

    pub fn pattern_error(message: impl Into<String>) -> Self {
        Self::PatternError {
            message: message.into(),
        }
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    pub fn aggregate_error(message: impl Into<String>) -> Self {
        Self::AggregateError {
            message: message.into(),
        }
    }
}

impl From<mew_pattern::PatternError> for QueryError {
    fn from(e: mew_pattern::PatternError) -> Self {
        Self::PatternError {
            message: e.to_string(),
        }
    }
}
