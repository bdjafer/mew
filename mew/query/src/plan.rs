//! Query planning.

use mew_core::{EdgeTypeId, TypeId};
use mew_parser::{Expr, MatchStmt, Projection, WalkStmt};
use mew_registry::Registry;

use crate::{QueryError, QueryResult};

/// A query execution plan.
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// The root operator of the plan.
    pub root: PlanOp,
    /// Output column names.
    pub columns: Vec<String>,
}

/// A plan operator (Volcano-style iterator model).
#[derive(Debug, Clone)]
pub enum PlanOp {
    /// Scan all nodes of a type.
    NodeScan { var: String, type_id: TypeId },

    /// Scan using an attribute index.
    IndexScan {
        var: String,
        type_id: TypeId,
        attr: String,
        value: Expr,
    },

    /// Follow edges from bound nodes.
    EdgeJoin {
        input: Box<PlanOp>,
        edge_type_id: EdgeTypeId,
        from_vars: Vec<String>,
        edge_var: Option<String>,
    },

    /// Filter rows by a condition.
    Filter { input: Box<PlanOp>, condition: Expr },

    /// Project columns.
    Project {
        input: Box<PlanOp>,
        projections: Vec<(String, Expr)>,
    },

    /// Sort by expressions.
    Sort {
        input: Box<PlanOp>,
        order_by: Vec<(Expr, bool)>, // (expr, ascending)
    },

    /// Limit and offset.
    LimitOffset {
        input: Box<PlanOp>,
        limit: Option<i64>,
        offset: Option<i64>,
    },

    /// Aggregate with optional grouping.
    Aggregate {
        input: Box<PlanOp>,
        group_by: Vec<Expr>,
        aggregates: Vec<AggregateSpec>,
    },

    /// Cartesian product of two inputs.
    CrossJoin {
        left: Box<PlanOp>,
        right: Box<PlanOp>,
    },

    /// Left outer join (for OPTIONAL MATCH).
    /// Returns all rows from left, with nulls for right if no match.
    LeftOuterJoin {
        left: Box<PlanOp>,
        right: Box<PlanOp>,
        /// Optional condition for the join
        condition: Option<Expr>,
    },

    /// Transitive closure for WALK.
    TransitiveClosure {
        start_var: String,
        start_expr: Expr,
        edge_types: Vec<EdgeTypeId>,
        min_depth: i64,
        max_depth: Option<i64>,
        direction: WalkDirection,
    },

    /// Remove duplicate rows.
    Distinct { input: Box<PlanOp> },

    /// Empty result (for patterns that can't match).
    Empty,
}

/// Aggregate function kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateKind {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// Specification for an aggregate computation.
#[derive(Debug, Clone)]
pub struct AggregateSpec {
    /// Output column name.
    pub name: String,
    /// Type of aggregate function.
    pub kind: AggregateKind,
    /// Expression to aggregate over.
    pub expr: Expr,
    /// Whether to only consider distinct values (e.g., COUNT(DISTINCT x)).
    pub distinct: bool,
}

/// Walk direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkDirection {
    Outbound,
    Inbound,
    Both,
}

/// Query planner.
pub struct QueryPlanner<'r> {
    registry: &'r Registry,
}

impl<'r> QueryPlanner<'r> {
    /// Create a new planner.
    pub fn new(registry: &'r Registry) -> Self {
        Self { registry }
    }

    /// Plan a MATCH statement.
    pub fn plan_match(&self, stmt: &MatchStmt) -> QueryResult<QueryPlan> {
        // Build the pattern matching plan
        let mut plan = self.plan_pattern(&stmt.pattern)?;

        // Add WHERE filter
        if let Some(ref cond) = stmt.where_clause {
            plan = PlanOp::Filter {
                input: Box::new(plan),
                condition: cond.clone(),
            };
        }

        // Handle OPTIONAL MATCH clauses (left outer joins)
        for opt_match in &stmt.optional_matches {
            let opt_plan = self.plan_pattern(&opt_match.pattern)?;
            plan = PlanOp::LeftOuterJoin {
                left: Box::new(plan),
                right: Box::new(opt_plan),
                condition: opt_match.where_clause.clone(),
            };
        }

        // Check if projections contain aggregates
        let aggregates = self.extract_aggregates(&stmt.return_clause.projections);
        let has_aggregates = !aggregates.is_empty();

        // Add aggregate operator if needed (before ORDER BY)
        if has_aggregates {
            // Extract non-aggregate expressions as implicit GROUP BY
            let group_by: Vec<Expr> = stmt
                .return_clause
                .projections
                .iter()
                .filter(|p| self.get_aggregate(&p.expr).is_none())
                .map(|p| p.expr.clone())
                .collect();

            plan = PlanOp::Aggregate {
                input: Box::new(plan),
                group_by,
                aggregates,
            };
        }

        // Add ORDER BY
        if let Some(ref order_by) = stmt.order_by {
            let order: Vec<(Expr, bool)> = order_by
                .iter()
                .map(|term| {
                    let asc = matches!(term.direction, mew_parser::OrderDirection::Asc);
                    (term.expr.clone(), asc)
                })
                .collect();

            plan = PlanOp::Sort {
                input: Box::new(plan),
                order_by: order,
            };
        }

        // Add LIMIT/OFFSET
        if stmt.limit.is_some() || stmt.offset.is_some() {
            plan = PlanOp::LimitOffset {
                input: Box::new(plan),
                limit: stmt.limit,
                offset: stmt.offset,
            };
        }

        // Add projections (for non-aggregate queries, or for the column naming)
        let (projections, columns) = self.plan_projections(&stmt.return_clause.projections)?;

        // If we have aggregates, the projection just names the output columns
        // The actual values are computed by the Aggregate operator
        if !has_aggregates {
            plan = PlanOp::Project {
                input: Box::new(plan),
                projections,
            };
        }

        // Add DISTINCT if requested
        if stmt.return_clause.distinct {
            plan = PlanOp::Distinct {
                input: Box::new(plan),
            };
        }

        Ok(QueryPlan {
            root: plan,
            columns,
        })
    }

    /// Extract aggregate functions from projections.
    fn extract_aggregates(&self, projections: &[Projection]) -> Vec<AggregateSpec> {
        projections
            .iter()
            .filter_map(|proj| {
                self.get_aggregate(&proj.expr).map(|(kind, expr, distinct)| {
                    AggregateSpec {
                        name: proj
                            .alias
                            .clone()
                            .unwrap_or_else(|| self.expr_to_name(&proj.expr)),
                        kind,
                        expr,
                        distinct,
                    }
                })
            })
            .collect()
    }

    /// Check if an expression is an aggregate function and return its kind, argument, and distinct flag.
    /// Note: min/max with 2 arguments are binary functions, not aggregates.
    fn get_aggregate(&self, expr: &Expr) -> Option<(AggregateKind, Expr, bool)> {
        match expr {
            Expr::FnCall(fc) => {
                let kind = match fc.name.to_lowercase().as_str() {
                    "count" => Some(AggregateKind::Count),
                    "sum" => Some(AggregateKind::Sum),
                    "avg" => Some(AggregateKind::Avg),
                    // min/max are aggregates only with 1 arg; with 2 args they're binary functions
                    "min" if fc.args.len() == 1 => Some(AggregateKind::Min),
                    "max" if fc.args.len() == 1 => Some(AggregateKind::Max),
                    _ => None,
                };
                kind.map(|k| {
                    // For count(), use a placeholder expression if no args
                    let arg = fc.args.first().cloned().unwrap_or({
                        Expr::Literal(mew_parser::Literal {
                            kind: mew_parser::LiteralKind::Int(1),
                            span: fc.span,
                        })
                    });
                    (k, arg, fc.distinct)
                })
            }
            _ => None,
        }
    }

    /// Plan the pattern matching portion.
    fn plan_pattern(&self, pattern: &[mew_parser::PatternElem]) -> QueryResult<PlanOp> {
        if pattern.is_empty() {
            return Ok(PlanOp::Empty);
        }

        let mut plan: Option<PlanOp> = None;

        for elem in pattern {
            match elem {
                mew_parser::PatternElem::Node(np) => {
                    let type_id = self
                        .registry
                        .get_type_id(&np.type_name)
                        .ok_or_else(|| QueryError::unknown_type(&np.type_name))?;

                    let scan = PlanOp::NodeScan {
                        var: np.var.clone(),
                        type_id,
                    };

                    plan = Some(match plan {
                        None => scan,
                        Some(p) => PlanOp::CrossJoin {
                            left: Box::new(p),
                            right: Box::new(scan),
                        },
                    });
                }
                mew_parser::PatternElem::Edge(ep) => {
                    let edge_type_id = self
                        .registry
                        .get_edge_type_id(&ep.edge_type)
                        .ok_or_else(|| QueryError::unknown_type(&ep.edge_type))?;

                    if let Some(p) = plan {
                        plan = Some(PlanOp::EdgeJoin {
                            input: Box::new(p),
                            edge_type_id,
                            from_vars: ep.targets.clone(),
                            edge_var: ep.alias.clone(),
                        });
                    }
                }
            }
        }

        Ok(plan.unwrap_or(PlanOp::Empty))
    }

    /// Plan projections from RETURN clause.
    #[allow(clippy::type_complexity)]
    fn plan_projections(
        &self,
        projections: &[Projection],
    ) -> QueryResult<(Vec<(String, Expr)>, Vec<String>)> {
        let mut result = Vec::new();
        let mut columns = Vec::new();

        for proj in projections {
            let name = proj
                .alias
                .clone()
                .unwrap_or_else(|| self.expr_to_name(&proj.expr));

            columns.push(name.clone());
            result.push((name, proj.expr.clone()));
        }

        Ok((result, columns))
    }

    /// Generate a column name from an expression.
    fn expr_to_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Var(name, _) => name.clone(),
            Expr::AttrAccess(base, attr, _) => {
                format!("{}.{}", self.expr_to_name(base), attr)
            }
            Expr::FnCall(fc) => fc.name.clone(),
            _ => "expr".to_string(),
        }
    }

    /// Plan a WALK statement.
    pub fn plan_walk(&self, stmt: &WalkStmt) -> QueryResult<QueryPlan> {
        // Collect edge types and direction from FOLLOW clauses
        let mut edge_types = Vec::new();
        let mut min_depth: i64 = 1;
        let mut max_depth: Option<i64> = None;
        let mut direction = WalkDirection::Outbound;

        for follow in &stmt.follow {
            for et_name in &follow.edge_types {
                if let Some(id) = self.registry.get_edge_type_id(et_name) {
                    edge_types.push(id);
                }
            }
            if let Some(min) = follow.min_depth {
                min_depth = min;
            }
            if let Some(max) = follow.max_depth {
                max_depth = Some(max);
            }
            direction = match follow.direction {
                mew_parser::WalkDirection::Outbound => WalkDirection::Outbound,
                mew_parser::WalkDirection::Inbound => WalkDirection::Inbound,
                mew_parser::WalkDirection::Any => WalkDirection::Both,
            };
        }

        let plan = PlanOp::TransitiveClosure {
            start_var: "start".to_string(),
            start_expr: stmt.from.clone(),
            edge_types,
            min_depth,
            max_depth,
            direction,
        };

        // Add UNTIL filter
        let plan = if let Some(ref until) = stmt.until {
            PlanOp::Filter {
                input: Box::new(plan),
                condition: until.clone(),
            }
        } else {
            plan
        };

        Ok(QueryPlan {
            root: plan,
            columns: vec!["node".to_string(), "path".to_string()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_parser::{NodePattern, PatternElem, ReturnClause, Span};
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
        builder
            .add_edge_type("owns")
            .param("owner", "Person")
            .param("task", "Task")
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_plan_single_node_scan() {
        // GIVEN
        let registry = test_registry();
        let planner = QueryPlanner::new(&registry);

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
                projections: vec![mew_parser::Projection {
                    expr: Expr::Var("t".to_string(), Span::default()),
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
        let plan = planner.plan_match(&stmt).unwrap();

        // THEN
        assert!(!plan.columns.is_empty());
        // The plan should be Project(NodeScan)
        matches!(plan.root, PlanOp::Project { .. });
    }

    #[test]
    fn test_plan_with_limit() {
        // GIVEN
        let registry = test_registry();
        let planner = QueryPlanner::new(&registry);

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
                projections: vec![mew_parser::Projection {
                    expr: Expr::Var("t".to_string(), Span::default()),
                    alias: None,
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            order_by: None,
            limit: Some(10),
            offset: Some(5),
            span: Span::default(),
        };

        // WHEN
        let plan = planner.plan_match(&stmt).unwrap();

        // THEN
        // The plan should be Project(LimitOffset(NodeScan))
        matches!(plan.root, PlanOp::Project { .. });
    }

    #[test]
    fn test_plan_unknown_type_error() {
        // GIVEN
        let registry = test_registry();
        let planner = QueryPlanner::new(&registry);

        let stmt = MatchStmt {
            pattern: vec![PatternElem::Node(NodePattern {
                var: "x".to_string(),
                type_name: "Unknown".to_string(),
                span: Span::default(),
            })],
            where_clause: None,
            optional_matches: vec![],
            return_clause: ReturnClause {
                distinct: false,
                projections: vec![],
                span: Span::default(),
            },
            order_by: None,
            limit: None,
            offset: None,
            span: Span::default(),
        };

        // WHEN
        let result = planner.plan_match(&stmt);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            QueryError::UnknownType { .. }
        ));
    }
}
