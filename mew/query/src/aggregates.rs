//! Aggregate computation for query execution.
//!
//! This module contains the logic for computing aggregate functions
//! (COUNT, SUM, AVG, MIN, MAX) over grouped result sets.

use mew_core::Value;
use mew_graph::Graph;
use mew_pattern::{Bindings, Evaluator};

use crate::plan::{AggregateKind, AggregateSpec};
use crate::QueryResult;

/// Compute an aggregate over a group of rows.
pub fn compute_aggregate(
    agg: &AggregateSpec,
    group: &[(Bindings, Vec<Value>)],
    evaluator: &Evaluator<'_>,
    graph: &Graph,
) -> QueryResult<Value> {
    match agg.kind {
        AggregateKind::Count => compute_count(agg, group, evaluator, graph),
        AggregateKind::Sum => compute_sum(agg, group, evaluator, graph),
        AggregateKind::Avg => compute_avg(agg, group, evaluator, graph),
        AggregateKind::Min => {
            compute_min_max(agg, group, evaluator, graph, std::cmp::Ordering::Less)
        }
        AggregateKind::Max => {
            compute_min_max(agg, group, evaluator, graph, std::cmp::Ordering::Greater)
        }
        AggregateKind::Collect => compute_collect(agg, group, evaluator, graph),
    }
}

/// Compute COUNT aggregate.
fn compute_count(
    agg: &AggregateSpec,
    group: &[(Bindings, Vec<Value>)],
    evaluator: &Evaluator<'_>,
    graph: &Graph,
) -> QueryResult<Value> {
    if agg.distinct {
        // Count distinct values
        let mut seen = std::collections::HashSet::new();
        for (bindings, _) in group {
            if let Ok(val) = evaluator.eval(&agg.expr, bindings, graph) {
                seen.insert(format!("{:?}", val));
            }
        }
        Ok(Value::Int(seen.len() as i64))
    } else {
        Ok(Value::Int(group.len() as i64))
    }
}

/// Compute SUM aggregate.
fn compute_sum(
    agg: &AggregateSpec,
    group: &[(Bindings, Vec<Value>)],
    evaluator: &Evaluator<'_>,
    graph: &Graph,
) -> QueryResult<Value> {
    let mut int_sum = 0i64;
    let mut float_sum = 0.0f64;
    let mut has_float = false;

    for (bindings, _) in group {
        if let Ok(val) = evaluator.eval(&agg.expr, bindings, graph) {
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

/// Compute AVG aggregate.
fn compute_avg(
    agg: &AggregateSpec,
    group: &[(Bindings, Vec<Value>)],
    evaluator: &Evaluator<'_>,
    graph: &Graph,
) -> QueryResult<Value> {
    let mut sum = 0.0f64;
    let mut count = 0;

    for (bindings, _) in group {
        if let Ok(val) = evaluator.eval(&agg.expr, bindings, graph) {
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

/// Compute MIN or MAX aggregate.
fn compute_min_max(
    agg: &AggregateSpec,
    group: &[(Bindings, Vec<Value>)],
    evaluator: &Evaluator<'_>,
    graph: &Graph,
    target_order: std::cmp::Ordering,
) -> QueryResult<Value> {
    let mut result: Option<Value> = None;

    for (bindings, _) in group {
        if let Ok(val) = evaluator.eval(&agg.expr, bindings, graph) {
            if !matches!(val, Value::Null) {
                result = Some(match result {
                    None => val,
                    Some(current) => {
                        if val.cmp_sortable(&current) == target_order {
                            val
                        } else {
                            current
                        }
                    }
                });
            }
        }
    }

    Ok(result.unwrap_or(Value::Null))
}

/// Compute COLLECT aggregate - collects all values into a list.
fn compute_collect(
    agg: &AggregateSpec,
    group: &[(Bindings, Vec<Value>)],
    evaluator: &Evaluator<'_>,
    graph: &Graph,
) -> QueryResult<Value> {
    let mut result = Vec::new();

    for (bindings, _) in group {
        if let Ok(val) = evaluator.eval(&agg.expr, bindings, graph) {
            // Skip null values
            if !matches!(val, Value::Null) {
                result.push(val);
            }
        }
    }

    Ok(Value::List(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_parser::{Expr, Literal, LiteralKind, Span};
    use mew_registry::RegistryBuilder;

    fn test_evaluator() -> Evaluator<'static> {
        // Create a minimal registry for testing
        let registry = Box::leak(Box::new(RegistryBuilder::new().build().unwrap()));
        Evaluator::new(registry)
    }

    fn make_literal_expr(value: i64) -> Expr {
        Expr::Literal(Literal {
            kind: LiteralKind::Int(value),
            span: Span::default(),
        })
    }

    #[test]
    fn test_count_empty() {
        // GIVEN
        let evaluator = test_evaluator();
        let graph = Graph::new();
        let agg = AggregateSpec {
            name: "count".to_string(),
            kind: AggregateKind::Count,
            expr: make_literal_expr(1),
            distinct: false,
        };

        // WHEN
        let result = compute_aggregate(&agg, &[], &evaluator, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_count_rows() {
        // GIVEN
        let evaluator = test_evaluator();
        let graph = Graph::new();
        let agg = AggregateSpec {
            name: "count".to_string(),
            kind: AggregateKind::Count,
            expr: make_literal_expr(1),
            distinct: false,
        };

        let group = vec![
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
        ];

        // WHEN
        let result = compute_aggregate(&agg, &group, &evaluator, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_sum_integers() {
        // GIVEN
        let evaluator = test_evaluator();
        let graph = Graph::new();

        // SUM of literal 5, evaluated 3 times
        let agg = AggregateSpec {
            name: "sum".to_string(),
            kind: AggregateKind::Sum,
            expr: make_literal_expr(5),
            distinct: false,
        };

        let group = vec![
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
        ];

        // WHEN
        let result = compute_aggregate(&agg, &group, &evaluator, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_avg_integers() {
        // GIVEN
        let evaluator = test_evaluator();
        let graph = Graph::new();

        // AVG of literal 6, evaluated 3 times = 6.0
        let agg = AggregateSpec {
            name: "avg".to_string(),
            kind: AggregateKind::Avg,
            expr: make_literal_expr(6),
            distinct: false,
        };

        let group = vec![
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
        ];

        // WHEN
        let result = compute_aggregate(&agg, &group, &evaluator, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Float(6.0));
    }

    #[test]
    fn test_collect_values() {
        // GIVEN
        let evaluator = test_evaluator();
        let graph = Graph::new();

        // COLLECT of literal 7, evaluated 3 times = [7, 7, 7]
        let agg = AggregateSpec {
            name: "collect".to_string(),
            kind: AggregateKind::Collect,
            expr: make_literal_expr(7),
            distinct: false,
        };

        let group = vec![
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
            (Bindings::new(), vec![]),
        ];

        // WHEN
        let result = compute_aggregate(&agg, &group, &evaluator, &graph).unwrap();

        // THEN
        assert_eq!(
            result,
            Value::List(vec![Value::Int(7), Value::Int(7), Value::Int(7)])
        );
    }

    #[test]
    fn test_collect_empty() {
        // GIVEN
        let evaluator = test_evaluator();
        let graph = Graph::new();

        let agg = AggregateSpec {
            name: "collect".to_string(),
            kind: AggregateKind::Collect,
            expr: make_literal_expr(1),
            distinct: false,
        };

        // WHEN
        let result = compute_aggregate(&agg, &[], &evaluator, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::List(vec![]));
    }
}
