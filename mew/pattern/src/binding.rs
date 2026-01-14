//! Variable bindings for pattern matching.

use mew_core::{EdgeId, NodeId, Value};
use std::collections::HashMap;

/// A binding value (node ref, edge ref, or value).
#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    /// Reference to a node.
    Node(NodeId),
    /// Reference to an edge.
    Edge(EdgeId),
    /// A scalar value.
    Value(Value),
    /// Null binding (for unmatched OPTIONAL MATCH variables).
    Null,
}

impl Binding {
    /// Get as node ID if this is a node binding.
    pub fn as_node(&self) -> Option<NodeId> {
        match self {
            Binding::Node(id) => Some(*id),
            _ => None,
        }
    }

    /// Get as edge ID if this is an edge binding.
    pub fn as_edge(&self) -> Option<EdgeId> {
        match self {
            Binding::Edge(id) => Some(*id),
            _ => None,
        }
    }

    /// Get as value if this is a value binding.
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            Binding::Value(v) => Some(v),
            _ => None,
        }
    }

    /// Convert to a Value.
    pub fn to_value(&self) -> Value {
        match self {
            Binding::Node(id) => Value::NodeRef(*id),
            Binding::Edge(id) => Value::EdgeRef(*id),
            Binding::Value(v) => v.clone(),
            Binding::Null => Value::Null,
        }
    }
}

impl From<NodeId> for Binding {
    fn from(id: NodeId) -> Self {
        Binding::Node(id)
    }
}

impl From<EdgeId> for Binding {
    fn from(id: EdgeId) -> Self {
        Binding::Edge(id)
    }
}

impl From<Value> for Binding {
    fn from(v: Value) -> Self {
        Binding::Value(v)
    }
}

/// A set of variable bindings.
#[derive(Debug, Clone, Default)]
pub struct Bindings {
    map: HashMap<String, Binding>,
}

impl Bindings {
    /// Create new empty bindings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create bindings with a single entry.
    pub fn with(name: impl Into<String>, binding: impl Into<Binding>) -> Self {
        let mut bindings = Self::new();
        bindings.insert(name, binding);
        bindings
    }

    /// Insert a binding.
    pub fn insert(&mut self, name: impl Into<String>, binding: impl Into<Binding>) {
        self.map.insert(name.into(), binding.into());
    }

    /// Get a binding by name.
    pub fn get(&self, name: &str) -> Option<&Binding> {
        self.map.get(name)
    }

    /// Check if a variable is bound.
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Get all variable names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(|s| s.as_str())
    }

    /// Clone with an additional binding.
    pub fn extend_with(&self, name: impl Into<String>, binding: impl Into<Binding>) -> Self {
        let mut new_bindings = self.clone();
        new_bindings.insert(name, binding);
        new_bindings
    }

    /// Merge with another set of bindings.
    pub fn merge(&mut self, other: &Bindings) {
        for (name, binding) in &other.map {
            self.map.insert(name.clone(), binding.clone());
        }
    }

    /// Get the number of bindings.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterate over bindings.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Binding)> {
        self.map.iter().map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_node() {
        let id = NodeId::new(1);
        let binding = Binding::from(id);
        assert_eq!(binding.as_node(), Some(id));
        assert_eq!(binding.as_edge(), None);
    }

    #[test]
    fn test_bindings_insert_and_get() {
        let mut bindings = Bindings::new();
        bindings.insert("x", NodeId::new(1));
        bindings.insert("y", Value::Int(42));

        assert!(bindings.contains("x"));
        assert!(bindings.contains("y"));
        assert!(!bindings.contains("z"));

        assert_eq!(bindings.get("x").unwrap().as_node(), Some(NodeId::new(1)));
        assert_eq!(bindings.get("y").unwrap().as_value(), Some(&Value::Int(42)));
    }

    #[test]
    fn test_bindings_extend_with() {
        let bindings = Bindings::with("x", NodeId::new(1));
        let extended = bindings.extend_with("y", NodeId::new(2));

        assert!(bindings.contains("x"));
        assert!(!bindings.contains("y"));
        assert!(extended.contains("x"));
        assert!(extended.contains("y"));
    }
}
