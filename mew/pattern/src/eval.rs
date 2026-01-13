//! Expression evaluation.

use crate::{Bindings, CompiledPattern, Matcher, PatternError, PatternResult};
use mew_core::Value;
use mew_graph::Graph;
use mew_parser::{BinaryOp, Expr, LiteralKind, PatternElem, UnaryOp};
use mew_registry::Registry;

/// Expression evaluator.
///
/// The evaluator is stateless - it takes the graph as a parameter to each eval call.
/// This allows safe use from contexts with mutable graph references.
pub struct Evaluator<'r> {
    #[allow(dead_code)]
    registry: &'r Registry,
}

impl<'r> Evaluator<'r> {
    /// Create a new evaluator.
    pub fn new(registry: &'r Registry) -> Self {
        Self { registry }
    }

    /// Evaluate an expression with the given bindings and graph.
    pub fn eval(&self, expr: &Expr, bindings: &Bindings, graph: &Graph) -> PatternResult<Value> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Var(name, _) => self.eval_var(name, bindings),
            Expr::AttrAccess(base, attr, _) => self.eval_attr_access(base, attr, bindings, graph),
            Expr::BinaryOp(op, left, right, _) => {
                self.eval_binary_op(*op, left, right, bindings, graph)
            }
            Expr::UnaryOp(op, operand, _) => self.eval_unary_op(*op, operand, bindings, graph),
            Expr::FnCall(fc) => self.eval_fn_call(&fc.name, &fc.args, bindings, graph),
            Expr::IdRef(_, _) => Ok(Value::Null), // TODO: resolve ID refs
            Expr::Param(name, _) => Err(PatternError::unbound_variable(name)),
            Expr::Exists(pattern_elems, where_clause, _) => {
                // Compile the subpattern and check if any matches exist
                let exists = self.eval_exists(pattern_elems, where_clause.as_deref(), bindings, graph)?;
                Ok(Value::Bool(exists))
            }
            Expr::NotExists(pattern_elems, where_clause, _) => {
                // Compile the subpattern and check if no matches exist
                let exists = self.eval_exists(pattern_elems, where_clause.as_deref(), bindings, graph)?;
                Ok(Value::Bool(!exists))
            }
            Expr::List(elements, _) => {
                // Evaluate each element and collect into a list
                let values: PatternResult<Vec<Value>> = elements
                    .iter()
                    .map(|e| self.eval(e, bindings, graph))
                    .collect();
                Ok(Value::List(values?))
            }
        }
    }

    /// Evaluate an EXISTS/NOT EXISTS subpattern.
    /// Uses short-circuit evaluation for better performance.
    fn eval_exists(
        &self,
        pattern_elems: &[PatternElem],
        where_clause: Option<&Expr>,
        bindings: &Bindings,
        graph: &Graph,
    ) -> PatternResult<bool> {
        // Get the names of already-bound variables
        let prebound: Vec<String> = bindings.names().map(|s| s.to_string()).collect();

        // Compile the subpattern with prebound variables
        let mut pattern =
            CompiledPattern::compile_with_prebound(pattern_elems, self.registry, &prebound)?;

        // Add where clause as filter if present
        if let Some(where_expr) = where_clause {
            pattern = pattern.with_filter(where_expr.clone());
        }

        // Use short-circuit exists check instead of finding all matches
        let matcher = Matcher::new(self.registry, graph);
        matcher.exists(&pattern, bindings.clone())
    }

    /// Evaluate a literal.
    fn eval_literal(&self, lit: &mew_parser::Literal) -> PatternResult<Value> {
        Ok(match &lit.kind {
            LiteralKind::Null => Value::Null,
            LiteralKind::Bool(b) => Value::Bool(*b),
            LiteralKind::Int(i) => Value::Int(*i),
            LiteralKind::Float(f) => Value::Float(*f),
            LiteralKind::String(s) => Value::String(s.clone()),
            LiteralKind::Duration(ms) => Value::Duration(*ms),
            LiteralKind::Timestamp(ms) => Value::Timestamp(*ms),
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
        graph: &Graph,
    ) -> PatternResult<Value> {
        let base_val = self.eval(base, bindings, graph)?;

        match base_val {
            Value::NodeRef(node_id) => {
                // Get the attribute from the node
                if let Some(node) = graph.get_node(node_id) {
                    Ok(node.get_attr(attr).cloned().unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            Value::EdgeRef(edge_id) => {
                // Get the attribute from the edge
                if let Some(edge) = graph.get_edge(edge_id) {
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
        graph: &Graph,
    ) -> PatternResult<Value> {
        let left_val = self.eval(left, bindings, graph)?;
        let right_val = self.eval(right, bindings, graph)?;

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

            // Null coalescing: return left if not null, otherwise right
            BinaryOp::NullCoalesce => {
                if matches!(left_val, Value::Null) {
                    Ok(right_val)
                } else {
                    Ok(left_val)
                }
            }
        }
    }

    /// Evaluate a unary operation.
    fn eval_unary_op(
        &self,
        op: UnaryOp,
        operand: &Expr,
        bindings: &Bindings,
        graph: &Graph,
    ) -> PatternResult<Value> {
        let val = self.eval(operand, bindings, graph)?;

        match op {
            UnaryOp::Neg => match val {
                Value::Int(i) => Ok(Value::Int(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(PatternError::type_error(format!("cannot negate {:?}", val))),
            },
            UnaryOp::Not => match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                Value::Null => Ok(Value::Null), // NOT NULL = NULL
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
        graph: &Graph,
    ) -> PatternResult<Value> {
        let name_lower = name.to_lowercase();

        match name_lower.as_str() {
            "now" => {
                // Return current timestamp in milliseconds since epoch
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Ok(Value::Timestamp(duration.as_millis() as i64))
            }
            "year" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::Timestamp(ms) = val {
                        let (year, _, _) = Self::timestamp_to_date(ms);
                        return Ok(Value::Int(year as i64));
                    }
                    return Err(PatternError::type_error("YEAR expects a timestamp argument"));
                }
                Err(PatternError::type_error("YEAR expects one argument"))
            }
            "month" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::Timestamp(ms) = val {
                        let (_, month, _) = Self::timestamp_to_date(ms);
                        return Ok(Value::Int(month as i64));
                    }
                    return Err(PatternError::type_error("MONTH expects a timestamp argument"));
                }
                Err(PatternError::type_error("MONTH expects one argument"))
            }
            "day" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::Timestamp(ms) = val {
                        let (_, _, day) = Self::timestamp_to_date(ms);
                        return Ok(Value::Int(day as i64));
                    }
                    return Err(PatternError::type_error("DAY expects a timestamp argument"));
                }
                Err(PatternError::type_error("DAY expects one argument"))
            }
            "hour" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::Timestamp(ms) = val {
                        let (hour, _, _) = Self::timestamp_to_time(ms);
                        return Ok(Value::Int(hour as i64));
                    }
                    return Err(PatternError::type_error("HOUR expects a timestamp argument"));
                }
                Err(PatternError::type_error("HOUR expects one argument"))
            }
            "minute" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::Timestamp(ms) = val {
                        let (_, minute, _) = Self::timestamp_to_time(ms);
                        return Ok(Value::Int(minute as i64));
                    }
                    return Err(PatternError::type_error("MINUTE expects a timestamp argument"));
                }
                Err(PatternError::type_error("MINUTE expects one argument"))
            }
            "second" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::Timestamp(ms) = val {
                        let (_, _, second) = Self::timestamp_to_time(ms);
                        return Ok(Value::Int(second as i64));
                    }
                    return Err(PatternError::type_error("SECOND expects a timestamp argument"));
                }
                Err(PatternError::type_error("SECOND expects one argument"))
            }
            "count" => {
                // COUNT expects 0 or 1 arguments
                // For now, just return 0 - proper aggregate handling is in Query
                Ok(Value::Int(0))
            }
            "sum" | "avg" => {
                // Aggregates - placeholder for now (handled in Query executor)
                Ok(Value::Float(0.0))
            }
            "min" => {
                // Binary min: min(a, b) returns smaller value
                if args.len() >= 2 {
                    return self.binary_numeric_compare(&args[0], &args[1], bindings, graph, |a, b| a < b);
                }
                Ok(Value::Null) // Aggregate placeholder
            }
            "max" => {
                // Binary max: max(a, b) returns larger value
                if args.len() >= 2 {
                    return self.binary_numeric_compare(&args[0], &args[1], bindings, graph, |a, b| a > b);
                }
                Ok(Value::Null) // Aggregate placeholder
            }
            "coalesce" => {
                // Return first non-null argument
                for arg in args {
                    let val = self.eval(arg, bindings, graph)?;
                    if !matches!(val, Value::Null) {
                        return Ok(val);
                    }
                }
                Ok(Value::Null)
            }
            "upper" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::String(s) = val {
                        return Ok(Value::String(s.to_uppercase()));
                    }
                }
                Err(PatternError::type_error("UPPER expects a string argument"))
            }
            "lower" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::String(s) = val {
                        return Ok(Value::String(s.to_lowercase()));
                    }
                }
                Err(PatternError::type_error("LOWER expects a string argument"))
            }
            "abs" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return match val {
                        Value::Int(i) => Ok(Value::Int(i.abs())),
                        Value::Float(f) => Ok(Value::Float(f.abs())),
                        _ => Err(PatternError::type_error("ABS expects a numeric argument")),
                    };
                }
                Err(PatternError::type_error("ABS expects one argument"))
            }
            "is_null" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return Ok(Value::Bool(matches!(val, Value::Null)));
                }
                Err(PatternError::type_error("IS_NULL expects one argument"))
            }
            "is_not_null" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return Ok(Value::Bool(!matches!(val, Value::Null)));
                }
                Err(PatternError::type_error("IS_NOT_NULL expects one argument"))
            }
            "length" | "len" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return match val {
                        Value::String(s) => Ok(Value::Int(s.len() as i64)),
                        Value::Null => Ok(Value::Null),
                        _ => Err(PatternError::type_error("LENGTH expects a string argument")),
                    };
                }
                Err(PatternError::type_error("LENGTH expects one argument"))
            }
            "concat" => {
                let mut result = String::new();
                for arg in args {
                    let val = self.eval(arg, bindings, graph)?;
                    match val {
                        Value::String(s) => result.push_str(&s),
                        Value::Int(i) => result.push_str(&i.to_string()),
                        Value::Float(f) => result.push_str(&f.to_string()),
                        Value::Bool(b) => result.push_str(&b.to_string()),
                        Value::Null => {}
                        _ => return Err(PatternError::type_error("CONCAT expects string or primitive arguments")),
                    }
                }
                Ok(Value::String(result))
            }
            "substring" | "substr" => {
                // substring(string, start, length)
                if args.len() >= 2 {
                    let s = self.eval(&args[0], bindings, graph)?;
                    let start = self.eval(&args[1], bindings, graph)?;

                    if let (Value::String(s), Value::Int(start)) = (s, start) {
                        let start_idx = start.max(0) as usize;

                        let result = if args.len() >= 3 {
                            let length = self.eval(&args[2], bindings, graph)?;
                            if let Value::Int(len) = length {
                                s.chars().skip(start_idx).take(len.max(0) as usize).collect()
                            } else {
                                s.chars().skip(start_idx).collect()
                            }
                        } else {
                            s.chars().skip(start_idx).collect()
                        };
                        return Ok(Value::String(result));
                    }
                }
                Err(PatternError::type_error("SUBSTRING expects (string, start[, length])"))
            }
            "trim" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    if let Value::String(s) = val {
                        return Ok(Value::String(s.trim().to_string()));
                    }
                }
                Err(PatternError::type_error("TRIM expects a string argument"))
            }
            "starts_with" => {
                if args.len() >= 2 {
                    let s = self.eval(&args[0], bindings, graph)?;
                    let prefix = self.eval(&args[1], bindings, graph)?;
                    if let (Value::String(s), Value::String(prefix)) = (s, prefix) {
                        return Ok(Value::Bool(s.starts_with(&prefix)));
                    }
                }
                Err(PatternError::type_error("STARTS_WITH expects (string, prefix)"))
            }
            "ends_with" => {
                if args.len() >= 2 {
                    let s = self.eval(&args[0], bindings, graph)?;
                    let suffix = self.eval(&args[1], bindings, graph)?;
                    if let (Value::String(s), Value::String(suffix)) = (s, suffix) {
                        return Ok(Value::Bool(s.ends_with(&suffix)));
                    }
                }
                Err(PatternError::type_error("ENDS_WITH expects (string, suffix)"))
            }
            "contains" => {
                if args.len() >= 2 {
                    let s = self.eval(&args[0], bindings, graph)?;
                    let pattern = self.eval(&args[1], bindings, graph)?;
                    if let (Value::String(s), Value::String(pattern)) = (s, pattern) {
                        return Ok(Value::Bool(s.contains(&pattern)));
                    }
                }
                Err(PatternError::type_error("CONTAINS expects (string, pattern)"))
            }
            "in" => {
                // x IN [a, b, c] - check if x is in the list
                if args.len() >= 2 {
                    let needle = self.eval(&args[0], bindings, graph)?;
                    let haystack = self.eval(&args[1], bindings, graph)?;
                    if let Value::List(list) = haystack {
                        return Ok(Value::Bool(list.contains(&needle)));
                    }
                }
                Err(PatternError::type_error("IN expects (value, list)"))
            }
            "replace" => {
                if args.len() >= 3 {
                    let s = self.eval(&args[0], bindings, graph)?;
                    let from = self.eval(&args[1], bindings, graph)?;
                    let to = self.eval(&args[2], bindings, graph)?;
                    if let (Value::String(s), Value::String(from), Value::String(to)) = (s, from, to) {
                        return Ok(Value::String(s.replace(&from, &to)));
                    }
                }
                Err(PatternError::type_error("REPLACE expects (string, from, to)"))
            }
            "floor" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return match val {
                        Value::Int(i) => Ok(Value::Int(i)),
                        Value::Float(f) => Ok(Value::Float(f.floor())),
                        _ => Err(PatternError::type_error("FLOOR expects a numeric argument")),
                    };
                }
                Err(PatternError::type_error("FLOOR expects one argument"))
            }
            "ceil" | "ceiling" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return match val {
                        Value::Int(i) => Ok(Value::Int(i)),
                        Value::Float(f) => Ok(Value::Float(f.ceil())),
                        _ => Err(PatternError::type_error("CEIL expects a numeric argument")),
                    };
                }
                Err(PatternError::type_error("CEIL expects one argument"))
            }
            "round" => {
                if let Some(arg) = args.first() {
                    let val = self.eval(arg, bindings, graph)?;
                    return match val {
                        Value::Int(i) => Ok(Value::Int(i)),
                        Value::Float(f) => Ok(Value::Float(f.round())),
                        _ => Err(PatternError::type_error("ROUND expects a numeric argument")),
                    };
                }
                Err(PatternError::type_error("ROUND expects one argument"))
            }
            _ => Err(PatternError::invalid_operation(format!(
                "unknown function '{}'",
                name
            ))),
        }
    }

    // ========== Arithmetic helpers ==========

    /// Compare two numeric values and return the one matching the predicate.
    /// Used for binary min/max functions.
    fn binary_numeric_compare<F>(
        &self,
        left_expr: &Expr,
        right_expr: &Expr,
        bindings: &Bindings,
        graph: &Graph,
        prefer_left: F,
    ) -> PatternResult<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        let a = self.eval(left_expr, bindings, graph)?;
        let b = self.eval(right_expr, bindings, graph)?;

        match (&a, &b) {
            (Value::Int(av), Value::Int(bv)) => {
                Ok(Value::Int(if prefer_left(*av as f64, *bv as f64) { *av } else { *bv }))
            }
            (Value::Float(av), Value::Float(bv)) => {
                Ok(Value::Float(if prefer_left(*av, *bv) { *av } else { *bv }))
            }
            (Value::Int(av), Value::Float(bv)) => {
                let af = *av as f64;
                Ok(Value::Float(if prefer_left(af, *bv) { af } else { *bv }))
            }
            (Value::Float(av), Value::Int(bv)) => {
                let bf = *bv as f64;
                Ok(Value::Float(if prefer_left(*av, bf) { *av } else { bf }))
            }
            _ => Err(PatternError::type_error("MIN/MAX expects numeric arguments")),
        }
    }

    fn eval_add(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            // Timestamp + Duration = Timestamp
            (Value::Timestamp(ts), Value::Duration(dur)) => Ok(Value::Timestamp(ts + dur)),
            (Value::Duration(dur), Value::Timestamp(ts)) => Ok(Value::Timestamp(ts + dur)),
            // Duration + Duration = Duration
            (Value::Duration(a), Value::Duration(b)) => Ok(Value::Duration(a + b)),
            _ => Err(PatternError::type_error(format!(
                "cannot add {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_sub(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            // Timestamp - Duration = Timestamp
            (Value::Timestamp(ts), Value::Duration(dur)) => Ok(Value::Timestamp(ts - dur)),
            // Timestamp - Timestamp = Duration
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(Value::Duration(a - b)),
            // Duration - Duration = Duration
            (Value::Duration(a), Value::Duration(b)) => Ok(Value::Duration(a - b)),
            _ => Err(PatternError::type_error(format!(
                "cannot subtract {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_mul(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
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
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
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
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
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
        match (left, right) {
            // Null propagation - comparison with null returns null
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(Value::Bool(a < b)),
            (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a < b)),
            _ => Err(PatternError::type_error(format!(
                "cannot compare {:?} < {:?}",
                left, right
            ))),
        }
    }

    fn eval_lte(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a <= (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(Value::Bool(a <= b)),
            (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(PatternError::type_error(format!(
                "cannot compare {:?} <= {:?}",
                left, right
            ))),
        }
    }

    fn eval_gt(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(Value::Bool(a > b)),
            (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a > b)),
            _ => Err(PatternError::type_error(format!(
                "cannot compare {:?} > {:?}",
                left, right
            ))),
        }
    }

    fn eval_gte(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            // Null propagation
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a >= (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(Value::Bool(a >= b)),
            (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(PatternError::type_error(format!(
                "cannot compare {:?} >= {:?}",
                left, right
            ))),
        }
    }

    // ========== Logical helpers ==========

    fn eval_and(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            // SQL three-valued logic for AND:
            // NULL AND true = NULL, NULL AND false = false
            (Value::Null, Value::Bool(b)) | (Value::Bool(b), Value::Null) => {
                if *b {
                    Ok(Value::Null)
                } else {
                    Ok(Value::Bool(false))
                }
            }
            (Value::Null, Value::Null) => Ok(Value::Null),
            _ => Err(PatternError::type_error(format!(
                "cannot AND {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn eval_or(&self, left: &Value, right: &Value) -> PatternResult<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            // SQL three-valued logic for OR:
            // NULL OR true = true, NULL OR false = NULL
            (Value::Null, Value::Bool(b)) | (Value::Bool(b), Value::Null) => {
                if *b {
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Null)
                }
            }
            (Value::Null, Value::Null) => Ok(Value::Null),
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

    // ========== Timestamp helpers ==========

    /// Convert milliseconds since epoch to (year, month, day).
    fn timestamp_to_date(ms: i64) -> (i32, u32, u32) {
        // Convert ms to days since epoch
        let days = (ms / 86_400_000) as i32;
        Self::days_to_date(days)
    }

    /// Convert milliseconds since epoch to (hour, minute, second).
    fn timestamp_to_time(ms: i64) -> (u32, u32, u32) {
        let ms_in_day = ms.rem_euclid(86_400_000) as u64;
        let total_seconds = ms_in_day / 1000;
        let hour = (total_seconds / 3600) as u32;
        let minute = ((total_seconds % 3600) / 60) as u32;
        let second = (total_seconds % 60) as u32;
        (hour, minute, second)
    }

    /// Convert days since Unix epoch (1970-01-01) to (year, month, day).
    fn days_to_date(days: i32) -> (i32, u32, u32) {
        // Algorithm from https://howardhinnant.github.io/date_algorithms.html
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = (z - era * 146097) as u32; // day of era [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
        let y = yoe as i32 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
        let mp = (5 * doy + 2) / 153; // month offset [0, 11]
        let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
        let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
        let y = if m <= 2 { y + 1 } else { y };
        (y, m, d)
    }

    /// Evaluate an expression and convert to bool.
    pub fn eval_bool(
        &self,
        expr: &Expr,
        bindings: &Bindings,
        graph: &Graph,
    ) -> PatternResult<bool> {
        let val = self.eval(expr, bindings, graph)?;
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
        let evaluator = Evaluator::new(&registry);
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
        let result = evaluator.eval(&expr, &bindings, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Int(16));
    }

    #[test]
    fn test_eval_comparison() {
        // GIVEN
        let registry = test_registry();
        let graph = test_graph();
        let evaluator = Evaluator::new(&registry);
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
        let result = evaluator.eval(&expr, &bindings, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_attribute_access() {
        // GIVEN
        let registry = test_registry();
        let mut graph = test_graph();
        let node_id = graph.create_node(TypeId::new(1), attrs! { "priority" => 5 });

        let evaluator = Evaluator::new(&registry);
        let mut bindings = Bindings::new();
        bindings.insert("t", Binding::Node(node_id));

        // t.priority
        let expr = Expr::AttrAccess(
            Box::new(Expr::Var("t".to_string(), Span::default())),
            "priority".to_string(),
            Span::default(),
        );

        // WHEN
        let result = evaluator.eval(&expr, &bindings, &graph).unwrap();

        // THEN
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_eval_unbound_variable_error() {
        // GIVEN
        let registry = test_registry();
        let graph = test_graph();
        let evaluator = Evaluator::new(&registry);
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
        let result = evaluator.eval(&expr, &bindings, &graph);

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
        let evaluator = Evaluator::new(&registry);
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
        let result = evaluator.eval(&expr, &bindings, &graph);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PatternError::DivisionByZero));
    }
}
