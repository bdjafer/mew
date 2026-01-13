//! Mutation error types.

use mew_core::{EdgeId, NodeId};
use thiserror::Error;

/// Result type for mutation operations.
pub type MutationResult<T> = Result<T, MutationError>;

/// Errors that can occur during mutation execution.
#[derive(Debug, Error)]
pub enum MutationError {
    #[error("Unknown type: {name}")]
    UnknownType { name: String },

    #[error("Unknown edge type: {name}")]
    UnknownEdgeType { name: String },

    #[error("Cannot instantiate abstract type: {name}")]
    AbstractType { name: String },

    #[error("Missing required attribute: {attr} on type {type_name}")]
    MissingRequired { type_name: String, attr: String },

    #[error("Cannot set required attribute to null: {attr} on type {type_name}")]
    RequiredNullViolation { type_name: String, attr: String },

    #[error("Invalid attribute type: expected {expected}, got {actual} for {attr}")]
    InvalidAttrType {
        attr: String,
        expected: String,
        actual: String,
    },

    #[error("Unknown attribute: {attr} on type {type_name}")]
    UnknownAttribute { type_name: String, attr: String },

    #[error("Invalid arity: expected {expected}, got {actual} for edge type {edge_type}")]
    InvalidArity {
        edge_type: String,
        expected: usize,
        actual: usize,
    },

    #[error("Target type mismatch at position {position}: expected {expected}, got {actual}")]
    TargetTypeMismatch {
        position: usize,
        expected: String,
        actual: String,
    },

    #[error("Acyclic constraint violated for edge type {edge_type}")]
    AcyclicViolation { edge_type: String },

    #[error("Deletion restricted by edge type {edge_type}")]
    OnKillRestrict { edge_type: String },

    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0:?}")]
    EdgeNotFound(EdgeId),

    #[error("Pattern error: {message}")]
    PatternError { message: String },

    #[error("Evaluation error: {message}")]
    EvalError { message: String },

    #[error("Range constraint violated: {attr} value {value} is out of range{range_desc}")]
    RangeViolation {
        attr: String,
        value: String,
        range_desc: String,
    },

    #[error("Cannot modify readonly attribute: {attr} on type {type_name}")]
    ReadonlyAttribute { type_name: String, attr: String },
}

impl MutationError {
    pub fn unknown_type(name: impl Into<String>) -> Self {
        Self::UnknownType { name: name.into() }
    }

    pub fn unknown_edge_type(name: impl Into<String>) -> Self {
        Self::UnknownEdgeType { name: name.into() }
    }

    pub fn abstract_type(name: impl Into<String>) -> Self {
        Self::AbstractType { name: name.into() }
    }

    pub fn missing_required(type_name: impl Into<String>, attr: impl Into<String>) -> Self {
        Self::MissingRequired {
            type_name: type_name.into(),
            attr: attr.into(),
        }
    }

    pub fn required_null_violation(type_name: impl Into<String>, attr: impl Into<String>) -> Self {
        Self::RequiredNullViolation {
            type_name: type_name.into(),
            attr: attr.into(),
        }
    }

    pub fn invalid_attr_type(
        attr: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::InvalidAttrType {
            attr: attr.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    pub fn unknown_attribute(type_name: impl Into<String>, attr: impl Into<String>) -> Self {
        Self::UnknownAttribute {
            type_name: type_name.into(),
            attr: attr.into(),
        }
    }

    pub fn invalid_arity(edge_type: impl Into<String>, expected: usize, actual: usize) -> Self {
        Self::InvalidArity {
            edge_type: edge_type.into(),
            expected,
            actual,
        }
    }

    pub fn target_type_mismatch(
        position: usize,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::TargetTypeMismatch {
            position,
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    pub fn acyclic_violation(edge_type: impl Into<String>) -> Self {
        Self::AcyclicViolation {
            edge_type: edge_type.into(),
        }
    }

    pub fn on_kill_restrict(edge_type: impl Into<String>) -> Self {
        Self::OnKillRestrict {
            edge_type: edge_type.into(),
        }
    }

    pub fn pattern_error(message: impl Into<String>) -> Self {
        Self::PatternError {
            message: message.into(),
        }
    }

    pub fn eval_error(message: impl Into<String>) -> Self {
        Self::EvalError {
            message: message.into(),
        }
    }

    pub fn range_violation(
        attr: impl Into<String>,
        value: impl Into<String>,
        range_desc: impl Into<String>,
    ) -> Self {
        Self::RangeViolation {
            attr: attr.into(),
            value: value.into(),
            range_desc: range_desc.into(),
        }
    }

    pub fn readonly_attribute(type_name: impl Into<String>, attr: impl Into<String>) -> Self {
        Self::ReadonlyAttribute {
            type_name: type_name.into(),
            attr: attr.into(),
        }
    }
}

impl From<mew_pattern::PatternError> for MutationError {
    fn from(e: mew_pattern::PatternError) -> Self {
        Self::PatternError {
            message: e.to_string(),
        }
    }
}
