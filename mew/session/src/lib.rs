//! MEW Session
//!
//! External interface (REPL, HTTP, embedded).
//!
//! Responsibilities:
//! - Accept statements (REPL, HTTP, embedded)
//! - Route statements to appropriate executor
//! - Track session state (current transaction)
//! - Format and return results
//! - Handle errors gracefully

mod error;
mod result;
mod session;

pub use error::{SessionError, SessionResult};
pub use result::{MutationResult, QueryResult, StatementResult, TransactionResult};
pub use session::{Session, SessionId, SessionManager};
