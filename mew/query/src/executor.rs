//! Query execution.

use mew_core::Value;
use mew_graph::Graph;
use mew_parser::{MatchStmt, WalkStmt};
use mew_pattern::{Bindings, Evaluator, Matcher};
use mew_registry::Registry;

use crate::operators::OperatorContext;
use crate::plan::{QueryPlan, QueryPlanner};
use crate::result::{QueryResults, QueryRow};
use crate::QueryResult;

/// Query executor.
pub struct QueryExecutor<'r, 'g> {
    #[allow(dead_code)]
    registry: &'r Registry,
    graph: &'g Graph,
    #[allow(dead_code)]
    matcher: Matcher<'r, 'g>,
    evaluator: Evaluator<'r>,
}

impl<'r, 'g> QueryExecutor<'r, 'g> {
    /// Create a new executor.
    pub fn new(registry: &'r Registry, graph: &'g Graph) -> Self {
        Self {
            registry,
            graph,
            matcher: Matcher::new(registry, graph),
            evaluator: Evaluator::new(registry),
        }
    }

    /// Execute a MATCH statement.
    pub fn execute_match(&self, stmt: &MatchStmt) -> QueryResult<QueryResults> {
        // Plan the query
        let planner = QueryPlanner::new(self.registry);
        let plan = planner.plan_match(stmt)?;

        // Execute the plan
        self.execute_plan(&plan, None)
    }

    /// Execute a WALK statement.
    pub fn execute_walk(&self, stmt: &WalkStmt) -> QueryResult<QueryResults> {
        self.execute_walk_with_bindings(stmt, None)
    }

    /// Execute a WALK statement with initial bindings (for resolving ID refs).
    pub fn execute_walk_with_bindings(
        &self,
        stmt: &WalkStmt,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<QueryResults> {
        // Plan the walk
        let planner = QueryPlanner::new(self.registry);
        let plan = planner.plan_walk(stmt)?;

        // Execute the plan
        self.execute_plan(&plan, initial_bindings)
    }

    /// Execute a MATCH...WALK compound statement.
    pub fn execute_match_walk(
        &self,
        stmt: &mew_parser::MatchWalkStmt,
    ) -> QueryResult<QueryResults> {
        // Plan the compound statement
        let planner = QueryPlanner::new(self.registry);
        let (pattern_plan, walk_plan, where_clause) = planner.plan_match_walk(stmt)?;

        // Execute pattern to get bindings
        let ctx = OperatorContext::new(self.registry, self.graph, &self.evaluator);
        let pattern_results = ctx.execute_op(&pattern_plan, None)?;

        // Filter by WHERE clause if present
        let filtered_bindings: Vec<_> = if let Some(ref cond) = where_clause {
            pattern_results
                .into_iter()
                .filter(|(bindings, _)| {
                    self.evaluator
                        .eval_bool(cond, bindings, self.graph)
                        .unwrap_or(false)
                })
                .map(|(bindings, _)| bindings)
                .collect()
        } else {
            pattern_results
                .into_iter()
                .map(|(bindings, _)| bindings)
                .collect()
        };

        // For each binding, execute the WALK
        let mut results = QueryResults::with_columns(walk_plan.columns.clone());

        for binding in filtered_bindings {
            let walk_results = self.execute_plan(&walk_plan, Some(&binding))?;
            for row in walk_results.iter() {
                results.push(row.clone());
            }
        }

        Ok(results)
    }

    /// Execute a MATCH statement using initial bindings.
    pub fn execute_match_with_bindings(
        &self,
        stmt: &MatchStmt,
        initial_bindings: &Bindings,
    ) -> QueryResult<QueryResults> {
        let planner = QueryPlanner::new(self.registry);
        let plan = planner.plan_match(stmt)?;

        self.execute_plan(&plan, Some(initial_bindings))
    }

    /// Execute a query plan.
    pub fn execute_plan(
        &self,
        plan: &QueryPlan,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<QueryResults> {
        // Create operator context and execute
        let ctx = OperatorContext::new(self.registry, self.graph, &self.evaluator);
        let bindings_list = ctx.execute_op(&plan.root, initial_bindings)?;

        let filtered_bindings = if let Some(initial) = initial_bindings {
            bindings_list
                .into_iter()
                .filter(|(bindings, _)| {
                    initial.iter().all(|(name, binding)| {
                        bindings
                            .get(name)
                            .map(|existing| existing == binding)
                            .unwrap_or(true)
                    })
                })
                .collect()
        } else {
            bindings_list
        };

        // Convert bindings to result rows
        let mut results = QueryResults::with_columns(plan.columns.clone());

        for (bindings, values) in filtered_bindings {
            let mut row = QueryRow::new();

            // If we have projected values, use those
            if !values.is_empty() {
                for (name, value) in plan.columns.iter().zip(values.iter()) {
                    row.push(name.clone(), value.clone());
                }
            } else {
                // Otherwise use bindings
                for name in &plan.columns {
                    let value = bindings
                        .get(name)
                        .map(|b| b.to_value())
                        .unwrap_or(Value::Null);
                    row.push(name.clone(), value);
                }
            }

            results.push(row);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;
    use mew_parser::{NodePattern, PatternElem, Projection, ReturnClause, Span};
    use mew_registry::{AttrDef, RegistryBuilder};

    fn test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .attr(AttrDef::new("priority", "Int"))
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_execute_simple_match() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        graph.create_node(
            task_type_id,
            attrs! { "title" => "Task A", "priority" => 1 },
        );
        graph.create_node(
            task_type_id,
            attrs! { "title" => "Task B", "priority" => 2 },
        );
        graph.create_node(
            task_type_id,
            attrs! { "title" => "Task C", "priority" => 3 },
        );

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH t: Task RETURN t
        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "t".to_string(),
                type_name: "Task".to_string(),
                span: Span::default(),
            })],
            where_clause: None,
            optional_matches: vec![],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![Projection {
                    expr: mew_parser::Expr::Var("t".to_string(), Span::default()),
                    alias: None,
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            order_by: None,
            limit: None,
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match(&stmt).unwrap();

        // THEN
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_execute_match_with_limit() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        for i in 1..=10 {
            graph.create_node(task_type_id, attrs! { "title" => format!("Task {}", i) });
        }

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH t: Task RETURN t LIMIT 5
        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "t".to_string(),
                type_name: "Task".to_string(),
                span: Span::default(),
            })],
            where_clause: None,
            optional_matches: vec![],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![Projection {
                    expr: mew_parser::Expr::Var("t".to_string(), Span::default()),
                    alias: None,
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            order_by: None,
            limit: Some(5),
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match(&stmt).unwrap();

        // THEN
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_execute_match_with_filter() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        graph.create_node(task_type_id, attrs! { "title" => "Low", "priority" => 1 });
        graph.create_node(task_type_id, attrs! { "title" => "High", "priority" => 10 });
        graph.create_node(
            task_type_id,
            attrs! { "title" => "Medium", "priority" => 5 },
        );

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH t: Task WHERE t.priority > 3 RETURN t
        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "t".to_string(),
                type_name: "Task".to_string(),
                span: Span::default(),
            })],
            where_clause: Some(mew_parser::Expr::BinaryOp(
                mew_parser::BinaryOp::Gt,
                Box::new(mew_parser::Expr::AttrAccess(
                    Box::new(mew_parser::Expr::Var("t".to_string(), Span::default())),
                    "priority".to_string(),
                    Span::default(),
                )),
                Box::new(mew_parser::Expr::Literal(mew_parser::Literal {
                    kind: mew_parser::LiteralKind::Int(3),
                    span: Span::default(),
                })),
                Span::default(),
            )),
            optional_matches: vec![],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![Projection {
                    expr: mew_parser::Expr::Var("t".to_string(), Span::default()),
                    alias: None,
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            order_by: None,
            limit: None,
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match(&stmt).unwrap();

        // THEN - should get High (10) and Medium (5)
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_execute_match_with_sort() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        graph.create_node(task_type_id, attrs! { "title" => "C", "priority" => 3 });
        graph.create_node(task_type_id, attrs! { "title" => "A", "priority" => 1 });
        graph.create_node(task_type_id, attrs! { "title" => "B", "priority" => 2 });

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH t: Task RETURN t.title ORDER BY t.priority
        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "t".to_string(),
                type_name: "Task".to_string(),
                span: Span::default(),
            })],
            where_clause: None,
            optional_matches: vec![],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![Projection {
                    expr: mew_parser::Expr::AttrAccess(
                        Box::new(mew_parser::Expr::Var("t".to_string(), Span::default())),
                        "title".to_string(),
                        Span::default(),
                    ),
                    alias: Some("title".to_string()),
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            order_by: Some(vec![mew_parser::OrderTerm {
                expr: mew_parser::Expr::AttrAccess(
                    Box::new(mew_parser::Expr::Var("t".to_string(), Span::default())),
                    "priority".to_string(),
                    Span::default(),
                ),
                direction: mew_parser::OrderDirection::Asc,
                span: Span::default(),
            }]),
            limit: None,
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match(&stmt).unwrap();

        // THEN - should be sorted by priority: A, B, C
        assert_eq!(results.len(), 3);
        let titles: Vec<_> = results
            .iter()
            .map(|r| r.get_by_name("title").cloned().unwrap_or(Value::Null))
            .collect();
        assert_eq!(titles[0], Value::String("A".to_string()));
        assert_eq!(titles[1], Value::String("B".to_string()));
        assert_eq!(titles[2], Value::String("C".to_string()));
    }

    // ==================== WALK TESTS ====================

    fn walk_test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("knows")
            .param("from", "Person")
            .param("to", "Person")
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_execute_walk_simple_chain() {
        // GIVEN - A -> B -> C chain
        let registry = walk_test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let knows_type_id = registry.get_edge_type_id("knows").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let bob = graph.create_node(person_type_id, attrs! { "name" => "Bob" });
        let carol = graph.create_node(person_type_id, attrs! { "name" => "Carol" });

        let _ = graph.create_edge(knows_type_id, vec![alice.into(), bob.into()], attrs! {});
        let _ = graph.create_edge(knows_type_id, vec![bob.into(), carol.into()], attrs! {});

        let executor = QueryExecutor::new(&registry, &graph);

        // WALK FROM #alice FOLLOW knows RETURN PATH
        let stmt = WalkStmt {
            from: mew_parser::Expr::Var("start".to_string(), Span::default()),
            follow: vec![mew_parser::FollowClause {
                edge_types: vec!["knows".to_string()],
                direction: mew_parser::WalkDirection::Outbound,
                min_depth: Some(1),
                max_depth: Some(3),
                span: Span::default(),
            }],
            until: None,
            return_type: mew_parser::WalkReturnType::Path { alias: None },
            span: Span::default(),
        };

        // Create initial bindings with start node
        let mut initial = Bindings::new();
        initial.insert("start", mew_pattern::Binding::Node(alice));

        let planner = crate::plan::QueryPlanner::new(&registry);
        let plan = planner.plan_walk(&stmt).unwrap();

        // WHEN
        let results = executor.execute_plan(&plan, Some(&initial)).unwrap();

        // THEN - should find Bob (depth 1) and Carol (depth 2)
        assert!(
            results.len() >= 2,
            "Expected at least 2 results, got {}",
            results.len()
        );
    }

    #[test]
    fn test_execute_walk_no_edges() {
        // GIVEN - isolated node
        let registry = walk_test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });

        let executor = QueryExecutor::new(&registry, &graph);

        // WALK FROM #alice FOLLOW knows RETURN PATH
        let stmt = WalkStmt {
            from: mew_parser::Expr::Var("start".to_string(), Span::default()),
            follow: vec![mew_parser::FollowClause {
                edge_types: vec!["knows".to_string()],
                direction: mew_parser::WalkDirection::Outbound,
                min_depth: Some(1),
                max_depth: Some(3),
                span: Span::default(),
            }],
            until: None,
            return_type: mew_parser::WalkReturnType::Path { alias: None },
            span: Span::default(),
        };

        let mut initial = Bindings::new();
        initial.insert("start", mew_pattern::Binding::Node(alice));

        let planner = crate::plan::QueryPlanner::new(&registry);
        let plan = planner.plan_walk(&stmt).unwrap();

        // WHEN
        let results = executor.execute_plan(&plan, Some(&initial)).unwrap();

        // THEN - no paths found (min_depth is 1)
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_execute_walk_with_cycle() {
        // GIVEN - A -> B -> A cycle
        let registry = walk_test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let knows_type_id = registry.get_edge_type_id("knows").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let bob = graph.create_node(person_type_id, attrs! { "name" => "Bob" });

        let _ = graph.create_edge(knows_type_id, vec![alice.into(), bob.into()], attrs! {});
        let _ = graph.create_edge(knows_type_id, vec![bob.into(), alice.into()], attrs! {});

        let executor = QueryExecutor::new(&registry, &graph);

        // WALK FROM #alice FOLLOW knows RETURN PATH
        let stmt = WalkStmt {
            from: mew_parser::Expr::Var("start".to_string(), Span::default()),
            follow: vec![mew_parser::FollowClause {
                edge_types: vec!["knows".to_string()],
                direction: mew_parser::WalkDirection::Outbound,
                min_depth: Some(1),
                max_depth: Some(10), // Should not infinite loop
                span: Span::default(),
            }],
            until: None,
            return_type: mew_parser::WalkReturnType::Path { alias: None },
            span: Span::default(),
        };

        let mut initial = Bindings::new();
        initial.insert("start", mew_pattern::Binding::Node(alice));

        let planner = crate::plan::QueryPlanner::new(&registry);
        let plan = planner.plan_walk(&stmt).unwrap();

        // WHEN
        let results = executor.execute_plan(&plan, Some(&initial)).unwrap();

        // THEN - should terminate and find Bob (cycle detection prevents revisiting)
        assert!(
            !results.is_empty(),
            "Expected at least 1 result, got {}",
            results.len()
        );
        assert!(
            results.len() <= 2,
            "Expected at most 2 results (cycle should be cut), got {}",
            results.len()
        );
    }

    // ==================== OPTIONAL MATCH TESTS ====================

    fn optional_match_test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_type("Pet")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("owns")
            .param("person", "Person")
            .param("pet", "Pet")
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_optional_match_with_match() {
        // GIVEN - Alice owns Fluffy, Bob has no pet
        let registry = optional_match_test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let pet_type_id = registry.get_type_id("Pet").unwrap();
        let owns_type_id = registry.get_edge_type_id("owns").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let _bob = graph.create_node(person_type_id, attrs! { "name" => "Bob" });
        let fluffy = graph.create_node(pet_type_id, attrs! { "name" => "Fluffy" });

        graph
            .create_edge(owns_type_id, vec![alice.into(), fluffy.into()], attrs! {})
            .unwrap();

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH p: Person OPTIONAL MATCH pet: Pet, owns(p, pet) RETURN p.name, pet.name
        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "p".to_string(),
                type_name: "Person".to_string(),
                span: Span::default(),
            })],
            where_clause: None,
            optional_matches: vec![mew_parser::OptionalMatch {
                pattern: vec![
                    PatternElem::Node(NodePattern {
                        var: "pet".to_string(),
                        type_name: "Pet".to_string(),
                        span: Span::default(),
                    }),
                    PatternElem::Edge(mew_parser::EdgePattern {
                        edge_type: "owns".to_string(),
                        targets: vec!["p".to_string(), "pet".to_string()],
                        alias: None,
                        transitive: None,
                        span: Span::default(),
                    }),
                ],
                where_clause: None,
                span: Span::default(),
            }],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![
                    Projection {
                        expr: mew_parser::Expr::AttrAccess(
                            Box::new(mew_parser::Expr::Var("p".to_string(), Span::default())),
                            "name".to_string(),
                            Span::default(),
                        ),
                        alias: Some("person_name".to_string()),
                        span: Span::default(),
                    },
                    Projection {
                        expr: mew_parser::Expr::AttrAccess(
                            Box::new(mew_parser::Expr::Var("pet".to_string(), Span::default())),
                            "name".to_string(),
                            Span::default(),
                        ),
                        alias: Some("pet_name".to_string()),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            },
            order_by: None,
            limit: None,
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match(&stmt).unwrap();

        // THEN - should get 2 rows: Alice with Fluffy, Bob with NULL
        assert_eq!(results.len(), 2, "Expected 2 rows, got {}", results.len());

        // Check that we have both Alice and Bob
        let person_names: Vec<_> = results
            .iter()
            .filter_map(|r| r.get_by_name("person_name").cloned())
            .collect();
        assert!(person_names.contains(&Value::String("Alice".to_string())));
        assert!(person_names.contains(&Value::String("Bob".to_string())));
    }

    #[test]
    fn test_optional_match_no_match() {
        // GIVEN - Bob has no pet
        let registry = optional_match_test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();

        graph.create_node(person_type_id, attrs! { "name" => "Bob" });

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH p: Person OPTIONAL MATCH pet: Pet, owns(p, pet) RETURN p.name, pet.name
        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "p".to_string(),
                type_name: "Person".to_string(),
                span: Span::default(),
            })],
            where_clause: None,
            optional_matches: vec![mew_parser::OptionalMatch {
                pattern: vec![
                    PatternElem::Node(NodePattern {
                        var: "pet".to_string(),
                        type_name: "Pet".to_string(),
                        span: Span::default(),
                    }),
                    PatternElem::Edge(mew_parser::EdgePattern {
                        edge_type: "owns".to_string(),
                        targets: vec!["p".to_string(), "pet".to_string()],
                        alias: None,
                        transitive: None,
                        span: Span::default(),
                    }),
                ],
                where_clause: None,
                span: Span::default(),
            }],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![
                    Projection {
                        expr: mew_parser::Expr::AttrAccess(
                            Box::new(mew_parser::Expr::Var("p".to_string(), Span::default())),
                            "name".to_string(),
                            Span::default(),
                        ),
                        alias: Some("person_name".to_string()),
                        span: Span::default(),
                    },
                    Projection {
                        expr: mew_parser::Expr::AttrAccess(
                            Box::new(mew_parser::Expr::Var("pet".to_string(), Span::default())),
                            "name".to_string(),
                            Span::default(),
                        ),
                        alias: Some("pet_name".to_string()),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            },
            order_by: None,
            limit: None,
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match(&stmt).unwrap();

        // THEN - should get 1 row: Bob with NULL pet
        assert_eq!(results.len(), 1, "Expected 1 row, got {}", results.len());

        let row = &results.iter().next().unwrap();
        assert_eq!(
            row.get_by_name("person_name").cloned(),
            Some(Value::String("Bob".to_string()))
        );
        assert_eq!(row.get_by_name("pet_name").cloned(), Some(Value::Null));
    }

    // ==================== MATCH-WALK TESTS ====================

    #[test]
    fn test_match_walk_compound() {
        // GIVEN - A -> B -> C chain
        let registry = walk_test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let knows_type_id = registry.get_edge_type_id("knows").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let bob = graph.create_node(person_type_id, attrs! { "name" => "Bob" });
        let carol = graph.create_node(person_type_id, attrs! { "name" => "Carol" });

        graph
            .create_edge(knows_type_id, vec![alice.into(), bob.into()], attrs! {})
            .unwrap();
        graph
            .create_edge(knows_type_id, vec![bob.into(), carol.into()], attrs! {})
            .unwrap();

        let executor = QueryExecutor::new(&registry, &graph);

        // MATCH p: Person WHERE p.name = "Alice" WALK FROM p FOLLOW knows RETURN NODES
        let stmt = mew_parser::MatchWalkStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "p".to_string(),
                type_name: "Person".to_string(),
                span: Span::default(),
            })],
            where_clause: Some(mew_parser::Expr::BinaryOp(
                mew_parser::BinaryOp::Eq,
                Box::new(mew_parser::Expr::AttrAccess(
                    Box::new(mew_parser::Expr::Var("p".to_string(), Span::default())),
                    "name".to_string(),
                    Span::default(),
                )),
                Box::new(mew_parser::Expr::Literal(mew_parser::Literal {
                    kind: mew_parser::LiteralKind::String("Alice".to_string()),
                    span: Span::default(),
                })),
                Span::default(),
            )),
            walk: WalkStmt {
                from: mew_parser::Expr::Var("p".to_string(), Span::default()),
                follow: vec![mew_parser::FollowClause {
                    edge_types: vec!["knows".to_string()],
                    direction: mew_parser::WalkDirection::Outbound,
                    min_depth: Some(1),
                    max_depth: Some(3),
                    span: Span::default(),
                }],
                until: None,
                return_type: mew_parser::WalkReturnType::Nodes { alias: None },
                span: Span::default(),
            },
            span: Span::default(),
        };

        // WHEN
        let results = executor.execute_match_walk(&stmt).unwrap();

        // THEN - should find Bob and Carol
        assert!(
            results.len() >= 2,
            "Expected at least 2 results (Bob and Carol), got {}",
            results.len()
        );
    }
}
