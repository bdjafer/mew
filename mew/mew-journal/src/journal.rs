//! Write-ahead log manager.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use mew_graph::Graph;

use crate::entry::{Lsn, TxnId, WalEntry, WalRecord};
use crate::error::{JournalError, JournalResult};

/// In-memory journal for testing and simple use cases.
#[derive(Debug, Default)]
pub struct MemoryJournal {
    /// All recorded entries.
    entries: Vec<WalRecord>,
    /// Next LSN to assign.
    next_lsn: Lsn,
    /// Next transaction ID.
    next_txn_id: TxnId,
}

impl MemoryJournal {
    /// Create a new empty memory journal.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_lsn: 1,
            next_txn_id: 1,
        }
    }

    /// Allocate a new transaction ID.
    pub fn alloc_txn_id(&mut self) -> TxnId {
        let id = self.next_txn_id;
        self.next_txn_id += 1;
        id
    }

    /// Append an entry to the journal.
    pub fn append(&mut self, entry: WalEntry) -> Lsn {
        let lsn = self.next_lsn;
        self.next_lsn += 1;
        self.entries.push(WalRecord::new(lsn, entry));
        lsn
    }

    /// Sync is a no-op for memory journal.
    pub fn sync(&mut self) -> JournalResult<()> {
        Ok(())
    }

    /// Get all entries.
    pub fn entries(&self) -> &[WalRecord] {
        &self.entries
    }

    /// Get entry by LSN.
    pub fn get(&self, lsn: Lsn) -> Option<&WalRecord> {
        self.entries.iter().find(|r| r.lsn == lsn)
    }

    /// Recover committed transactions into a graph.
    pub fn recover(&self, graph: &mut Graph) -> JournalResult<RecoveryStats> {
        let mut stats = RecoveryStats::default();

        // First pass: identify committed transactions
        let mut committed: HashSet<TxnId> = HashSet::new();
        let mut aborted: HashSet<TxnId> = HashSet::new();

        for record in &self.entries {
            if let Some(txn_id) = record.entry.txn_id() {
                if record.entry.is_commit() {
                    committed.insert(txn_id);
                } else if record.entry.is_abort() {
                    aborted.insert(txn_id);
                }
            }
        }

        stats.committed_transactions = committed.len();
        stats.aborted_transactions = aborted.len();

        // Second pass: replay only committed transactions
        for record in &self.entries {
            if let Some(txn_id) = record.entry.txn_id() {
                if !committed.contains(&txn_id) {
                    // Skip uncommitted or aborted transactions
                    continue;
                }

                stats.entries_replayed += 1;

                match &record.entry {
                    WalEntry::SpawnNode {
                        node_id,
                        type_id,
                        attrs,
                        ..
                    } => {
                        // For recovery, we need to create node with specific ID
                        // Since Graph doesn't support this directly, we'll track it differently
                        // In a real implementation, we'd extend Graph
                        let _new_id = graph.create_node(*type_id, attrs.clone());
                        // Note: new_id may differ from node_id - would need ID mapping in real impl
                        let _ = node_id; // Acknowledge unused
                        stats.nodes_created += 1;
                    }

                    WalEntry::KillNode { node_id, .. } => {
                        let _ = graph.delete_node(*node_id);
                        stats.nodes_deleted += 1;
                    }

                    WalEntry::LinkEdge {
                        edge_id,
                        type_id,
                        targets,
                        attrs,
                        ..
                    } => {
                        let _ = graph.create_edge(*type_id, targets.clone(), attrs.clone());
                        let _ = edge_id; // Acknowledge unused
                        stats.edges_created += 1;
                    }

                    WalEntry::UnlinkEdge { edge_id, .. } => {
                        let _ = graph.delete_edge(*edge_id);
                        stats.edges_deleted += 1;
                    }

                    WalEntry::SetAttr {
                        node_id,
                        attr_name,
                        new_value,
                        ..
                    } => {
                        let _ = graph.set_node_attr(*node_id, attr_name, new_value.clone());
                        stats.attrs_updated += 1;
                    }

                    // Skip transaction control entries
                    WalEntry::Begin { .. }
                    | WalEntry::Commit { .. }
                    | WalEntry::Abort { .. }
                    | WalEntry::Checkpoint { .. } => {}
                }
            }
        }

        Ok(stats)
    }

    /// Clear the journal (for testing).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_lsn = 1;
    }
}

/// Statistics from recovery.
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    /// Number of committed transactions.
    pub committed_transactions: usize,
    /// Number of aborted transactions.
    pub aborted_transactions: usize,
    /// Number of entries replayed.
    pub entries_replayed: usize,
    /// Number of nodes created.
    pub nodes_created: usize,
    /// Number of nodes deleted.
    pub nodes_deleted: usize,
    /// Number of edges created.
    pub edges_created: usize,
    /// Number of edges deleted.
    pub edges_deleted: usize,
    /// Number of attributes updated.
    pub attrs_updated: usize,
}

/// File-based journal for durability.
pub struct FileJournal {
    /// Path to the journal file.
    path: PathBuf,
    /// File for writing.
    writer: Option<BufWriter<File>>,
    /// Next LSN to assign.
    next_lsn: Lsn,
    /// Next transaction ID.
    next_txn_id: TxnId,
    /// In-memory buffer of recent entries (for recovery without re-reading file).
    recent_entries: Vec<WalRecord>,
}

impl FileJournal {
    /// Open or create a journal file.
    pub fn open(path: impl AsRef<Path>) -> JournalResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Read existing entries to determine next LSN
        let (next_lsn, next_txn_id, recent_entries) = if path.exists() {
            Self::scan_file(&path)?
        } else {
            (1, 1, Vec::new())
        };

        // Open for append
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            path,
            writer: Some(BufWriter::new(file)),
            next_lsn,
            next_txn_id,
            recent_entries,
        })
    }

    /// Scan the file to get next LSN and transaction ID.
    fn scan_file(path: &Path) -> JournalResult<(Lsn, TxnId, Vec<WalRecord>)> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut max_lsn: Lsn = 0;
        let mut max_txn_id: TxnId = 0;
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            // Parse entry (simple format: "LSN|TXN_ID|TYPE|DATA")
            let record = Self::parse_entry(&line)?;

            if record.lsn > max_lsn {
                max_lsn = record.lsn;
            }

            if let Some(txn_id) = record.entry.txn_id() {
                if txn_id > max_txn_id {
                    max_txn_id = txn_id;
                }
            }

            entries.push(record);
        }

        Ok((max_lsn + 1, max_txn_id + 1, entries))
    }

    /// Parse an entry from a line.
    fn parse_entry(line: &str) -> JournalResult<WalRecord> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 3 {
            return Err(JournalError::invalid_format("not enough fields"));
        }

        let lsn: Lsn = parts[0]
            .parse()
            .map_err(|_| JournalError::invalid_format("invalid LSN"))?;

        let txn_id: TxnId = parts[1]
            .parse()
            .map_err(|_| JournalError::invalid_format("invalid txn_id"))?;

        let entry_type = parts[2];

        let entry = match entry_type {
            "BEGIN" => WalEntry::Begin { txn_id },
            "COMMIT" => WalEntry::Commit { txn_id },
            "ABORT" => WalEntry::Abort { txn_id },
            // For other entry types, we'd need more complex parsing
            // Simplified for now - in production we'd use a proper serialization format
            _ => {
                return Err(JournalError::invalid_format(format!(
                    "unknown type: {}",
                    entry_type
                )))
            }
        };

        Ok(WalRecord::new(lsn, entry))
    }

    /// Format an entry for writing.
    fn format_entry(record: &WalRecord) -> String {
        let txn_id = record.entry.txn_id().unwrap_or(0);
        let entry_type = match &record.entry {
            WalEntry::Begin { .. } => "BEGIN",
            WalEntry::Commit { .. } => "COMMIT",
            WalEntry::Abort { .. } => "ABORT",
            WalEntry::SpawnNode { .. } => "SPAWN",
            WalEntry::KillNode { .. } => "KILL",
            WalEntry::LinkEdge { .. } => "LINK",
            WalEntry::UnlinkEdge { .. } => "UNLINK",
            WalEntry::SetAttr { .. } => "SET",
            WalEntry::Checkpoint { .. } => "CHECKPOINT",
        };

        format!("{}|{}|{}", record.lsn, txn_id, entry_type)
    }

    /// Allocate a new transaction ID.
    pub fn alloc_txn_id(&mut self) -> TxnId {
        let id = self.next_txn_id;
        self.next_txn_id += 1;
        id
    }

    /// Append an entry to the journal.
    pub fn append(&mut self, entry: WalEntry) -> JournalResult<Lsn> {
        let lsn = self.next_lsn;
        self.next_lsn += 1;

        let record = WalRecord::new(lsn, entry);
        let line = Self::format_entry(&record);

        if let Some(ref mut writer) = self.writer {
            writeln!(writer, "{}", line)?;
        }

        self.recent_entries.push(record);

        Ok(lsn)
    }

    /// Sync to disk.
    pub fn sync(&mut self) -> JournalResult<()> {
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
            // On Unix, we'd also call fsync() here
        }
        Ok(())
    }

    /// Get all recent entries.
    pub fn entries(&self) -> &[WalRecord] {
        &self.recent_entries
    }

    /// Recover into a graph.
    pub fn recover(&self, graph: &mut Graph) -> JournalResult<RecoveryStats> {
        // Use the same recovery logic as MemoryJournal
        let mem = MemoryJournal {
            entries: self.recent_entries.clone(),
            next_lsn: self.next_lsn,
            next_txn_id: self.next_txn_id,
        };
        mem.recover(graph)
    }

    /// Get the journal file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::{attrs, TypeId};

    #[test]
    fn test_memory_journal_append() {
        // GIVEN
        let mut journal = MemoryJournal::new();

        // WHEN
        let txn_id = journal.alloc_txn_id();
        let lsn1 = journal.append(WalEntry::Begin { txn_id });
        let lsn2 = journal.append(WalEntry::Commit { txn_id });

        // THEN
        assert_eq!(lsn1, 1);
        assert_eq!(lsn2, 2);
        assert_eq!(journal.entries().len(), 2);
    }

    #[test]
    fn test_memory_journal_get() {
        // GIVEN
        let mut journal = MemoryJournal::new();
        let txn_id = journal.alloc_txn_id();
        let lsn = journal.append(WalEntry::Begin { txn_id });

        // WHEN
        let record = journal.get(lsn);

        // THEN
        assert!(record.is_some());
        assert_eq!(record.unwrap().lsn, lsn);
    }

    #[test]
    fn test_recovery_committed_only() {
        // GIVEN
        let mut journal = MemoryJournal::new();
        let mut graph = Graph::new();

        // Transaction 1: committed
        let txn1 = journal.alloc_txn_id();
        journal.append(WalEntry::Begin { txn_id: txn1 });
        journal.append(WalEntry::SpawnNode {
            txn_id: txn1,
            node_id: mew_core::NodeId::new(1),
            type_id: TypeId(1),
            attrs: attrs! { "name" => "committed" },
        });
        journal.append(WalEntry::Commit { txn_id: txn1 });

        // Transaction 2: uncommitted (no commit entry)
        let txn2 = journal.alloc_txn_id();
        journal.append(WalEntry::Begin { txn_id: txn2 });
        journal.append(WalEntry::SpawnNode {
            txn_id: txn2,
            node_id: mew_core::NodeId::new(2),
            type_id: TypeId(2),
            attrs: attrs! { "name" => "uncommitted" },
        });
        // No commit!

        // WHEN
        let stats = journal.recover(&mut graph).unwrap();

        // THEN - only committed transaction was replayed
        assert_eq!(stats.committed_transactions, 1);
        assert_eq!(stats.nodes_created, 1);
        // Graph should have 1 node (from committed transaction)
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_recovery_aborted_not_replayed() {
        // GIVEN
        let mut journal = MemoryJournal::new();
        let mut graph = Graph::new();

        // Transaction: aborted
        let txn = journal.alloc_txn_id();
        journal.append(WalEntry::Begin { txn_id: txn });
        journal.append(WalEntry::SpawnNode {
            txn_id: txn,
            node_id: mew_core::NodeId::new(1),
            type_id: TypeId(1),
            attrs: attrs! {},
        });
        journal.append(WalEntry::Abort { txn_id: txn });

        // WHEN
        let stats = journal.recover(&mut graph).unwrap();

        // THEN
        assert_eq!(stats.aborted_transactions, 1);
        assert_eq!(stats.committed_transactions, 0);
        assert_eq!(stats.nodes_created, 0);
    }

    #[test]
    fn test_txn_id_allocation() {
        // GIVEN
        let mut journal = MemoryJournal::new();

        // WHEN
        let id1 = journal.alloc_txn_id();
        let id2 = journal.alloc_txn_id();
        let id3 = journal.alloc_txn_id();

        // THEN
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_sync_memory() {
        // GIVEN
        let mut journal = MemoryJournal::new();

        // WHEN/THEN - sync should succeed for memory journal
        assert!(journal.sync().is_ok());
    }
}
