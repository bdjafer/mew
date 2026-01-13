//! The Registry - immutable schema lookup.

use crate::{ConstraintDef, EdgeTypeDef, RuleDef, SubtypeIndex, TypeDef};
use mew_core::{EdgeTypeId, TypeId};
use std::collections::HashMap;

/// The Registry provides runtime lookup of schema definitions.
/// It is immutable after construction.
#[derive(Debug)]
pub struct Registry {
    /// Node type definitions by ID.
    types: HashMap<TypeId, TypeDef>,
    /// Node type ID lookup by name.
    type_names: HashMap<String, TypeId>,

    /// Edge type definitions by ID.
    edge_types: HashMap<EdgeTypeId, EdgeTypeDef>,
    /// Edge type ID lookup by name.
    edge_type_names: HashMap<String, EdgeTypeId>,

    /// Constraint definitions.
    constraints: Vec<ConstraintDef>,
    /// Constraints indexed by type ID.
    constraints_by_type: HashMap<TypeId, Vec<usize>>,
    /// Constraints indexed by edge type ID.
    constraints_by_edge_type: HashMap<EdgeTypeId, Vec<usize>>,

    /// Rule definitions (sorted by priority within each type).
    rules: Vec<RuleDef>,
    /// Rules indexed by type ID.
    rules_by_type: HashMap<TypeId, Vec<usize>>,
    /// Rules indexed by edge type ID.
    rules_by_edge_type: HashMap<EdgeTypeId, Vec<usize>>,

    /// Precomputed subtype relationships.
    subtype_index: SubtypeIndex,
}

impl Registry {
    /// Create an empty registry (use RegistryBuilder for construction).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        types: HashMap<TypeId, TypeDef>,
        type_names: HashMap<String, TypeId>,
        edge_types: HashMap<EdgeTypeId, EdgeTypeDef>,
        edge_type_names: HashMap<String, EdgeTypeId>,
        constraints: Vec<ConstraintDef>,
        constraints_by_type: HashMap<TypeId, Vec<usize>>,
        constraints_by_edge_type: HashMap<EdgeTypeId, Vec<usize>>,
        rules: Vec<RuleDef>,
        rules_by_type: HashMap<TypeId, Vec<usize>>,
        rules_by_edge_type: HashMap<EdgeTypeId, Vec<usize>>,
        subtype_index: SubtypeIndex,
    ) -> Self {
        Self {
            types,
            type_names,
            edge_types,
            edge_type_names,
            constraints,
            constraints_by_type,
            constraints_by_edge_type,
            rules,
            rules_by_type,
            rules_by_edge_type,
            subtype_index,
        }
    }

    // ==================== Type Lookups ====================

    /// Get a type definition by name.
    pub fn get_type_by_name(&self, name: &str) -> Option<&TypeDef> {
        self.type_names.get(name).and_then(|id| self.types.get(id))
    }

    /// Get a type definition by ID.
    pub fn get_type(&self, id: TypeId) -> Option<&TypeDef> {
        self.types.get(&id)
    }

    /// Get a type ID by name.
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.type_names.get(name).copied()
    }

    /// Get all type definitions.
    pub fn all_types(&self) -> impl Iterator<Item = &TypeDef> {
        self.types.values()
    }

    /// Get the number of types.
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    // ==================== Edge Type Lookups ====================

    /// Get an edge type definition by name.
    pub fn get_edge_type_by_name(&self, name: &str) -> Option<&EdgeTypeDef> {
        self.edge_type_names
            .get(name)
            .and_then(|id| self.edge_types.get(id))
    }

    /// Get an edge type definition by ID.
    pub fn get_edge_type(&self, id: EdgeTypeId) -> Option<&EdgeTypeDef> {
        self.edge_types.get(&id)
    }

    /// Get an edge type ID by name.
    pub fn get_edge_type_id(&self, name: &str) -> Option<EdgeTypeId> {
        self.edge_type_names.get(name).copied()
    }

    /// Get all edge type definitions.
    pub fn all_edge_types(&self) -> impl Iterator<Item = &EdgeTypeDef> {
        self.edge_types.values()
    }

    /// Get the number of edge types.
    pub fn edge_type_count(&self) -> usize {
        self.edge_types.len()
    }

    /// Get an attribute definition from a type, including inherited attributes.
    pub fn get_type_attr(&self, type_id: TypeId, attr_name: &str) -> Option<&crate::AttrDef> {
        // Check own attributes first
        if let Some(type_def) = self.types.get(&type_id) {
            if let Some(attr) = type_def.get_attr(attr_name) {
                return Some(attr);
            }
            // Check parent types
            for &parent_id in &type_def.parent_ids {
                if let Some(attr) = self.get_type_attr(parent_id, attr_name) {
                    return Some(attr);
                }
            }
        }
        None
    }

    /// Check if a type has an attribute (including inherited).
    pub fn type_has_attr(&self, type_id: TypeId, attr_name: &str) -> bool {
        self.get_type_attr(type_id, attr_name).is_some()
    }

    /// Get all attributes for a type including inherited ones.
    pub fn get_all_type_attrs(&self, type_id: TypeId) -> Vec<&crate::AttrDef> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        self.collect_type_attrs(type_id, &mut result, &mut seen);
        result
    }

    /// Helper to collect attributes from type and parents.
    fn collect_type_attrs<'a>(
        &'a self,
        type_id: TypeId,
        result: &mut Vec<&'a crate::AttrDef>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        if let Some(type_def) = self.types.get(&type_id) {
            // First collect from parents
            for &parent_id in &type_def.parent_ids {
                self.collect_type_attrs(parent_id, result, seen);
            }
            // Then add own attrs (may override parent attrs)
            for (name, attr) in &type_def.attributes {
                if !seen.contains(name) {
                    seen.insert(name.clone());
                    result.push(attr);
                }
            }
        }
    }

    // ==================== Subtype Queries ====================

    /// Check if `sub` is a subtype of `super_type`.
    pub fn is_subtype(&self, sub: TypeId, super_type: TypeId) -> bool {
        self.subtype_index.is_subtype(sub, super_type)
    }

    /// Get all subtypes of a type (not including the type itself).
    pub fn get_subtypes(&self, type_id: TypeId) -> impl Iterator<Item = TypeId> + '_ {
        self.subtype_index.get_subtypes(type_id)
    }

    /// Get all supertypes of a type (not including the type itself).
    pub fn get_supertypes(&self, type_id: TypeId) -> impl Iterator<Item = TypeId> + '_ {
        self.subtype_index.get_supertypes(type_id)
    }

    // ==================== Constraint Lookups ====================

    /// Get all constraints for a type.
    pub fn get_constraints_for_type(&self, type_id: TypeId) -> Vec<&ConstraintDef> {
        self.constraints_by_type
            .get(&type_id)
            .map(|indices| indices.iter().map(|&i| &self.constraints[i]).collect())
            .unwrap_or_default()
    }

    /// Get all constraints for an edge type.
    pub fn get_constraints_for_edge_type(&self, edge_type_id: EdgeTypeId) -> Vec<&ConstraintDef> {
        self.constraints_by_edge_type
            .get(&edge_type_id)
            .map(|indices| indices.iter().map(|&i| &self.constraints[i]).collect())
            .unwrap_or_default()
    }

    /// Get all deferred constraints.
    pub fn get_deferred_constraints(&self) -> Vec<&ConstraintDef> {
        self.constraints.iter().filter(|c| c.deferred).collect()
    }

    /// Get all constraints.
    pub fn all_constraints(&self) -> impl Iterator<Item = &ConstraintDef> {
        self.constraints.iter()
    }

    // ==================== Rule Lookups ====================

    /// Get all rules for a type, sorted by priority (descending).
    pub fn get_rules_for_type(&self, type_id: TypeId) -> Vec<&RuleDef> {
        self.rules_by_type
            .get(&type_id)
            .map(|indices| indices.iter().map(|&i| &self.rules[i]).collect())
            .unwrap_or_default()
    }

    /// Get all rules for an edge type, sorted by priority (descending).
    pub fn get_rules_for_edge_type(&self, edge_type_id: EdgeTypeId) -> Vec<&RuleDef> {
        self.rules_by_edge_type
            .get(&edge_type_id)
            .map(|indices| indices.iter().map(|&i| &self.rules[i]).collect())
            .unwrap_or_default()
    }

    /// Get all auto rules for a type.
    pub fn get_auto_rules_for_type(&self, type_id: TypeId) -> Vec<&RuleDef> {
        self.get_rules_for_type(type_id)
            .into_iter()
            .filter(|r| r.auto)
            .collect()
    }

    /// Get all rules.
    pub fn all_rules(&self) -> impl Iterator<Item = &RuleDef> {
        self.rules.iter()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            types: HashMap::new(),
            type_names: HashMap::new(),
            edge_types: HashMap::new(),
            edge_type_names: HashMap::new(),
            constraints: Vec::new(),
            constraints_by_type: HashMap::new(),
            constraints_by_edge_type: HashMap::new(),
            rules: Vec::new(),
            rules_by_type: HashMap::new(),
            rules_by_edge_type: HashMap::new(),
            subtype_index: SubtypeIndex::new(),
        }
    }
}
