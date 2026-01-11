//! WAL entry types.

use mew_core::{Attributes, EdgeId, EdgeTypeId, EntityId, NodeId, TypeId, Value};

/// Log Sequence Number - unique identifier for each WAL entry.
pub type Lsn = u64;

/// Transaction ID.
pub type TxnId = u64;

/// WAL entry types.
#[derive(Debug, Clone)]
pub enum WalEntry {
    /// Begin a transaction.
    Begin { txn_id: TxnId },

    /// Commit a transaction.
    Commit { txn_id: TxnId },

    /// Abort a transaction.
    Abort { txn_id: TxnId },

    /// Create a node.
    SpawnNode {
        txn_id: TxnId,
        node_id: NodeId,
        type_id: TypeId,
        attrs: Attributes,
    },

    /// Delete a node.
    KillNode {
        txn_id: TxnId,
        node_id: NodeId,
    },

    /// Create an edge.
    LinkEdge {
        txn_id: TxnId,
        edge_id: EdgeId,
        type_id: EdgeTypeId,
        targets: Vec<EntityId>,
        attrs: Attributes,
    },

    /// Delete an edge.
    UnlinkEdge {
        txn_id: TxnId,
        edge_id: EdgeId,
    },

    /// Set an attribute.
    SetAttr {
        txn_id: TxnId,
        node_id: NodeId,
        attr_name: String,
        old_value: Option<Value>,
        new_value: Value,
    },

    /// Checkpoint marker (for log truncation).
    Checkpoint {
        last_committed_lsn: Lsn,
    },
}

impl WalEntry {
    /// Get the transaction ID for this entry.
    pub fn txn_id(&self) -> Option<TxnId> {
        match self {
            WalEntry::Begin { txn_id } => Some(*txn_id),
            WalEntry::Commit { txn_id } => Some(*txn_id),
            WalEntry::Abort { txn_id } => Some(*txn_id),
            WalEntry::SpawnNode { txn_id, .. } => Some(*txn_id),
            WalEntry::KillNode { txn_id, .. } => Some(*txn_id),
            WalEntry::LinkEdge { txn_id, .. } => Some(*txn_id),
            WalEntry::UnlinkEdge { txn_id, .. } => Some(*txn_id),
            WalEntry::SetAttr { txn_id, .. } => Some(*txn_id),
            WalEntry::Checkpoint { .. } => None,
        }
    }

    /// Check if this is a commit entry.
    pub fn is_commit(&self) -> bool {
        matches!(self, WalEntry::Commit { .. })
    }

    /// Check if this is an abort entry.
    pub fn is_abort(&self) -> bool {
        matches!(self, WalEntry::Abort { .. })
    }

    /// Check if this is a begin entry.
    pub fn is_begin(&self) -> bool {
        matches!(self, WalEntry::Begin { .. })
    }
}

/// A WAL record with its LSN.
#[derive(Debug, Clone)]
pub struct WalRecord {
    /// Log sequence number.
    pub lsn: Lsn,
    /// The entry data.
    pub entry: WalEntry,
}

impl WalRecord {
    /// Create a new WAL record.
    pub fn new(lsn: Lsn, entry: WalEntry) -> Self {
        Self { lsn, entry }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txn_id_extraction() {
        // GIVEN
        let entry = WalEntry::Begin { txn_id: 42 };

        // WHEN/THEN
        assert_eq!(entry.txn_id(), Some(42));
    }

    #[test]
    fn test_checkpoint_has_no_txn_id() {
        // GIVEN
        let entry = WalEntry::Checkpoint {
            last_committed_lsn: 100,
        };

        // WHEN/THEN
        assert_eq!(entry.txn_id(), None);
    }

    #[test]
    fn test_is_commit() {
        // GIVEN
        let commit = WalEntry::Commit { txn_id: 1 };
        let begin = WalEntry::Begin { txn_id: 1 };

        // WHEN/THEN
        assert!(commit.is_commit());
        assert!(!begin.is_commit());
    }
}
