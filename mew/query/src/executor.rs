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
        // Plan the walk
        let planner = QueryPlanner::new(self.registry);
        let plan = planner.plan_walk(stmt)?;

        // Execute the plan
        self.execute_plan(&plan, None)
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

        graph.create_edge(knows_type_id, vec![alice.into(), bob.into()], attrs! {});
        graph.create_edge(knows_type_id, vec![bob.into(), carol.into()], attrs! {});

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
            return_type: mew_parser::WalkReturnType::Path,
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
            return_type: mew_parser::WalkReturnType::Path,
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

        graph.create_edge(knows_type_id, vec![alice.into(), bob.into()], attrs! {});
        graph.create_edge(knows_type_id, vec![bob.into(), alice.into()], attrs! {});

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
            return_type: mew_parser::WalkReturnType::Path,
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
            results.len() >= 1,
            "Expected at least 1 result, got {}",
            results.len()
        );
        assert!(
            results.len() <= 2,
            "Expected at most 2 results (cycle should be cut), got {}",
            results.len()
        );
    }
}
