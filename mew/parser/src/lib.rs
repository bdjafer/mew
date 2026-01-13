//! MEW Parser
//!
//! This crate provides parsing for MEW source text:
//! - Statement parsing (MATCH, SPAWN, KILL, LINK, UNLINK, SET, BEGIN, COMMIT, ROLLBACK)
//! - Expression parsing (arithmetic, comparison, function calls)
//! - Ontology parsing (node, edge, constraint, rule definitions)
//! - Error handling with location information

mod ast;
mod error;
mod lexer;
mod parser;

pub use ast::*;
pub use error::*;
pub use parser::{parse_ontology, parse_stmt, parse_stmts, Parser};
