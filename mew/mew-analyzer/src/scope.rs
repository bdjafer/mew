//! Variable scope management.

use crate::Type;
use std::collections::HashMap;

/// A variable binding in scope.
#[derive(Debug, Clone)]
pub struct VarBinding {
    /// The name of the variable.
    pub name: String,
    /// The resolved type.
    pub ty: Type,
    /// Whether this variable is mutable (can be assigned to).
    pub mutable: bool,
}

impl VarBinding {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
            mutable: false,
        }
    }

    pub fn mutable(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
            mutable: true,
        }
    }
}

/// A scope for variable bindings.
/// Supports nested scopes for match blocks, etc.
#[derive(Debug)]
pub struct Scope {
    /// Stack of scope frames. Each frame is a mapping from name to binding.
    frames: Vec<HashMap<String, VarBinding>>,
}

impl Scope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
        }
    }

    /// Push a new scope frame.
    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Pop the current scope frame.
    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Define a variable in the current scope.
    /// Returns `false` if the variable is already defined in the current frame.
    pub fn define(&mut self, binding: VarBinding) -> bool {
        let name = binding.name.clone();
        if let Some(frame) = self.frames.last_mut() {
            if frame.contains_key(&name) {
                return false;
            }
            frame.insert(name, binding);
            true
        } else {
            false
        }
    }

    /// Look up a variable by name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&VarBinding> {
        for frame in self.frames.iter().rev() {
            if let Some(binding) = frame.get(name) {
                return Some(binding);
            }
        }
        None
    }

    /// Check if a variable is defined in any scope.
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Check if a variable is defined in the current (innermost) scope frame.
    pub fn is_defined_in_current(&self, name: &str) -> bool {
        self.frames
            .last()
            .map(|frame| frame.contains_key(name))
            .unwrap_or(false)
    }

    /// Get the type of a variable if defined.
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.lookup(name).map(|b| &b.ty)
    }

    /// Get all variable names in the current scope (including outer scopes).
    pub fn all_names(&self) -> Vec<&str> {
        let mut names = Vec::new();
        for frame in &self.frames {
            for name in frame.keys() {
                names.push(name.as_str());
            }
        }
        names
    }

    /// Get the current depth (number of nested scopes).
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_define_and_lookup() {
        // GIVEN
        let mut scope = Scope::new();

        // WHEN
        scope.define(VarBinding::new("x", Type::Int));

        // THEN
        assert!(scope.is_defined("x"));
        assert_eq!(scope.get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_scope_nested() {
        // GIVEN
        let mut scope = Scope::new();
        scope.define(VarBinding::new("x", Type::Int));

        // WHEN - push nested scope and define same name
        scope.push();
        scope.define(VarBinding::new("x", Type::String));
        scope.define(VarBinding::new("y", Type::Bool));

        // THEN - inner scope shadows outer
        assert_eq!(scope.get_type("x"), Some(&Type::String));
        assert_eq!(scope.get_type("y"), Some(&Type::Bool));

        // WHEN - pop scope
        scope.pop();

        // THEN - outer scope restored
        assert_eq!(scope.get_type("x"), Some(&Type::Int));
        assert!(!scope.is_defined("y"));
    }

    #[test]
    fn test_scope_duplicate_in_same_frame() {
        // GIVEN
        let mut scope = Scope::new();
        scope.define(VarBinding::new("x", Type::Int));

        // WHEN
        let result = scope.define(VarBinding::new("x", Type::String));

        // THEN
        assert!(!result);
        assert_eq!(scope.get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_scope_is_defined_in_current() {
        // GIVEN
        let mut scope = Scope::new();
        scope.define(VarBinding::new("x", Type::Int));
        scope.push();

        // THEN
        assert!(!scope.is_defined_in_current("x"));
        assert!(scope.is_defined("x"));
    }
}
