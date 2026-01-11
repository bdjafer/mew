//! Transaction error types.

use thiserror::Error;

/// Transaction errors.
#[derive(Debug, Error)]
pub enum TransactionError {
    /// No transaction is active.
    #[error("no transaction is active")]
    NoActiveTransaction,

    /// Transaction is already active.
    #[error("transaction already active")]
    AlreadyActive,

    /// Constraint violation during commit.
    #[error("constraint violation: {message}")]
    ConstraintViolation { message: String },

    /// Mutation error during transaction.
    #[error("mutation error: {0}")]
    MutationError(#[from] mew_mutation::MutationError),

    /// Rule error during transaction.
    #[error("rule error: {0}")]
    RuleError(#[from] mew_rule::RuleError),

    /// Constraint error during transaction.
    #[error("constraint error: {0}")]
    ConstraintError(#[from] mew_constraint::ConstraintError),

    /// Savepoint not found.
    #[error("savepoint not found: {name}")]
    SavepointNotFound { name: String },

    /// Transaction rolled back.
    #[error("transaction rolled back")]
    RolledBack,
}

impl TransactionError {
    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            message: message.into(),
        }
    }

    pub fn savepoint_not_found(name: impl Into<String>) -> Self {
        Self::SavepointNotFound { name: name.into() }
    }
}

/// Result type for transaction operations.
pub type TransactionResult<T> = Result<T, TransactionError>;
