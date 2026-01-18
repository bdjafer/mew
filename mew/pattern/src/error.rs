//! Pattern error types.

use thiserror::Error;

/// Errors that can occur during pattern matching.
#[derive(Debug, Error)]
pub enum PatternError {
    /// Unbound variable in expression.
    #[error("Unbound variable '{name}'")]
    UnboundVariable { name: String },

    /// Missing parameter in expression.
    #[error("missing_parameter '${name}' not provided")]
    MissingParameter { name: String },

    /// Type mismatch in expression.
    #[error("type error: {message}")]
    TypeError { message: String },

    /// Unknown type name.
    #[error("Unknown type '{name}'")]
    UnknownType { name: String },

    /// Unknown edge type name.
    #[error("Unknown edge type '{name}'")]
    UnknownEdgeType { name: String },

    /// Unknown attribute name.
    #[error("Unknown attribute '{attr}' on type '{type_name}'")]
    UnknownAttribute { attr: String, type_name: String },

    /// Invalid operation.
    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    /// Division by zero.
    #[error("Division by zero")]
    DivisionByZero,

    /// Node not found by ID reference.
    #[error("Node not found: no node with ID '{id}'")]
    NodeNotFound { id: String },
}

impl PatternError {
    pub fn unbound_variable(name: impl Into<String>) -> Self {
        Self::UnboundVariable { name: name.into() }
    }

    pub fn missing_parameter(name: impl Into<String>) -> Self {
        Self::MissingParameter { name: name.into() }
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    pub fn unknown_type(name: impl Into<String>) -> Self {
        Self::UnknownType { name: name.into() }
    }

    pub fn unknown_edge_type(name: impl Into<String>) -> Self {
        Self::UnknownEdgeType { name: name.into() }
    }

    pub fn unknown_attribute(attr: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self::UnknownAttribute {
            attr: attr.into(),
            type_name: type_name.into(),
        }
    }

    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::InvalidOperation {
            message: message.into(),
        }
    }

    pub fn node_not_found(id: impl Into<String>) -> Self {
        Self::NodeNotFound { id: id.into() }
    }
}

/// Result type for pattern operations.
pub type PatternResult<T> = Result<T, PatternError>;
