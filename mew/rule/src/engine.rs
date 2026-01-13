//! Rule execution engine.

use std::collections::HashSet;

use mew_core::{EdgeId, NodeId};
use mew_graph::Graph;
use mew_pattern::{Bindings, Matcher};
use mew_registry::{Registry, RuleDef};

use crate::error::{RuleError, RuleResult};
use crate::{MAX_ACTIONS, MAX_DEPTH};

/// Statistics from rule execution.
#[derive(Debug, Clone, Default)]
pub struct RuleExecutionStats {
    /// Number of rules triggered.
    pub rules_triggered: usize,
    /// Number of actions executed.
    pub actions_executed: usize,
    /// Maximum depth reached.
    pub max_depth_reached: usize,
    /// Whether quiescence was reached.
    pub quiescence_reached: bool,
}

/// The rule engine.
pub struct RuleEngine<'r, 'g> {
    registry: &'r Registry,
    graph: &'g Graph,
    #[allow(dead_code)]
    matcher: Matcher<'r, 'g>,
    /// Track executed (rule_id, bindings_hash) to prevent re-execution.
    executed: HashSet<(u32, u64)>,
    /// Current execution depth.
    current_depth: usize,
    /// Total actions executed.
    action_count: usize,
}

impl<'r, 'g> RuleEngine<'r, 'g> {
    /// Create a new rule engine.
    pub fn new(registry: &'r Registry, graph: &'g Graph) -> Self {
        Self {
            registry,
            graph,
            matcher: Matcher::new(registry, graph),
            executed: HashSet::new(),
            current_depth: 0,
            action_count: 0,
        }
    }

    /// Reset execution state for a new transaction.
    pub fn reset(&mut self) {
        self.executed.clear();
        self.current_depth = 0;
        self.action_count = 0;
    }

    /// Find rules triggered by node creation.
    pub fn find_triggered_by_node(&self, node_id: NodeId) -> Vec<&'r RuleDef> {
        let node = match self.graph.get_node(node_id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        // Get rules that might be triggered by this node type
        let mut triggered = Vec::new();

        for rule in self.registry.get_rules_for_type(node.type_id) {
            if rule.auto {
                triggered.push(rule);
            }
        }

        // Sort by priority (highest first)
        triggered.sort_by(|a, b| b.priority.cmp(&a.priority));

        triggered
    }

    /// Find rules triggered by edge creation.
    pub fn find_triggered_by_edge(&self, edge_id: EdgeId) -> Vec<&'r RuleDef> {
        let edge = match self.graph.get_edge(edge_id) {
            Some(e) => e,
            None => return Vec::new(),
        };

        // Get rules that might be triggered by this edge type
        let mut triggered = Vec::new();

        for rule in self.registry.get_rules_for_edge_type(edge.type_id) {
            if rule.auto {
                triggered.push(rule);
            }
        }

        // Sort by priority (highest first)
        triggered.sort_by(|a, b| b.priority.cmp(&a.priority));

        triggered
    }

    /// Fire all triggered rules to quiescence.
    pub fn fire_to_quiescence(
        &mut self,
        initial_nodes: &[NodeId],
        initial_edges: &[EdgeId],
    ) -> RuleResult<RuleExecutionStats> {
        let mut stats = RuleExecutionStats::default();

        // Collect initially triggered rules
        let mut pending_nodes: Vec<NodeId> = initial_nodes.to_vec();
        let mut pending_edges: Vec<EdgeId> = initial_edges.to_vec();

        loop {
            // Check depth limit
            if self.current_depth >= MAX_DEPTH {
                return Err(RuleError::max_depth_exceeded(self.current_depth));
            }

            // Check action limit
            if self.action_count >= MAX_ACTIONS {
                return Err(RuleError::max_actions_exceeded(self.action_count));
            }

            // Find triggered rules
            let mut rules_to_fire: Vec<(&RuleDef, Bindings)> = Vec::new();

            for &node_id in &pending_nodes {
                let triggered = self.find_triggered_by_node(node_id);
                for rule in triggered {
                    // Create initial bindings with the triggering node
                    let mut bindings = Bindings::new();
                    bindings.insert("trigger", mew_pattern::Binding::Node(node_id));

                    let key = (rule.id, self.hash_bindings(&bindings));
                    if !self.executed.contains(&key) {
                        rules_to_fire.push((rule, bindings));
                    }
                }
            }

            for &edge_id in &pending_edges {
                let triggered = self.find_triggered_by_edge(edge_id);
                for rule in triggered {
                    let mut bindings = Bindings::new();
                    bindings.insert("trigger", mew_pattern::Binding::Edge(edge_id));

                    let key = (rule.id, self.hash_bindings(&bindings));
                    if !self.executed.contains(&key) {
                        rules_to_fire.push((rule, bindings));
                    }
                }
            }

            // If no rules to fire, we've reached quiescence
            if rules_to_fire.is_empty() {
                stats.quiescence_reached = true;
                break;
            }

            // Clear pending for next iteration
            pending_nodes.clear();
            pending_edges.clear();

            // Fire the rules
            self.current_depth += 1;
            stats.max_depth_reached = stats.max_depth_reached.max(self.current_depth);

            for (rule, bindings) in rules_to_fire {
                let key = (rule.id, self.hash_bindings(&bindings));
                self.executed.insert(key);

                stats.rules_triggered += 1;
                self.action_count += 1;

                // In a real implementation, we would:
                // 1. Match the rule's pattern with the bindings
                // 2. Execute the rule's actions
                // 3. Collect any new nodes/edges created
                // For now, we just track that the rule was triggered
            }

            self.current_depth -= 1;
        }

        stats.actions_executed = self.action_count;
        Ok(stats)
    }

    /// Execute a single rule with given bindings.
    pub fn execute_rule(&mut self, rule: &RuleDef, bindings: &Bindings) -> RuleResult<()> {
        let key = (rule.id, self.hash_bindings(bindings));

        // Check if already executed
        if self.executed.contains(&key) {
            return Ok(());
        }

        // Check limits
        if self.current_depth >= MAX_DEPTH {
            return Err(RuleError::max_depth_exceeded(self.current_depth));
        }
        if self.action_count >= MAX_ACTIONS {
            return Err(RuleError::max_actions_exceeded(self.action_count));
        }

        // Mark as executed
        self.executed.insert(key);
        self.action_count += 1;

        // In a real implementation, execute the rule's actions here

        Ok(())
    }

    /// Manually fire a rule by name.
    pub fn fire_rule_by_name(&mut self, name: &str, bindings: &Bindings) -> RuleResult<()> {
        let rule = self
            .registry
            .all_rules()
            .find(|r| r.name == name)
            .ok_or_else(|| RuleError::unknown_rule(name))?;

        self.execute_rule(rule, bindings)
    }

    /// Hash bindings for cycle detection.
    fn hash_bindings(&self, bindings: &Bindings) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        // Hash all binding names and values
        for name in bindings.names() {
            name.hash(&mut hasher);
            if let Some(binding) = bindings.get(name) {
                match binding.as_node() {
                    Some(id) => id.raw().hash(&mut hasher),
                    None => {
                        if let Some(id) = binding.as_edge() {
                            id.raw().hash(&mut hasher);
                        }
                    }
                }
            }
        }

        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;
    use mew_registry::{AttrDef, RegistryBuilder};

    fn test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .done()
            .unwrap();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_rule_engine_creation() {
        // GIVEN
        let registry = test_registry();
        let graph = Graph::new();

        // WHEN
        let engine = RuleEngine::new(&registry, &graph);

        // THEN
        assert_eq!(engine.current_depth, 0);
        assert_eq!(engine.action_count, 0);
    }

    #[test]
    fn test_fire_to_quiescence_empty() {
        // GIVEN
        let registry = test_registry();
        let graph = Graph::new();
        let mut engine = RuleEngine::new(&registry, &graph);

        // WHEN - fire with no initial entities
        let stats = engine.fire_to_quiescence(&[], &[]).unwrap();

        // THEN
        assert!(stats.quiescence_reached);
        assert_eq!(stats.rules_triggered, 0);
    }

    #[test]
    fn test_fire_to_quiescence_with_nodes() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let mut engine = RuleEngine::new(&registry, &graph);

        // WHEN
        let stats = engine.fire_to_quiescence(&[node], &[]).unwrap();

        // THEN - no rules defined in test registry, should reach quiescence immediately
        assert!(stats.quiescence_reached);
    }

    #[test]
    fn test_reset_clears_state() {
        // GIVEN
        let registry = test_registry();
        let graph = Graph::new();
        let mut engine = RuleEngine::new(&registry, &graph);

        // Simulate some execution
        engine.action_count = 100;
        engine.current_depth = 5;
        engine.executed.insert((1, 12345));

        // WHEN
        engine.reset();

        // THEN
        assert_eq!(engine.action_count, 0);
        assert_eq!(engine.current_depth, 0);
        assert!(engine.executed.is_empty());
    }

    // ========== Acceptance Tests ==========

    fn registry_with_rules() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .attr(AttrDef::new("created_at", "String"))
            .attr(AttrDef::new("updated_at", "String"))
            .attr(AttrDef::new("owner_name", "String"))
            .attr(AttrDef::new("has_owner", "Bool"))
            .done()
            .unwrap();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("owns")
            .param("owner", "Person")
            .param("task", "Task")
            .done()
            .unwrap();
        builder
            .add_edge_type("default_owner")
            .param("owner", "Person")
            .param("task", "Task")
            .done()
            .unwrap();

        // Rule that triggers on Task SPAWN (auto-fire)
        builder
            .add_rule("set_created_at", "SET t.created_at = NOW()")
            .for_type("Task")
            .priority(50)
            .auto()
            .done()
            .unwrap();

        // Rule that triggers on owns edge (auto-fire)
        builder
            .add_rule("set_owner_name", "SET t.owner_name = p.name")
            .for_edge_type("owns")
            .priority(50)
            .auto()
            .done()
            .unwrap();

        // Manual rule (auto: false by default)
        builder
            .add_rule("manual_rule", "SET t.updated_at = NOW()")
            .for_type("Task")
            // No .auto() call - default is false (manual rule)
            .priority(50)
            .done()
            .unwrap();

        builder.build().unwrap()
    }

    #[test]
    fn test_rule_triggers_on_spawn() {
        // GIVEN rule for Task type
        let registry = registry_with_rules();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let engine = RuleEngine::new(&registry, &graph);

        // WHEN finding triggered rules for spawned node
        let triggered = engine.find_triggered_by_node(node);

        // THEN rule is found
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].name, "set_created_at");
    }

    #[test]
    fn test_rule_triggers_on_link() {
        // GIVEN rule for owns edge type
        let registry = registry_with_rules();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let owns_type_id = registry.get_edge_type_id("owns").unwrap();

        let person = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let task = graph.create_node(task_type_id, attrs! { "title" => "Test" });
        let edge = graph
            .create_edge(owns_type_id, vec![person.into(), task.into()], attrs! {})
            .unwrap();

        let engine = RuleEngine::new(&registry, &graph);

        // WHEN finding triggered rules for linked edge
        let triggered = engine.find_triggered_by_edge(edge);

        // THEN rule is found
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].name, "set_owner_name");
    }

    #[test]
    fn test_rules_execute_in_priority_order() {
        // GIVEN rules with different priorities
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .done()
            .unwrap();
        builder
            .add_rule("low_priority", "SET t.x = 1")
            .for_type("Task")
            .priority(10)
            .auto()
            .done()
            .unwrap();
        builder
            .add_rule("high_priority", "SET t.y = 2")
            .for_type("Task")
            .priority(50)
            .auto()
            .done()
            .unwrap();
        builder
            .add_rule("medium_priority", "SET t.z = 3")
            .for_type("Task")
            .priority(20)
            .auto()
            .done()
            .unwrap();

        let registry = builder.build().unwrap();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let engine = RuleEngine::new(&registry, &graph);

        // WHEN finding triggered rules
        let triggered = engine.find_triggered_by_node(node);

        // THEN rules are in priority order (highest first)
        assert_eq!(triggered.len(), 3);
        assert_eq!(triggered[0].name, "high_priority");
        assert_eq!(triggered[1].name, "medium_priority");
        assert_eq!(triggered[2].name, "low_priority");
    }

    #[test]
    fn test_same_binding_executes_once() {
        // GIVEN a rule
        let registry = registry_with_rules();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let mut engine = RuleEngine::new(&registry, &graph);

        // WHEN firing rules multiple times for the same node
        let stats1 = engine.fire_to_quiescence(&[node], &[]).unwrap();
        let stats2 = engine.fire_to_quiescence(&[node], &[]).unwrap();

        // THEN rule only executes once (first time) due to dedup
        assert!(stats1.rules_triggered >= 1);
        assert_eq!(stats2.rules_triggered, 0); // Already executed
    }

    #[test]
    fn test_depth_limit_prevents_infinite_recursion() {
        // GIVEN engine at max depth
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let mut engine = RuleEngine::new(&registry, &graph);
        engine.current_depth = crate::MAX_DEPTH; // Simulate deep recursion

        // WHEN trying to fire rules
        let result = engine.fire_to_quiescence(&[node], &[]);

        // THEN error about depth limit
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuleError::MaxDepthExceeded { .. }
        ));
    }

    #[test]
    fn test_action_limit_prevents_runaway() {
        // GIVEN engine at max actions
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let mut engine = RuleEngine::new(&registry, &graph);
        engine.action_count = crate::MAX_ACTIONS; // Simulate many actions

        // WHEN trying to fire rules
        let result = engine.fire_to_quiescence(&[node], &[]);

        // THEN error about action limit
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuleError::MaxActionsExceeded { .. }
        ));
    }

    #[test]
    fn test_reach_quiescence() {
        // GIVEN rules that will stop producing matches
        let registry = registry_with_rules();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let mut engine = RuleEngine::new(&registry, &graph);

        // WHEN firing rules
        let stats = engine.fire_to_quiescence(&[node], &[]).unwrap();

        // THEN quiescence is reached
        assert!(stats.quiescence_reached);
    }

    #[test]
    fn test_manual_rule_does_not_auto_fire() {
        // GIVEN a manual rule (auto: false)
        let registry = registry_with_rules();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let engine = RuleEngine::new(&registry, &graph);

        // WHEN finding triggered rules
        let triggered = engine.find_triggered_by_node(node);

        // THEN manual rule is NOT included (only auto rules)
        for rule in &triggered {
            assert!(rule.auto, "Manual rule should not auto fire");
            assert_ne!(rule.name, "manual_rule");
        }
    }

    #[test]
    fn test_manual_rule_fires_on_explicit_call() {
        // GIVEN a manual rule
        let registry = registry_with_rules();
        let graph = Graph::new();
        let mut engine = RuleEngine::new(&registry, &graph);
        let bindings = Bindings::new();

        // WHEN explicitly firing the rule
        let result = engine.fire_rule_by_name("manual_rule", &bindings);

        // THEN rule fires successfully
        assert!(result.is_ok());
        assert_eq!(engine.action_count, 1);
    }

    #[test]
    fn test_fire_unknown_rule_fails() {
        // GIVEN no such rule
        let registry = test_registry();
        let graph = Graph::new();
        let mut engine = RuleEngine::new(&registry, &graph);
        let bindings = Bindings::new();

        // WHEN trying to fire unknown rule
        let result = engine.fire_rule_by_name("nonexistent", &bindings);

        // THEN error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuleError::UnknownRule { .. }));
    }
}
