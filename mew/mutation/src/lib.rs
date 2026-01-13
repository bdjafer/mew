//! MEW Mutation
//!
//! Execute write operations (SPAWN/KILL/LINK/UNLINK/SET).
//!
//! Responsibilities:
//! - Validate mutations against schema
//! - Apply mutations to transaction buffer
//! - Handle cascade deletions
//! - Return created IDs
//!
//! # Module Structure
//!
//! - `executor` - Main MutationExecutor that coordinates operations
//! - `ops/` - Individual operation implementations (spawn, kill, link, unlink, set)
//! - `validation` - Shared attribute validation helpers
//! - `error` - Error types for mutation failures
//! - `result` - Result types for mutation outcomes

mod error;
mod executor;
mod ops;
mod result;
mod validation;

pub use error::{MutationError, MutationResult};
pub use executor::MutationExecutor;
pub use result::MutationOutcome;
