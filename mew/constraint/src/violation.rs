//! Constraint violation types.

use mew_core::{EdgeId, NodeId};

/// Severity of a constraint violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity {
    /// Hard constraint - must abort transaction.
    Error,
    /// Soft constraint - warning only.
    Warning,
}

/// A constraint violation.
#[derive(Debug, Clone)]
pub struct Violation {
    /// The constraint that was violated.
    pub constraint_name: String,
    /// The severity of the violation.
    pub severity: ViolationSeverity,
    /// Human-readable message describing the violation.
    pub message: String,
    /// Optional node ID involved in the violation.
    pub node_id: Option<NodeId>,
    /// Optional edge ID involved in the violation.
    pub edge_id: Option<EdgeId>,
}

impl Violation {
    /// Create a new violation.
    pub fn new(
        constraint_name: impl Into<String>,
        severity: ViolationSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            constraint_name: constraint_name.into(),
            severity,
            message: message.into(),
            node_id: None,
            edge_id: None,
        }
    }

    /// Create an error-level violation.
    pub fn error(constraint_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(constraint_name, ViolationSeverity::Error, message)
    }

    /// Create a warning-level violation.
    pub fn warning(constraint_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(constraint_name, ViolationSeverity::Warning, message)
    }

    /// Add a node ID to the violation context.
    pub fn with_node(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    /// Add an edge ID to the violation context.
    pub fn with_edge(mut self, edge_id: EdgeId) -> Self {
        self.edge_id = Some(edge_id);
        self
    }

    /// Check if this is an error-level violation.
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ViolationSeverity::Error)
    }

    /// Check if this is a warning-level violation.
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, ViolationSeverity::Warning)
    }
}

/// Collection of violations.
#[derive(Debug, Clone, Default)]
pub struct Violations {
    violations: Vec<Violation>,
}

impl Violations {
    /// Create a new empty violations collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a violation.
    pub fn push(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// Check if there are any violations.
    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }

    /// Check if there are any error-level violations.
    pub fn has_errors(&self) -> bool {
        self.violations.iter().any(|v| v.is_error())
    }

    /// Check if there are only warnings.
    pub fn has_only_warnings(&self) -> bool {
        !self.violations.is_empty() && !self.has_errors()
    }

    /// Get all violations.
    pub fn all(&self) -> &[Violation] {
        &self.violations
    }

    /// Get error-level violations.
    pub fn errors(&self) -> impl Iterator<Item = &Violation> {
        self.violations.iter().filter(|v| v.is_error())
    }

    /// Get warning-level violations.
    pub fn warnings(&self) -> impl Iterator<Item = &Violation> {
        self.violations.iter().filter(|v| v.is_warning())
    }

    /// Get the number of violations.
    pub fn len(&self) -> usize {
        self.violations.len()
    }

    /// Merge another violations collection.
    pub fn merge(&mut self, other: Violations) {
        self.violations.extend(other.violations);
    }
}

impl IntoIterator for Violations {
    type Item = Violation;
    type IntoIter = std::vec::IntoIter<Violation>;

    fn into_iter(self) -> Self::IntoIter {
        self.violations.into_iter()
    }
}

impl<'a> IntoIterator for &'a Violations {
    type Item = &'a Violation;
    type IntoIter = std::slice::Iter<'a, Violation>;

    fn into_iter(self) -> Self::IntoIter {
        self.violations.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        // GIVEN/WHEN
        let violation = Violation::error("unique_email", "Email already exists");

        // THEN
        assert_eq!(violation.constraint_name, "unique_email");
        assert!(violation.is_error());
        assert!(!violation.is_warning());
    }

    #[test]
    fn test_violations_has_errors() {
        // GIVEN
        let mut violations = Violations::new();
        violations.push(Violation::warning("soft", "Just a warning"));

        // THEN - only warnings
        assert!(!violations.has_errors());
        assert!(violations.has_only_warnings());

        // WHEN - add an error
        violations.push(Violation::error("hard", "Critical error"));

        // THEN
        assert!(violations.has_errors());
        assert!(!violations.has_only_warnings());
    }
}
