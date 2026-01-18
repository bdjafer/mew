//! Transaction handling for MEW sessions.
//!
//! This module contains the logic for handling transaction control statements
//! (BEGIN, COMMIT, ROLLBACK).

use crate::error::{SessionError, SessionResult};
use crate::result::{StatementResult, TransactionResult};
use mew_core::{EdgeId, NodeId};
use mew_parser::TxnStmt;

/// Transaction state tracker.
pub struct TransactionState {
    /// Whether a transaction is active.
    pub in_transaction: bool,
    /// Nodes created during this transaction (for rollback).
    pub created_nodes: Vec<NodeId>,
    /// Edges created during this transaction (for rollback).
    pub created_edges: Vec<EdgeId>,
}

impl TransactionState {
    /// Create a new transaction state.
    pub fn new() -> Self {
        Self {
            in_transaction: false,
            created_nodes: Vec::new(),
            created_edges: Vec::new(),
        }
    }

    /// Track a created node for potential rollback.
    pub fn track_created_node(&mut self, id: NodeId) {
        if self.in_transaction {
            self.created_nodes.push(id);
        }
    }

    /// Track a created edge for potential rollback.
    pub fn track_created_edge(&mut self, id: EdgeId) {
        if self.in_transaction {
            self.created_edges.push(id);
        }
    }

    /// Clear tracked changes (for commit or after rollback).
    pub fn clear_tracked(&mut self) {
        self.created_nodes.clear();
        self.created_edges.clear();
    }
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a transaction control statement.
pub fn execute_txn(state: &mut TransactionState, stmt: &TxnStmt) -> SessionResult<StatementResult> {
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
            state.clear_tracked(); // Clear tracked entities on commit
            Ok(StatementResult::Transaction(TransactionResult::Committed))
        }

        TxnStmt::Rollback => {
            if !state.in_transaction {
                return Err(SessionError::TransactionError(
                    mew_transaction::TransactionError::NoActiveTransaction,
                ));
            }
            // Note: Caller must handle the actual rollback using state.created_nodes/created_edges
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
