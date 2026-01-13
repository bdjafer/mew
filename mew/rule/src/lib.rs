//! MEW Rule
//!
//! Trigger and execute rules reactively on mutations.
//!
//! Responsibilities:
//! - Find rules triggered by mutations
//! - Execute rule productions
//! - Fire rules to quiescence
//! - Prevent infinite loops

mod engine;
mod error;

pub use engine::RuleEngine;
pub use error::{RuleError, RuleResult};

/// Maximum rule execution depth.
pub const MAX_DEPTH: usize = 100;

/// Maximum actions per transaction.
pub const MAX_ACTIONS: usize = 10_000;
