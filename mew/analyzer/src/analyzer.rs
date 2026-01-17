//! Main analyzer implementation.

use crate::{AnalyzerError, AnalyzerResult, BinaryOpType, Scope, Type, UnaryOpType, VarBinding};
use mew_parser::{
    AttrAssignment, EdgePattern, Expr, FnCall, Literal, LiteralKind, MatchStmt, NodePattern,
    PatternElem, Projection, Span, Stmt,
};
use mew_registry::Registry;

/// The Analyzer performs name resolution and type checking.
pub struct Analyzer<'r> {
    /// The schema registry for type lookups.
    registry: &'r Registry,
    /// Current variable scope.
    scope: Scope,
    /// Accumulated errors (for error recovery).
    errors: Vec<AnalyzerError>,
}

impl<'r> Analyzer<'r> {
    /// Create a new analyzer with the given registry.
    pub fn new(registry: &'r Registry) -> Self {
        Self {
            registry,
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    /// Analyze a statement.
    pub fn analyze_stmt(&mut self, stmt: &Stmt) -> AnalyzerResult<Type> {
        match stmt {
            Stmt::Match(m) => self.analyze_match(m),
            Stmt::MatchMutate(mm) => self.analyze_match_mutate(mm),
            Stmt::MatchWalk(mw) => self.analyze_match_walk(mw),
            Stmt::Spawn(s) => self.analyze_spawn(s),
            Stmt::Kill(k) => self.analyze_kill(k),
            Stmt::Link(l) => self.analyze_link(l),
            Stmt::Unlink(u) => self.analyze_unlink(u),
            Stmt::Set(s) => self.analyze_set(s),
            Stmt::Walk(w) => self.analyze_walk(w),
            Stmt::Inspect(_) => Ok(Type::Any), // INSPECT returns entity data
            Stmt::Txn(_) => Ok(Type::Null), // Txn statements don't produce a value
            Stmt::Explain(e) => {
                // Analyze inner statement but return plan type
                self.analyze_stmt(&e.statement)?;
                Ok(Type::Any) // Returns plan structure
            }
            Stmt::Profile(p) => {
                // Analyze inner statement but return metrics type
                self.analyze_stmt(&p.statement)?;
                Ok(Type::Any) // Returns execution metrics
            }
        }
    }

    /// Analyze a MATCH...mutation compound statement.
    fn analyze_match_mutate(&mut self, stmt: &mew_parser::MatchMutateStmt) -> AnalyzerResult<Type> {
        // Push a new scope for pattern bindings
        self.scope.push();

        // Analyze pattern elements
        for elem in &stmt.pattern {
            self.analyze_pattern_elem(elem)?;
        }

        // Analyze WHERE clause if present
        if let Some(ref where_expr) = stmt.where_clause {
            self.analyze_expr(where_expr)?;
        }

        // Analyze each mutation
        for mutation in &stmt.mutations {
            match mutation {
                mew_parser::MutationAction::Spawn(s) => {
                    self.analyze_spawn(s)?;
                }
                mew_parser::MutationAction::Link(l) => {
                    self.analyze_link(l)?;
                }
                mew_parser::MutationAction::Set(s) => {
                    self.analyze_set(s)?;
                }
                mew_parser::MutationAction::Kill(k) => {
                    self.analyze_kill(k)?;
                }
                mew_parser::MutationAction::Unlink(u) => {
                    self.analyze_unlink(u)?;
                }
            }
        }

        // Pop scope
        self.scope.pop();

        // Returns a count of affected entities
        Ok(Type::Int)
    }

    /// Analyze a MATCH...WALK compound statement.
    fn analyze_match_walk(&mut self, stmt: &mew_parser::MatchWalkStmt) -> AnalyzerResult<Type> {
        // Push a new scope for pattern bindings
        self.scope.push();

        // Analyze pattern elements
        for elem in &stmt.pattern {
            self.analyze_pattern_elem(elem)?;
        }

        // Analyze WHERE clause if present
        if let Some(ref where_expr) = stmt.where_clause {
            self.analyze_expr(where_expr)?;
        }

        // Analyze the WALK statement (uses bound variables)
        self.analyze_walk(&stmt.walk)?;

        // Pop scope
        self.scope.pop();

        // Returns same as WALK
        Ok(Type::Any)
    }

    /// Analyze a MATCH statement.
    fn analyze_match(&mut self, stmt: &MatchStmt) -> AnalyzerResult<Type> {
        // Push a new scope for pattern bindings
        self.scope.push();

        // Analyze pattern elements
        for elem in &stmt.pattern {
            self.analyze_pattern_elem(elem)?;
        }

        // Analyze WHERE clause if present
        if let Some(where_expr) = &stmt.where_clause {
            let where_type = self.analyze_expr(where_expr)?;
            if where_type != Type::Bool && where_type != Type::Any {
                return Err(AnalyzerError::type_mismatch(
                    &Type::Bool,
                    &where_type,
                    where_expr.span(),
                ));
            }
        }

        // Analyze RETURN clause
        for proj in &stmt.return_clause.projections {
            self.analyze_projection(proj)?;
        }

        // Analyze ORDER BY if present
        if let Some(order_terms) = &stmt.order_by {
            for term in order_terms {
                self.analyze_expr(&term.expr)?;
            }
        }

        // Pop the pattern scope
        self.scope.pop();

        // MATCH returns a result set type (simplified as Any for now)
        Ok(Type::Any)
    }

    /// Analyze a pattern element.
    fn analyze_pattern_elem(&mut self, elem: &PatternElem) -> AnalyzerResult<()> {
        match elem {
            PatternElem::Node(node) => self.analyze_node_pattern(node),
            PatternElem::Edge(edge) => self.analyze_edge_pattern(edge),
        }
    }

    /// Analyze a node pattern.
    fn analyze_node_pattern(&mut self, pattern: &NodePattern) -> AnalyzerResult<()> {
        // Resolve the type name
        let type_id = self
            .registry
            .get_type_id(&pattern.type_name)
            .ok_or_else(|| AnalyzerError::unknown_type(&pattern.type_name, pattern.span))?;

        // Check for duplicate variable in current scope
        if self.scope.is_defined_in_current(&pattern.var) {
            return Err(AnalyzerError::duplicate_variable(
                &pattern.var,
                pattern.span,
            ));
        }

        // Add variable to scope
        let binding = VarBinding::new(&pattern.var, Type::NodeRef(type_id));
        self.scope.define(binding);

        Ok(())
    }

    /// Analyze an edge pattern.
    fn analyze_edge_pattern(&mut self, pattern: &EdgePattern) -> AnalyzerResult<()> {
        // Resolve the edge type name
        let edge_type_id = self
            .registry
            .get_edge_type_id(&pattern.edge_type)
            .ok_or_else(|| AnalyzerError::unknown_edge_type(&pattern.edge_type, pattern.span))?;

        // Get edge type definition to check target count
        let edge_def = self.registry.get_edge_type(edge_type_id).unwrap();
        if edge_def.arity() != pattern.targets.len() {
            return Err(AnalyzerError::WrongTargetCount {
                edge: pattern.edge_type.clone(),
                expected: edge_def.arity(),
                actual: pattern.targets.len(),
                line: pattern.span.line,
                column: pattern.span.column,
            });
        }

        // Check that target variables exist (skip "_" wildcard)
        for target in &pattern.targets {
            if target != "_" && !self.scope.is_defined(target) {
                return Err(AnalyzerError::undefined_variable(target, pattern.span));
            }
        }

        // If alias is provided, bind it
        if let Some(alias) = &pattern.alias {
            if self.scope.is_defined_in_current(alias) {
                return Err(AnalyzerError::duplicate_variable(alias, pattern.span));
            }
            let binding = VarBinding::new(alias, Type::EdgeRef(edge_type_id));
            self.scope.define(binding);
        }

        Ok(())
    }

    /// Analyze a projection.
    fn analyze_projection(&mut self, proj: &Projection) -> AnalyzerResult<Type> {
        self.analyze_expr(&proj.expr)
    }

    /// Analyze a SPAWN statement.
    fn analyze_spawn(&mut self, stmt: &mew_parser::SpawnStmt) -> AnalyzerResult<Type> {
        let mut last_type_id = None;

        // Analyze each spawn item
        for item in &stmt.items {
            // Resolve type name
            let type_id = self
                .registry
                .get_type_id(&item.type_name)
                .ok_or_else(|| AnalyzerError::unknown_type(&item.type_name, item.span))?;

            let type_def = self.registry.get_type(type_id).unwrap();

            // Check that type is not abstract
            if type_def.is_abstract {
                return Err(AnalyzerError::TypeMismatch {
                    expected: "concrete type".to_string(),
                    actual: format!("abstract type '{}'", item.type_name),
                    line: item.span.line,
                    column: item.span.column,
                });
            }

            // Analyze attribute assignments
            for attr in &item.attrs {
                self.analyze_attr_assignment(attr, &item.type_name)?;
            }

            // Bind the variable
            if !item.var.is_empty() {
                let binding = VarBinding::new(&item.var, Type::NodeRef(type_id));
                if !self.scope.define(binding) {
                    return Err(AnalyzerError::duplicate_variable(&item.var, item.span));
                }
            }

            last_type_id = Some(type_id);
        }

        // Return type of last item (for single spawns, this is the only type)
        Ok(Type::NodeRef(last_type_id.unwrap_or(mew_core::TypeId::new(0))))
    }

    /// Analyze an attribute assignment.
    fn analyze_attr_assignment(
        &mut self,
        attr: &AttrAssignment,
        type_name: &str,
    ) -> AnalyzerResult<Type> {
        // Check that the attribute exists on the type
        let type_def = self.registry.get_type_by_name(type_name);
        if let Some(td) = type_def {
            if !td.has_attr(&attr.name) {
                return Err(AnalyzerError::unknown_attribute(
                    &attr.name, type_name, attr.span,
                ));
            }
        }

        // Analyze the value expression
        self.analyze_expr(&attr.value)
    }

    /// Analyze a KILL statement.
    fn analyze_kill(&mut self, stmt: &mew_parser::KillStmt) -> AnalyzerResult<Type> {
        self.analyze_target(&stmt.target, stmt.span)?;
        Ok(Type::Null)
    }

    /// Analyze a LINK statement.
    fn analyze_link(&mut self, stmt: &mew_parser::LinkStmt) -> AnalyzerResult<Type> {
        // Resolve edge type
        let edge_type_id = self
            .registry
            .get_edge_type_id(&stmt.edge_type)
            .ok_or_else(|| AnalyzerError::unknown_edge_type(&stmt.edge_type, stmt.span))?;

        let edge_def = self.registry.get_edge_type(edge_type_id).unwrap();

        // Check target count
        if edge_def.arity() != stmt.targets.len() {
            return Err(AnalyzerError::WrongTargetCount {
                edge: stmt.edge_type.clone(),
                expected: edge_def.arity(),
                actual: stmt.targets.len(),
                line: stmt.span.line,
                column: stmt.span.column,
            });
        }

        // Analyze targets
        for target_ref in &stmt.targets {
            self.analyze_target_ref(target_ref, stmt.span)?;
        }

        // Analyze attributes
        for attr in &stmt.attrs {
            // For edges, we'd need to check against edge type attributes
            self.analyze_expr(&attr.value)?;
        }

        // Bind variable if provided
        if let Some(var) = &stmt.var {
            let binding = VarBinding::new(var, Type::EdgeRef(edge_type_id));
            if !self.scope.define(binding) {
                return Err(AnalyzerError::duplicate_variable(var, stmt.span));
            }
        }

        Ok(Type::EdgeRef(edge_type_id))
    }

    /// Analyze an UNLINK statement.
    fn analyze_unlink(&mut self, stmt: &mew_parser::UnlinkStmt) -> AnalyzerResult<Type> {
        self.analyze_target(&stmt.target, stmt.span)?;
        Ok(Type::Null)
    }

    /// Analyze a SET statement.
    fn analyze_set(&mut self, stmt: &mew_parser::SetStmt) -> AnalyzerResult<Type> {
        // Analyze the target to get its type
        let target_type = self.analyze_target(&stmt.target, stmt.span)?;

        // Get the type name for attribute checking
        if let Type::NodeRef(type_id) = &target_type {
            if let Some(type_def) = self.registry.get_type(*type_id) {
                for attr in &stmt.assignments {
                    if !type_def.has_attr(&attr.name) {
                        return Err(AnalyzerError::unknown_attribute(
                            &attr.name,
                            &type_def.name,
                            attr.span,
                        ));
                    }
                    self.analyze_expr(&attr.value)?;
                }
            }
        } else {
            // For non-node types, just analyze expressions
            for attr in &stmt.assignments {
                self.analyze_expr(&attr.value)?;
            }
        }

        Ok(Type::Null)
    }

    /// Analyze a WALK statement.
    fn analyze_walk(&mut self, stmt: &mew_parser::WalkStmt) -> AnalyzerResult<Type> {
        // Analyze the FROM expression
        let from_type = self.analyze_expr(&stmt.from)?;
        if !from_type.is_ref() && from_type != Type::Any {
            return Err(AnalyzerError::cannot_access_attribute(
                &from_type, stmt.span,
            ));
        }

        // Analyze FOLLOW clauses
        for follow in &stmt.follow {
            for edge_type_name in &follow.edge_types {
                if self.registry.get_edge_type_id(edge_type_name).is_none() {
                    return Err(AnalyzerError::unknown_edge_type(
                        edge_type_name,
                        follow.span,
                    ));
                }
            }
        }

        // Analyze UNTIL condition if present
        if let Some(until_expr) = &stmt.until {
            let until_type = self.analyze_expr(until_expr)?;
            if until_type != Type::Bool && until_type != Type::Any {
                return Err(AnalyzerError::type_mismatch(
                    &Type::Bool,
                    &until_type,
                    until_expr.span(),
                ));
            }
        }

        // WALK returns a collection
        Ok(Type::Any)
    }

    /// Analyze a target.
    fn analyze_target(&mut self, target: &mew_parser::Target, span: Span) -> AnalyzerResult<Type> {
        match target {
            mew_parser::Target::Var(name) => {
                if let Some(binding) = self.scope.lookup(name) {
                    Ok(binding.ty.clone())
                } else {
                    Err(AnalyzerError::undefined_variable(name, span))
                }
            }
            mew_parser::Target::Id(_) => Ok(Type::AnyNodeRef),
            mew_parser::Target::Pattern(match_stmt) => self.analyze_match(match_stmt),
            mew_parser::Target::EdgePattern { .. } => {
                // Edge pattern is used in UNLINK, returns an edge reference
                Ok(Type::AnyEdgeRef)
            }
        }
    }

    /// Analyze a target reference.
    fn analyze_target_ref(
        &mut self,
        target: &mew_parser::TargetRef,
        span: Span,
    ) -> AnalyzerResult<Type> {
        match target {
            mew_parser::TargetRef::Var(name) => {
                if let Some(binding) = self.scope.lookup(name) {
                    Ok(binding.ty.clone())
                } else {
                    Err(AnalyzerError::undefined_variable(name, span))
                }
            }
            mew_parser::TargetRef::Id(_) => Ok(Type::AnyNodeRef),
            mew_parser::TargetRef::Pattern(match_stmt) => self.analyze_match(match_stmt),
            mew_parser::TargetRef::InlineSpawn(spawn_stmt) => self.analyze_spawn(spawn_stmt)
        }
    }

    /// Analyze an expression and return its type.
    pub fn analyze_expr(&mut self, expr: &Expr) -> AnalyzerResult<Type> {
        match expr {
            Expr::Literal(lit) => self.analyze_literal(lit),
            Expr::Var(name, span) => self.analyze_var(name, *span),
            Expr::AttrAccess(base, attr, span) => self.analyze_attr_access(base, attr, *span),
            Expr::BinaryOp(op, left, right, span) => {
                self.analyze_binary_op(*op, left, right, *span)
            }
            Expr::UnaryOp(op, operand, span) => self.analyze_unary_op(*op, operand, *span),
            Expr::FnCall(fc) => self.analyze_fn_call(fc),
            Expr::IdRef(_, _) => Ok(Type::AnyNodeRef),
            Expr::Param(_, _) => Ok(Type::Any), // Parameters have unknown types
            Expr::Exists(pattern, where_clause, span) => {
                self.analyze_exists(pattern, where_clause.as_deref(), *span)
            }
            Expr::NotExists(pattern, where_clause, span) => {
                self.analyze_exists(pattern, where_clause.as_deref(), *span)
            }
            Expr::List(elements, _) => {
                // Analyze all elements but return a generic list type
                for elem in elements {
                    self.analyze_expr(elem)?;
                }
                Ok(Type::Any) // List type - could be refined to Type::List in future
            }
            Expr::TypeCheck(base, _type_name, _) => {
                // Type check: expr:Type - returns bool
                self.analyze_expr(base)?;
                Ok(Type::Bool)
            }
        }
    }

    /// Analyze a literal.
    fn analyze_literal(&self, lit: &Literal) -> AnalyzerResult<Type> {
        Ok(match &lit.kind {
            LiteralKind::Null => Type::Null,
            LiteralKind::Bool(_) => Type::Bool,
            LiteralKind::Int(_) => Type::Int,
            LiteralKind::Float(_) => Type::Float,
            LiteralKind::String(_) => Type::String,
            LiteralKind::Duration(_) => Type::Duration,
            LiteralKind::Timestamp(_) => Type::Timestamp,
        })
    }

    /// Analyze a variable reference.
    fn analyze_var(&self, name: &str, span: Span) -> AnalyzerResult<Type> {
        if let Some(binding) = self.scope.lookup(name) {
            Ok(binding.ty.clone())
        } else {
            Err(AnalyzerError::undefined_variable(name, span))
        }
    }

    /// Analyze an attribute access.
    fn analyze_attr_access(&mut self, base: &Expr, attr: &str, span: Span) -> AnalyzerResult<Type> {
        let base_type = self.analyze_expr(base)?;

        match &base_type {
            Type::NodeRef(type_id) => {
                // Look up the type and check if it has the attribute
                if let Some(type_def) = self.registry.get_type(*type_id) {
                    if let Some(attr_def) = type_def.get_attr(attr) {
                        // Convert attribute type name to Type
                        Ok(self.type_name_to_type(&attr_def.type_name))
                    } else {
                        Err(AnalyzerError::unknown_attribute(attr, &type_def.name, span))
                    }
                } else {
                    // Type not found - return Any
                    Ok(Type::Any)
                }
            }
            Type::AnyNodeRef => {
                // Any node ref - we can't check attributes, return Any
                Ok(Type::Any)
            }
            Type::EdgeRef(edge_type_id) => {
                // Check edge attributes
                if let Some(edge_def) = self.registry.get_edge_type(*edge_type_id) {
                    if edge_def.get_attr(attr).is_some() {
                        Ok(Type::Any) // Edge attributes simplified
                    } else {
                        Err(AnalyzerError::unknown_attribute(attr, &edge_def.name, span))
                    }
                } else {
                    Ok(Type::Any)
                }
            }
            Type::AnyEdgeRef => Ok(Type::Any),
            Type::Any => Ok(Type::Any),
            _ => Err(AnalyzerError::cannot_access_attribute(&base_type, span)),
        }
    }

    /// Convert a type name string to a Type.
    fn type_name_to_type(&self, name: &str) -> Type {
        match name {
            "Bool" | "bool" => Type::Bool,
            "Int" | "int" | "i64" => Type::Int,
            "Float" | "float" | "f64" => Type::Float,
            "String" | "string" => Type::String,
            "Timestamp" | "timestamp" => Type::Timestamp,
            "Duration" | "duration" => Type::Duration,
            _ => {
                // Check if it's a node type
                if let Some(type_id) = self.registry.get_type_id(name) {
                    Type::NodeRef(type_id)
                } else if let Some(edge_type_id) = self.registry.get_edge_type_id(name) {
                    Type::EdgeRef(edge_type_id)
                } else {
                    Type::Any
                }
            }
        }
    }

    /// Analyze a binary operation.
    fn analyze_binary_op(
        &mut self,
        op: mew_parser::BinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> AnalyzerResult<Type> {
        let left_type = self.analyze_expr(left)?;
        let right_type = self.analyze_expr(right)?;

        let op_type: BinaryOpType = op.into();

        // Check type compatibility
        match op_type {
            BinaryOpType::Eq | BinaryOpType::NotEq => {
                if !left_type.can_eq(&right_type) {
                    return Err(AnalyzerError::invalid_operator(
                        op.to_string(),
                        &left_type,
                        &right_type,
                        span,
                    ));
                }
            }
            BinaryOpType::Lt | BinaryOpType::LtEq | BinaryOpType::Gt | BinaryOpType::GtEq => {
                if !left_type.can_order(&right_type) {
                    return Err(AnalyzerError::invalid_operator(
                        op.to_string(),
                        &left_type,
                        &right_type,
                        span,
                    ));
                }
            }
            _ => {}
        }

        // Get result type
        left_type
            .binary_result(op_type, &right_type)
            .ok_or_else(|| {
                AnalyzerError::invalid_operator(op.to_string(), &left_type, &right_type, span)
            })
    }

    /// Analyze a unary operation.
    fn analyze_unary_op(
        &mut self,
        op: mew_parser::UnaryOp,
        operand: &Expr,
        span: Span,
    ) -> AnalyzerResult<Type> {
        let operand_type = self.analyze_expr(operand)?;
        let op_type: UnaryOpType = op.into();

        operand_type.unary_result(op_type).ok_or_else(|| {
            AnalyzerError::invalid_unary_operator(
                match op {
                    mew_parser::UnaryOp::Neg => "-",
                    mew_parser::UnaryOp::Not => "NOT",
                },
                &operand_type,
                span,
            )
        })
    }

    /// Analyze a function call.
    fn analyze_fn_call(&mut self, fc: &FnCall) -> AnalyzerResult<Type> {
        // Analyze all arguments
        for arg in &fc.args {
            self.analyze_expr(arg)?;
        }

        // Return type depends on function - simplified for now
        Ok(match fc.name.to_lowercase().as_str() {
            "count" => Type::Int,
            "sum" | "avg" | "min" | "max" => Type::Float,
            "concat" | "upper" | "lower" | "trim" => Type::String,
            "now" => Type::Timestamp,
            "coalesce" => {
                // Return type of first non-null argument
                if let Some(arg) = fc.args.first() {
                    self.analyze_expr(arg)?
                } else {
                    Type::Any
                }
            }
            _ => Type::Any,
        })
    }

    /// Analyze an EXISTS/NOT EXISTS expression.
    fn analyze_exists(
        &mut self,
        pattern: &[PatternElem],
        where_clause: Option<&Expr>,
        _span: Span,
    ) -> AnalyzerResult<Type> {
        // Push scope for pattern
        self.scope.push();

        // Analyze pattern elements
        for elem in pattern {
            self.analyze_pattern_elem(elem)?;
        }

        // Analyze WHERE clause
        if let Some(where_expr) = where_clause {
            let where_type = self.analyze_expr(where_expr)?;
            if where_type != Type::Bool && where_type != Type::Any {
                return Err(AnalyzerError::type_mismatch(
                    &Type::Bool,
                    &where_type,
                    where_expr.span(),
                ));
            }
        }

        // Pop scope
        self.scope.pop();

        Ok(Type::Bool)
    }

    /// Get accumulated errors.
    pub fn errors(&self) -> &[AnalyzerError] {
        &self.errors
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Clear scope and errors for fresh analysis.
    pub fn reset(&mut self) {
        self.scope = Scope::new();
        self.errors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_registry::{AttrDef, RegistryBuilder};

    fn test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .attr(AttrDef::new("age", "Int"))
            .done()
            .unwrap();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("Knows")
            .param("from", "Person")
            .param("to", "Person")
            .done()
            .unwrap();
        builder
            .add_edge_type("Assigned")
            .param("who", "Person")
            .param("what", "Task")
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_analyze_literal_int() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let expr = Expr::Literal(Literal {
            kind: LiteralKind::Int(42),
            span: Span::default(),
        });

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert_eq!(result.unwrap(), Type::Int);
    }

    #[test]
    fn test_analyze_literal_string() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let expr = Expr::Literal(Literal {
            kind: LiteralKind::String("hello".to_string()),
            span: Span::default(),
        });

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert_eq!(result.unwrap(), Type::String);
    }

    #[test]
    fn test_analyze_undefined_variable() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let expr = Expr::Var("undefined".to_string(), Span::default());

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnalyzerError::UndefinedVariable { .. }
        ));
    }

    #[test]
    fn test_analyze_binary_op_arithmetic() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let left = Expr::Literal(Literal {
            kind: LiteralKind::Int(1),
            span: Span::default(),
        });
        let right = Expr::Literal(Literal {
            kind: LiteralKind::Int(2),
            span: Span::default(),
        });
        let expr = Expr::BinaryOp(
            mew_parser::BinaryOp::Add,
            Box::new(left),
            Box::new(right),
            Span::default(),
        );

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert_eq!(result.unwrap(), Type::Int);
    }

    #[test]
    fn test_analyze_binary_op_comparison() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let left = Expr::Literal(Literal {
            kind: LiteralKind::Int(1),
            span: Span::default(),
        });
        let right = Expr::Literal(Literal {
            kind: LiteralKind::Int(2),
            span: Span::default(),
        });
        let expr = Expr::BinaryOp(
            mew_parser::BinaryOp::Lt,
            Box::new(left),
            Box::new(right),
            Span::default(),
        );

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_analyze_binary_op_type_mismatch() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let left = Expr::Literal(Literal {
            kind: LiteralKind::String("hello".to_string()),
            span: Span::default(),
        });
        let right = Expr::Literal(Literal {
            kind: LiteralKind::Int(2),
            span: Span::default(),
        });
        let expr = Expr::BinaryOp(
            mew_parser::BinaryOp::Add,
            Box::new(left),
            Box::new(right),
            Span::default(),
        );

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnalyzerError::InvalidOperator { .. }
        ));
    }

    #[test]
    fn test_analyze_unary_not() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let operand = Expr::Literal(Literal {
            kind: LiteralKind::Bool(true),
            span: Span::default(),
        });
        let expr = Expr::UnaryOp(mew_parser::UnaryOp::Not, Box::new(operand), Span::default());

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_analyze_unary_neg_type_mismatch() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let operand = Expr::Literal(Literal {
            kind: LiteralKind::String("hello".to_string()),
            span: Span::default(),
        });
        let expr = Expr::UnaryOp(mew_parser::UnaryOp::Neg, Box::new(operand), Span::default());

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnalyzerError::InvalidUnaryOperator { .. }
        ));
    }

    #[test]
    fn test_analyze_node_pattern() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let pattern = NodePattern {
            var: "p".to_string(),
            type_name: "Person".to_string(),
            span: Span::default(),
        };

        // WHEN
        analyzer.scope.push();
        let result = analyzer.analyze_node_pattern(&pattern);
        analyzer.scope.pop();

        // THEN
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_node_pattern_unknown_type() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let pattern = NodePattern {
            var: "x".to_string(),
            type_name: "Unknown".to_string(),
            span: Span::default(),
        };

        // WHEN
        let result = analyzer.analyze_node_pattern(&pattern);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnalyzerError::UnknownType { .. }
        ));
    }

    #[test]
    fn test_analyze_fn_call_count() {
        // GIVEN
        let registry = test_registry();
        let mut analyzer = Analyzer::new(&registry);
        let expr = Expr::FnCall(FnCall {
            name: "count".to_string(),
            args: vec![],
            distinct: false,
            limit: None,
            filter: None,
            span: Span::default(),
        });

        // WHEN
        let result = analyzer.analyze_expr(&expr);

        // THEN
        assert_eq!(result.unwrap(), Type::Int);
    }
}
