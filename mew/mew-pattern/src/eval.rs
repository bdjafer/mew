//! Expression evaluation.

use crate::{Bindings, PatternError, PatternResult};
use mew_core::Value;
use mew_graph::Graph;
use mew_parser::{BinaryOp, Expr, LiteralKind, UnaryOp};
use mew_registry::Registry;

/// Expression evaluator.
pub struct Evaluator<'r, 'g> {
    registry: &'r Registry,
    graph: &'g Graph,
}

impl<'r, 'g> Evaluator<'r, 'g> {
    /// Create a new evaluator.
    pub fn new(registry: &'r Registry, graph: &'g Graph) -> Self {
        Self { registry, graph }
    }

    /// Evaluate an expression with the given bindings.
    pub fn eval(&self, expr: &Expr, bindings: &Bindings) -> PatternResult<Value> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Var(name, _) => self.eval_var(name, bindings),
            Expr::AttrAccess(base, attr, _) => self.eval_attr_access(base, attr, bindings),
            Expr::BinaryOp(op, left, right, _) => self.eval_binary_op(*op, left, right, bindings),
            Expr::UnaryOp(op, operand, _) => self.eval_unary_op(*op, operand, bindings),
            Expr::FnCall(fc) => self.eval_fn_call(&fc.name, &fc.args, bindings),
            Expr::IdRef(_, _) => Ok(Value::Null), // TODO: resolve ID refs
            Expr::Param(name, _) => Err(PatternError::unbound_variable(name)),
            Expr::Exists(_, _, _) | Expr::NotExists(_, _, _) => {
                // EXISTS/NOT EXISTS are handled by the matcher, not the evaluator
                Ok(Value::Bool(false))
            }
        }
    }

    /// Evaluate a literal.
    fn eval_literal(&self, lit: &mew_parser::Literal) -> PatternResult<Value> {
        Ok(match &lit.kind {
            LiteralKind::Null => Value::Null,
            LiteralKind::Bool(b) => Value::Bool(*b),
            LiteralKind::Int(i) => Value::Int(*i),
            LiteralKind::Float(f) => Value::Float(*f),
            LiteralKind::String(s) => Value::String(s.clone()),
        })
    }

    /// Evaluate a variable reference.
    fn eval_var(&self, name: &str, bindings: &Bindings) -> PatternResult<Value> {
        bindings
            .get(name)
            .map(|b| b.to_value())
            .ok_or_else(|| PatternError::unbound_variable(name))
    }

    /// Evaluate an attribute access.
    fn eval_attr_access(
        &self,
        base: &Expr,
        attr: &str,
        bindings: &Bindings,
    ) -> PatternResult<Value> {
        let base_val = self.eval(base, bindings)?;

        match base_val {
            Value::NodeRef(node_id) => {
                // Get the attribute from the node
                if let Some(node) = self.graph.get_node(node_id) {
                    Ok(node.get_attr(attr).cloned().unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            Value::EdgeRef(edge_id) => {
                // Get the attribute from the edge
                if let Some(edge) = self.graph.get_edge(edge_id) {
                    Ok(edge.get_attr(attr).cloned().unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Err(PatternError::type_error(format!(
                "cannot access attribute on {:?}",
                base_val
            ))),
        }
    }

    /// Evaluate a binary operation.
    fn eval_binary_op(
        &self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        bindings: &Bindings,
    ) -> PatternResult<Value> {
        let left_val = self.eval(left, bindings)?;
        let right_val = self.eval(right, bindings)?;

        match op {
            // Arithmetic
            BinaryOp::Add => self.eval_add(&left_val, &right_val),
            BinaryOp::Sub => self.eval_sub(&left_val, &right_val),
            BinaryOp::Mul => self.eval_mul(&left_val, &right_val),
            BinaryOp::Div => self.eval_div(&left_val, &right_val),
            BinaryOp::Mod => self.eval_mod(&left_val, &right_val),

            // Comparison
            BinaryOp::Eq => Ok(Value::Bool(self.values_equal(&left_val, &right_val))),
            BinaryOp::NotEq => Ok(Value::Bool(!self.values_equal(&left_val, &right_val))),
            BinaryOp::Lt => self.eval_lt(&left_val, &right_val),
            BinaryOp::LtEq => self.eval_lte(&left_val, &right_val),
            BinaryOp::Gt => self.eval_gt(&left_val, &right_val),
            BinaryOp::GtEq => self.eval_gte(&left_val, &right_val),

            // Logical
            BinaryOp::And => self.eval_and(&left_val, &right_val),
            BinaryOp::Or => self.eval_or(&left_val, &right_val),

            // String
            BinaryOp::Concat => self.eval_concat(&left_val, &right_val),
        }
    }

    /// Evaluate a unary operation.
    fn eval_unary_op(
        &self,
        op: UnaryOp,
        operand: &Expr,
        bindings: &Bindings,
    ) -> PatternResult<Value> {
        let val = self.eval(operand, bindings)?;

        match op {
            UnaryOp::Neg => match val {
                Value::Int(i) => Ok(Value::Int(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(PatternError::type_error(format!(
                    "cannot negate {:?}",
                    val
                ))),
            },
            UnaryOp::Not => match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(PatternError::type_error(format!(
                    "cannot apply NOT to {:?}",
                    val
                ))),
            },
        }
    }

    /// Evaluate a function call.
    fn eval_fn_call(
        &self,
        name: &str,
        args: &[Expr],
        bindings: &Bindings,
    ) -> PatternResult<Value> {
        let name_lower = name.to_lowercase();

        match name_lower.as_str() {
            "count" => {
                // COUNT expects 0 or 1 arguments
                // For now, just return 0 - proper aggregate handling is in Query
                Ok(Value::Int(0))
            }
            "sum" | "avg" | "min" | "max" => {
                // Aggregates - placeholder for now
                Ok(Value::Float(0.0))
            }
            "coalesce" => {
                // Return first non-null argument
                for arg in args {
                    let val = self.eval(arg, bindings)?;
                    if !matches!(val, Value::Null) {
                        return Ok(val);
                    }
                }
                Ok(Value::Null)
            }
            "upper" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings)?;
                    if let Value::String(s) = val {
                        return Ok(Value::String(s.to_uppercase()));
                    }
                }
                Err(PatternError::type_error("UPPER expects a string argument"))
            }
            "lower" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings)?;
                    if let Value::String(s) = val {
                        return Ok(Value::String(s.to_lowercase()));
                    }
                }
                Err(PatternError::type_error("LOWER expects a string argument"))
            }
            "abs" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings)?;
                    return match val {
                        Value::Int(i) => Ok(Value::Int(i.abs())),
                        Value::Float(f) => Ok(Value::Float(f.abs())),
                        _ => Err(PatternError::type_error("ABS expects a numeric argument")),
                    };
                }
                Err(PatternError::type_error("ABS expects one argument"))
            }
            _ => Err(PatternError::invalid_operation(format!(
                "unknown function '{}'",
                name
            ))),
        }
    }

    // ========== Arithmetic helpers ==========

    fn eval_add(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            _ => Err(PatternError::type_error(format!(
                "cannot add {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_sub(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(PatternError::type_error(format!(
                "cannot subtract {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_mul(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(PatternError::type_error(format!(
                "cannot multiply {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_div(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(PatternError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(PatternError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(PatternError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(PatternError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            _ => Err(PatternError::type_error(format!(
                "cannot divide {:?} by {:?}",
                left, right
            ))),
        }
    }

    fn eval_mod(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(PatternError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(PatternError::type_error(format!(
                "cannot mod {:?} and {:?}",
                left, right
            ))),
        }
    }

    // ========== Comparison helpers ==========

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
            (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::NodeRef(a), Value::NodeRef(b)) => a == b,
            (Value::EdgeRef(a), Value::EdgeRef(b)) => a == b,
            _ => false,
        }
    }

    fn eval_lt(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        Ok(Value::Bool(match (left, right) {
            (Value::Int(a), Value::Int(b)) => a < b,
            (Value::Float(a), Value::Float(b)) => a < b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) < *b,
            (Value::Float(a), Value::Int(b)) => *a < (*b as f64),
            (Value::String(a), Value::String(b)) => a < b,
            _ => {
                return Err(PatternError::type_error(format!(
                    "cannot compare {:?} < {:?}",
                    left, right
                )))
            }
        }))
    }

    fn eval_lte(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        Ok(Value::Bool(match (left, right) {
            (Value::Int(a), Value::Int(b)) => a <= b,
            (Value::Float(a), Value::Float(b)) => a <= b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) <= *b,
            (Value::Float(a), Value::Int(b)) => *a <= (*b as f64),
            (Value::String(a), Value::String(b)) => a <= b,
            _ => {
                return Err(PatternError::type_error(format!(
                    "cannot compare {:?} <= {:?}",
                    left, right
                )))
            }
        }))
    }

    fn eval_gt(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        Ok(Value::Bool(match (left, right) {
            (Value::Int(a), Value::Int(b)) => a > b,
            (Value::Float(a), Value::Float(b)) => a > b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) > *b,
            (Value::Float(a), Value::Int(b)) => *a > (*b as f64),
            (Value::String(a), Value::String(b)) => a > b,
            _ => {
                return Err(PatternError::type_error(format!(
                    "cannot compare {:?} > {:?}",
                    left, right
                )))
            }
        }))
    }

    fn eval_gte(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        Ok(Value::Bool(match (left, right) {
            (Value::Int(a), Value::Int(b)) => a >= b,
            (Value::Float(a), Value::Float(b)) => a >= b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) >= *b,
            (Value::Float(a), Value::Int(b)) => *a >= (*b as f64),
            (Value::String(a), Value::String(b)) => a >= b,
            _ => {
                return Err(PatternError::type_error(format!(
                    "cannot compare {:?} >= {:?}",
                    left, right
                )))
            }
        }))
    }

    // ========== Logical helpers ==========

    fn eval_and(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(PatternError::type_error(format!(
                "cannot AND {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_or(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(PatternError::type_error(format!(
                "cannot OR {:?} and {:?}",
                left, right
            ))),
        }
    }

    // ========== String helpers ==========

    fn eval_concat(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(PatternError::type_error(format!(
                "cannot concat {:?} and {:?}",
                left, right
            ))),
        }
    }

    /// Evaluate an expression and convert to bool.
    pub fn eval_bool(&self, expr: &Expr, bindings: &Bindings) -> PatternResult<bool> {
        let val = self.eval(expr, bindings)?;
        match val {
            Value::Bool(b) => Ok(b),
            Value::Null => Ok(false),
            _ => Err(PatternError::type_error(format!(
                "expected bool, got {:?}",
                val
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Binding;
    use mew_core::{attrs, TypeId};
    use mew_parser::{Literal, Span};
    use mew_registry::RegistryBuilder;

    fn test_registry() -> Registry {
        RegistryBuilder::new().build().unwrap()
    }

    fn test_graph() -> Graph {
        Graph::new()
    }

    #[test]
    fn test_eval_arithmetic() {
        // GIVEN
        let registry = test_registry();
        let graph = test_graph();
        let evaluator = Evaluator::new(&registry, &graph);
        let bindings = Bindings::new();

        // x + y * 2 where x=10, y=3 => 10 + 3 * 2 = 16
        let expr = Expr::BinaryOp(
            BinaryOp::Add,
            Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Int(10),
                span: Span::default(),
            })),
            Box::new(Expr::BinaryOp(
                BinaryOp::Mul,
                Box::new(Expr::Literal(Literal {
                    kind: LiteralKind::Int(3),
                    span: Span::default(),
                })),
                Box::new(Expr::Literal(Literal {
                    kind: LiteralKind::Int(2),
                    span: Span::default(),
                })),
                Span::default(),
            )),
            Span::default(),
        );

        // WHEN
        let result = evaluator.eval(&expr, &bindings).unwrap();

        // THEN
        assert_eq!(result, Value::Int(16));
    }

    #[test]
    fn test_eval_comparison() {
        // GIVEN
        let registry = test_registry();
        let graph = test_graph();
        let evaluator = Evaluator::new(&registry, &graph);
        let mut bindings = Bindings::new();
        bindings.insert("x", Value::Int(10));

        // x >= 5 AND x < 20
        let expr = Expr::BinaryOp(
            BinaryOp::And,
            Box::new(Expr::BinaryOp(
                BinaryOp::GtEq,
                Box::new(Expr::Var("x".to_string(), Span::default())),
                Box::new(Expr::Literal(Literal {
                    kind: LiteralKind::Int(5),
                    span: Span::default(),
                })),
                Span::default(),
            )),
            Box::new(Expr::BinaryOp(
                BinaryOp::Lt,
                Box::new(Expr::Var("x".to_string(), Span::default())),
                Box::new(Expr::Literal(Literal {
                    kind: LiteralKind::Int(20),
                    span: Span::default(),
                })),
                Span::default(),
            )),
            Span::default(),
        );

        // WHEN
        let result = evaluator.eval(&expr, &bindings).unwrap();

        // THEN
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_attribute_access() {
        // GIVEN
        let registry = test_registry();
        let mut graph = test_graph();
        let node_id = graph.create_node(TypeId::new(1), attrs! { "priority" => 5 });

        let evaluator = Evaluator::new(&registry, &graph);
        let mut bindings = Bindings::new();
        bindings.insert("t", Binding::Node(node_id));

        // t.priority
        let expr = Expr::AttrAccess(
            Box::new(Expr::Var("t".to_string(), Span::default())),
            "priority".to_string(),
            Span::default(),
        );

        // WHEN
        let result = evaluator.eval(&expr, &bindings).unwrap();

        // THEN
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_eval_unbound_variable_error() {
        // GIVEN
        let registry = test_registry();
        let graph = test_graph();
        let evaluator = Evaluator::new(&registry, &graph);
        let mut bindings = Bindings::new();
        bindings.insert("x", Value::Int(10));

        // y + 1 (y is unbound)
        let expr = Expr::BinaryOp(
            BinaryOp::Add,
            Box::new(Expr::Var("y".to_string(), Span::default())),
            Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Int(1),
                span: Span::default(),
            })),
            Span::default(),
        );

        // WHEN
        let result = evaluator.eval(&expr, &bindings);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PatternError::UnboundVariable { .. }
        ));
    }

    #[test]
    fn test_eval_division_by_zero() {
        // GIVEN
        let registry = test_registry();
        let graph = test_graph();
        let evaluator = Evaluator::new(&registry, &graph);
        let bindings = Bindings::new();

        // 10 / 0
        let expr = Expr::BinaryOp(
            BinaryOp::Div,
            Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Int(10),
                span: Span::default(),
            })),
            Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Int(0),
                span: Span::default(),
            })),
            Span::default(),
        );

        // WHEN
        let result = evaluator.eval(&expr, &bindings);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PatternError::DivisionByZero));
    }
}
