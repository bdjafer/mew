//! Analyzer error types.

use crate::Type;
use mew_parser::Span;
use thiserror::Error;

/// Errors that can occur during analysis.
#[derive(Debug, Error)]
pub enum AnalyzerError {
    /// Unknown type name.
    #[error("Unknown type '{name}' at line {line}, column {column}")]
    UnknownType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Unknown edge type name.
    #[error("Unknown edge type '{name}' at line {line}, column {column}")]
    UnknownEdgeType {
        name: String,
        line: usize,
        column: usize,
    },

    /// Unknown attribute on type.
    #[error("Unknown attribute '{attr}' on type '{type_name}' at line {line}, column {column}")]
    UnknownAttribute {
        attr: String,
        type_name: String,
        line: usize,
        column: usize,
    },

    /// Undefined variable.
    #[error("Undefined variable '{name}' at line {line}, column {column}")]
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },

    /// Duplicate variable.
    #[error("Duplicate variable '{name}' at line {line}, column {column}")]
    DuplicateVariable {
        name: String,
        line: usize,
        column: usize,
    },

    /// Type mismatch.
    #[error("Type mismatch: expected {expected}, got {actual} at line {line}, column {column}")]
    TypeMismatch {
        expected: String,
        actual: String,
        line: usize,
        column: usize,
    },

    /// Invalid operator for types.
    #[error(
        "Invalid operator '{op}' for types {left} and {right} at line {line}, column {column}"
    )]
    InvalidOperator {
        op: String,
        left: String,
        right: String,
        line: usize,
        column: usize,
    },

    /// Invalid unary operator.
    #[error("Invalid unary operator '{op}' for type {operand} at line {line}, column {column}")]
    InvalidUnaryOperator {
        op: String,
        operand: String,
        line: usize,
        column: usize,
    },

    /// Cannot access attribute on non-node type.
    #[error("Cannot access attribute on non-node type {actual} at line {line}, column {column}")]
    CannotAccessAttribute {
        actual: String,
        line: usize,
        column: usize,
    },

    /// Wrong number of targets for edge.
    #[error(
        "Edge '{edge}' expects {expected} targets, got {actual} at line {line}, column {column}"
    )]
    WrongTargetCount {
        edge: String,
        expected: usize,
        actual: usize,
        line: usize,
        column: usize,
    },
}

impl AnalyzerError {
    pub fn unknown_type(name: impl Into<String>, span: Span) -> Self {
        Self::UnknownType {
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

    pub fn unknown_attribute(
        attr: impl Into<String>,
        type_name: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::UnknownAttribute {
            attr: attr.into(),
            type_name: type_name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn undefined_variable(name: impl Into<String>, span: Span) -> Self {
        Self::UndefinedVariable {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn duplicate_variable(name: impl Into<String>, span: Span) -> Self {
        Self::DuplicateVariable {
            name: name.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn type_mismatch(expected: &Type, actual: &Type, span: Span) -> Self {
        Self::TypeMismatch {
            expected: expected.name().to_string(),
            actual: actual.name().to_string(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn invalid_operator(op: impl Into<String>, left: &Type, right: &Type, span: Span) -> Self {
        Self::InvalidOperator {
            op: op.into(),
            left: left.name().to_string(),
            right: right.name().to_string(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn invalid_unary_operator(op: impl Into<String>, operand: &Type, span: Span) -> Self {
        Self::InvalidUnaryOperator {
            op: op.into(),
            operand: operand.name().to_string(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn cannot_access_attribute(actual: &Type, span: Span) -> Self {
        Self::CannotAccessAttribute {
            actual: actual.name().to_string(),
            line: span.line,
            column: span.column,
        }
    }
}

/// Result type for analyzer operations.
pub type AnalyzerResult<T> = Result<T, AnalyzerError>;
