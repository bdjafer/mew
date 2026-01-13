//! Core types for the test generation framework

use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during test generation
#[derive(Error, Debug)]
pub enum TestGenError {
    #[error("Failed to parse ontology: {0}")]
    ParseError(String),
    #[error("Schema analysis failed: {0}")]
    SchemaError(String),
    #[error("World generation failed: {0}")]
    WorldError(String),
    #[error("Query generation failed: {0}")]
    QueryError(String),
    #[error("Mutation generation failed: {0}")]
    MutationError(String),
    #[error("Execution failed: {0}")]
    ExecutionError(String),
}

/// Trust level for test expectations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustLevel {
    /// Derived from mathematical axioms (e.g., empty set properties)
    Axiomatic,
    /// Built into generation process (we created it, we know the answer)
    Constructive,
    /// Computed by independent oracle (algebraic properties)
    Predicted,
    /// Statistical properties verified over many runs
    Statistical,
}

impl TrustLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustLevel::Axiomatic => "axiomatic",
            TrustLevel::Constructive => "constructive",
            TrustLevel::Predicted => "predicted",
            TrustLevel::Statistical => "statistical",
        }
    }
}

/// Complexity score for a test case (0-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Complexity(pub u8);

impl Complexity {
    pub fn simple() -> Self {
        Self(20)
    }

    pub fn medium() -> Self {
        Self(50)
    }

    pub fn complex() -> Self {
        Self(80)
    }

    pub fn extreme() -> Self {
        Self(100)
    }
}

/// A complete test suite
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub ontology_source: String,
    pub schema: AnalyzedSchema,
    pub world: WorldState,
    pub test_cases: Vec<TestCase>,
    pub seed: u64,
}

/// A single test case
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Unique identifier
    pub id: String,
    /// The MEW statement to execute
    pub statement: String,
    /// Setup statements that must run first
    pub setup: Vec<String>,
    /// Expected result
    pub expected: Expected,
    /// Trust level of the expected result
    pub trust_level: TrustLevel,
    /// Complexity score
    pub complexity: Complexity,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Expected result for a test case
#[derive(Debug, Clone)]
pub enum Expected {
    /// Query returns specific rows
    Rows(Vec<Row>),
    /// Query returns specific count
    Count(usize),
    /// Mutation succeeds with specific effect
    Success(MutationEffect),
    /// Statement should fail with error pattern
    Error(String),
    /// Property that should hold
    Property(PropertySpec),
}

/// A row of query results
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub columns: Vec<Value>,
}

/// A value in results
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Id(u64),
    /// A function call like now()
    FunctionCall(String),
}

impl Value {
    /// Check if this value is dynamic (changes each time it's evaluated)
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Value::FunctionCall(_))
    }
}

/// A generated value with its kind (static or dynamic)
#[derive(Debug, Clone)]
pub struct GeneratedValue {
    /// The value to use in SPAWN statements
    pub value: Value,
    /// Whether this value is dynamic (like now())
    pub is_dynamic: bool,
}

impl GeneratedValue {
    pub fn static_val(value: Value) -> Self {
        Self {
            value,
            is_dynamic: false,
        }
    }

    pub fn dynamic(value: Value) -> Self {
        Self {
            value,
            is_dynamic: true,
        }
    }
}

impl From<&mew_core::Value> for Value {
    fn from(v: &mew_core::Value) -> Self {
        match v {
            mew_core::Value::Null => Value::Null,
            mew_core::Value::Bool(b) => Value::Bool(*b),
            mew_core::Value::Int(i) => Value::Int(*i),
            mew_core::Value::Float(f) => Value::Float(*f),
            mew_core::Value::String(s) => Value::String(s.clone()),
            mew_core::Value::Timestamp(t) => Value::Int(*t),
            mew_core::Value::Duration(d) => Value::Int(*d),
            mew_core::Value::NodeRef(id) => Value::Id(id.0),
            mew_core::Value::EdgeRef(id) => Value::Id(id.0),
            mew_core::Value::List(items) => {
                // Convert list to string representation for testgen
                let formatted: Vec<String> = items
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect();
                Value::String(format!("[{}]", formatted.join(", ")))
            }
        }
    }
}

/// Effect of a mutation
#[derive(Debug, Clone)]
pub struct MutationEffect {
    pub nodes_created: usize,
    pub nodes_deleted: usize,
    pub edges_created: usize,
    pub edges_deleted: usize,
    pub attrs_modified: usize,
}

impl MutationEffect {
    pub fn spawn_one() -> Self {
        Self {
            nodes_created: 1,
            nodes_deleted: 0,
            edges_created: 0,
            edges_deleted: 0,
            attrs_modified: 0,
        }
    }

    pub fn link_one() -> Self {
        Self {
            nodes_created: 0,
            nodes_deleted: 0,
            edges_created: 1,
            edges_deleted: 0,
            attrs_modified: 0,
        }
    }
}

/// A property that should hold
#[derive(Debug, Clone)]
pub enum PropertySpec {
    /// Result count is within range
    CountInRange { min: usize, max: usize },
    /// All results satisfy predicate
    AllMatch { column: String, pattern: String },
    /// Idempotent: running twice gives same result
    Idempotent,
    /// Commutative: A then B = B then A
    Commutative { other: String },
}

/// Analyzed schema from an ontology
#[derive(Debug, Clone)]
pub struct AnalyzedSchema {
    pub node_types: HashMap<String, NodeTypeInfo>,
    pub edge_types: HashMap<String, EdgeTypeInfo>,
    pub type_aliases: HashMap<String, TypeAliasInfo>,
    pub constraints: Vec<ConstraintInfo>,
    pub rules: Vec<RuleInfo>,
}

/// Information about a type alias
#[derive(Debug, Clone)]
pub struct TypeAliasInfo {
    pub name: String,
    pub base_type: String,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub allowed_values: Option<Vec<Value>>,
}

/// Information about a node type
#[derive(Debug, Clone)]
pub struct NodeTypeInfo {
    pub name: String,
    pub attrs: Vec<AttrInfo>,
    pub parents: Vec<String>,
    /// Constraints that apply to this type
    pub applicable_constraints: Vec<String>,
}

/// Information about an attribute
#[derive(Debug, Clone)]
pub struct AttrInfo {
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
    pub required: bool,
    pub unique: bool,
    pub default: Option<Value>,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub allowed_values: Option<Vec<Value>>,
    pub pattern: Option<String>,
}

impl AttrInfo {
    /// Generate a valid value for this attribute
    pub fn generate_value(&self, rng: &mut impl rand::Rng) -> GeneratedValue {
        self.generate_value_with_aliases(rng, &HashMap::new())
    }

    /// Generate a valid value for this attribute, resolving type aliases
    pub fn generate_value_with_aliases(
        &self,
        rng: &mut impl rand::Rng,
        type_aliases: &HashMap<String, TypeAliasInfo>,
    ) -> GeneratedValue {
        // First check if this type is an alias
        if let Some(alias) = type_aliases.get(&self.type_name) {
            // Use alias constraints
            if let Some(ref values) = alias.allowed_values {
                let idx = rng.gen_range(0..values.len());
                return GeneratedValue::static_val(values[idx].clone());
            }

            // Create a virtual AttrInfo with the resolved base type and constraints
            let resolved = AttrInfo {
                name: self.name.clone(),
                type_name: alias.base_type.clone(),
                nullable: self.nullable,
                required: self.required,
                unique: self.unique,
                default: self.default.clone(),
                min: alias.min.clone().or(self.min.clone()),
                max: alias.max.clone().or(self.max.clone()),
                allowed_values: alias.allowed_values.clone().or(self.allowed_values.clone()),
                pattern: self.pattern.clone(),
            };
            return resolved.generate_value_with_aliases(rng, type_aliases);
        }

        // If we have allowed values, pick one
        if let Some(ref values) = self.allowed_values {
            let idx = rng.gen_range(0..values.len());
            return GeneratedValue::static_val(values[idx].clone());
        }

        // Generate based on type
        match self.type_name.as_str() {
            "String" => {
                // Generate a random string
                let len = rng.gen_range(3..15);
                let s: String = (0..len)
                    .map(|_| rng.gen_range(b'a'..=b'z') as char)
                    .collect();
                GeneratedValue::static_val(Value::String(s))
            }
            "Int" => {
                let min = self
                    .min
                    .as_ref()
                    .and_then(|v| {
                        if let Value::Int(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                let max = self
                    .max
                    .as_ref()
                    .and_then(|v| {
                        if let Value::Int(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(100);
                GeneratedValue::static_val(Value::Int(rng.gen_range(min..=max)))
            }
            "Float" => {
                let min = self
                    .min
                    .as_ref()
                    .and_then(|v| match v {
                        Value::Float(f) => Some(*f),
                        Value::Int(i) => Some(*i as f64),
                        _ => None,
                    })
                    .unwrap_or(0.0);
                let max = self
                    .max
                    .as_ref()
                    .and_then(|v| match v {
                        Value::Float(f) => Some(*f),
                        Value::Int(i) => Some(*i as f64),
                        _ => None,
                    })
                    .unwrap_or(100.0);
                GeneratedValue::static_val(Value::Float(rng.gen_range(min..=max)))
            }
            "Bool" => GeneratedValue::static_val(Value::Bool(rng.gen_bool(0.5))),
            "Timestamp" => GeneratedValue::dynamic(Value::FunctionCall("now".to_string())),
            "Duration" => GeneratedValue::static_val(Value::Int(rng.gen_range(0..86400000))), // Up to 1 day in ms
            _ => GeneratedValue::static_val(Value::Null),
        }
    }
}

/// Information about an edge type
#[derive(Debug, Clone)]
pub struct EdgeTypeInfo {
    pub name: String,
    pub params: Vec<(String, String)>, // (param_name, type_name)
    pub attrs: Vec<AttrInfo>,
    pub acyclic: bool,
    pub unique: bool,
    pub symmetric: bool,
    pub no_self: bool,
}

/// Information about a constraint
#[derive(Debug, Clone)]
pub struct ConstraintInfo {
    pub name: String,
    pub on_type: String,
    pub description: String,
}

/// Information about a rule
#[derive(Debug, Clone)]
pub struct RuleInfo {
    pub name: String,
    pub on_type: String,
    pub auto: bool,
}

/// The generated world state
#[derive(Debug, Clone)]
pub struct WorldState {
    pub nodes: Vec<GeneratedNode>,
    pub edges: Vec<GeneratedEdge>,
    /// Index: type_name -> node indices
    pub nodes_by_type: HashMap<String, Vec<usize>>,
    /// Index: (from_idx, to_idx) -> edge indices
    pub edges_by_endpoints: HashMap<(usize, usize), Vec<usize>>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            nodes_by_type: HashMap::new(),
            edges_by_endpoints: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: GeneratedNode) -> usize {
        let idx = self.nodes.len();
        self.nodes_by_type
            .entry(node.type_name.clone())
            .or_default()
            .push(idx);
        self.nodes.push(node);
        idx
    }

    pub fn add_edge(&mut self, edge: GeneratedEdge) -> usize {
        let idx = self.edges.len();
        let key = (edge.from_idx, edge.to_idx);
        self.edges_by_endpoints.entry(key).or_default().push(idx);
        self.edges.push(edge);
        idx
    }

    pub fn nodes_of_type(&self, type_name: &str) -> impl Iterator<Item = &GeneratedNode> {
        self.nodes_by_type
            .get(type_name)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|&i| &self.nodes[i]))
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// A generated node
#[derive(Debug, Clone)]
pub struct GeneratedNode {
    pub var_name: String,
    pub type_name: String,
    pub attrs: HashMap<String, Value>,
    /// Names of attributes that have dynamic values (like now())
    pub dynamic_attrs: HashSet<String>,
}

impl GeneratedNode {
    /// Check if an attribute has a dynamic value
    pub fn is_attr_dynamic(&self, attr_name: &str) -> bool {
        self.dynamic_attrs.contains(attr_name)
    }

    /// Check if this node has any dynamic attributes
    pub fn has_dynamic_attrs(&self) -> bool {
        !self.dynamic_attrs.is_empty()
    }
}

/// A generated edge
#[derive(Debug, Clone)]
pub struct GeneratedEdge {
    pub var_name: Option<String>,
    pub edge_type: String,
    pub from_idx: usize,
    pub to_idx: usize,
    pub attrs: HashMap<String, Value>,
    /// Names of attributes that have dynamic values
    pub dynamic_attrs: HashSet<String>,
}

/// Generated query with expected results
#[derive(Debug, Clone)]
pub struct GeneratedQuery {
    pub statement: String,
    pub required_setup: Vec<String>,
    pub expected: Expected,
    pub trust_level: TrustLevel,
    pub complexity: Complexity,
    pub tags: Vec<String>,
}

/// Generated mutation with expected results
#[derive(Debug, Clone)]
pub struct GeneratedMutation {
    pub statement: String,
    pub required_setup: Vec<String>,
    pub expected: Expected,
    pub trust_level: TrustLevel,
    pub complexity: Complexity,
    pub tags: Vec<String>,
}

/// Test execution result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub passed: bool,
    pub expected: Expected,
    pub actual: ActualResult,
    pub duration_us: u64,
    pub trust_level: TrustLevel,
}

/// Actual result from execution
#[derive(Debug, Clone)]
pub enum ActualResult {
    Rows(Vec<Row>),
    Count(usize),
    Success,
    Error(String),
}

/// Test suite execution summary
#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub by_trust_level: HashMap<TrustLevel, (usize, usize)>, // (passed, total)
    pub by_complexity: HashMap<u8, (usize, usize)>,
    pub by_tag: HashMap<String, (usize, usize)>,
    pub total_duration_us: u64,
}
