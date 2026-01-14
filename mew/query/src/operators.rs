//! Plan operator execution.
//!
//! This module contains the execution logic for each query plan operator.
//! Operators transform and filter binding sets during query execution.

use mew_core::{EdgeTypeId, Value};
use mew_graph::Graph;
use mew_parser::Expr;
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::aggregates::compute_aggregate;
use crate::plan::{PlanOp, WalkDirection};
use crate::QueryResult;

/// Operator executor context.
///
/// Holds references needed during operator execution.
pub struct OperatorContext<'r, 'g> {
    pub registry: &'r Registry,
    pub graph: &'g Graph,
    pub evaluator: &'r Evaluator<'r>,
}

impl<'r, 'g> OperatorContext<'r, 'g> {
    /// Create a new operator context.
    pub fn new(registry: &'r Registry, graph: &'g Graph, evaluator: &'r Evaluator<'r>) -> Self {
        Self {
            registry,
            graph,
            evaluator,
        }
    }

    /// Execute a plan operator.
    pub fn execute_op(
        &self,
        op: &PlanOp,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        match op {
            PlanOp::NodeScan { var, type_id } => {
                self.execute_node_scan(var, *type_id, initial_bindings)
            }

            PlanOp::IndexScan {
                var,
                type_id,
                attr,
                value,
            } => self.execute_index_scan(var, *type_id, attr, value, initial_bindings),

            PlanOp::EdgeJoin {
                input,
                edge_type_id,
                from_vars,
                edge_var,
            } => self.execute_edge_join(input, *edge_type_id, from_vars, edge_var, initial_bindings),

            PlanOp::Filter { input, condition } => {
                self.execute_filter(input, condition, initial_bindings)
            }

            PlanOp::Project { input, projections } => {
                self.execute_project(input, projections, initial_bindings)
            }

            PlanOp::Sort { input, order_by } => {
                self.execute_sort(input, order_by, initial_bindings)
            }

            PlanOp::LimitOffset {
                input,
                limit,
                offset,
            } => self.execute_limit_offset(input, *limit, *offset, initial_bindings),

            PlanOp::Aggregate {
                input,
                group_by,
                aggregates,
            } => self.execute_aggregate(input, group_by, aggregates, initial_bindings),

            PlanOp::CrossJoin { left, right } => {
                self.execute_cross_join(left, right, initial_bindings)
            }

            PlanOp::LeftOuterJoin {
                left,
                right,
                condition,
                right_vars,
            } => self.execute_left_outer_join(left, right, condition, right_vars, initial_bindings),

            PlanOp::TransitiveClosure {
                start_var: _,
                start_expr,
                edge_types,
                min_depth,
                max_depth,
                direction,
            } => self.execute_transitive_closure(
                start_expr,
                edge_types,
                *min_depth,
                *max_depth,
                direction,
                initial_bindings,
            ),

            PlanOp::Distinct { input } => self.execute_distinct(input, initial_bindings),

            PlanOp::Empty => {
                if let Some(initial) = initial_bindings {
                    Ok(vec![(initial.clone(), Vec::new())])
                } else {
                    Ok(Vec::new())
                }
            }
        }
    }

    fn execute_node_scan(
        &self,
        var: &str,
        type_id: mew_core::TypeId,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        let mut results = Vec::new();

        // Collect all nodes of the given type AND all subtypes (polymorphic matching)
        let mut node_ids: Vec<mew_core::NodeId> = self.graph.nodes_by_type(type_id).collect();

        // Also include nodes of all subtypes
        for subtype_id in self.registry.get_subtypes(type_id) {
            node_ids.extend(self.graph.nodes_by_type(subtype_id));
        }

        for node_id in node_ids {
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

    fn execute_index_scan(
        &self,
        var: &str,
        type_id: mew_core::TypeId,
        attr: &str,
        value: &Expr,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        // Evaluate the search value
        let search_val = self
            .evaluator
            .eval(value, &Bindings::new(), self.graph)?;

        let mut results = Vec::new();

        // Collect all type IDs to scan (including subtypes for polymorphic matching)
        let mut type_ids = vec![type_id];
        type_ids.extend(self.registry.get_subtypes(type_id));

        // Use attribute index
        if !matches!(search_val, Value::Null) {
            for tid in &type_ids {
                for node_id in self.graph.nodes_by_attr(*tid, attr, &search_val) {
                    let mut bindings = initial_bindings.cloned().unwrap_or_default();
                    if let Some(existing) = bindings.get(var) {
                        if existing.as_node() != Some(node_id) {
                            continue;
                        }
                    }
                    bindings.insert(var, mew_pattern::Binding::Node(node_id));
                    results.push((bindings, Vec::new()));
                }
            }
        } else {
            // Fall back to scan with filter
            for tid in &type_ids {
                for node_id in self.graph.nodes_by_type(*tid) {
                    if let Some(node) = self.graph.get_node(node_id) {
                        if let Some(attr_val) = node.get_attr(attr) {
                            if values_equal(attr_val, &search_val) {
                                let mut bindings = initial_bindings.cloned().unwrap_or_default();
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
        }

        Ok(results)
    }

    fn execute_edge_join(
        &self,
        input: &PlanOp,
        edge_type_id: EdgeTypeId,
        from_vars: &[String],
        edge_var: &Option<String>,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
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
                for edge_id in self.graph.edges_from(source_id, Some(edge_type_id)) {
                    if let Some(edge) = self.graph.get_edge(edge_id) {
                        // Check that all target variables match
                        let mut all_match = true;
                        for (i, var) in from_vars.iter().enumerate() {
                            if let Some(binding) = bindings.get(var) {
                                if let Some(expected_id) = binding.as_node() {
                                    if i < edge.targets.len()
                                        && edge.targets[i].as_node() != Some(expected_id)
                                    {
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

    fn execute_filter(
        &self,
        input: &PlanOp,
        condition: &Expr,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
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

    fn execute_project(
        &self,
        input: &PlanOp,
        projections: &[(String, Expr)],
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
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

    fn execute_sort(
        &self,
        input: &PlanOp,
        order_by: &[(Expr, bool)],
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        let mut results = self.execute_op(input, initial_bindings)?;

        // Sort by the order expressions
        results.sort_by(|(a_bindings, _a_values), (b_bindings, _b_values)| {
            for (expr, ascending) in order_by {
                // Evaluate the expression for both rows
                let a_val = self.evaluator.eval(expr, a_bindings, self.graph).ok();
                let b_val = self.evaluator.eval(expr, b_bindings, self.graph).ok();

                let cmp = compare_values(&a_val, &b_val);
                if cmp != std::cmp::Ordering::Equal {
                    return if *ascending { cmp } else { cmp.reverse() };
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(results)
    }

    fn execute_limit_offset(
        &self,
        input: &PlanOp,
        limit: Option<i64>,
        offset: Option<i64>,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        let results = self.execute_op(input, initial_bindings)?;

        let start = offset.unwrap_or(0) as usize;
        let end = limit.map(|l| start + l as usize).unwrap_or(results.len());

        Ok(results.into_iter().skip(start).take(end - start).collect())
    }

    fn execute_aggregate(
        &self,
        input: &PlanOp,
        group_by: &[Expr],
        aggregates: &[crate::plan::AggregateSpec],
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        let results = self.execute_op(input, initial_bindings)?;

        if group_by.is_empty() && results.is_empty() {
            // Empty input with no grouping returns single row with defaults
            let values: Vec<Value> = aggregates
                .iter()
                .map(|agg| match agg.kind {
                    crate::plan::AggregateKind::Count => Value::Int(0),
                    _ => Value::Null,
                })
                .collect();
            return Ok(vec![(Bindings::new(), values)]);
        }

        // Group rows by key (using string serialization since Value doesn't impl Hash)
        let mut groups: std::collections::HashMap<String, Vec<(Bindings, Vec<Value>)>> =
            std::collections::HashMap::new();

        for (bindings, values) in results {
            let key = compute_group_key(group_by, &bindings, self.evaluator, self.graph);
            groups.entry(key).or_default().push((bindings, values));
        }

        // Compute output for each group
        let mut output = Vec::new();
        for group in groups.into_values() {
            let first_bindings = group.first().map(|(b, _)| b.clone()).unwrap_or_default();

            // Collect group_by values then aggregate values
            let mut row_values: Vec<Value> = group_by
                .iter()
                .map(|expr| {
                    self.evaluator
                        .eval(expr, &first_bindings, self.graph)
                        .unwrap_or(Value::Null)
                })
                .collect();

            for agg in aggregates {
                let val = compute_aggregate(agg, &group, self.evaluator, self.graph)?;
                row_values.push(val);
            }

            output.push((first_bindings, row_values));
        }

        Ok(output)
    }

    fn execute_cross_join(
        &self,
        left: &PlanOp,
        right: &PlanOp,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
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

    fn execute_left_outer_join(
        &self,
        left: &PlanOp,
        right: &PlanOp,
        condition: &Option<Expr>,
        right_vars: &[String],
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        let left_results = self.execute_op(left, initial_bindings)?;
        let mut results = Vec::new();

        for (left_bindings, left_values) in &left_results {
            // Execute right side with left bindings as context
            let right_results = self.execute_op(right, Some(left_bindings))?;

            // Filter right results by condition if present
            let matching_right: Vec<_> = if let Some(cond) = condition {
                right_results
                    .into_iter()
                    .filter(|(right_bindings, _)| {
                        let mut merged = left_bindings.clone();
                        for (k, v) in right_bindings.iter() {
                            merged.insert(k, v.clone());
                        }
                        self.evaluator
                            .eval_bool(cond, &merged, self.graph)
                            .unwrap_or(false)
                    })
                    .collect()
            } else {
                right_results
            };

            if matching_right.is_empty() {
                // No match: keep left row with null bindings for right side variables
                let mut bindings = left_bindings.clone();
                for var in right_vars {
                    bindings.insert(var, mew_pattern::Binding::Null);
                }
                results.push((bindings, left_values.clone()));
            } else {
                // Has matches: produce a row for each match
                for (right_bindings, right_values) in matching_right {
                    let mut merged = left_bindings.clone();
                    for (k, v) in right_bindings.iter() {
                        merged.insert(k, v.clone());
                    }
                    let mut merged_values = left_values.clone();
                    merged_values.extend(right_values);
                    results.push((merged, merged_values));
                }
            }
        }

        Ok(results)
    }

    fn execute_transitive_closure(
        &self,
        start_expr: &Expr,
        edge_types: &[EdgeTypeId],
        min_depth: i64,
        max_depth: Option<i64>,
        direction: &WalkDirection,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        // Evaluate the start expression to get the starting node(s)
        let init_bindings = initial_bindings.cloned().unwrap_or_default();
        let start_value = self
            .evaluator
            .eval(start_expr, &init_bindings, self.graph)?;

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
        let mut frontier: std::collections::VecDeque<(
            mew_core::NodeId,
            i64,
            Vec<mew_core::EntityId>,
        )> = std::collections::VecDeque::new();

        // Start with (node_id, depth, path)
        frontier.push_back((start_id, 0, vec![start_id.into()]));

        let max_d = max_depth.unwrap_or(100); // Default max depth

        while let Some((current_id, depth, path)) = frontier.pop_front() {
            // Skip if already visited (cycle prevention) - check BEFORE yielding
            if !visited.insert(current_id) {
                continue;
            }

            // Check if we should yield this node
            if depth >= min_depth && depth <= max_d {
                let mut bindings = init_bindings.clone();
                // Use "node" as the output variable to avoid overwriting the input "start" binding
                bindings.insert("node", mew_pattern::Binding::Node(current_id));

                // Add path as a value (list of node refs)
                let path_values: Vec<Value> = path
                    .iter()
                    .map(|id| match id {
                        mew_core::EntityId::Node(n) => Value::NodeRef(*n),
                        mew_core::EntityId::Edge(e) => Value::EdgeRef(*e),
                    })
                    .collect();

                // Output: node (the current node), path (the path taken)
                results.push((
                    bindings,
                    vec![
                        Value::NodeRef(current_id),
                        Value::String(format!("{:?}", path_values)),
                    ],
                ));
            }

            // Stop expanding if at max depth
            if depth >= max_d {
                continue;
            }

            // Expand frontier by following edges
            for edge_type_id in edge_types {
                // Outbound edges
                if matches!(direction, WalkDirection::Outbound | WalkDirection::Both) {
                    self.expand_outbound(
                        current_id,
                        *edge_type_id,
                        depth,
                        &path,
                        &visited,
                        &mut frontier,
                    );
                }

                // Inbound edges
                if matches!(direction, WalkDirection::Inbound | WalkDirection::Both) {
                    self.expand_inbound(
                        current_id,
                        *edge_type_id,
                        depth,
                        &path,
                        &visited,
                        &mut frontier,
                    );
                }
            }
        }

        Ok(results)
    }

    fn expand_outbound(
        &self,
        current_id: mew_core::NodeId,
        edge_type_id: EdgeTypeId,
        depth: i64,
        path: &[mew_core::EntityId],
        visited: &std::collections::HashSet<mew_core::NodeId>,
        frontier: &mut std::collections::VecDeque<(mew_core::NodeId, i64, Vec<mew_core::EntityId>)>,
    ) {
        for edge_id in self.graph.edges_from(current_id, Some(edge_type_id)) {
            if let Some(edge) = self.graph.get_edge(edge_id) {
                // Get the target node (position 1 for binary edges)
                if edge.targets.len() > 1 {
                    if let Some(target_id) = edge.targets[1].as_node() {
                        if !visited.contains(&target_id) {
                            let mut new_path = path.to_vec();
                            new_path.push(edge_id.into());
                            new_path.push(target_id.into());
                            frontier.push_back((target_id, depth + 1, new_path));
                        }
                    }
                }
            }
        }
    }

    fn expand_inbound(
        &self,
        current_id: mew_core::NodeId,
        edge_type_id: EdgeTypeId,
        depth: i64,
        path: &[mew_core::EntityId],
        visited: &std::collections::HashSet<mew_core::NodeId>,
        frontier: &mut std::collections::VecDeque<(mew_core::NodeId, i64, Vec<mew_core::EntityId>)>,
    ) {
        for edge_id in self.graph.edges_to(current_id, Some(edge_type_id)) {
            if let Some(edge) = self.graph.get_edge(edge_id) {
                // Get the source node (position 0 for binary edges)
                if !edge.targets.is_empty() {
                    if let Some(source_id) = edge.targets[0].as_node() {
                        if !visited.contains(&source_id) {
                            let mut new_path = path.to_vec();
                            new_path.push(edge_id.into());
                            new_path.push(source_id.into());
                            frontier.push_back((source_id, depth + 1, new_path));
                        }
                    }
                }
            }
        }
    }

    fn execute_distinct(
        &self,
        input: &PlanOp,
        initial_bindings: Option<&Bindings>,
    ) -> QueryResult<Vec<(Bindings, Vec<Value>)>> {
        let results = self.execute_op(input, initial_bindings)?;
        let mut seen = std::collections::HashSet::new();
        let mut distinct_results = Vec::new();

        for (bindings, values) in results {
            // Use the values for deduplication (as a string for hashing)
            let key: Vec<String> = values.iter().map(|v| format!("{:?}", v)).collect();
            if seen.insert(key) {
                distinct_results.push((bindings, values));
            }
        }

        Ok(distinct_results)
    }
}

/// Compare two optional values for sorting.
pub fn compare_values(a: &Option<Value>, b: &Option<Value>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => a.cmp_sortable(b),
    }
}

/// Check if two values are equal.
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

/// Compute a group key from expressions (for GROUP BY).
pub fn compute_group_key(
    group_by: &[Expr],
    bindings: &Bindings,
    evaluator: &Evaluator<'_>,
    graph: &Graph,
) -> String {
    group_by
        .iter()
        .map(|e| {
            let v = evaluator.eval(e, bindings, graph).unwrap_or(Value::Null);
            format!("{:?}", v)
        })
        .collect::<Vec<_>>()
        .join("|")
}
