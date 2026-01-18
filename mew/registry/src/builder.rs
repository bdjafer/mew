//! RegistryBuilder for constructing an immutable Registry.

use crate::{
    AttrDef, Cardinality, ConstraintDef, EdgeParam, EdgeTypeDef, OnKillAction, Registry, RuleDef,
    SubtypeIndex, TypeDef,
};
use mew_core::{EdgeTypeId, TypeId};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during registry construction.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Duplicate type name: {0}")]
    DuplicateTypeName(String),

    #[error("Duplicate edge type name: {0}")]
    DuplicateEdgeTypeName(String),

    #[error("Unknown parent type: {0}")]
    UnknownParentType(String),

    #[error("Inheritance cycle detected involving type: {0}")]
    InheritanceCycle(String),

    #[error("Unknown type in constraint: {0}")]
    UnknownTypeInConstraint(String),

    #[error("Unknown edge type in constraint: {0}")]
    UnknownEdgeTypeInConstraint(String),
}

/// Builder for constructing an immutable Registry.
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    /// Next type ID to allocate.
    next_type_id: u32,
    /// Next edge type ID to allocate.
    next_edge_type_id: u32,
    /// Next constraint ID to allocate.
    next_constraint_id: u32,
    /// Next rule ID to allocate.
    next_rule_id: u32,

    /// Types being built.
    types: HashMap<TypeId, TypeDef>,
    /// Type name to ID mapping.
    type_names: HashMap<String, TypeId>,

    /// Edge types being built.
    edge_types: HashMap<EdgeTypeId, EdgeTypeDef>,
    /// Edge type name to ID mapping.
    edge_type_names: HashMap<String, EdgeTypeId>,

    /// Constraints being built.
    constraints: Vec<ConstraintDef>,

    /// Rules being built.
    rules: Vec<RuleDef>,
}

impl RegistryBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a type definition.
    pub fn add_type(&mut self, name: impl Into<String>) -> TypeBuilder<'_> {
        let name = name.into();
        let id = TypeId::new(self.next_type_id);
        self.next_type_id += 1;

        TypeBuilder {
            builder: self,
            id,
            name,
            parent_names: Vec::new(),
            attributes: HashMap::new(),
            is_abstract: false,
            is_sealed: false,
        }
    }

    /// Add an edge type definition.
    pub fn add_edge_type(&mut self, name: impl Into<String>) -> EdgeTypeBuilder<'_> {
        let name = name.into();
        let id = EdgeTypeId::new(self.next_edge_type_id);
        self.next_edge_type_id += 1;

        EdgeTypeBuilder {
            builder: self,
            id,
            name,
            params: Vec::new(),
            attributes: HashMap::new(),
            symmetric: false,
            on_kill: Vec::new(),
            acyclic: false,
            unique: false,
        }
    }

    /// Add a constraint definition.
    pub fn add_constraint(
        &mut self,
        name: impl Into<String>,
        condition: impl Into<String>,
    ) -> ConstraintBuilder<'_> {
        let name = name.into();
        let id = self.next_constraint_id;
        self.next_constraint_id += 1;

        ConstraintBuilder {
            builder: self,
            id,
            name,
            type_name: None,
            edge_type_name: None,
            hard: true,
            deferred: false,
            condition: condition.into(),
        }
    }

    /// Add a rule definition.
    pub fn add_rule(
        &mut self,
        name: impl Into<String>,
        production: impl Into<String>,
    ) -> RuleBuilder<'_> {
        let name = name.into();
        let id = self.next_rule_id;
        self.next_rule_id += 1;

        RuleBuilder {
            builder: self,
            id,
            name,
            type_name: None,
            edge_type_name: None,
            priority: 0,
            auto: false,
            production: production.into(),
        }
    }

    /// Build the immutable Registry.
    pub fn build(self) -> Result<Registry, RegistryError> {
        // Validate and resolve parent types
        let mut resolved_types = HashMap::new();
        for (id, type_def) in &self.types {
            resolved_types.insert(*id, type_def.clone());
        }

        // Build subtype index
        let subtype_index = SubtypeIndex::build(&resolved_types);

        // Index constraints by type
        let mut constraints_by_type: HashMap<TypeId, Vec<usize>> = HashMap::new();
        let mut constraints_by_edge_type: HashMap<EdgeTypeId, Vec<usize>> = HashMap::new();

        for (i, constraint) in self.constraints.iter().enumerate() {
            if let Some(type_id) = constraint.type_id {
                constraints_by_type.entry(type_id).or_default().push(i);
            }
            if let Some(edge_type_id) = constraint.edge_type_id {
                constraints_by_edge_type
                    .entry(edge_type_id)
                    .or_default()
                    .push(i);
            }
        }

        // Sort and index rules by type (sorted by priority descending)
        let mut rules = self.rules;
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut rules_by_type: HashMap<TypeId, Vec<usize>> = HashMap::new();
        let mut rules_by_edge_type: HashMap<EdgeTypeId, Vec<usize>> = HashMap::new();

        for (i, rule) in rules.iter().enumerate() {
            if let Some(type_id) = rule.type_id {
                rules_by_type.entry(type_id).or_default().push(i);
            }
            if let Some(edge_type_id) = rule.edge_type_id {
                rules_by_edge_type.entry(edge_type_id).or_default().push(i);
            }
        }

        Ok(Registry::new(
            resolved_types,
            self.type_names,
            self.edge_types,
            self.edge_type_names,
            self.constraints,
            constraints_by_type,
            constraints_by_edge_type,
            rules,
            rules_by_type,
            rules_by_edge_type,
            subtype_index,
        ))
    }
}

/// Builder for a type definition.
pub struct TypeBuilder<'a> {
    builder: &'a mut RegistryBuilder,
    id: TypeId,
    name: String,
    parent_names: Vec<String>,
    attributes: HashMap<String, AttrDef>,
    is_abstract: bool,
    is_sealed: bool,
}

impl<'a> TypeBuilder<'a> {
    /// Add a parent type by name.
    pub fn extends(mut self, parent_name: impl Into<String>) -> Self {
        self.parent_names.push(parent_name.into());
        self
    }

    /// Add an attribute.
    pub fn attr(mut self, attr: AttrDef) -> Self {
        self.attributes.insert(attr.name.clone(), attr);
        self
    }

    /// Mark as abstract.
    pub fn abstract_type(mut self) -> Self {
        self.is_abstract = true;
        self
    }

    /// Mark as sealed.
    pub fn sealed(mut self) -> Self {
        self.is_sealed = true;
        self
    }

    /// Finish building this type.
    pub fn done(self) -> Result<TypeId, RegistryError> {
        // Check for duplicate name
        if self.builder.type_names.contains_key(&self.name) {
            return Err(RegistryError::DuplicateTypeName(self.name));
        }

        // Resolve parent IDs
        let mut parent_ids = Vec::new();
        for parent_name in &self.parent_names {
            match self.builder.type_names.get(parent_name) {
                Some(&parent_id) => parent_ids.push(parent_id),
                None => return Err(RegistryError::UnknownParentType(parent_name.clone())),
            }
        }

        let type_def = TypeDef {
            id: self.id,
            name: self.name.clone(),
            parent_ids,
            attributes: self.attributes,
            is_abstract: self.is_abstract,
            is_sealed: self.is_sealed,
        };

        self.builder.type_names.insert(self.name, self.id);
        self.builder.types.insert(self.id, type_def);

        Ok(self.id)
    }
}

/// Builder for an edge type definition.
pub struct EdgeTypeBuilder<'a> {
    builder: &'a mut RegistryBuilder,
    id: EdgeTypeId,
    name: String,
    params: Vec<EdgeParam>,
    attributes: HashMap<String, AttrDef>,
    symmetric: bool,
    on_kill: Vec<OnKillAction>,
    acyclic: bool,
    unique: bool,
}

impl<'a> EdgeTypeBuilder<'a> {
    /// Add a parameter.
    pub fn param(mut self, name: impl Into<String>, type_constraint: impl Into<String>) -> Self {
        self.params.push(EdgeParam {
            name: name.into(),
            type_constraint: type_constraint.into(),
            cardinality: Cardinality::default(),
        });
        self
    }

    /// Set cardinality constraint for a parameter by name.
    /// `min` and `max` define the allowed range; `max = None` means unbounded.
    pub fn with_cardinality(mut self, param_name: &str, min: u32, max: Option<u32>) -> Self {
        for param in &mut self.params {
            if param.name == param_name {
                param.cardinality = Cardinality::new(min, max);
                break;
            }
        }
        self
    }

    /// Add an attribute.
    pub fn attr(mut self, attr: AttrDef) -> Self {
        self.attributes.insert(attr.name.clone(), attr);
        self
    }

    /// Mark as symmetric.
    pub fn symmetric(mut self) -> Self {
        self.symmetric = true;
        self
    }

    /// Set on-kill action for a parameter (deprecated, use on_kill_at instead).
    pub fn on_kill(mut self, action: OnKillAction) -> Self {
        self.on_kill.push(action);
        self
    }

    /// Set on-kill action for a specific parameter by index.
    /// Index 0 = source (first param), Index 1 = target (second param), etc.
    pub fn on_kill_at(mut self, param_index: usize, action: OnKillAction) -> Self {
        // Extend the vector if needed, filling with default (Delete = unlink per spec)
        while self.on_kill.len() <= param_index {
            self.on_kill.push(OnKillAction::Delete); // Default: unlink (remove edge)
        }
        self.on_kill[param_index] = action;
        self
    }

    /// Mark as acyclic.
    pub fn acyclic(mut self) -> Self {
        self.acyclic = true;
        self
    }

    /// Mark as unique.
    pub fn unique_edge(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Finish building this edge type.
    pub fn done(self) -> Result<EdgeTypeId, RegistryError> {
        // Check for duplicate name
        if self.builder.edge_type_names.contains_key(&self.name) {
            return Err(RegistryError::DuplicateEdgeTypeName(self.name));
        }

        // Fill in on_kill actions if not specified
        let on_kill = if self.on_kill.is_empty() {
            vec![OnKillAction::Delete; self.params.len()]
        } else {
            self.on_kill
        };

        let edge_type_def = EdgeTypeDef {
            id: self.id,
            name: self.name.clone(),
            params: self.params,
            attributes: self.attributes,
            symmetric: self.symmetric,
            on_kill,
            acyclic: self.acyclic,
            unique: self.unique,
        };

        self.builder.edge_type_names.insert(self.name, self.id);
        self.builder.edge_types.insert(self.id, edge_type_def);

        Ok(self.id)
    }
}

/// Builder for a constraint definition.
pub struct ConstraintBuilder<'a> {
    builder: &'a mut RegistryBuilder,
    id: u32,
    name: String,
    type_name: Option<String>,
    edge_type_name: Option<String>,
    hard: bool,
    deferred: bool,
    condition: String,
}

impl<'a> ConstraintBuilder<'a> {
    /// Apply to a type.
    pub fn for_type(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    /// Apply to an edge type.
    pub fn for_edge_type(mut self, edge_type_name: impl Into<String>) -> Self {
        self.edge_type_name = Some(edge_type_name.into());
        self
    }

    /// Mark as soft constraint.
    pub fn soft(mut self) -> Self {
        self.hard = false;
        self
    }

    /// Mark as deferred.
    pub fn deferred(mut self) -> Self {
        self.deferred = true;
        self
    }

    /// Finish building this constraint.
    pub fn done(self) -> Result<u32, RegistryError> {
        // Resolve type ID if specified
        let type_id = if let Some(name) = &self.type_name {
            match self.builder.type_names.get(name) {
                Some(&id) => Some(id),
                None => return Err(RegistryError::UnknownTypeInConstraint(name.clone())),
            }
        } else {
            None
        };

        // Resolve edge type ID if specified
        let edge_type_id = if let Some(name) = &self.edge_type_name {
            match self.builder.edge_type_names.get(name) {
                Some(&id) => Some(id),
                None => return Err(RegistryError::UnknownEdgeTypeInConstraint(name.clone())),
            }
        } else {
            None
        };

        let constraint = ConstraintDef {
            id: self.id,
            name: self.name,
            type_id,
            edge_type_id,
            hard: self.hard,
            deferred: self.deferred,
            condition: self.condition,
        };

        self.builder.constraints.push(constraint);
        Ok(self.id)
    }
}

/// Builder for a rule definition.
pub struct RuleBuilder<'a> {
    builder: &'a mut RegistryBuilder,
    id: u32,
    name: String,
    type_name: Option<String>,
    edge_type_name: Option<String>,
    priority: i32,
    auto: bool,
    production: String,
}

impl<'a> RuleBuilder<'a> {
    /// Apply to a type.
    pub fn for_type(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    /// Apply to an edge type.
    pub fn for_edge_type(mut self, edge_type_name: impl Into<String>) -> Self {
        self.edge_type_name = Some(edge_type_name.into());
        self
    }

    /// Set priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Mark as auto.
    pub fn auto(mut self) -> Self {
        self.auto = true;
        self
    }

    /// Finish building this rule.
    pub fn done(self) -> Result<u32, RegistryError> {
        // Resolve type ID if specified
        let type_id = if let Some(name) = &self.type_name {
            self.builder.type_names.get(name).copied()
        } else {
            None
        };

        // Resolve edge type ID if specified
        let edge_type_id = if let Some(name) = &self.edge_type_name {
            self.builder.edge_type_names.get(name).copied()
        } else {
            None
        };

        let rule = RuleDef {
            id: self.id,
            name: self.name,
            type_id,
            edge_type_id,
            priority: self.priority,
            auto: self.auto,
            production: self.production,
        };

        self.builder.rules.push(rule);
        Ok(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== TEST: get_type_by_name ==========
    #[test]
    fn test_get_type_by_name() {
        // GIVEN registry with type Task
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String").required())
            .done()
            .unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_type_by_name("Task")
        let result = registry.get_type_by_name("Task");

        // THEN returns TypeDef with name="Task"
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Task");
    }

    // ========== TEST: get_type_by_name_not_found ==========
    #[test]
    fn test_get_type_by_name_not_found() {
        // GIVEN empty registry
        let registry = RegistryBuilder::new().build().unwrap();

        // WHEN get_type_by_name("NonExistent")
        let result = registry.get_type_by_name("NonExistent");

        // THEN returns None
        assert!(result.is_none());
    }

    // ========== TEST: get_type_by_id ==========
    #[test]
    fn test_get_type_by_id() {
        // GIVEN registry with type Task
        let mut builder = RegistryBuilder::new();
        let type_id = builder.add_type("Task").done().unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_type(type_id)
        let result = registry.get_type(type_id);

        // THEN returns TypeDef
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, type_id);
    }

    // ========== TEST: get_edge_type_by_name ==========
    #[test]
    fn test_get_edge_type_by_name() {
        // GIVEN registry with edge type owns
        let mut builder = RegistryBuilder::new();
        builder.add_type("Person").done().unwrap();
        builder.add_type("Task").done().unwrap();
        builder
            .add_edge_type("owns")
            .param("owner", "Person")
            .param("task", "Task")
            .done()
            .unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_edge_type_by_name("owns")
        let result = registry.get_edge_type_by_name("owns");

        // THEN returns EdgeTypeDef with name="owns"
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "owns");
        assert_eq!(result.unwrap().arity(), 2);
    }

    // ========== TEST: check_subtype_relationship ==========
    #[test]
    fn test_check_subtype_relationship() {
        // GIVEN registry with Entity <- Task
        let mut builder = RegistryBuilder::new();
        builder.add_type("Entity").done().unwrap();
        let task_id = builder.add_type("Task").extends("Entity").done().unwrap();
        let entity_id = builder.get_type_id("Entity").unwrap();
        let registry = builder.build().unwrap();

        // WHEN is_subtype(Task, Entity)
        let result = registry.is_subtype(task_id, entity_id);

        // THEN returns true
        assert!(result);

        // AND is_subtype(Entity, Task) returns false
        assert!(!registry.is_subtype(entity_id, task_id));
    }

    // ========== TEST: get_all_subtypes ==========
    #[test]
    fn test_get_all_subtypes() {
        // GIVEN registry with Entity <- Task <- Bug
        let mut builder = RegistryBuilder::new();
        builder.add_type("Entity").done().unwrap();
        builder.add_type("Task").extends("Entity").done().unwrap();
        builder.add_type("Bug").extends("Task").done().unwrap();
        let entity_id = builder.get_type_id("Entity").unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_subtypes(Entity)
        let subtypes: Vec<TypeId> = registry.get_subtypes(entity_id).collect();

        // THEN returns [Task, Bug]
        assert_eq!(subtypes.len(), 2);
    }

    // ========== TEST: get_constraints_for_type ==========
    #[test]
    fn test_get_constraints_for_type() {
        // GIVEN registry with Task type and constraint
        let mut builder = RegistryBuilder::new();
        builder.add_type("Task").done().unwrap();
        builder
            .add_constraint("priority_positive", "t.priority >= 0")
            .for_type("Task")
            .done()
            .unwrap();
        let task_id = builder.get_type_id("Task").unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_constraints_for_type(Task)
        let constraints = registry.get_constraints_for_type(task_id);

        // THEN returns [priority_positive]
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0].name, "priority_positive");
    }

    // ========== TEST: get_deferred_constraints ==========
    #[test]
    fn test_get_deferred_constraints() {
        // GIVEN registry with deferred and non-deferred constraints
        let mut builder = RegistryBuilder::new();
        builder.add_type("Task").done().unwrap();
        builder
            .add_constraint("immediate", "t.x > 0")
            .for_type("Task")
            .done()
            .unwrap();
        builder
            .add_constraint("deferred_check", "t.y > 0")
            .for_type("Task")
            .deferred()
            .done()
            .unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_deferred_constraints()
        let deferred = registry.get_deferred_constraints();

        // THEN returns only deferred constraint
        assert_eq!(deferred.len(), 1);
        assert_eq!(deferred[0].name, "deferred_check");
    }

    // ========== TEST: get_rules_for_type_sorted ==========
    #[test]
    fn test_get_rules_for_type_sorted() {
        // GIVEN registry with rules of different priorities
        let mut builder = RegistryBuilder::new();
        builder.add_type("Task").done().unwrap();
        builder
            .add_rule("low_priority", "SET t.x = 1")
            .for_type("Task")
            .priority(10)
            .done()
            .unwrap();
        builder
            .add_rule("high_priority", "SET t.y = 2")
            .for_type("Task")
            .priority(100)
            .done()
            .unwrap();
        builder
            .add_rule("medium_priority", "SET t.z = 3")
            .for_type("Task")
            .priority(50)
            .done()
            .unwrap();
        let task_id = builder.get_type_id("Task").unwrap();
        let registry = builder.build().unwrap();

        // WHEN get_rules_for_type(Task)
        let rules = registry.get_rules_for_type(task_id);

        // THEN returns rules sorted by priority descending
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].name, "high_priority");
        assert_eq!(rules[1].name, "medium_priority");
        assert_eq!(rules[2].name, "low_priority");
    }

    // ========== TEST: duplicate_type_name_error ==========
    #[test]
    fn test_duplicate_type_name_error() {
        // GIVEN registry with type Task
        let mut builder = RegistryBuilder::new();
        builder.add_type("Task").done().unwrap();

        // WHEN add another type with same name
        let result = builder.add_type("Task").done();

        // THEN returns DuplicateTypeName error
        assert!(matches!(result, Err(RegistryError::DuplicateTypeName(_))));
    }

    // ========== TEST: unknown_parent_type_error ==========
    #[test]
    fn test_unknown_parent_type_error() {
        // GIVEN empty registry
        let mut builder = RegistryBuilder::new();

        // WHEN add type extending non-existent parent
        let result = builder.add_type("Task").extends("NonExistent").done();

        // THEN returns UnknownParentType error
        assert!(matches!(result, Err(RegistryError::UnknownParentType(_))));
    }

    impl RegistryBuilder {
        fn get_type_id(&self, name: &str) -> Option<TypeId> {
            self.type_names.get(name).copied()
        }
    }
}
