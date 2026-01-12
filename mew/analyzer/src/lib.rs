//! MEW Analyzer
//!
//! Name resolution and type checking against Registry.
//! Transforms raw AST into analyzed AST with resolved types.

mod analyzer;
mod error;
mod scope;
mod types;

pub use analyzer::Analyzer;
pub use error::{AnalyzerError, AnalyzerResult};
pub use scope::{Scope, VarBinding};
pub use types::*;
