//! MEW Journal
//!
//! Write-ahead log and crash recovery.
//!
//! Responsibilities:
//! - Append mutation records to durable log before commit
//! - Sync log to disk on transaction commit
//! - Replay log on startup to recover state
//! - Manage log segments (rotation, cleanup)

mod entry;
mod error;
mod journal;

pub use entry::{Lsn, TxnId, WalEntry, WalRecord};
pub use error::{JournalError, JournalResult};
pub use journal::{FileJournal, MemoryJournal, RecoveryStats};
