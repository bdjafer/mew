//! Compiler error types.

use mew_parser::Span;
use thiserror::Error;

/// Errors that can occur during compilation.
#[derive(Debug, Error)]
pub enum CompileError {
    /// Parse error from the parser.
    #[error("Parse error: {0}")]
    Parse(#[from] mew_parser::ParseError),

    /// Duplicate type name.
    #[error("Duplicate type name '{name}' at line {line}, column {column}")]
    DuplicateType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Duplicate edge type name.
    #[error("Duplicate edge type name '{name}' at line {line}, column {column}")]
    DuplicateEdgeType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Unknown type reference.
    #[error("Unknown type '{name}' at line {line}, column {column}")]
    UnknownType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Unknown edge type in edge reference.
    #[error("Unknown edge type '{name}' in edge reference at line {line}, column {column}")]
    UnknownEdgeType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Unknown parent type.
    #[error("Unknown parent type '{name}' at line {line}, column {column}")]
    UnknownParentType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Inheritance cycle detected.
    #[error("Inheritance cycle detected: {cycle}")]
    InheritanceCycle { cycle: String },

    /// Validation error.
    #[error("Validation error: {message} at line {line}, column {column}")]
    Validation {
        message: String,
        line: usize,
        column: usize,
    },

    /// Registry build error.
    #[error("Registry error: {0}")]
    Registry(#[from] mew_registry::RegistryError),
}

impl CompileError {
    pub fn duplicate_type(name: impl Into<String>, span: Span) -> Self {
        Self::DuplicateType {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn duplicate_edge_type(name: impl Into<String>, span: Span) -> Self {
        Self::DuplicateEdgeType {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn unknown_type(name: impl Into<String>, span: Span) -> Self {
        Self::UnknownType {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn unknown_parent_type(name: impl Into<String>, span: Span) -> Self {
        Self::UnknownParentType {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn unknown_edge_type(name: impl Into<String>, span: Span) -> Self {
        Self::UnknownEdgeType {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn validation(message: impl Into<String>, span: Span) -> Self {
        Self::Validation {
            message: message.into(),
            line: span.line,
            column: span.column,
        }
    }
}

/// Result type for compiler operations.
pub type CompileResult<T> = Result<T, CompileError>;
