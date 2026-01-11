//! Value types for MEW attributes.
//!
//! Values are the atomic data stored in node and edge attributes.
//! MEW supports scalar types (String, Int, Float, Bool, Timestamp, Duration)
//! and reference types (NodeRef, EdgeRef).

use crate::{EdgeId, NodeId};
use std::fmt;

/// A value that can be stored in an attribute.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null/missing value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit floating point.
    Float(f64),
    /// UTF-8 string.
    String(String),
    /// Timestamp as milliseconds since Unix epoch.
    Timestamp(i64),
    /// Duration in milliseconds.
    Duration(i64),
    /// Reference to a node.
    NodeRef(NodeId),
    /// Reference to an edge.
    EdgeRef(EdgeId),
}

impl Value {
    /// Returns true if this is a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if this is a boolean value.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Returns true if this is an integer value.
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    /// Returns true if this is a float value.
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Returns true if this is a string value.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns true if this is a timestamp value.
    pub fn is_timestamp(&self) -> bool {
        matches!(self, Value::Timestamp(_))
    }

    /// Returns true if this is a duration value.
    pub fn is_duration(&self) -> bool {
        matches!(self, Value::Duration(_))
    }

    /// Returns true if this is a node reference.
    pub fn is_node_ref(&self) -> bool {
        matches!(self, Value::NodeRef(_))
    }

    /// Returns true if this is an edge reference.
    pub fn is_edge_ref(&self) -> bool {
        matches!(self, Value::EdgeRef(_))
    }

    /// Get as boolean if this is a Bool value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as integer if this is an Int value.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float if this is a Float value.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get as string reference if this is a String value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as timestamp if this is a Timestamp value.
    pub fn as_timestamp(&self) -> Option<i64> {
        match self {
            Value::Timestamp(t) => Some(*t),
            _ => None,
        }
    }

    /// Get as duration if this is a Duration value.
    pub fn as_duration(&self) -> Option<i64> {
        match self {
            Value::Duration(d) => Some(*d),
            _ => None,
        }
    }

    /// Get as node ID if this is a NodeRef value.
    pub fn as_node_ref(&self) -> Option<NodeId> {
        match self {
            Value::NodeRef(id) => Some(*id),
            _ => None,
        }
    }

    /// Get as edge ID if this is an EdgeRef value.
    pub fn as_edge_ref(&self) -> Option<EdgeId> {
        match self {
            Value::EdgeRef(id) => Some(*id),
            _ => None,
        }
    }

    /// Returns the type name of this value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Timestamp(_) => "Timestamp",
            Value::Duration(_) => "Duration",
            Value::NodeRef(_) => "NodeRef",
            Value::EdgeRef(_) => "EdgeRef",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Timestamp(t) => write!(f, "ts:{}", t),
            Value::Duration(d) => write!(f, "dur:{}", d),
            Value::NodeRef(id) => write!(f, "#{}", id),
            Value::EdgeRef(id) => write!(f, "#{}", id),
        }
    }
}

// Convenient From implementations
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<NodeId> for Value {
    fn from(id: NodeId) -> Self {
        Value::NodeRef(id)
    }
}

impl From<EdgeId> for Value {
    fn from(id: EdgeId) -> Self {
        Value::EdgeRef(id)
    }
}

/// Type alias for attribute storage.
pub type Attributes = std::collections::HashMap<String, Value>;

/// Helper macro to create attribute maps.
#[macro_export]
macro_rules! attrs {
    () => {
        std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key.to_string(), $crate::Value::from($value));
            )+
            map
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_checks() {
        assert!(Value::Null.is_null());
        assert!(Value::Bool(true).is_bool());
        assert!(Value::Int(42).is_int());
        assert!(Value::Float(3.14).is_float());
        assert!(Value::String("hello".into()).is_string());
        assert!(Value::Timestamp(1234567890).is_timestamp());
        assert!(Value::Duration(1000).is_duration());
        assert!(Value::NodeRef(NodeId::new(1)).is_node_ref());
        assert!(Value::EdgeRef(EdgeId::new(1)).is_edge_ref());
    }

    #[test]
    fn test_value_accessors() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::String("hello".into()).as_str(), Some("hello"));
    }

    #[test]
    fn test_attrs_macro() {
        let empty: Attributes = attrs!();
        assert!(empty.is_empty());

        let attrs = attrs! {
            "name" => "Alice",
            "age" => 30i64,
            "active" => true,
        };
        assert_eq!(attrs.get("name"), Some(&Value::String("Alice".into())));
        assert_eq!(attrs.get("age"), Some(&Value::Int(30)));
        assert_eq!(attrs.get("active"), Some(&Value::Bool(true)));
    }
}
