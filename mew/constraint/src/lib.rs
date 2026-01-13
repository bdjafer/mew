//! MEW Constraint
//!
//! Validate graph state against declared constraints.
//!
//! Responsibilities:
//! - Check immediate constraints after mutations
//! - Check deferred constraints at commit
//! - Distinguish hard (abort) vs soft (warn) constraints
//! - Produce meaningful violation messages

mod checker;
mod error;
mod violation;

pub use checker::ConstraintChecker;
pub use error::{ConstraintError, ConstraintResult};
pub use violation::{Violation, ViolationSeverity};
