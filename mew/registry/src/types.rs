//! Schema definition types.

use mew_core::{EdgeTypeId, TypeId, Value};
use std::collections::{HashMap, HashSet};

/// Attribute definition within a type.
#[derive(Debug, Clone)]
pub struct AttrDef {
    /// Attribute name.
    pub name: String,
    /// Type name (String, Int, Float, Bool, etc.).
    pub type_name: String,
    /// Whether this attribute is required.
    pub required: bool,
    /// Whether this attribute can be null.
    pub nullable: bool,
    /// Whether this attribute must be unique across instances.
    pub unique: bool,
    /// Default value if not provided.
    pub default: Option<Value>,
    /// Minimum value constraint (for Int, Float).
    pub min: Option<Value>,
    /// Maximum value constraint (for Int, Float).
    pub max: Option<Value>,
    /// Format constraint (e.g., "slug", "email", "url", "uuid").
    pub format: Option<String>,
    /// Match pattern constraint (regex).
    pub match_pattern: Option<String>,
    /// Allowed values (in: [...] constraint).
    pub allowed_values: Option<Vec<Value>>,
    /// Minimum string length constraint.
    pub length_min: Option<i64>,
    /// Maximum string length constraint.
    pub length_max: Option<i64>,
}

impl AttrDef {
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            required: false,
            nullable: false,
            unique: false,
            default: None,
            min: None,
            max: None,
            format: None,
            match_pattern: None,
            allowed_values: None,
            length_min: None,
            length_max: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn with_range(mut self, min: Option<Value>, max: Option<Value>) -> Self {
        // Merge rather than replace: only update if Some is provided
        if min.is_some() {
            self.min = min;
        }
        if max.is_some() {
            self.max = max;
        }
        self
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn with_match_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.match_pattern = Some(pattern.into());
        self
    }

    pub fn with_allowed_values(mut self, values: Vec<Value>) -> Self {
        self.allowed_values = Some(values);
        self
    }

    pub fn with_length(mut self, min: i64, max: i64) -> Self {
        self.length_min = Some(min);
        self.length_max = Some(max);
        self
    }
}

/// Node type definition.
#[derive(Debug, Clone)]
pub struct TypeDef {
    /// Unique identifier.
    pub id: TypeId,
    /// Type name.
    pub name: String,
    /// Parent type IDs (for inheritance).
    pub parent_ids: Vec<TypeId>,
    /// Attribute definitions.
    pub attributes: HashMap<String, AttrDef>,
    /// Whether this type is abstract (cannot be instantiated directly).
    pub is_abstract: bool,
    /// Whether this type is sealed (cannot be extended).
    pub is_sealed: bool,
}

impl TypeDef {
    pub fn new(id: TypeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            parent_ids: Vec::new(),
            attributes: HashMap::new(),
            is_abstract: false,
            is_sealed: false,
        }
    }

    /// Get an attribute definition by name.
    pub fn get_attr(&self, name: &str) -> Option<&AttrDef> {
        self.attributes.get(name)
    }

    /// Check if this type has an attribute.
    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get all attribute names.
    pub fn attr_names(&self) -> impl Iterator<Item = &str> {
        self.attributes.keys().map(|s| s.as_str())
    }
}

/// Edge type parameter.
#[derive(Debug, Clone)]
pub struct EdgeParam {
    /// Parameter name (e.g., "from", "to").
    pub name: String,
    /// Type constraint (e.g., "Person", "Task", or "any").
    pub type_constraint: String,
}

/// Edge type definition.
#[derive(Debug, Clone)]
pub struct EdgeTypeDef {
    /// Unique identifier.
    pub id: EdgeTypeId,
    /// Edge type name.
    pub name: String,
    /// Parameters (ordered targets with type constraints).
    pub params: Vec<EdgeParam>,
    /// Attribute definitions.
    pub attributes: HashMap<String, AttrDef>,
    /// Whether this edge is symmetric (order of targets doesn't matter).
    pub symmetric: bool,
    /// On-kill behavior for each parameter.
    pub on_kill: Vec<OnKillAction>,
    /// Whether this edge must be acyclic.
    pub acyclic: bool,
    /// Whether this edge must be unique (no duplicate edges between same targets).
    pub unique: bool,
}

impl EdgeTypeDef {
    pub fn new(id: EdgeTypeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            params: Vec::new(),
            attributes: HashMap::new(),
            symmetric: false,
            on_kill: Vec::new(),
            acyclic: false,
            unique: false,
        }
    }

    /// Get the arity (number of parameters).
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Get an attribute definition by name.
    pub fn get_attr(&self, name: &str) -> Option<&AttrDef> {
        self.attributes.get(name)
    }
}

/// Action to take when a referenced node is killed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnKillAction {
    /// Delete the edge (default).
    #[default]
    Delete,
    /// Delete the edge and cascade to other targets.
    Cascade,
    /// Prevent deletion of the node.
    Restrict,
    /// Set the target to null (only for optional edges).
    SetNull,
}

/// Constraint definition.
#[derive(Debug, Clone)]
pub struct ConstraintDef {
    /// Unique identifier.
    pub id: u32,
    /// Constraint name.
    pub name: String,
    /// Type ID this constraint applies to (if any).
    pub type_id: Option<TypeId>,
    /// Edge type ID this constraint applies to (if any).
    pub edge_type_id: Option<EdgeTypeId>,
    /// Whether this is a hard constraint (must be satisfied).
    pub hard: bool,
    /// Whether this constraint is deferred until commit.
    pub deferred: bool,
    /// Condition expression (serialized or stored somehow).
    /// For now, we store a string representation.
    pub condition: String,
}

impl ConstraintDef {
    pub fn new(id: u32, name: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            type_id: None,
            edge_type_id: None,
            hard: true,
            deferred: false,
            condition: condition.into(),
        }
    }

    pub fn for_type(mut self, type_id: TypeId) -> Self {
        self.type_id = Some(type_id);
        self
    }

    pub fn for_edge_type(mut self, edge_type_id: EdgeTypeId) -> Self {
        self.edge_type_id = Some(edge_type_id);
        self
    }

    pub fn soft(mut self) -> Self {
        self.hard = false;
        self
    }

    pub fn deferred(mut self) -> Self {
        self.deferred = true;
        self
    }
}

/// Rule definition.
#[derive(Debug, Clone)]
pub struct RuleDef {
    /// Unique identifier.
    pub id: u32,
    /// Rule name.
    pub name: String,
    /// Type ID this rule applies to (if any).
    pub type_id: Option<TypeId>,
    /// Edge type ID this rule applies to (if any).
    pub edge_type_id: Option<EdgeTypeId>,
    /// Execution priority (higher = runs first).
    pub priority: i32,
    /// Whether this rule fires automatically.
    pub auto: bool,
    /// Production (action to take) - stored as string for now.
    pub production: String,
}

impl RuleDef {
    pub fn new(id: u32, name: impl Into<String>, production: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            type_id: None,
            edge_type_id: None,
            priority: 0,
            auto: false,
            production: production.into(),
        }
    }

    pub fn for_type(mut self, type_id: TypeId) -> Self {
        self.type_id = Some(type_id);
        self
    }

    pub fn for_edge_type(mut self, edge_type_id: EdgeTypeId) -> Self {
        self.edge_type_id = Some(edge_type_id);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn auto(mut self) -> Self {
        self.auto = true;
        self
    }
}

/// Precomputed subtype relationships.
#[derive(Debug, Default)]
pub struct SubtypeIndex {
    /// For each type, the set of all its subtypes (transitive).
    subtypes: HashMap<TypeId, HashSet<TypeId>>,
    /// For each type, the set of all its supertypes (transitive).
    supertypes: HashMap<TypeId, HashSet<TypeId>>,
}

impl SubtypeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the subtype index from type definitions.
    pub fn build(types: &HashMap<TypeId, TypeDef>) -> Self {
        let mut index = Self::new();

        // Initialize empty sets for all types
        for &type_id in types.keys() {
            index.subtypes.insert(type_id, HashSet::new());
            index.supertypes.insert(type_id, HashSet::new());
        }

        // Build supertype relationships (direct parents first)
        for (type_id, type_def) in types {
            for &parent_id in &type_def.parent_ids {
                // type_id is a subtype of parent_id
                if let Some(parent_subtypes) = index.subtypes.get_mut(&parent_id) {
                    parent_subtypes.insert(*type_id);
                }
                if let Some(type_supertypes) = index.supertypes.get_mut(type_id) {
                    type_supertypes.insert(parent_id);
                }
            }
        }

        // Transitively close the relationships
        let type_ids: Vec<TypeId> = types.keys().copied().collect();

        // Keep iterating until no changes
        let mut changed = true;
        while changed {
            changed = false;
            for &type_id in &type_ids {
                // Get supertypes of this type
                let supertypes: Vec<TypeId> = index
                    .supertypes
                    .get(&type_id)
                    .map(|s| s.iter().copied().collect())
                    .unwrap_or_default();

                // For each supertype, add its supertypes to our supertypes
                for super_id in supertypes {
                    let transitive: Vec<TypeId> = index
                        .supertypes
                        .get(&super_id)
                        .map(|s| s.iter().copied().collect())
                        .unwrap_or_default();

                    for trans_id in transitive {
                        if let Some(set) = index.supertypes.get_mut(&type_id) {
                            if set.insert(trans_id) {
                                changed = true;
                            }
                        }
                        // Also update subtypes
                        if let Some(set) = index.subtypes.get_mut(&trans_id) {
                            set.insert(type_id);
                        }
                    }
                }
            }
        }

        index
    }

    /// Check if `sub` is a subtype of `super_type`.
    pub fn is_subtype(&self, sub: TypeId, super_type: TypeId) -> bool {
        if sub == super_type {
            return true;
        }
        self.supertypes
            .get(&sub)
            .map(|set| set.contains(&super_type))
            .unwrap_or(false)
    }

    /// Get all subtypes of a type (not including the type itself).
    pub fn get_subtypes(&self, type_id: TypeId) -> impl Iterator<Item = TypeId> + '_ {
        self.subtypes
            .get(&type_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Get all supertypes of a type (not including the type itself).
    pub fn get_supertypes(&self, type_id: TypeId) -> impl Iterator<Item = TypeId> + '_ {
        self.supertypes
            .get(&type_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }
}
