//! MEW Pattern
//!
//! Compile patterns, match against graph, evaluate expressions.
//!
//! Responsibilities:
//! - Compile pattern AST into executable form
//! - Find all matches of pattern in graph
//! - Evaluate expressions given variable bindings
//! - Support transitive closure (edge+, edge*)
//! - Support negative patterns (NOT EXISTS)
//! - Shared target resolution utilities

mod binding;
mod error;
mod eval;
mod matcher;
mod pattern;
pub mod target;

pub use binding::{Binding, Bindings};
pub use error::{PatternError, PatternResult};
pub use eval::Evaluator;
pub use matcher::Matcher;
pub use pattern::{CompiledPattern, PatternOp};
pub use target::{resolve_target, resolve_target_ref, resolve_var_target, TargetError};
