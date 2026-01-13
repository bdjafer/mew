//! Query execution.

use mew_core::Value;
use mew_graph::Graph;
use mew_parser::{MatchStmt, WalkStmt};
use mew_pattern::{Bindings, Evaluator, Matcher};
use mew_registry::Registry;

use crate::plan::{PlanOp, QueryPlan, QueryPlanner};
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
        // Get all matching bindings using the pattern matcher
        let bindings_list = self.execute_op(&plan.root, initial_bindings)?;

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

    /// Execute a single plan operator.
    fn execute_op(
        &self,
        op: &PlanOp,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        match op {
            PlanOp::NodeScan { var, type_id } => {
                let mut results = Vec::new();

                for node_id in self.graph.nodes_by_type(*type_id) {
                    let mut bindings = initial_bindings.cloned().unwrap_or_default();
                    if let Some(existing) = bindings.get(var) {
                        if existing.as_node() != Some(node_id) {
                            continue;
                        }
                    }
                    bindings.insert(var, mew_pattern::Binding::Node(node_id));
                    results.push((bindings, Vec::new()));
                }

                Ok(results)
            }

            PlanOp::IndexScan {
                var,
                type_id,
                attr,
                value,
            } => {
                // Evaluate the search value
                let search_val = self.evaluator.eval(value, &Bindings::new(), self.graph)?;

                let mut results = Vec::new();

                // Use attribute index
                if !matches!(search_val, Value::Null) {
                    for node_id in self.graph.nodes_by_attr(*type_id, attr, &search_val) {
                        let mut bindings = initial_bindings.cloned().unwrap_or_default();
                        if let Some(existing) = bindings.get(var) {
                            if existing.as_node() != Some(node_id) {
                                continue;
                            }
                        }
                        bindings.insert(var, mew_pattern::Binding::Node(node_id));
                        results.push((bindings, Vec::new()));
                    }
                } else {
                    // Fall back to scan with filter
                    for node_id in self.graph.nodes_by_type(*type_id) {
                        if let Some(node) = self.graph.get_node(node_id) {
                            if let Some(attr_val) = node.get_attr(attr) {
                                if self.values_equal(attr_val, &search_val) {
                                    let mut bindings =
                                        initial_bindings.cloned().unwrap_or_default();
                                    if let Some(existing) = bindings.get(var) {
                                        if existing.as_node() != Some(node_id) {
                                            continue;
                                        }
                                    }
                                    bindings.insert(var, mew_pattern::Binding::Node(node_id));
                                    results.push((bindings, Vec::new()));
                                }
                            }
                        }
                    }
                }

                Ok(results)
            }

            PlanOp::EdgeJoin {
                input,
                edge_type_id,
                from_vars,
                edge_var,
            } => {
                let input_results = self.execute_op(input, initial_bindings)?;
                let mut results = Vec::new();

                for (bindings, _) in input_results {
                    // Get the source node from the first variable
                    if from_vars.is_empty() {
                        results.push((bindings, Vec::new()));
                        continue;
                    }

                    let source_binding = bindings.get(&from_vars[0]);
                    let source_id = source_binding.and_then(|b| b.as_node());

                    if let Some(source_id) = source_id {
                        // Find matching edges
                        for edge_id in self.graph.edges_from(source_id, Some(*edge_type_id)) {
                            if let Some(edge) = self.graph.get_edge(edge_id) {
                                // Check that all target variables match
                                let mut all_match = true;
                                for (i, var) in from_vars.iter().enumerate() {
                                    if let Some(binding) = bindings.get(var) {
                                        if let Some(expected_id) = binding.as_node() {
                                            if i < edge.targets.len()
                                                && edge.targets[i].as_node() != Some(expected_id) {
                                                    all_match = false;
                                                    break;
                                                }
                                        }
                                    }
                                }

                                if all_match {
                                    let mut new_bindings = bindings.clone();
                                    if let Some(alias) = edge_var {
                                        new_bindings
                                            .insert(alias, mew_pattern::Binding::Edge(edge_id));
                                    }
                                    results.push((new_bindings, Vec::new()));
                                }
                            }
                        }
                    }
                }

                Ok(results)
            }

            PlanOp::Filter { input, condition } => {
                let input_results = self.execute_op(input, initial_bindings)?;
                let mut results = Vec::new();

                for (bindings, values) in input_results {
                    let result = self.evaluator.eval_bool(condition, &bindings, self.graph)?;
                    if result {
                        results.push((bindings, values));
                    }
                }

                Ok(results)
            }

            PlanOp::Project { input, projections } => {
                let input_results = self.execute_op(input, initial_bindings)?;
                let mut results = Vec::new();

                for (bindings, _) in input_results {
                    let mut values = Vec::new();

                    for (_name, expr) in projections {
                        let val = self.evaluator.eval(expr, &bindings, self.graph)?;
                        values.push(val);
                    }

                    results.push((bindings, values));
                }

                Ok(results)
            }

            PlanOp::Sort { input, order_by } => {
                let mut results = self.execute_op(input, initial_bindings)?;

                // Sort by the order expressions
                results.sort_by(|(a_bindings, _a_values), (b_bindings, _b_values)| {
                    for (expr, ascending) in order_by {
                        // Evaluate the expression for both rows
                        let a_val = self.evaluator.eval(expr, a_bindings, self.graph).ok();
                        let b_val = self.evaluator.eval(expr, b_bindings, self.graph).ok();

                        let cmp = self.compare_values(&a_val, &b_val);
                        if cmp != std::cmp::Ordering::Equal {
                            return if *ascending { cmp } else { cmp.reverse() };
                        }
                    }
                    std::cmp::Ordering::Equal
                });

                Ok(results)
            }

            PlanOp::LimitOffset {
                input,
                limit,
                offset,
            } => {
                let results = self.execute_op(input, initial_bindings)?;

                let start = offset.unwrap_or(0) as usize;
                let end = limit.map(|l| start + l as usize).unwrap_or(results.len());

                Ok(results.into_iter().skip(start).take(end - start).collect())
            }

            PlanOp::Aggregate {
                input,
                group_by,
                aggregates,
            } => {
                let results = self.execute_op(input, initial_bindings)?;

                if group_by.is_empty() && results.is_empty() {
                    // Empty input with no grouping returns single row with defaults
                    let mut values = Vec::new();
                    for (_, kind, _) in aggregates {
                        values.push(match kind {
                            crate::plan::AggregateKind::Count => Value::Int(0),
                            _ => Value::Null,
                        });
                    }
                    return Ok(vec![(Bindings::new(), values)]);
                }

                // Group by keys (using string serialization since Value doesn't impl Hash)
                let mut groups: std::collections::HashMap<String, Vec<(Bindings, Vec<Value>)>> =
                    std::collections::HashMap::new();

                for (bindings, values) in results {
                    let key: String = group_by
                        .iter()
                        .map(|e| {
                            let v = self
                                .evaluator
                                .eval(e, &bindings, self.graph)
                                .unwrap_or(Value::Null);
                            format!("{:?}", v)
                        })
                        .collect::<Vec<_>>()
                        .join("|");

                    groups.entry(key).or_default().push((bindings, values));
                }

                // Compute aggregates for each group
                let mut output = Vec::new();

                for (_, group) in groups {
                    let first_bindings = group.first().map(|(b, _)| b.clone()).unwrap_or_default();
                    let mut agg_values = Vec::new();

                    for (_, kind, expr) in aggregates {
                        let agg_val = self.compute_aggregate(*kind, &group, expr)?;
                        agg_values.push(agg_val);
                    }

                    output.push((first_bindings, agg_values));
                }

                Ok(output)
            }

            PlanOp::CrossJoin { left, right } => {
                let left_results = self.execute_op(left, initial_bindings)?;
                let right_results = self.execute_op(right, initial_bindings)?;

                let mut results = Vec::new();

                for (left_bindings, left_values) in &left_results {
                    for (right_bindings, right_values) in &right_results {
                        let mut merged = left_bindings.clone();
                        for (k, v) in right_bindings.iter() {
                            merged.insert(k, v.clone());
                        }
                        let mut merged_values = left_values.clone();
                        merged_values.extend(right_values.clone());
                        results.push((merged, merged_values));
                    }
                }

                Ok(results)
            }

            PlanOp::TransitiveClosure {
                start_var: _,
                start_expr,
                edge_types,
                min_depth,
                max_depth,
                direction,
            } => {
                // Evaluate the start expression to get the starting node(s)
                let init_bindings = initial_bindings.cloned().unwrap_or_default();
                let start_value = self.evaluator.eval(start_expr, &init_bindings, self.graph)?;

                // Get the starting node ID
                let start_node = match &start_value {
                    Value::NodeRef(id) => Some(*id),
                    _ => None,
                };

                let Some(start_id) = start_node else {
                    return Ok(Vec::new());
                };

                // BFS traversal with depth tracking
                let mut results = Vec::new();
                let mut visited = std::collections::HashSet::new();
                let mut frontier: std::collections::VecDeque<(mew_core::NodeId, i64, Vec<mew_core::EntityId>)> =
                    std::collections::VecDeque::new();

                // Start with (node_id, depth, path)
                frontier.push_back((start_id, 0, vec![start_id.into()]));

                let max_d = max_depth.unwrap_or(100); // Default max depth

                while let Some((current_id, depth, path)) = frontier.pop_front() {
                    // Skip if already visited (cycle prevention) - check BEFORE yielding
                    if !visited.insert(current_id) {
                        continue;
                    }

                    // Check if we should yield this node
                    if depth >= *min_depth && depth <= max_d {

                        let mut bindings = init_bindings.clone();
                        // Use "node" as the output variable to avoid overwriting the input "start" binding
                        bindings.insert("node", mew_pattern::Binding::Node(current_id));

                        // Add path as a value (list of node refs)
                        let path_values: Vec<Value> = path.iter().map(|id| {
                            match id {
                                mew_core::EntityId::Node(n) => Value::NodeRef(*n),
                                mew_core::EntityId::Edge(e) => Value::EdgeRef(*e),
                            }
                        }).collect();

                        // Output: node (the current node), path (the path taken)
                        results.push((bindings, vec![
                            Value::NodeRef(current_id),
                            Value::String(format!("{:?}", path_values)),
                        ]));
                    }

                    // Stop expanding if at max depth
                    if depth >= max_d {
                        continue;
                    }

                    // Expand frontier by following edges
                    for edge_type_id in edge_types {
                        // Outbound edges
                        if matches!(direction, crate::plan::WalkDirection::Outbound | crate::plan::WalkDirection::Both) {
                            for edge_id in self.graph.edges_from(current_id, Some(*edge_type_id)) {
                                if let Some(edge) = self.graph.get_edge(edge_id) {
                                    // Get the target node (position 1 for binary edges)
                                    if edge.targets.len() > 1 {
                                        if let Some(target_id) = edge.targets[1].as_node() {
                                            if !visited.contains(&target_id) {
                                                let mut new_path = path.clone();
                                                new_path.push(edge_id.into());
                                                new_path.push(target_id.into());
                                                frontier.push_back((target_id, depth + 1, new_path));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Inbound edges
                        if matches!(direction, crate::plan::WalkDirection::Inbound | crate::plan::WalkDirection::Both) {
                            for edge_id in self.graph.edges_to(current_id, Some(*edge_type_id)) {
                                if let Some(edge) = self.graph.get_edge(edge_id) {
                                    // Get the source node (position 0 for binary edges)
                                    if !edge.targets.is_empty() {
                                        if let Some(source_id) = edge.targets[0].as_node() {
                                            if !visited.contains(&source_id) {
                                                let mut new_path = path.clone();
                                                new_path.push(edge_id.into());
                                                new_path.push(source_id.into());
                                                frontier.push_back((source_id, depth + 1, new_path));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(results)
            }

            PlanOp::Empty => {
                if let Some(initial) = initial_bindings {
                    Ok(vec![(initial.clone(), Vec::new())])
                } else {
                    Ok(Vec::new())
                }
            }
        }
    }

    /// Compare two optional values for sorting.
    fn compare_values(&self, a: &Option<Value>, b: &Option<Value>) -> std::cmp::Ordering {
        match (a, b) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => self.compare_values_inner(a, b),
        }
    }

    fn compare_values_inner(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        a.cmp_sortable(b)
    }

    /// Check if two values are equal.
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    /// Compute an aggregate over a group.
    fn compute_aggregate(
        &self,
        kind: crate::plan::AggregateKind,
        group: &[(Bindings, Vec<Value>)],
        expr: &mew_parser::Expr,
    ) -> QueryResult<Value> {
        match kind {
            crate::plan::AggregateKind::Count => Ok(Value::Int(group.len() as i64)),

            crate::plan::AggregateKind::Sum => {
                let mut int_sum = 0i64;
                let mut float_sum = 0.0f64;
                let mut has_float = false;

                for (bindings, _) in group {
                    if let Ok(val) = self.evaluator.eval(expr, bindings, self.graph) {
                        match val {
                            Value::Int(i) => {
                                if has_float {
                                    float_sum += i as f64;
                                } else {
                                    int_sum += i;
                                }
                            }
                            Value::Float(f) => {
                                if !has_float {
                                    // Convert accumulated int sum to float
                                    float_sum = int_sum as f64;
                                    has_float = true;
                                }
                                float_sum += f;
                            }
                            _ => {}
                        }
                    }
                }

                if has_float {
                    Ok(Value::Float(float_sum))
                } else {
                    Ok(Value::Int(int_sum))
                }
            }

            crate::plan::AggregateKind::Avg => {
                let mut sum = 0.0f64;
                let mut count = 0;
                for (bindings, _) in group {
                    if let Ok(val) = self.evaluator.eval(expr, bindings, self.graph) {
                        match val {
                            Value::Int(i) => {
                                sum += i as f64;
                                count += 1;
                            }
                            Value::Float(f) => {
                                sum += f;
                                count += 1;
                            }
                            _ => {}
                        }
                    }
                }
                if count == 0 {
                    Ok(Value::Null)
                } else {
                    Ok(Value::Float(sum / count as f64))
                }
            }

            crate::plan::AggregateKind::Min => {
                let mut min: Option<Value> = None;
                for (bindings, _) in group {
                    if let Ok(val) = self.evaluator.eval(expr, bindings, self.graph) {
                        if !matches!(val, Value::Null) {
                            min = Some(match min {
                                None => val,
                                Some(m) => {
                                    if self.compare_values_inner(&val, &m)
                                        == std::cmp::Ordering::Less
                                    {
                                        val
                                    } else {
                                        m
                                    }
                                }
                            });
                        }
                    }
                }
                Ok(min.unwrap_or(Value::Null))
            }

            crate::plan::AggregateKind::Max => {
                let mut max: Option<Value> = None;
                for (bindings, _) in group {
                    if let Ok(val) = self.evaluator.eval(expr, bindings, self.graph) {
                        if !matches!(val, Value::Null) {
                            max = Some(match max {
                                None => val,
                                Some(m) => {
                                    if self.compare_values_inner(&val, &m)
                                        == std::cmp::Ordering::Greater
                                    {
                                        val
                                    } else {
                                        m
                                    }
                                }
                            });
                        }
                    }
                }
                Ok(max.unwrap_or(Value::Null))
            }
        }
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
        assert!(results.len() >= 2, "Expected at least 2 results, got {}", results.len());
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
        assert!(results.len() >= 1, "Expected at least 1 result, got {}", results.len());
        assert!(results.len() <= 2, "Expected at most 2 results (cycle should be cut), got {}", results.len());
    }
}
