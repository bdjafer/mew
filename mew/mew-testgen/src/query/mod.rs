//! Query generation with expected results

use crate::types::*;
use rand::Rng;

/// Generates queries with constructive expected results
pub struct QueryGenerator<'a> {
    schema: &'a AnalyzedSchema,
    world: &'a WorldState,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(schema: &'a AnalyzedSchema, world: &'a WorldState) -> Self {
        Self { schema, world }
    }

    /// Generate queries with known expected results
    pub fn generate(
        &mut self,
        count: usize,
        rng: &mut impl Rng,
    ) -> Result<Vec<GeneratedQuery>, TestGenError> {
        let mut queries = Vec::new();

        // Generate different query types
        let query_types = [
            QueryType::MatchAll,
            QueryType::MatchByAttr,
            QueryType::MatchWithProjection,
            QueryType::MatchEmpty,
            QueryType::MatchCount,
        ];

        for i in 0..count {
            let query_type = &query_types[i % query_types.len()];
            if let Some(query) = self.generate_query(query_type, rng) {
                queries.push(query);
            }
        }

        Ok(queries)
    }

    fn generate_query(
        &self,
        query_type: &QueryType,
        rng: &mut impl Rng,
    ) -> Option<GeneratedQuery> {
        match query_type {
            QueryType::MatchAll => self.gen_match_all(rng),
            QueryType::MatchByAttr => self.gen_match_by_attr(rng),
            QueryType::MatchWithProjection => self.gen_match_projection(rng),
            QueryType::MatchEmpty => self.gen_match_empty(rng),
            QueryType::MatchCount => self.gen_match_count(rng),
        }
    }

    /// MATCH t: Type RETURN t
    fn gen_match_all(&self, rng: &mut impl Rng) -> Option<GeneratedQuery> {
        let type_names: Vec<&String> = self.schema.node_types.keys().collect();
        if type_names.is_empty() {
            return None;
        }

        let type_name = type_names[rng.gen_range(0..type_names.len())];
        let var = type_name.chars().next()?.to_lowercase().to_string();

        let statement = format!("MATCH {}: {} RETURN {}", var, type_name, var);

        // Count expected results from world
        let count = self.world.nodes_of_type(type_name).count();

        Some(GeneratedQuery {
            statement,
            required_setup: self.setup_statements(),
            expected: Expected::Count(count),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::simple(),
            tags: vec!["match".to_string(), "all".to_string()],
        })
    }

    /// MATCH t: Type WHERE t.attr = value RETURN t
    fn gen_match_by_attr(&self, rng: &mut impl Rng) -> Option<GeneratedQuery> {
        // Pick a type with nodes in the world
        let type_name = self.pick_populated_type(rng)?;
        let type_info = self.schema.node_types.get(type_name)?;

        if type_info.attrs.is_empty() {
            return None;
        }

        // Pick a random node of this type to query for
        let nodes: Vec<&GeneratedNode> = self.world.nodes_of_type(type_name).collect();
        if nodes.is_empty() {
            return None;
        }

        let target_node = nodes[rng.gen_range(0..nodes.len())];

        // Pick a STATIC attribute that this node has (skip dynamic ones like now())
        let static_attrs: Vec<&String> = target_node.attrs.keys()
            .filter(|k| !target_node.is_attr_dynamic(k))
            .collect();

        if static_attrs.is_empty() {
            // Fall back to a simple MATCH ALL if no static attrs
            return self.gen_match_all(rng);
        }

        let attr_name = static_attrs[rng.gen_range(0..static_attrs.len())];
        let attr_value = target_node.attrs.get(attr_name)?;

        let var = type_name.chars().next()?.to_lowercase().to_string();

        let value_str = self.value_to_mew(attr_value);
        let statement = format!(
            "MATCH {}: {} WHERE {}.{} = {} RETURN {}",
            var, type_name, var, attr_name, value_str, var
        );

        // Count matching nodes
        let count = self.world
            .nodes_of_type(type_name)
            .filter(|n| n.attrs.get(attr_name) == Some(attr_value))
            .count();

        Some(GeneratedQuery {
            statement,
            required_setup: self.setup_statements(),
            expected: Expected::Count(count),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::medium(),
            tags: vec!["match".to_string(), "where".to_string()],
        })
    }

    /// MATCH t: Type RETURN t.attr
    fn gen_match_projection(&self, rng: &mut impl Rng) -> Option<GeneratedQuery> {
        let type_name = self.pick_populated_type(rng)?;
        let type_info = self.schema.node_types.get(type_name)?;

        if type_info.attrs.is_empty() {
            return None;
        }

        let attr = &type_info.attrs[rng.gen_range(0..type_info.attrs.len())];
        let var = type_name.chars().next()?.to_lowercase().to_string();

        let statement = format!(
            "MATCH {}: {} RETURN {}.{}",
            var, type_name, var, attr.name
        );

        // Check if any nodes have this attribute as dynamic
        let has_dynamic = self.world
            .nodes_of_type(type_name)
            .any(|n| n.is_attr_dynamic(&attr.name));

        // If the attribute can be dynamic, use Count expectation (can't compare values)
        if has_dynamic {
            let count = self.world.nodes_of_type(type_name).count();
            return Some(GeneratedQuery {
                statement,
                required_setup: self.setup_statements(),
                expected: Expected::Count(count),
                trust_level: TrustLevel::Constructive,
                complexity: Complexity::simple(),
                tags: vec!["match".to_string(), "projection".to_string()],
            });
        }

        // Build expected rows for static attributes
        let rows: Vec<Row> = self.world
            .nodes_of_type(type_name)
            .map(|n| {
                let value = n.attrs.get(&attr.name)
                    .cloned()
                    .unwrap_or(Value::Null);
                Row { columns: vec![value] }
            })
            .collect();

        Some(GeneratedQuery {
            statement,
            required_setup: self.setup_statements(),
            expected: Expected::Rows(rows),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::simple(),
            tags: vec!["match".to_string(), "projection".to_string()],
        })
    }

    /// MATCH with impossible condition - should return 0 rows
    fn gen_match_empty(&self, rng: &mut impl Rng) -> Option<GeneratedQuery> {
        let type_name = self.pick_populated_type(rng)?;
        let var = type_name.chars().next()?.to_lowercase().to_string();

        // Use an impossible value
        let statement = format!(
            "MATCH {}: {} WHERE {}.impossible_attr_xyz = 999999 RETURN {}",
            var, type_name, var, var
        );

        Some(GeneratedQuery {
            statement,
            required_setup: self.setup_statements(),
            expected: Expected::Count(0),
            trust_level: TrustLevel::Axiomatic, // Axiomatically true - impossible condition
            complexity: Complexity::simple(),
            tags: vec!["match".to_string(), "negative".to_string()],
        })
    }

    /// Test count aggregate
    fn gen_match_count(&self, rng: &mut impl Rng) -> Option<GeneratedQuery> {
        let type_name = self.pick_populated_type(rng)?;
        let var = type_name.chars().next()?.to_lowercase().to_string();

        let statement = format!(
            "MATCH {}: {} RETURN count({})",
            var, type_name, var
        );

        let count = self.world.nodes_of_type(type_name).count();

        Some(GeneratedQuery {
            statement,
            required_setup: self.setup_statements(),
            expected: Expected::Rows(vec![Row { columns: vec![Value::Int(count as i64)] }]),
            trust_level: TrustLevel::Constructive,
            complexity: Complexity::medium(),
            tags: vec!["match".to_string(), "aggregate".to_string()],
        })
    }

    fn pick_populated_type(&self, rng: &mut impl Rng) -> Option<&String> {
        let populated: Vec<&String> = self.schema.node_types.keys()
            .filter(|t| self.world.nodes_by_type.get(*t).map(|v| !v.is_empty()).unwrap_or(false))
            .collect();

        if populated.is_empty() {
            None
        } else {
            Some(populated[rng.gen_range(0..populated.len())])
        }
    }

    fn setup_statements(&self) -> Vec<String> {
        let mut setup = Vec::new();

        // Generate SPAWN statements for all nodes
        for node in &self.world.nodes {
            let attrs_str = node.attrs.iter()
                .map(|(k, v)| format!("{} = {}", k, self.value_to_mew(v)))
                .collect::<Vec<_>>()
                .join(", ");

            let stmt = if attrs_str.is_empty() {
                format!("SPAWN {}: {}", node.var_name, node.type_name)
            } else {
                format!("SPAWN {}: {} {{ {} }}", node.var_name, node.type_name, attrs_str)
            };
            setup.push(stmt);
        }

        // Generate LINK statements for all edges
        for edge in &self.world.edges {
            let from_var = &self.world.nodes[edge.from_idx].var_name;
            let to_var = &self.world.nodes[edge.to_idx].var_name;
            let stmt = format!("LINK {}({}, {})", edge.edge_type, from_var, to_var);
            setup.push(stmt);
        }

        setup
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

enum QueryType {
    MatchAll,
    MatchByAttr,
    MatchWithProjection,
    MatchEmpty,
    MatchCount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaAnalyzer;
    use crate::world::WorldGenerator;
    use crate::config::TestConfig;
    use rand::SeedableRng;

    #[test]
    fn test_generate_queries() {
        let source = r#"
            node Person {
                name: String [required],
                age: Int
            }
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        let config = TestConfig::minimal();
        let mut world_gen = WorldGenerator::new(&schema, config.clone());
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let world = world_gen.generate(&mut rng).unwrap();
        let mut query_gen = QueryGenerator::new(&schema, &world);

        let queries = query_gen.generate(5, &mut rng).unwrap();

        assert!(!queries.is_empty());
        for q in &queries {
            assert!(q.statement.starts_with("MATCH"));
            assert!(matches!(q.trust_level, TrustLevel::Constructive | TrustLevel::Axiomatic));
        }
    }
}
