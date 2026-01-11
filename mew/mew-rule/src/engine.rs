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
}
