//! Transaction handling for MEW sessions.
//!
//! This module contains the logic for handling transaction control statements
//! (BEGIN, COMMIT, ROLLBACK).

use crate::error::{SessionError, SessionResult};
use crate::result::{StatementResult, TransactionResult};
use mew_parser::TxnStmt;

/// Transaction state tracker.
pub struct TransactionState {
    /// Whether a transaction is active.
    pub in_transaction: bool,
}

impl TransactionState {
    /// Create a new transaction state.
    pub fn new() -> Self {
        Self {
            in_transaction: false,
        }
    }
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a transaction control statement.
pub fn execute_txn(
    state: &mut TransactionState,
    stmt: &TxnStmt,
) -> SessionResult<StatementResult> {
    match stmt {
        TxnStmt::Begin { .. } => {
            if state.in_transaction {
                return Err(SessionError::TransactionError(
                    mew_transaction::TransactionError::AlreadyActive,
                ));
            }
            state.in_transaction = true;
            Ok(StatementResult::Transaction(TransactionResult::Begun))
        }

        TxnStmt::Commit => {
            if !state.in_transaction {
                return Err(SessionError::TransactionError(
                    mew_transaction::TransactionError::NoActiveTransaction,
                ));
            }
            state.in_transaction = false;
            Ok(StatementResult::Transaction(TransactionResult::Committed))
        }

        TxnStmt::Rollback => {
            if !state.in_transaction {
                return Err(SessionError::TransactionError(
                    mew_transaction::TransactionError::NoActiveTransaction,
                ));
            }
            state.in_transaction = false;
            Ok(StatementResult::Transaction(TransactionResult::RolledBack))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_transaction() {
        // GIVEN
        let mut state = TransactionState::new();

        // WHEN
        let result = execute_txn(&mut state, &TxnStmt::Begin { isolation: None });

        // THEN
        assert!(result.is_ok());
        assert!(state.in_transaction);
    }

    #[test]
    fn test_begin_when_already_active() {
        // GIVEN
        let mut state = TransactionState::new();
        state.in_transaction = true;

        // WHEN
        let result = execute_txn(&mut state, &TxnStmt::Begin { isolation: None });

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_transaction() {
        // GIVEN
        let mut state = TransactionState::new();
        state.in_transaction = true;

        // WHEN
        let result = execute_txn(&mut state, &TxnStmt::Commit);

        // THEN
        assert!(result.is_ok());
        assert!(!state.in_transaction);
    }

    #[test]
    fn test_commit_without_transaction() {
        // GIVEN
        let mut state = TransactionState::new();

        // WHEN
        let result = execute_txn(&mut state, &TxnStmt::Commit);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_transaction() {
        // GIVEN
        let mut state = TransactionState::new();
        state.in_transaction = true;

        // WHEN
        let result = execute_txn(&mut state, &TxnStmt::Rollback);

        // THEN
        assert!(result.is_ok());
        assert!(!state.in_transaction);
    }

    #[test]
    fn test_rollback_without_transaction() {
        // GIVEN
        let mut state = TransactionState::new();

        // WHEN
        let result = execute_txn(&mut state, &TxnStmt::Rollback);

        // THEN
        assert!(result.is_err());
    }
}
