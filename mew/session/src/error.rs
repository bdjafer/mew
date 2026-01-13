//! Session error types.

use thiserror::Error;

/// Session errors.
#[derive(Debug, Error)]
pub enum SessionError {
    /// Parse error.
    #[error("parse error: {0}")]
    ParseError(#[from] mew_parser::ParseError),

    /// Analysis error.
    #[error("analysis error: {message}")]
    AnalysisError { message: String },

    /// Query error.
    #[error("query error: {0}")]
    QueryError(#[from] mew_query::QueryError),

    /// Mutation error.
    #[error("mutation error: {0}")]
    MutationError(#[from] mew_mutation::MutationError),

    /// Transaction error.
    #[error("transaction error: {0}")]
    TransactionError(#[from] mew_transaction::TransactionError),

    /// Compilation error.
    #[error("compilation error: {0}")]
    CompileError(#[from] mew_compiler::CompileError),

    /// Pattern error.
    #[error("pattern error: {0}")]
    PatternError(#[from] mew_pattern::PatternError),

    /// Session not found.
    #[error("session not found: {id}")]
    SessionNotFound { id: u64 },

    /// Invalid statement type.
    #[error("invalid statement type: {message}")]
    InvalidStatementType { message: String },
}

impl SessionError {
    pub fn analysis_error(message: impl Into<String>) -> Self {
        Self::AnalysisError {
            message: message.into(),
        }
    }

    pub fn session_not_found(id: u64) -> Self {
        Self::SessionNotFound { id }
    }

    pub fn invalid_statement_type(message: impl Into<String>) -> Self {
        Self::InvalidStatementType {
            message: message.into(),
        }
    }
}

impl From<mew_pattern::TargetError> for SessionError {
    fn from(err: mew_pattern::TargetError) -> Self {
        SessionError::invalid_statement_type(err.to_string())
    }
}

/// Result type for session operations.
pub type SessionResult<T> = Result<T, SessionError>;
