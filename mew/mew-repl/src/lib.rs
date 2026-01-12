//! MEW REPL library - Read-Eval-Print-Loop for MEW hypergraph database.
//!
//! This crate provides the REPL functionality for interacting with MEW.
//! It is split into modules for better maintainability:
//!
//! - `repl`: Core REPL state and execution
//! - `executor`: Statement execution (MATCH, SPAWN, KILL, etc.)
//! - `block`: Block collection for multi-line input
//! - `format`: Output formatting utilities

mod block;
mod executor;
mod format;
mod repl;

pub use format::{format_value, print_help};
pub use repl::Repl;
