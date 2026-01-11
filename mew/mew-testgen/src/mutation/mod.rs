//! Mutation generation with expected effects

use crate::types::*;
use rand::Rng;

/// Generates mutations with constructive expected results
pub struct MutationGenerator<'a> {
    schema: &'a AnalyzedSchema,
    world: &'a WorldState,
    var_counter: usize,
}

impl<'a> MutationGenerator<'a> {
    pub fn new(schema: &'a AnalyzedSchema, world: &'a WorldState) -> Self {
        Self {
            schema,
            world,
            var_counter: 0,
        }
    }

    /// Generate mutations with known expected effects
    pub fn generate(
        &mut self,
        count: usize,
        rng: &mut impl Rng,
    ) -> Result<Vec<GeneratedMutation>, TestGenError> {
        let mut mutations = Vec::new();

        let mutation_types = [
            MutationType::SpawnValid,
            MutationType::SpawnMissingRequired,
            MutationType::SpawnOutOfRange,
            MutationType::LinkValid,
        ];

        for i in 0..count {
            let mutation_type = &mutation_types[i % mutation_types.len()];
            if let Some(mutation) = self.generate_mutation(mutation_type, rng) {
                mutations.push(mutation);
            }
        }

        Ok(mutations)
    }

    fn generate_mutation(
        &mut self,
        mutation_type: &MutationType,
        rng: &mut impl Rng,
    ) -> Option<GeneratedMutation> {
        match mutation_type {
            MutationType::SpawnValid => self.gen_spawn_valid(rng),
            MutationType::SpawnMissingRequired => self.gen_spawn_missing_required(rng),
            MutationType::SpawnOutOfRange => self.gen_spawn_out_of_range(rng),
            MutationType::LinkValid => self.gen_link_valid(rng),
        }
    }

    /// SPAWN with all valid attributes
    fn gen_spawn_valid(&mut self, rng: &mut impl Rng) -> Option<GeneratedMutation> {
        let type_names: Vec<&String> = self.schema.node_types.keys().collect();
        if type_names.is_empty() {
            return None;
        }

        let type_name = type_names[rng.gen_range(0..type_names.len())];
        let type_info = self.schema.node_types.get(type_name)?;

        let var = self.next_var();

        // Generate all required attributes (including inherited)
        let all_attrs = self.collect_all_attrs(type_info);
        let mut attrs = Vec::new();
        for attr in &all_attrs {
            if attr.required || rng.gen_bool(0.5) {
                let gen_value = attr.generate_value_with_aliases(rng, &self.schema.type_aliases);
                attrs.push((attr.name.clone(), self.value_to_mew(&gen_value.value)));
            }
        }

        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(
                " {{ {} }}",
                attrs.iter()
                    .map(|(k, v)| format!("{} = {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let statement = format!("SPAWN {}: {}{}", var, type_name, attrs_str);

        Some(GeneratedMutation {
            statement,
            required_setup: Vec::new(),
            expected: Expected::Success(MutationEffect::spawn_one()),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::simple(),
            tags: vec!["spawn".to_string(), "valid".to_string()],
        })
    }

    /// SPAWN missing required attribute - should fail
    fn gen_spawn_missing_required(&mut self, rng: &mut impl Rng) -> Option<GeneratedMutation> {
        // Find a type with required attributes (including inherited)
        let node_types = &self.schema.node_types;
        let types_with_required: Vec<(&String, &NodeTypeInfo)> = self.schema.node_types.iter()
            .filter(|(_, info)| {
                let all_attrs = Self::collect_all_attrs_static(info, node_types);
                all_attrs.iter().any(|a| a.required)
            })
            .collect();

        if types_with_required.is_empty() {
            return None;
        }

        let (type_name, type_info) = types_with_required[rng.gen_range(0..types_with_required.len())];
        let var = self.next_var();

        // Get all attrs including inherited
        let all_attrs = self.collect_all_attrs(type_info);

        // Skip one required attribute
        let required_attr = all_attrs.iter()
            .find(|a| a.required)?;

        // Generate other required attrs but skip selected required one
        let type_aliases = &self.schema.type_aliases;
        let attrs: Vec<(String, String)> = all_attrs.iter()
            .filter(|a| a.name != required_attr.name)
            .filter(|a| a.required) // Only include other required attrs
            .map(|a| {
                let gen_value = a.generate_value_with_aliases(rng, type_aliases);
                (a.name.clone(), self.value_to_mew(&gen_value.value))
            })
            .collect();

        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(
                " {{ {} }}",
                attrs.iter()
                    .map(|(k, v)| format!("{} = {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let statement = format!("SPAWN {}: {}{}", var, type_name, attrs_str);

        Some(GeneratedMutation {
            statement,
            required_setup: Vec::new(),
            expected: Expected::Error(format!("required.*{}", required_attr.name)),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::medium(),
            tags: vec!["spawn".to_string(), "negative".to_string(), "constraint".to_string()],
        })
    }

    /// SPAWN with out-of-range value - should fail
    fn gen_spawn_out_of_range(&mut self, rng: &mut impl Rng) -> Option<GeneratedMutation> {
        // Find a type with range constraints (including inherited)
        let node_types = &self.schema.node_types;
        let types_with_range: Vec<(&String, &NodeTypeInfo)> = self.schema.node_types.iter()
            .filter(|(_, info)| {
                let all_attrs = Self::collect_all_attrs_static(info, node_types);
                all_attrs.iter().any(|a| a.min.is_some() || a.max.is_some())
            })
            .collect();

        if types_with_range.is_empty() {
            return None;
        }

        let (type_name, type_info) = types_with_range[rng.gen_range(0..types_with_range.len())];
        let var = self.next_var();

        // Get all attrs including inherited
        let all_attrs = self.collect_all_attrs(type_info);

        let range_attr = all_attrs.iter()
            .find(|a| a.min.is_some() || a.max.is_some())?;

        // Generate an out-of-range value
        let out_of_range = if let Some(Value::Int(max)) = &range_attr.max {
            Value::Int(max + 100)
        } else if let Some(Value::Int(min)) = &range_attr.min {
            Value::Int(min - 100)
        } else {
            return None;
        };

        // Generate attrs with one out of range (include all required, including inherited)
        let mut attrs = Vec::new();
        for attr in &all_attrs {
            if attr.name == range_attr.name {
                attrs.push((attr.name.clone(), self.value_to_mew(&out_of_range)));
            } else if attr.required {
                let gen_value = attr.generate_value_with_aliases(rng, &self.schema.type_aliases);
                attrs.push((attr.name.clone(), self.value_to_mew(&gen_value.value)));
            }
        }

        let attrs_str = format!(
            " {{ {} }}",
            attrs.iter()
                .map(|(k, v)| format!("{} = {}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let statement = format!("SPAWN {}: {}{}", var, type_name, attrs_str);

        Some(GeneratedMutation {
            statement,
            required_setup: Vec::new(),
            expected: Expected::Error(format!("range|constraint.*{}", range_attr.name)),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::medium(),
            tags: vec!["spawn".to_string(), "negative".to_string(), "constraint".to_string()],
        })
    }

    /// LINK valid edge
    fn gen_link_valid(&mut self, rng: &mut impl Rng) -> Option<GeneratedMutation> {
        if self.schema.edge_types.is_empty() {
            return None;
        }

        let edge_types: Vec<(&String, &EdgeTypeInfo)> = self.schema.edge_types.iter().collect();
        let (edge_name, edge_info) = edge_types[rng.gen_range(0..edge_types.len())];

        if edge_info.params.len() < 2 {
            return None;
        }

        let source_type = &edge_info.params[0].1;
        let target_type = &edge_info.params[1].1;

        // Check we have nodes
        let source_nodes: Vec<&GeneratedNode> = self.world.nodes_of_type(source_type).collect();
        let target_nodes: Vec<&GeneratedNode> = self.world.nodes_of_type(target_type).collect();

        if source_nodes.is_empty() || target_nodes.is_empty() {
            return None;
        }

        // Pick random source and target
        let source = source_nodes[rng.gen_range(0..source_nodes.len())];
        let target = target_nodes[rng.gen_range(0..target_nodes.len())];

        // Make sure we don't violate no_self
        if edge_info.no_self && source.var_name == target.var_name {
            return None;
        }

        let statement = format!(
            "LINK {}({}, {})",
            edge_name, source.var_name, target.var_name
        );

        // Setup spawns the nodes first
        let setup = self.setup_for_nodes(&[source, target]);

        Some(GeneratedMutation {
            statement,
            required_setup: setup,
            expected: Expected::Success(MutationEffect::link_one()),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::medium(),
            tags: vec!["link".to_string(), "valid".to_string()],
        })
    }

    fn setup_for_nodes(&self, nodes: &[&GeneratedNode]) -> Vec<String> {
        nodes.iter()
            .map(|n| {
                let attrs_str = n.attrs.iter()
                    .map(|(k, v)| format!("{} = {}", k, self.value_to_mew(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                if attrs_str.is_empty() {
                    format!("SPAWN {}: {}", n.var_name, n.type_name)
                } else {
                    format!("SPAWN {}: {} {{ {} }}", n.var_name, n.type_name, attrs_str)
                }
            })
            .collect()
    }

    fn next_var(&mut self) -> String {
        self.var_counter += 1;
        format!("v{}", self.var_counter)
    }

    /// Collect all attributes for a type, including inherited ones
    fn collect_all_attrs(&self, type_info: &NodeTypeInfo) -> Vec<AttrInfo> {
        Self::collect_all_attrs_static(type_info, &self.schema.node_types)
    }

    /// Static version for use in closures
    fn collect_all_attrs_static(
        type_info: &NodeTypeInfo,
        node_types: &std::collections::HashMap<String, NodeTypeInfo>,
    ) -> Vec<AttrInfo> {
        let mut attrs = type_info.attrs.clone();

        // Add parent attrs
        for parent_name in &type_info.parents {
            if let Some(parent) = node_types.get(parent_name) {
                let parent_attrs = Self::collect_all_attrs_static(parent, node_types);
                for attr in parent_attrs {
                    if !attrs.iter().any(|a| a.name == attr.name) {
                        attrs.push(attr);
                    }
                }
            }
        }

        attrs
    }

    fn value_to_mew(&self, v: &Value) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Id(id) => format!("#{}", id),
            Value::FunctionCall(name) => format!("{}()", name),
        }
    }
}

enum MutationType {
    SpawnValid,
    SpawnMissingRequired,
    SpawnOutOfRange,
    LinkValid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaAnalyzer;
    use crate::world::WorldGenerator;
    use crate::config::TestConfig;
    use rand::SeedableRng;

    #[test]
    fn test_generate_mutations() {
        let source = r#"
            node Person {
                name: String [required],
                age: Int [0..120]
            }
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        let config = TestConfig::minimal();
        let mut world_gen = WorldGenerator::new(&schema, config.clone());
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let world = world_gen.generate(&mut rng).unwrap();
        let mut mutation_gen = MutationGenerator::new(&schema, &world);

        let mutations = mutation_gen.generate(4, &mut rng).unwrap();

        assert!(!mutations.is_empty());
        for m in &mutations {
            assert!(m.statement.starts_with("SPAWN") || m.statement.starts_with("LINK"));
        }
    }
}
