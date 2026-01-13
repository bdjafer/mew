//! Type system for the analyzer.

use mew_core::{EdgeTypeId, TypeId};

/// The type of a value in MEW.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Null type.
    Null,
    /// Boolean type.
    Bool,
    /// 64-bit signed integer.
    Int,
    /// 64-bit floating point.
    Float,
    /// UTF-8 string.
    String,
    /// Timestamp (milliseconds since epoch).
    Timestamp,
    /// Duration (milliseconds).
    Duration,
    /// Reference to a node of a specific type.
    NodeRef(TypeId),
    /// Reference to an edge of a specific type.
    EdgeRef(EdgeTypeId),
    /// Any node reference.
    AnyNodeRef,
    /// Any edge reference.
    AnyEdgeRef,
    /// Any type (used for unknown/unresolved).
    Any,
    /// Unknown type (for errors).
    Unknown,
}

impl Type {
    /// Check if this type is numeric.
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    /// Check if this type is a reference.
    pub fn is_ref(&self) -> bool {
        matches!(
            self,
            Type::NodeRef(_) | Type::EdgeRef(_) | Type::AnyNodeRef | Type::AnyEdgeRef
        )
    }

    /// Check if this type can be compared for equality with another.
    pub fn can_eq(&self, other: &Type) -> bool {
        match (self, other) {
            // Same types can always be compared
            (a, b) if a == b => true,
            // Any matches anything
            (Type::Any, _) | (_, Type::Any) => true,
            // Null can be compared with anything
            (Type::Null, _) | (_, Type::Null) => true,
            // Int and Float can be compared
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            // Node refs can be compared
            (Type::NodeRef(_), Type::NodeRef(_)) => true,
            (Type::NodeRef(_), Type::AnyNodeRef) | (Type::AnyNodeRef, Type::NodeRef(_)) => true,
            // Edge refs can be compared
            (Type::EdgeRef(_), Type::EdgeRef(_)) => true,
            (Type::EdgeRef(_), Type::AnyEdgeRef) | (Type::AnyEdgeRef, Type::EdgeRef(_)) => true,
            _ => false,
        }
    }

    /// Check if this type can be ordered (compared with <, >, etc).
    pub fn can_order(&self, other: &Type) -> bool {
        matches!(
            (self, other),
            (Type::Int, Type::Int)
                | (Type::Float, Type::Float)
                | (Type::Int, Type::Float)
                | (Type::Float, Type::Int)
                | (Type::String, Type::String)
                | (Type::Timestamp, Type::Timestamp)
                | (Type::Duration, Type::Duration)
                | (Type::Any, _)
                | (_, Type::Any)
        )
    }

    /// Get the result type of a binary operation.
    pub fn binary_result(&self, op: BinaryOpType, other: &Type) -> Option<Type> {
        match op {
            BinaryOpType::Add | BinaryOpType::Sub | BinaryOpType::Mul | BinaryOpType::Div => {
                match (self, other) {
                    (Type::Int, Type::Int) => Some(Type::Int),
                    (Type::Float, Type::Float) => Some(Type::Float),
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) => Some(Type::Float),
                    (Type::Any, _) | (_, Type::Any) => Some(Type::Any),
                    _ => None,
                }
            }
            BinaryOpType::Mod => match (self, other) {
                (Type::Int, Type::Int) => Some(Type::Int),
                (Type::Any, _) | (_, Type::Any) => Some(Type::Any),
                _ => None,
            },
            BinaryOpType::Concat => match (self, other) {
                (Type::String, Type::String) => Some(Type::String),
                (Type::Any, _) | (_, Type::Any) => Some(Type::Any),
                _ => None,
            },
            BinaryOpType::Eq
            | BinaryOpType::NotEq
            | BinaryOpType::Lt
            | BinaryOpType::LtEq
            | BinaryOpType::Gt
            | BinaryOpType::GtEq => Some(Type::Bool),
            BinaryOpType::And | BinaryOpType::Or => match (self, other) {
                (Type::Bool, Type::Bool) => Some(Type::Bool),
                (Type::Any, _) | (_, Type::Any) => Some(Type::Bool),
                _ => None,
            },
            // Null coalesce returns type of left operand (or right if left is null)
            // Since we allow nullable types, accept any type combination
            BinaryOpType::NullCoalesce => {
                match (self, other) {
                    // Same types
                    (t, u) if t == u => Some(t.clone()),
                    // If one is Any, use the other type
                    (Type::Any, t) | (t, Type::Any) => Some(t.clone()),
                    // Different types - use Any as result
                    _ => Some(Type::Any),
                }
            }
        }
    }

    /// Get the result type of a unary operation.
    pub fn unary_result(&self, op: UnaryOpType) -> Option<Type> {
        match op {
            UnaryOpType::Neg => match self {
                Type::Int => Some(Type::Int),
                Type::Float => Some(Type::Float),
                Type::Any => Some(Type::Any),
                _ => None,
            },
            UnaryOpType::Not => match self {
                Type::Bool => Some(Type::Bool),
                Type::Any => Some(Type::Bool),
                _ => None,
            },
        }
    }

    /// Get the name of this type for error messages.
    pub fn name(&self) -> &'static str {
        match self {
            Type::Null => "Null",
            Type::Bool => "Bool",
            Type::Int => "Int",
            Type::Float => "Float",
            Type::String => "String",
            Type::Timestamp => "Timestamp",
            Type::Duration => "Duration",
            Type::NodeRef(_) => "NodeRef",
            Type::EdgeRef(_) => "EdgeRef",
            Type::AnyNodeRef => "NodeRef",
            Type::AnyEdgeRef => "EdgeRef",
            Type::Any => "Any",
            Type::Unknown => "Unknown",
        }
    }
}

/// Type for binary operators (for type checking).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpType {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    NullCoalesce,
}

/// Type for unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpType {
    Neg,
    Not,
}

/// Convert parser BinaryOp to analyzer BinaryOpType.
impl From<mew_parser::BinaryOp> for BinaryOpType {
    fn from(op: mew_parser::BinaryOp) -> Self {
        match op {
            mew_parser::BinaryOp::Add => BinaryOpType::Add,
            mew_parser::BinaryOp::Sub => BinaryOpType::Sub,
            mew_parser::BinaryOp::Mul => BinaryOpType::Mul,
            mew_parser::BinaryOp::Div => BinaryOpType::Div,
            mew_parser::BinaryOp::Mod => BinaryOpType::Mod,
            mew_parser::BinaryOp::Concat => BinaryOpType::Concat,
            mew_parser::BinaryOp::Eq => BinaryOpType::Eq,
            mew_parser::BinaryOp::NotEq => BinaryOpType::NotEq,
            mew_parser::BinaryOp::Lt => BinaryOpType::Lt,
            mew_parser::BinaryOp::LtEq => BinaryOpType::LtEq,
            mew_parser::BinaryOp::Gt => BinaryOpType::Gt,
            mew_parser::BinaryOp::GtEq => BinaryOpType::GtEq,
            mew_parser::BinaryOp::And => BinaryOpType::And,
            mew_parser::BinaryOp::Or => BinaryOpType::Or,
            mew_parser::BinaryOp::NullCoalesce => BinaryOpType::NullCoalesce,
        }
    }
}

/// Convert parser UnaryOp to analyzer UnaryOpType.
impl From<mew_parser::UnaryOp> for UnaryOpType {
    fn from(op: mew_parser::UnaryOp) -> Self {
        match op {
            mew_parser::UnaryOp::Neg => UnaryOpType::Neg,
            mew_parser::UnaryOp::Not => UnaryOpType::Not,
        }
    }
}
