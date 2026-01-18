//! World state generation

use crate::config::TestConfig;
use crate::types::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};

/// Generates a valid world state from a schema
pub struct WorldGenerator<'a> {
    schema: &'a AnalyzedSchema,
    config: TestConfig,
    var_counter: usize,
}

impl<'a> WorldGenerator<'a> {
    pub fn new(schema: &'a AnalyzedSchema, config: TestConfig) -> Self {
        Self {
            schema,
            config,
            var_counter: 0,
        }
    }

    /// Generate a complete world state
    pub fn generate(&mut self, rng: &mut impl Rng) -> Result<WorldState, TestGenError> {
        let mut world = WorldState::new();

        // 1. Generate nodes for each type
        for (type_name, type_info) in &self.schema.node_types {
            for _ in 0..self.config.nodes_per_type {
                let node = self.generate_node(type_name, type_info, rng);
                world.add_node(node);
            }
        }

        // 2. Generate edges for each edge type
        for (edge_name, edge_info) in &self.schema.edge_types {
            self.generate_edges_for_type(edge_name, edge_info, &mut world, rng)?;
        }

        Ok(world)
    }

    fn generate_node(
        &mut self,
        type_name: &str,
        type_info: &NodeTypeInfo,
        rng: &mut impl Rng,
    ) -> GeneratedNode {
        let var_name = self.next_var(type_name);
        let mut attrs = HashMap::new();
        let mut dynamic_attrs = HashSet::new();

        // Generate all attributes (including inherited)
        let all_attrs = self.collect_attrs(type_info);

        for attr in &all_attrs {
            // Skip nullable non-required attrs sometimes
            if attr.nullable && !attr.required && rng.gen_bool(0.3) {
                continue;
            }

            // Use default if available and random says so
            if let Some(ref default) = attr.default {
                if rng.gen_bool(0.5) {
                    // Check if default is dynamic
                    if default.is_dynamic() {
                        dynamic_attrs.insert(attr.name.clone());
                    }
                    attrs.insert(attr.name.clone(), default.clone());
                    continue;
                }
            }

            // Generate a value using type aliases for proper enum/range handling
            let gen_value = attr.generate_value_with_aliases(rng, &self.schema.type_aliases);
            if gen_value.is_dynamic {
                dynamic_attrs.insert(attr.name.clone());
            }
            attrs.insert(attr.name.clone(), gen_value.value);
        }

        GeneratedNode {
            var_name,
            type_name: type_name.to_string(),
            attrs,
            dynamic_attrs,
        }
    }

    fn collect_attrs(&self, type_info: &NodeTypeInfo) -> Vec<AttrInfo> {
        let mut attrs = type_info.attrs.clone();

        // Add parent attrs
        for parent_name in &type_info.parents {
            if let Some(parent) = self.schema.node_types.get(parent_name) {
                let parent_attrs = self.collect_attrs(parent);
                for attr in parent_attrs {
                    if !attrs.iter().any(|a| a.name == attr.name) {
                        attrs.push(attr);
                    }
                }
            }
        }

        attrs
    }

    fn generate_edges_for_type(
        &mut self,
        edge_name: &str,
        edge_info: &EdgeTypeInfo,
        world: &mut WorldState,
        rng: &mut impl Rng,
    ) -> Result<(), TestGenError> {
        // Determine valid source and target types from params
        if edge_info.params.len() < 2 {
            return Ok(()); // Need at least 2 params
        }

        let source_type = &edge_info.params[0].1;
        let target_type = &edge_info.params[1].1;

        // Get candidate nodes
        let source_indices: Vec<usize> = world
            .nodes_by_type
            .get(source_type)
            .cloned()
            .unwrap_or_default();
        let target_indices: Vec<usize> = world
            .nodes_by_type
            .get(target_type)
            .cloned()
            .unwrap_or_default();

        if source_indices.is_empty() || target_indices.is_empty() {
            return Ok(());
        }

        // Track created edges for uniqueness
        let mut created: std::collections::HashSet<(usize, usize)> =
            std::collections::HashSet::new();

        for _ in 0..self.config.edges_per_type {
            // Pick random source and target
            let from_idx = source_indices[rng.gen_range(0..source_indices.len())];
            let to_idx = target_indices[rng.gen_range(0..target_indices.len())];

            // Check no_self constraint
            if edge_info.no_self && from_idx == to_idx {
                continue;
            }

            // Check uniqueness
            if edge_info.unique && created.contains(&(from_idx, to_idx)) {
                continue;
            }

            created.insert((from_idx, to_idx));

            // Generate edge attributes
            let mut attrs = HashMap::new();
            let mut dynamic_attrs = HashSet::new();
            for attr in &edge_info.attrs {
                let gen_value = attr.generate_value_with_aliases(rng, &self.schema.type_aliases);
                if gen_value.is_dynamic {
                    dynamic_attrs.insert(attr.name.clone());
                }
                attrs.insert(attr.name.clone(), gen_value.value);
            }

            let edge = GeneratedEdge {
                var_name: None,
                edge_type: edge_name.to_string(),
                from_idx,
                to_idx,
                attrs: attrs.clone(),
                dynamic_attrs: dynamic_attrs.clone(),
            };

            world.add_edge(edge);

            // If symmetric, create reverse edge too
            if edge_info.symmetric && from_idx != to_idx && !created.contains(&(to_idx, from_idx)) {
                created.insert((to_idx, from_idx));
                let reverse = GeneratedEdge {
                    var_name: None,
                    edge_type: edge_name.to_string(),
                    from_idx: to_idx,
                    to_idx: from_idx,
                    attrs,
                    dynamic_attrs,
                };
                world.add_edge(reverse);
            }
        }

        Ok(())
    }

    fn next_var(&mut self, type_name: &str) -> String {
        self.var_counter += 1;
        format!("{}_{}", type_name.to_lowercase(), self.var_counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaAnalyzer;
    use rand::SeedableRng;

    #[test]
    fn test_generate_nodes() {
        let source = r#"
            node Person {
                name: String [required],
                age: Int [0..120]
            }
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        let config = TestConfig::minimal();
        let mut gen = WorldGenerator::new(&schema, config.clone());
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let world = gen.generate(&mut rng).unwrap();

        assert_eq!(world.nodes.len(), config.nodes_per_type);
        for node in &world.nodes {
            assert_eq!(node.type_name, "Person");
            assert!(node.attrs.contains_key("name"));
        }
    }

    #[test]
    fn test_generate_edges() {
        let source = r#"
            node Person { name: String }
            edge knows(a: Person, b: Person) [no_self]
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        let config = TestConfig::minimal();
        let mut gen = WorldGenerator::new(&schema, config);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let world = gen.generate(&mut rng).unwrap();

        // Check no self-edges
        for edge in &world.edges {
            assert_ne!(edge.from_idx, edge.to_idx, "Should not have self-edges");
        }
    }
}
