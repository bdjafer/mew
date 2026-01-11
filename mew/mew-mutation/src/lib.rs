//! MEW Mutation
//!
//! Execute write operations (SPAWN/KILL/LINK/UNLINK/SET).
//!
//! Responsibilities:
//! - Validate mutations against schema
//! - Apply mutations to transaction buffer
//! - Handle cascade deletions
//! - Return created IDs

mod error;
mod executor;
mod result;

pub use error::{MutationError, MutationResult};
pub use executor::MutationExecutor;
pub use result::MutationResult as MutationOutput;
