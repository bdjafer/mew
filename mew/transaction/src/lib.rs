//! MEW Transaction
//!
//! ACID transaction management, orchestrate mutation->rule->constraint flow.
//!
//! Responsibilities:
//! - Track pending changes (transaction buffer)
//! - Orchestrate: mutation → rules → constraints
//! - Implement BEGIN/COMMIT/ROLLBACK
//! - Coordinate with Journal for durability
//! - Provide read-your-writes isolation

mod buffer;
mod error;
mod manager;

pub use buffer::{PendingEdge, PendingNode, PendingUpdate, TransactionBuffer};
pub use error::{TransactionError, TransactionResult};
pub use manager::{TransactionManager, TransactionState};
