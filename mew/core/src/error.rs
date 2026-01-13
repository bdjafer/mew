//! Common error types for MEW.

use crate::{EdgeId, EdgeTypeId, NodeId, TypeId};
use thiserror::Error;

/// Errors that can occur during graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    /// Node not found.
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    /// Edge not found.
    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    /// Type not found.
    #[error("Type not found: {0}")]
    TypeNotFound(TypeId),

    /// Edge type not found.
    #[error("Edge type not found: {0}")]
    EdgeTypeNotFound(EdgeTypeId),

    /// Cannot delete node because edges reference it.
    #[error("Cannot delete node {0}: referenced by edges")]
    NodeHasEdges(NodeId),

    /// Cannot delete edge because higher-order edges reference it.
    #[error("Cannot delete edge {0}: referenced by higher-order edges")]
    EdgeHasHigherOrder(EdgeId),

    /// Attribute not found.
    #[error("Attribute not found: {attr} on entity {entity}")]
    AttributeNotFound { entity: String, attr: String },

    /// Type mismatch in attribute.
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type for graph operations.
pub type GraphResult<T> = Result<T, GraphError>;
