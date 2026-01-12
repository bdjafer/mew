//! Parser for MEW source text.

use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::{Lexer, Token, TokenKind};

/// Parser state.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from source text.
    pub fn new(input: &str) -> ParseResult<Self> {
        let tokens = Lexer::new(input).tokenize()?;
        Ok(Self { tokens, pos: 0 })
    }

    /// Parse a single statement.
    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Match => self.parse_match().map(Stmt::Match),
            TokenKind::Spawn => self.parse_spawn().map(Stmt::Spawn),
            TokenKind::Kill => self.parse_kill().map(Stmt::Kill),
            TokenKind::Link => self.parse_link().map(Stmt::Link),
            TokenKind::Unlink => self.parse_unlink().map(Stmt::Unlink),
            TokenKind::Set => self.parse_set().map(Stmt::Set),
            TokenKind::Walk => self.parse_walk().map(Stmt::Walk),
            TokenKind::Begin => {
                let _span = self.advance().span;
                let isolation = if self.check(&TokenKind::Read) {
                    self.advance();
                    self.expect(&TokenKind::Committed)?;
                    Some(IsolationLevel::ReadCommitted)
                } else if self.check(&TokenKind::Serializable) {
                    self.advance();
                    Some(IsolationLevel::Serializable)
                } else {
                    None
                };
                Ok(Stmt::Txn(TxnStmt::Begin { isolation }))
            }
            TokenKind::Commit => {
                self.advance();
                Ok(Stmt::Txn(TxnStmt::Commit))
            }
            TokenKind::Rollback => {
                self.advance();
                Ok(Stmt::Txn(TxnStmt::Rollback))
            }
            _ => Err(ParseError::unexpected_token(
                token.span,
                "statement",
                token.kind.name(),
            )),
        }
    }

    // ==================== MATCH ====================

    fn parse_match(&mut self) -> ParseResult<MatchStmt> {
        let start = self.expect(&TokenKind::Match)?.span;

        // Parse pattern
        let pattern = self.parse_pattern()?;

        // Parse optional WHERE
        let where_clause = if self.check(&TokenKind::Where) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        // Parse RETURN (required)
        let return_clause = self.parse_return_clause()?;

        // Parse optional ORDER BY
        let order_by = if self.check(&TokenKind::Order) {
            self.advance();
            self.expect(&TokenKind::By)?;
            Some(self.parse_order_terms()?)
        } else {
            None
        };

        // Parse optional LIMIT
        let limit = if self.check(&TokenKind::Limit) {
            self.advance();
            Some(self.expect_int()?)
        } else {
            None
        };

        // Parse optional OFFSET
        let offset = if self.check(&TokenKind::Offset) {
            self.advance();
            Some(self.expect_int()?)
        } else {
            None
        };

        let span = self.span_from(start);

        Ok(MatchStmt {
            pattern,
            where_clause,
            return_clause,
            order_by,
            limit,
            offset,
            span,
        })
    }

    fn parse_pattern(&mut self) -> ParseResult<Vec<PatternElem>> {
        let mut elements = Vec::new();

        // First element
        elements.push(self.parse_pattern_elem()?);

        // Additional elements separated by comma
        while self.check(&TokenKind::Comma) {
            self.advance();
            elements.push(self.parse_pattern_elem()?);
        }

        Ok(elements)
    }

    /// Parse a pattern element: node pattern (var: Type) or edge pattern (edge(targets)).
    /// Supports transitive edge patterns with + or * modifiers.
    fn parse_pattern_elem(&mut self) -> ParseResult<PatternElem> {
        let start = self.peek().span;
        let name = self.expect_ident()?;

        if self.check(&TokenKind::Colon) {
            // Node pattern: var: Type
            self.advance();
            let type_name = self.expect_ident()?;
            let span = self.span_from(start);
            Ok(PatternElem::Node(NodePattern {
                var: name,
                type_name,
                span,
            }))
        } else if self.check(&TokenKind::LParen)
            || self.check(&TokenKind::Plus)
            || self.check(&TokenKind::Star)
        {
            self.parse_edge_pattern_body(name, start)
        } else {
            Err(ParseError::unexpected_token(
                self.peek().span,
                ": or (",
                self.peek().kind.name(),
            ))
        }
    }

    /// Parse the body of an edge pattern after the edge type name.
    /// Handles transitive modifiers (+, *), targets, and optional alias.
    fn parse_edge_pattern_body(&mut self, edge_type: String, start: Span) -> ParseResult<PatternElem> {
        // Parse optional transitive modifier
        let transitive = if self.check(&TokenKind::Plus) {
            self.advance();
            Some(TransitiveKind::Plus)
        } else if self.check(&TokenKind::Star) {
            self.advance();
            Some(TransitiveKind::Star)
        } else {
            None
        };

        // Parse targets list
        self.expect(&TokenKind::LParen)?;
        let mut targets = Vec::new();
        if !self.check(&TokenKind::RParen) {
            targets.push(self.parse_edge_target()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                targets.push(self.parse_edge_target()?);
            }
        }
        self.expect(&TokenKind::RParen)?;

        // Parse optional alias
        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };

        let span = self.span_from(start);
        Ok(PatternElem::Edge(EdgePattern {
            edge_type,
            targets,
            alias,
            transitive,
            span,
        }))
    }

    /// Parse an edge target: identifier or underscore for wildcard.
    fn parse_edge_target(&mut self) -> ParseResult<String> {
        if self.check_ident("_") {
            self.advance();
            Ok("_".to_string())
        } else {
            self.expect_ident()
        }
    }

    fn parse_return_clause(&mut self) -> ParseResult<ReturnClause> {
        let start = self.expect(&TokenKind::Return)?.span;

        let distinct = if self.check(&TokenKind::Distinct) {
            self.advance();
            true
        } else {
            false
        };

        let projections = self.parse_projections()?;
        let span = self.span_from(start);

        Ok(ReturnClause {
            distinct,
            projections,
            span,
        })
    }

    fn parse_projections(&mut self) -> ParseResult<Vec<Projection>> {
        let mut projections = Vec::new();
        projections.push(self.parse_projection()?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            projections.push(self.parse_projection()?);
        }

        Ok(projections)
    }

    fn parse_projection(&mut self) -> ParseResult<Projection> {
        let start = self.peek().span;

        // Check for *
        if self.check(&TokenKind::Star) {
            let span = self.advance().span;
            return Ok(Projection {
                expr: Expr::Var("*".to_string(), span),
                alias: None,
                span,
            });
        }

        let expr = self.parse_expr()?;

        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };

        let span = self.span_from(start);
        Ok(Projection { expr, alias, span })
    }

    fn parse_order_terms(&mut self) -> ParseResult<Vec<OrderTerm>> {
        let mut terms = Vec::new();
        terms.push(self.parse_order_term()?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            terms.push(self.parse_order_term()?);
        }

        Ok(terms)
    }

    fn parse_order_term(&mut self) -> ParseResult<OrderTerm> {
        let start = self.peek().span;
        let expr = self.parse_expr()?;

        let direction = if self.check(&TokenKind::Asc) {
            self.advance();
            OrderDirection::Asc
        } else if self.check(&TokenKind::Desc) {
            self.advance();
            OrderDirection::Desc
        } else {
            OrderDirection::Asc
        };

        let span = self.span_from(start);
        Ok(OrderTerm {
            expr,
            direction,
            span,
        })
    }

    // ==================== SPAWN ====================

    fn parse_spawn(&mut self) -> ParseResult<SpawnStmt> {
        let start = self.expect(&TokenKind::Spawn)?.span;

        let var = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let type_name = self.expect_ident()?;

        let attrs = if self.check(&TokenKind::LBrace) {
            self.parse_attr_block()?
        } else {
            Vec::new()
        };

        let returning = self.parse_optional_returning()?;
        let span = self.span_from(start);

        Ok(SpawnStmt {
            var,
            type_name,
            attrs,
            returning,
            span,
        })
    }

    fn parse_attr_block(&mut self) -> ParseResult<Vec<AttrAssignment>> {
        self.expect(&TokenKind::LBrace)?;

        let mut attrs = Vec::new();
        if !self.check(&TokenKind::RBrace) {
            attrs.push(self.parse_attr_assignment()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::RBrace) {
                    break; // trailing comma
                }
                attrs.push(self.parse_attr_assignment()?);
            }
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(attrs)
    }

    fn parse_attr_assignment(&mut self) -> ParseResult<AttrAssignment> {
        let start = self.peek().span;
        // Use expect_name to allow keywords like 'order' as attribute names
        let name = self.expect_name()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr()?;
        let span = self.span_from(start);

        Ok(AttrAssignment { name, value, span })
    }

    fn parse_optional_returning(&mut self) -> ParseResult<Option<ReturningClause>> {
        if self.check(&TokenKind::Returning) {
            self.advance();
            Ok(Some(self.parse_returning_clause()?))
        } else {
            Ok(None)
        }
    }

    fn parse_returning_clause(&mut self) -> ParseResult<ReturningClause> {
        if self.check_ident("id") {
            self.advance();
            Ok(ReturningClause::Id)
        } else if self.check(&TokenKind::Star) {
            self.advance();
            Ok(ReturningClause::All)
        } else {
            let mut fields = Vec::new();
            fields.push(self.expect_ident()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                fields.push(self.expect_ident()?);
            }
            Ok(ReturningClause::Fields(fields))
        }
    }

    // ==================== KILL ====================

    fn parse_kill(&mut self) -> ParseResult<KillStmt> {
        let start = self.expect(&TokenKind::Kill)?.span;

        let target = self.parse_target()?;

        let cascade = if self.check(&TokenKind::Cascade) {
            self.advance();
            Some(true)
        } else if self.check(&TokenKind::No) {
            self.advance();
            self.expect(&TokenKind::Cascade)?;
            Some(false)
        } else {
            None
        };

        let returning = self.parse_optional_returning()?;
        let span = self.span_from(start);

        Ok(KillStmt {
            target,
            cascade,
            returning,
            span,
        })
    }

    fn parse_target(&mut self) -> ParseResult<Target> {
        if self.check(&TokenKind::Hash) {
            self.advance();
            let id = self.expect_ident()?;
            Ok(Target::Id(id))
        } else if self.check(&TokenKind::LBrace) {
            self.advance();
            let pattern = self.parse_match()?;
            self.expect(&TokenKind::RBrace)?;
            Ok(Target::Pattern(Box::new(pattern)))
        } else {
            let var = self.expect_ident()?;
            Ok(Target::Var(var))
        }
    }

    // ==================== LINK ====================

    fn parse_link(&mut self) -> ParseResult<LinkStmt> {
        let start = self.expect(&TokenKind::Link)?.span;

        // Check for optional variable: "LINK e: edge_type(...)"
        let (var, edge_type) = if self.peek_is_ident() {
            let first = self.expect_ident()?;
            if self.check(&TokenKind::Colon) {
                self.advance();
                let edge_type = self.expect_ident()?;
                (Some(first), edge_type)
            } else {
                (None, first)
            }
        } else {
            return Err(ParseError::unexpected_token(
                self.peek().span,
                "identifier",
                self.peek().kind.name(),
            ));
        };

        self.expect(&TokenKind::LParen)?;

        let mut targets = Vec::new();
        if !self.check(&TokenKind::RParen) {
            targets.push(self.parse_target_ref()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                targets.push(self.parse_target_ref()?);
            }
        }

        self.expect(&TokenKind::RParen)?;

        let attrs = if self.check(&TokenKind::LBrace) {
            self.parse_attr_block()?
        } else {
            Vec::new()
        };

        let returning = self.parse_optional_returning()?;
        let span = self.span_from(start);

        Ok(LinkStmt {
            var,
            edge_type,
            targets,
            attrs,
            returning,
            span,
        })
    }

    fn parse_target_ref(&mut self) -> ParseResult<TargetRef> {
        if self.check(&TokenKind::Hash) {
            self.advance();
            let id = self.expect_ident()?;
            Ok(TargetRef::Id(id))
        } else if self.check(&TokenKind::LBrace) {
            self.advance();
            let pattern = self.parse_match()?;
            self.expect(&TokenKind::RBrace)?;
            Ok(TargetRef::Pattern(Box::new(pattern)))
        } else {
            let var = self.expect_ident()?;
            Ok(TargetRef::Var(var))
        }
    }

    // ==================== UNLINK ====================

    fn parse_unlink(&mut self) -> ParseResult<UnlinkStmt> {
        let start = self.expect(&TokenKind::Unlink)?.span;

        let target = self.parse_target()?;
        let returning = self.parse_optional_returning()?;
        let span = self.span_from(start);

        Ok(UnlinkStmt {
            target,
            returning,
            span,
        })
    }

    // ==================== SET ====================

    fn parse_set(&mut self) -> ParseResult<SetStmt> {
        let start = self.expect(&TokenKind::Set)?.span;

        let target = self.parse_target()?;

        let assignments = if self.check(&TokenKind::LBrace) {
            // Block syntax: SET target { attr = val, ... }
            self.parse_attr_block()?
        } else {
            // Single syntax: SET target.attr = val
            self.expect(&TokenKind::Dot)?;
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let value = self.parse_expr()?;
            let span = self.span_from(start);
            vec![AttrAssignment { name, value, span }]
        };

        let returning = self.parse_optional_returning()?;
        let span = self.span_from(start);

        Ok(SetStmt {
            target,
            assignments,
            returning,
            span,
        })
    }

    // ==================== WALK ====================

    fn parse_walk(&mut self) -> ParseResult<WalkStmt> {
        let start = self.expect(&TokenKind::Walk)?.span;

        self.expect(&TokenKind::From)?;
        let from = self.parse_expr()?;

        let mut follow = Vec::new();
        while self.check(&TokenKind::Follow) {
            follow.push(self.parse_follow_clause()?);
        }

        let until = if self.check(&TokenKind::Until) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let return_type = self.parse_walk_return()?;
        let span = self.span_from(start);

        Ok(WalkStmt {
            from,
            follow,
            until,
            return_type,
            span,
        })
    }

    fn parse_follow_clause(&mut self) -> ParseResult<FollowClause> {
        let start = self.expect(&TokenKind::Follow)?.span;

        let mut edge_types = Vec::new();
        if self.check(&TokenKind::Star) {
            self.advance();
            edge_types.push("*".to_string());
        } else {
            edge_types.push(self.expect_ident()?);
            while self.check(&TokenKind::Pipe) {
                self.advance();
                edge_types.push(self.expect_ident()?);
            }
        }

        let direction = if self.check(&TokenKind::Outbound) {
            self.advance();
            WalkDirection::Outbound
        } else if self.check(&TokenKind::Inbound) {
            self.advance();
            WalkDirection::Inbound
        } else if self.check(&TokenKind::Any) {
            self.advance();
            WalkDirection::Any
        } else {
            WalkDirection::Outbound
        };

        let (min_depth, max_depth) = if self.check(&TokenKind::LBracket) {
            self.advance();
            self.expect_keyword("depth")?;
            self.expect(&TokenKind::Colon)?;
            let min = self.expect_int()?;
            let max = if self.check(&TokenKind::Range) {
                self.advance();
                Some(self.expect_int()?)
            } else {
                None
            };
            self.expect(&TokenKind::RBracket)?;
            (Some(min), max)
        } else {
            (None, None)
        };

        let span = self.span_from(start);

        Ok(FollowClause {
            edge_types,
            direction,
            min_depth,
            max_depth,
            span,
        })
    }

    fn parse_walk_return(&mut self) -> ParseResult<WalkReturnType> {
        self.expect(&TokenKind::Return)?;

        if self.check(&TokenKind::Path) {
            self.advance();
            Ok(WalkReturnType::Path)
        } else if self.check(&TokenKind::Nodes) {
            self.advance();
            Ok(WalkReturnType::Nodes)
        } else if self.check(&TokenKind::Edges) {
            self.advance();
            Ok(WalkReturnType::Edges)
        } else if self.check(&TokenKind::Terminal) {
            self.advance();
            Ok(WalkReturnType::Terminal)
        } else {
            let projections = self.parse_projections()?;
            Ok(WalkReturnType::Projections(projections))
        }
    }

    // ==================== EXPRESSIONS ====================

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and()?;

        while self.check(&TokenKind::Or) {
            let start = left.span();
            self.advance();
            let right = self.parse_and()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(BinaryOp::Or, Box::new(left), Box::new(right), span);
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_not()?;

        while self.check(&TokenKind::And) {
            let start = left.span();
            self.advance();
            let right = self.parse_not()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(BinaryOp::And, Box::new(left), Box::new(right), span);
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> ParseResult<Expr> {
        if self.check(&TokenKind::Not) {
            let start = self.advance().span;
            // Check for NOT EXISTS
            if self.check(&TokenKind::Exists) {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let pattern = self.parse_pattern()?;
                let where_clause = if self.check(&TokenKind::Where) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect(&TokenKind::RParen)?;
                let span = self.span_from(start);
                return Ok(Expr::NotExists(pattern, where_clause, span));
            }

            let expr = self.parse_not()?;
            let span = self.span_from(start);
            Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr), span))
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_additive()?;

        loop {
            let op = if self.check(&TokenKind::Eq) {
                BinaryOp::Eq
            } else if self.check(&TokenKind::NotEq) {
                BinaryOp::NotEq
            } else if self.check(&TokenKind::Lt) {
                BinaryOp::Lt
            } else if self.check(&TokenKind::LtEq) {
                BinaryOp::LtEq
            } else if self.check(&TokenKind::Gt) {
                BinaryOp::Gt
            } else if self.check(&TokenKind::GtEq) {
                BinaryOp::GtEq
            } else {
                break;
            };

            let start = left.span();
            self.advance();
            let right = self.parse_additive()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right), span);
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = if self.check(&TokenKind::Plus) {
                BinaryOp::Add
            } else if self.check(&TokenKind::Minus) {
                BinaryOp::Sub
            } else if self.check(&TokenKind::Concat) {
                BinaryOp::Concat
            } else {
                break;
            };

            let start = left.span();
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right), span);
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = if self.check(&TokenKind::Star) {
                BinaryOp::Mul
            } else if self.check(&TokenKind::Slash) {
                BinaryOp::Div
            } else if self.check(&TokenKind::Percent) {
                BinaryOp::Mod
            } else {
                break;
            };

            let start = left.span();
            self.advance();
            let right = self.parse_unary()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right), span);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if self.check(&TokenKind::Minus) {
            let start = self.advance().span;
            let expr = self.parse_unary()?;
            let span = self.span_from(start);
            Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr), span))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;

        // Handle attribute access: expr.attr
        while self.check(&TokenKind::Dot) {
            let start = expr.span();
            self.advance();
            let attr = self.expect_ident()?;
            let span = self.span_from(start);
            expr = Expr::AttrAccess(Box::new(expr), attr, span);
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let token = self.peek().clone();

        match &token.kind {
            // Literals
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::Null,
                    span: token.span,
                }))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::Bool(true),
                    span: token.span,
                }))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::Bool(false),
                    span: token.span,
                }))
            }
            TokenKind::Int(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::Int(n),
                    span: token.span,
                }))
            }
            TokenKind::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::Float(f),
                    span: token.span,
                }))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::String(s),
                    span: token.span,
                }))
            }

            // ID reference: #id
            TokenKind::Hash => {
                self.advance();
                let id = self.expect_ident()?;
                let span = self.span_from(token.span);
                Ok(Expr::IdRef(id, span))
            }

            // Parameter: $param
            TokenKind::Dollar => {
                self.advance();
                let name = self.expect_ident()?;
                let span = self.span_from(token.span);
                Ok(Expr::Param(name, span))
            }

            // EXISTS
            TokenKind::Exists => {
                let start = self.advance().span;
                self.expect(&TokenKind::LParen)?;
                let pattern = self.parse_pattern()?;
                let where_clause = if self.check(&TokenKind::Where) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect(&TokenKind::RParen)?;
                let span = self.span_from(start);
                Ok(Expr::Exists(pattern, where_clause, span))
            }

            // Parenthesized expression
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }

            // Identifier (variable or function call)
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for function call
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while self.check(&TokenKind::Comma) {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    let span = self.span_from(token.span);
                    Ok(Expr::FnCall(FnCall { name, args, span }))
                } else {
                    Ok(Expr::Var(name, token.span))
                }
            }

            _ => Err(ParseError::unexpected_token(
                token.span,
                "expression",
                token.kind.name(),
            )),
        }
    }

    // ==================== HELPERS ====================

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("tokens should always end with EOF")
        })
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn check_ident(&self, name: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(s) if s.eq_ignore_ascii_case(name))
    }

    fn peek_is_ident(&self) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(_))
    }

    fn expect(&mut self, kind: &TokenKind) -> ParseResult<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(ParseError::unexpected_token(
                token.span,
                kind.name(),
                token.kind.name(),
            ))
        }
    }

    fn expect_ident(&mut self) -> ParseResult<String> {
        match self.peek().kind.clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok(name)
            }
            _ => {
                let token = self.peek();
                Err(ParseError::unexpected_token(
                    token.span,
                    "identifier",
                    token.kind.name(),
                ))
            }
        }
    }

    /// Expect an identifier or a keyword that can be used as a name.
    /// This allows keywords like 'order', 'type', 'match' to be used as attribute names.
    fn expect_name(&mut self) -> ParseResult<String> {
        let token = self.peek().clone();
        let name = match &token.kind {
            TokenKind::Ident(name) => name.clone(),
            // Allow keywords to be used as names (convert to lowercase)
            kind if kind.is_keyword() => kind.name().to_lowercase(),
            _ => {
                return Err(ParseError::unexpected_token(
                    token.span,
                    "name",
                    token.kind.name(),
                ));
            }
        };
        self.advance();
        Ok(name)
    }

    fn expect_int(&mut self) -> ParseResult<i64> {
        match self.peek().kind {
            TokenKind::Int(n) => {
                self.advance();
                Ok(n)
            }
            _ => {
                let token = self.peek();
                Err(ParseError::unexpected_token(
                    token.span,
                    "integer",
                    token.kind.name(),
                ))
            }
        }
    }

    fn expect_keyword(&mut self, name: &str) -> ParseResult<Token> {
        if self.check_ident(name) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(ParseError::unexpected_token(
                token.span,
                name,
                token.kind.name(),
            ))
        }
    }

    fn span_from(&self, start: Span) -> Span {
        let end_token = if self.pos > 0 {
            &self.tokens[self.pos - 1]
        } else {
            self.peek()
        };
        Span::new(start.start, end_token.span.end, start.line, start.column)
    }
}

// ==================== PUBLIC API ====================

/// Parse a statement from source text.
#[allow(dead_code)]
pub fn parse_stmt(input: &str) -> ParseResult<Stmt> {
    Parser::new(input)?.parse_stmt()
}

/// Parse a MATCH statement from source text.
#[allow(dead_code)]
pub fn parse_match(input: &str) -> ParseResult<MatchStmt> {
    match parse_stmt(input)? {
        Stmt::Match(m) => Ok(m),
        _ => Err(ParseError::new("expected MATCH statement", Span::default())),
    }
}

/// Parse ontology definitions from source text.
pub fn parse_ontology(input: &str) -> ParseResult<Vec<OntologyDef>> {
    Parser::new(input)?.parse_ontology_defs()
}

impl Parser {
    /// Parse multiple ontology definitions.
    /// Supports both bare definitions and `ontology Name { ... }` wrapper syntax.
    pub fn parse_ontology_defs(&mut self) -> ParseResult<Vec<OntologyDef>> {
        let mut defs = Vec::new();

        while !self.check(&TokenKind::Eof) {
            // Check for ontology wrapper
            if self.check(&TokenKind::Ontology) {
                self.advance(); // consume 'ontology'
                let _name = self.expect_ident()?; // consume name (ignored for now)
                self.expect(&TokenKind::LBrace)?;

                // Parse definitions inside the wrapper
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    defs.push(self.parse_ontology_def()?);
                }

                self.expect(&TokenKind::RBrace)?;
            } else {
                defs.push(self.parse_ontology_def()?);
            }
        }

        Ok(defs)
    }

    /// Parse a single ontology definition.
    fn parse_ontology_def(&mut self) -> ParseResult<OntologyDef> {
        let token = self.peek().clone();
        match &token.kind {
            TokenKind::Type => self.parse_type_alias_def().map(OntologyDef::TypeAlias),
            TokenKind::Node => self.parse_node_type_def().map(OntologyDef::Node),
            TokenKind::Edge => self.parse_edge_type_def().map(OntologyDef::Edge),
            TokenKind::Constraint => self.parse_constraint_def().map(OntologyDef::Constraint),
            TokenKind::Rule => self.parse_rule_def().map(OntologyDef::Rule),
            _ => Err(ParseError::unexpected_token(
                token.span,
                "type, node, edge, constraint, or rule",
                token.kind.name(),
            )),
        }
    }

    /// Parse a type alias definition.
    /// Syntax: type Name = BaseType [modifiers]
    fn parse_type_alias_def(&mut self) -> ParseResult<TypeAliasDef> {
        let start = self.expect(&TokenKind::Type)?.span;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;
        let base_type = self.expect_ident()?;

        // Parse optional modifiers in brackets
        let modifiers = if self.check(&TokenKind::LBracket) {
            self.parse_attr_modifiers()?
        } else {
            Vec::new()
        };

        let span = self.span_from(start);
        Ok(TypeAliasDef {
            name,
            base_type,
            modifiers,
            span,
        })
    }

    /// Parse a node type definition.
    /// Syntax: node TypeName [: Parent1, Parent2] { attr: Type [modifiers], ... }
    fn parse_node_type_def(&mut self) -> ParseResult<NodeTypeDef> {
        let start = self.expect(&TokenKind::Node)?.span;
        let name = self.expect_ident()?;

        // Parse optional inheritance clause: : Parent1, Parent2
        let parents = if self.check(&TokenKind::Colon) {
            self.advance();
            let mut parents = vec![self.expect_ident()?];
            while self.check(&TokenKind::Comma) {
                self.advance();
                // Check if next is LBrace (end of parents) or another ident
                if self.check(&TokenKind::LBrace) {
                    break;
                }
                parents.push(self.expect_ident()?);
            }
            parents
        } else {
            Vec::new()
        };

        // Parse attribute definitions in braces
        let attrs = if self.check(&TokenKind::LBrace) {
            self.parse_attr_defs()?
        } else {
            Vec::new()
        };

        let span = self.span_from(start);
        Ok(NodeTypeDef {
            name,
            parents,
            attrs,
            span,
        })
    }

    /// Parse attribute definitions: { name: Type [modifiers], ... }
    fn parse_attr_defs(&mut self) -> ParseResult<Vec<AttrDef>> {
        self.expect(&TokenKind::LBrace)?;

        let mut attrs = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            attrs.push(self.parse_attr_def()?);
            // Comma is optional between attrs
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(attrs)
    }

    /// Parse a single attribute definition: name: Type? [modifiers] = default
    fn parse_attr_def(&mut self) -> ParseResult<AttrDef> {
        let start = self.peek().span;
        // Use expect_name to allow keywords like 'order' as attribute names
        let name = self.expect_name()?;
        self.expect(&TokenKind::Colon)?;
        let type_name = self.expect_ident()?;

        // Parse optional nullable marker (?)
        let nullable = if self.check(&TokenKind::Question) {
            self.advance();
            true
        } else {
            false
        };

        // Parse optional modifiers in brackets
        let modifiers = if self.check(&TokenKind::LBracket) {
            self.parse_attr_modifiers()?
        } else {
            Vec::new()
        };

        // Parse optional default value (= expr)
        let default_value = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = self.span_from(start);
        Ok(AttrDef {
            name,
            type_name,
            nullable,
            modifiers,
            default_value,
            span,
        })
    }

    /// Parse attribute modifiers: [required, unique, default = x, >= n, <= n]
    fn parse_attr_modifiers(&mut self) -> ParseResult<Vec<AttrModifier>> {
        self.expect(&TokenKind::LBracket)?;

        let mut modifiers = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
            modifiers.push(self.parse_attr_modifier()?);
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RBracket)?;
        Ok(modifiers)
    }

    /// Parse a single attribute modifier.
    fn parse_attr_modifier(&mut self) -> ParseResult<AttrModifier> {
        if self.check_ident("required") {
            self.advance();
            Ok(AttrModifier::Required)
        } else if self.check_ident("unique") {
            self.advance();
            Ok(AttrModifier::Unique)
        } else if self.check_ident("default") {
            self.advance();
            if self.check(&TokenKind::Eq) || self.check(&TokenKind::Colon) {
                self.advance();
            } else {
                let token = self.peek();
                return Err(ParseError::unexpected_token(
                    token.span,
                    "= or :",
                    token.kind.name(),
                ));
            }
            let value = self.parse_expr()?;
            Ok(AttrModifier::Default(value))
        } else if self.check(&TokenKind::In) || self.check_ident("in") {
            // in: ["a", "b", "c"] - allowed values
            self.advance();
            self.expect(&TokenKind::Colon)?;
            let values = self.parse_array_literal()?;
            Ok(AttrModifier::InValues(values))
        } else if self.check(&TokenKind::Match) || self.check_ident("match") {
            // match: "regex" - regex pattern
            self.advance();
            self.expect(&TokenKind::Colon)?;
            let pattern = self.expect_string()?;
            Ok(AttrModifier::Match(pattern))
        } else if self.check(&TokenKind::GtEq) {
            self.advance();
            let min = self.parse_expr()?;
            Ok(AttrModifier::Range {
                min: Some(min),
                max: None,
            })
        } else if self.check(&TokenKind::LtEq) {
            self.advance();
            let max = self.parse_expr()?;
            Ok(AttrModifier::Range {
                min: None,
                max: Some(max),
            })
        } else if let TokenKind::Int(min_val) = self.peek().kind {
            // Range shorthand: [N..M]
            let min = min_val;
            self.advance();
            if self.check(&TokenKind::Range) {
                self.advance();
                if let TokenKind::Int(max_val) = self.peek().kind {
                    let max = max_val;
                    self.advance();
                    Ok(AttrModifier::Range {
                        min: Some(Expr::Literal(Literal {
                            kind: LiteralKind::Int(min),
                            span: Span::default(),
                        })),
                        max: Some(Expr::Literal(Literal {
                            kind: LiteralKind::Int(max),
                            span: Span::default(),
                        })),
                    })
                } else {
                    let token = self.peek();
                    Err(ParseError::unexpected_token(
                        token.span,
                        "integer for range end",
                        token.kind.name(),
                    ))
                }
            } else {
                let token = self.peek();
                Err(ParseError::unexpected_token(
                    token.span,
                    ".. for range",
                    token.kind.name(),
                ))
            }
        } else {
            let token = self.peek();
            Err(ParseError::unexpected_token(
                token.span,
                "modifier",
                token.kind.name(),
            ))
        }
    }

    /// Parse an array literal: [expr, expr, ...]
    fn parse_array_literal(&mut self) -> ParseResult<Vec<Expr>> {
        self.expect(&TokenKind::LBracket)?;
        let mut values = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
            values.push(self.parse_expr()?);
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(values)
    }

    /// Expect and consume a string literal, returning the string value.
    fn expect_string(&mut self) -> ParseResult<String> {
        let token = self.peek().clone();
        if let TokenKind::String(s) = token.kind {
            self.advance();
            Ok(s)
        } else {
            Err(ParseError::unexpected_token(
                token.span,
                "string",
                token.kind.name(),
            ))
        }
    }

    /// Parse an edge type definition.
    /// Syntax: edge EdgeName(param: Type, ...) [modifiers] { attrs }
    fn parse_edge_type_def(&mut self) -> ParseResult<EdgeTypeDef> {
        let start = self.expect(&TokenKind::Edge)?.span;
        let name = self.expect_ident()?;

        // Parse parameters
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
            let param_name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            // Accept 'any' keyword as a type constraint
            let param_type = if self.check(&TokenKind::Any) {
                self.advance();
                "any".to_string()
            } else {
                self.expect_ident()?
            };
            params.push((param_name, param_type));
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(&TokenKind::RParen)?;

        // Parse optional modifiers (can come before or after attrs)
        let mut modifiers = if self.check(&TokenKind::LBracket) {
            self.parse_edge_modifiers()?
        } else {
            Vec::new()
        };

        // Parse optional attributes in braces
        let attrs = if self.check(&TokenKind::LBrace) {
            self.parse_attr_defs()?
        } else {
            Vec::new()
        };

        // Parse optional modifiers after attrs (if not already parsed)
        if modifiers.is_empty() && self.check(&TokenKind::LBracket) {
            modifiers = self.parse_edge_modifiers()?;
        }

        let span = self.span_from(start);
        Ok(EdgeTypeDef {
            name,
            params,
            attrs,
            modifiers,
            span,
        })
    }

    /// Parse edge modifiers: [acyclic, unique, on_kill: cascade]
    fn parse_edge_modifiers(&mut self) -> ParseResult<Vec<EdgeModifier>> {
        self.expect(&TokenKind::LBracket)?;

        let mut modifiers = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
            modifiers.push(self.parse_edge_modifier()?);
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RBracket)?;
        Ok(modifiers)
    }

    /// Parse a single edge modifier.
    fn parse_edge_modifier(&mut self) -> ParseResult<EdgeModifier> {
        if self.check_ident("acyclic") {
            self.advance();
            Ok(EdgeModifier::Acyclic)
        } else if self.check_ident("unique") {
            self.advance();
            Ok(EdgeModifier::Unique)
        } else if self.check_ident("no_self") {
            self.advance();
            Ok(EdgeModifier::NoSelf)
        } else if self.check_ident("symmetric") {
            self.advance();
            Ok(EdgeModifier::Symmetric)
        } else if self.check_ident("indexed") {
            self.advance();
            Ok(EdgeModifier::Indexed)
        } else if self.check_ident("on_kill_target") {
            self.advance();
            self.expect(&TokenKind::Colon)?;
            let action = self.parse_referential_action()?;
            Ok(EdgeModifier::OnKillTarget(action))
        } else if self.check_ident("on_kill_source") {
            self.advance();
            self.expect(&TokenKind::Colon)?;
            let action = self.parse_referential_action()?;
            Ok(EdgeModifier::OnKillSource(action))
        } else if let TokenKind::Ident(name) = &self.peek().kind {
            // Check if this is a cardinality constraint: param -> N or param -> N..M
            let param = name.clone();
            self.advance();
            if self.check(&TokenKind::RightArrow) {
                self.advance();
                let (min, max) = self.parse_cardinality()?;
                Ok(EdgeModifier::Cardinality { param, min, max })
            } else {
                let token = self.peek();
                Err(ParseError::unexpected_token(
                    token.span,
                    "-> for cardinality constraint",
                    token.kind.name(),
                ))
            }
        } else {
            let token = self.peek();
            Err(ParseError::unexpected_token(
                token.span,
                "edge modifier (acyclic, unique, no_self, symmetric, indexed, on_kill_*, or cardinality)",
                token.kind.name(),
            ))
        }
    }

    /// Parse referential action: cascade, unlink, or prevent
    fn parse_referential_action(&mut self) -> ParseResult<ReferentialAction> {
        if self.check(&TokenKind::Cascade) || self.check_ident("cascade") {
            self.advance();
            Ok(ReferentialAction::Cascade)
        } else if self.check(&TokenKind::Unlink) || self.check_ident("unlink") {
            self.advance();
            Ok(ReferentialAction::Unlink)
        } else if self.check_ident("prevent") {
            self.advance();
            Ok(ReferentialAction::Prevent)
        } else {
            let token = self.peek();
            Err(ParseError::unexpected_token(
                token.span,
                "cascade, unlink, or prevent",
                token.kind.name(),
            ))
        }
    }

    /// Parse cardinality: N, N..M, N..*, or just *
    fn parse_cardinality(&mut self) -> ParseResult<(i64, CardinalityMax)> {
        // Handle * (unbounded)
        if self.check(&TokenKind::Star) {
            self.advance();
            return Ok((0, CardinalityMax::Unbounded));
        }

        let min = self.expect_int()?;

        // Check for range
        if self.check(&TokenKind::Range) {
            self.advance();
            if self.check(&TokenKind::Star) {
                self.advance();
                Ok((min, CardinalityMax::Unbounded))
            } else {
                let max = self.expect_int()?;
                Ok((min, CardinalityMax::Value(max)))
            }
        } else {
            // Single value means exactly N (min == max)
            Ok((min, CardinalityMax::Value(min)))
        }
    }

    /// Parse a constraint definition.
    /// Syntax: constraint Name [modifiers]: Pattern => Condition
    fn parse_constraint_def(&mut self) -> ParseResult<ConstraintDef> {
        let start = self.expect(&TokenKind::Constraint)?.span;
        let name = self.expect_ident()?;

        // Parse optional modifiers [soft, message: "..."]
        let modifiers = if self.check(&TokenKind::LBracket) {
            self.parse_constraint_modifiers()?
        } else {
            ConstraintModifiers::default()
        };

        self.expect(&TokenKind::Colon)?;

        // Parse pattern (node/edge patterns with optional WHERE)
        let pattern = self.parse_ontology_pattern()?;

        // Expect =>
        self.expect(&TokenKind::Arrow)?;

        // Parse condition expression
        let condition = self.parse_expr()?;

        let span = self.span_from(start);
        Ok(ConstraintDef {
            name,
            pattern,
            condition,
            modifiers,
            span,
        })
    }

    /// Parse constraint modifiers: [soft, message: "..."]
    fn parse_constraint_modifiers(&mut self) -> ParseResult<ConstraintModifiers> {
        self.expect(&TokenKind::LBracket)?;

        let mut mods = ConstraintModifiers::default();
        while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
            if self.check_ident("soft") {
                self.advance();
                mods.soft = true;
            } else if self.check_ident("hard") {
                self.advance();
                mods.soft = false;
            } else if self.check_ident("message") {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                if let TokenKind::String(s) = &self.peek().kind {
                    mods.message = Some(s.clone());
                    self.advance();
                } else {
                    return Err(ParseError::new(
                        "expected string after message:",
                        self.peek().span,
                    ));
                }
            } else if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(&TokenKind::RBracket)?;
        Ok(mods)
    }

    /// Parse an ontology pattern for constraints/rules: PatternElement ("," PatternElement)* WhereClause?
    fn parse_ontology_pattern(&mut self) -> ParseResult<Pattern> {
        let start_span = self.peek().span;
        let mut elements = Vec::new();

        // Parse first element (reuse parse_pattern_elem for consistency)
        elements.push(self.parse_pattern_elem()?);

        // Parse remaining elements separated by comma
        while self.check(&TokenKind::Comma) {
            self.advance();
            // Stop if we hit WHERE or =>
            if self.check(&TokenKind::Where) || self.check(&TokenKind::Arrow) {
                break;
            }
            elements.push(self.parse_pattern_elem()?);
        }

        // Parse optional WHERE clause
        let where_clause = if self.check(&TokenKind::Where) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = self.span_from(start_span);
        Ok(Pattern {
            elements,
            where_clause,
            span,
        })
    }

    /// Parse a rule definition.
    /// Syntax: rule Name [modifiers]: Pattern => Production
    fn parse_rule_def(&mut self) -> ParseResult<RuleDef> {
        let start = self.expect(&TokenKind::Rule)?.span;
        let name = self.expect_ident()?;

        let mut auto = true; // Default is auto
        let mut priority = None;

        // Parse optional modifiers [auto, manual, priority: N]
        if self.check(&TokenKind::LBracket) {
            self.advance();
            while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
                if self.check_ident("auto") {
                    self.advance();
                    auto = true;
                } else if self.check_ident("manual") {
                    self.advance();
                    auto = false;
                } else if self.check_ident("priority") {
                    self.advance();
                    if self.check(&TokenKind::Colon) || self.check(&TokenKind::Eq) {
                        self.advance();
                    }
                    priority = Some(self.expect_int()?);
                } else if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    let token = self.peek().clone();
                    return Err(ParseError::unexpected_token(
                        token.span,
                        "auto, manual, priority, or ]",
                        token.kind.name(),
                    ));
                }
            }
            self.expect(&TokenKind::RBracket)?;
        }

        self.expect(&TokenKind::Colon)?;

        // Parse pattern (node/edge patterns with optional WHERE)
        let pattern = self.parse_ontology_pattern()?;

        // Expect =>
        self.expect(&TokenKind::Arrow)?;

        // Parse production (rule actions)
        let mut production = Vec::new();
        production.push(self.parse_rule_action()?);

        // Parse additional actions separated by comma
        while self.check(&TokenKind::Comma) {
            self.advance();
            production.push(self.parse_rule_action()?);
        }

        let span = self.span_from(start);
        Ok(RuleDef {
            name,
            pattern,
            auto,
            priority,
            production,
            span,
        })
    }

    /// Parse a rule action: SPAWN, KILL, LINK, UNLINK, SET
    fn parse_rule_action(&mut self) -> ParseResult<RuleAction> {
        let start_span = self.peek().span;

        if self.check(&TokenKind::Spawn) {
            self.advance();
            let var = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let type_name = self.expect_ident()?;

            let attrs = if self.check(&TokenKind::LBrace) {
                self.parse_attr_block()?
            } else {
                Vec::new()
            };

            let span = self.span_from(start_span);
            Ok(RuleAction::Spawn {
                var,
                type_name,
                attrs,
                span,
            })
        } else if self.check(&TokenKind::Kill) {
            self.advance();
            let var = self.expect_ident()?;
            let span = self.span_from(start_span);
            Ok(RuleAction::Kill { var, span })
        } else if self.check(&TokenKind::Link) {
            self.advance();
            let edge_type = self.expect_ident()?;
            self.expect(&TokenKind::LParen)?;

            let mut targets = Vec::new();
            if !self.check(&TokenKind::RParen) {
                targets.push(self.expect_ident()?);
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    targets.push(self.expect_ident()?);
                }
            }
            self.expect(&TokenKind::RParen)?;

            let alias = if self.check(&TokenKind::As) {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };

            let attrs = if self.check(&TokenKind::LBrace) {
                self.parse_attr_block()?
            } else {
                Vec::new()
            };

            let span = self.span_from(start_span);
            Ok(RuleAction::Link {
                edge_type,
                targets,
                alias,
                attrs,
                span,
            })
        } else if self.check(&TokenKind::Unlink) {
            self.advance();
            let var = self.expect_ident()?;
            let span = self.span_from(start_span);
            Ok(RuleAction::Unlink { var, span })
        } else if self.check(&TokenKind::Set) {
            self.advance();
            let target = self.expect_ident()?;
            self.expect(&TokenKind::Dot)?;
            let attr = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let value = self.parse_expr()?;
            let span = self.span_from(start_span);
            Ok(RuleAction::Set {
                target,
                attr,
                value,
                span,
            })
        } else {
            let token = self.peek().clone();
            Err(ParseError::unexpected_token(
                token.span,
                "SPAWN, KILL, LINK, UNLINK, or SET",
                token.kind.name(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== MATCH TESTS ====================

    #[test]
    fn test_parse_match_single_type() {
        let stmt = parse_match("MATCH t: Task RETURN t").unwrap();

        assert_eq!(stmt.pattern.len(), 1);
        match &stmt.pattern[0] {
            PatternElem::Node(n) => {
                assert_eq!(n.var, "t");
                assert_eq!(n.type_name, "Task");
            }
            _ => panic!("Expected node pattern"),
        }

        assert!(stmt.where_clause.is_none());
        assert_eq!(stmt.return_clause.projections.len(), 1);
    }

    #[test]
    fn test_parse_match_with_edge() {
        let stmt = parse_match("MATCH a: Person, b: Person, knows(a, b) RETURN a, b").unwrap();

        assert_eq!(stmt.pattern.len(), 3);
        match &stmt.pattern[2] {
            PatternElem::Edge(e) => {
                assert_eq!(e.edge_type, "knows");
                assert_eq!(e.targets, vec!["a", "b"]);
            }
            _ => panic!("Expected edge pattern"),
        }
    }

    #[test]
    fn test_parse_match_with_where() {
        let stmt = parse_match("MATCH t: Task WHERE t.priority > 5 RETURN t.title").unwrap();

        assert!(stmt.where_clause.is_some());
        match &stmt.where_clause.unwrap() {
            Expr::BinaryOp(BinaryOp::Gt, left, right, _) => {
                match left.as_ref() {
                    Expr::AttrAccess(base, attr, _) => {
                        match base.as_ref() {
                            Expr::Var(name, _) => assert_eq!(name, "t"),
                            _ => panic!("Expected var"),
                        }
                        assert_eq!(attr, "priority");
                    }
                    _ => panic!("Expected attr access"),
                }
                match right.as_ref() {
                    Expr::Literal(Literal {
                        kind: LiteralKind::Int(5),
                        ..
                    }) => {}
                    _ => panic!("Expected int literal"),
                }
            }
            _ => panic!("Expected GT binary op"),
        }
    }

    // ==================== SPAWN TESTS ====================

    #[test]
    fn test_parse_spawn() {
        let stmt = parse_stmt(r#"SPAWN t: Task { title = "Hello", priority = 1 }"#).unwrap();

        match stmt {
            Stmt::Spawn(s) => {
                assert_eq!(s.var, "t");
                assert_eq!(s.type_name, "Task");
                assert_eq!(s.attrs.len(), 2);
                assert_eq!(s.attrs[0].name, "title");
                assert_eq!(s.attrs[1].name, "priority");
            }
            _ => panic!("Expected SPAWN"),
        }
    }

    // ==================== KILL TESTS ====================

    #[test]
    fn test_parse_kill() {
        let stmt = parse_stmt("KILL t").unwrap();

        match stmt {
            Stmt::Kill(k) => match k.target {
                Target::Var(v) => assert_eq!(v, "t"),
                _ => panic!("Expected var target"),
            },
            _ => panic!("Expected KILL"),
        }
    }

    // ==================== LINK TESTS ====================

    #[test]
    fn test_parse_link() {
        let stmt = parse_stmt("LINK e: owns(p, t)").unwrap();

        match stmt {
            Stmt::Link(l) => {
                assert_eq!(l.var, Some("e".to_string()));
                assert_eq!(l.edge_type, "owns");
                assert_eq!(l.targets.len(), 2);
            }
            _ => panic!("Expected LINK"),
        }
    }

    // ==================== UNLINK TESTS ====================

    #[test]
    fn test_parse_unlink() {
        let stmt = parse_stmt("UNLINK e").unwrap();

        match stmt {
            Stmt::Unlink(u) => match u.target {
                Target::Var(v) => assert_eq!(v, "e"),
                _ => panic!("Expected var target"),
            },
            _ => panic!("Expected UNLINK"),
        }
    }

    // ==================== SET TESTS ====================

    #[test]
    fn test_parse_set() {
        let stmt = parse_stmt(r#"SET t.status = "done""#).unwrap();

        match stmt {
            Stmt::Set(s) => {
                match s.target {
                    Target::Var(v) => assert_eq!(v, "t"),
                    _ => panic!("Expected var target"),
                }
                assert_eq!(s.assignments.len(), 1);
                assert_eq!(s.assignments[0].name, "status");
            }
            _ => panic!("Expected SET"),
        }
    }

    // ==================== TRANSACTION TESTS ====================

    #[test]
    fn test_parse_transaction_statements() {
        assert!(matches!(
            parse_stmt("BEGIN").unwrap(),
            Stmt::Txn(TxnStmt::Begin { isolation: None })
        ));
        assert!(matches!(
            parse_stmt("COMMIT").unwrap(),
            Stmt::Txn(TxnStmt::Commit)
        ));
        assert!(matches!(
            parse_stmt("ROLLBACK").unwrap(),
            Stmt::Txn(TxnStmt::Rollback)
        ));
    }

    // ==================== EXPRESSION TESTS ====================

    #[test]
    fn test_parse_arithmetic() {
        let stmt = parse_match("MATCH x: T RETURN a + b * 2").unwrap();

        // Should parse as a + (b * 2) due to precedence
        let proj = &stmt.return_clause.projections[0].expr;
        match proj {
            Expr::BinaryOp(BinaryOp::Add, left, right, _) => {
                assert!(matches!(left.as_ref(), Expr::Var(n, _) if n == "a"));
                match right.as_ref() {
                    Expr::BinaryOp(BinaryOp::Mul, _, _, _) => {}
                    _ => panic!("Expected Mul"),
                }
            }
            _ => panic!("Expected Add"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let stmt = parse_match("MATCH x: T WHERE x >= 10 AND y < 20 RETURN x").unwrap();

        match stmt.where_clause.unwrap() {
            Expr::BinaryOp(BinaryOp::And, left, right, _) => {
                assert!(matches!(
                    left.as_ref(),
                    Expr::BinaryOp(BinaryOp::GtEq, _, _, _)
                ));
                assert!(matches!(
                    right.as_ref(),
                    Expr::BinaryOp(BinaryOp::Lt, _, _, _)
                ));
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let stmt = parse_match("MATCH x: T RETURN COUNT(items)").unwrap();

        match &stmt.return_clause.projections[0].expr {
            Expr::FnCall(fc) => {
                assert_eq!(fc.name, "COUNT");
                assert_eq!(fc.args.len(), 1);
            }
            _ => panic!("Expected function call"),
        }
    }

    // ==================== ERROR TESTS ====================

    #[test]
    fn test_syntax_error_has_location() {
        let result = parse_stmt("MATCH t: Task WHER t.x > 1 RETURN t");

        match result {
            Err(e) => {
                assert!(e.message.contains("expected") || e.message.contains("unexpected"));
                assert!(e.span.line >= 1);
                assert!(e.span.column >= 1);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_unexpected_eof() {
        let result = parse_stmt("MATCH t:");

        match result {
            Err(e) => {
                assert!(
                    e.message.contains("end of input") || e.message.contains("identifier"),
                    "message was: {}",
                    e.message
                );
            }
            Ok(_) => panic!("Expected error"),
        }
    }
}
