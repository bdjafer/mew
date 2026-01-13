//! MEW Compiler
//!
//! Transform ontology source into populated Registry + Layer 0 graph.
//!
//! Responsibilities:
//! - Parse ontology DSL
//! - Expand syntactic sugar (modifiers → constraints/rules)
//! - Validate ontology consistency
//! - Generate Layer 0 nodes and edges
//! - Build and return Registry

mod compiler;
mod error;

pub use compiler::{compile, Compiler};
pub use error::{CompileError, CompileResult};
