//! Statement parsing.
//!
//! Handles parsing of GQL statements:
//! - MATCH: pattern matching and queries
//! - SPAWN: node creation
//! - KILL: node/edge deletion
//! - LINK: edge creation
//! - UNLINK: edge deletion
//! - SET: attribute updates
//! - WALK: graph traversal
//! - Transactions: BEGIN, COMMIT, ROLLBACK

use super::Parser;
use crate::ast::*;
use crate::error::ParseResult;
use crate::lexer::TokenKind;

impl Parser {
    /// Parse a single statement.
    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Match => self.parse_match_or_mutate(),
            TokenKind::Spawn => self.parse_spawn().map(Stmt::Spawn),
            TokenKind::Kill => self.parse_kill().map(Stmt::Kill),
            TokenKind::Link => self.parse_link().map(Stmt::Link),
            TokenKind::Unlink => self.parse_unlink().map(Stmt::Unlink),
            TokenKind::Set => self.parse_set().map(Stmt::Set),
            TokenKind::Walk => self.parse_walk().map(Stmt::Walk),
            TokenKind::Inspect => self.parse_inspect().map(Stmt::Inspect),
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
            TokenKind::Explain => self.parse_explain().map(Stmt::Explain),
            TokenKind::Profile => self.parse_profile().map(Stmt::Profile),
            _ => Err(crate::ParseError::unexpected_token(
                token.span,
                "statement",
                token.kind.name(),
            )),
        }
    }

    /// Parse EXPLAIN statement.
    fn parse_explain(&mut self) -> ParseResult<ExplainStmt> {
        let start = self.expect(&TokenKind::Explain)?.span;
        let inner = self.parse_stmt()?;
        let span = self.span_from(start);
        Ok(ExplainStmt {
            statement: Box::new(inner),
            span,
        })
    }

    /// Parse PROFILE statement.
    fn parse_profile(&mut self) -> ParseResult<ProfileStmt> {
        let start = self.expect(&TokenKind::Profile)?.span;
        let inner = self.parse_stmt()?;
        let span = self.span_from(start);
        Ok(ProfileStmt {
            statement: Box::new(inner),
            span,
        })
    }

    /// Parse MATCH, which can be either:
    /// - MATCH ... RETURN ... (query)
    /// - MATCH ... LINK/SET/KILL/UNLINK ... (compound mutation)
    /// - MATCH ... WALK ... (compound walk)
    fn parse_match_or_mutate(&mut self) -> ParseResult<Stmt> {
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

        // Parse OPTIONAL MATCH clauses
        let optional_matches = self.parse_optional_matches()?;

        // Check what comes next: RETURN, mutation keyword, or WALK
        if self.check(&TokenKind::Return) {
            // Parse as normal MATCH query
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

            Ok(Stmt::Match(MatchStmt {
                pattern,
                where_clause,
                optional_matches,
                return_clause,
                order_by,
                limit,
                offset,
                span,
            }))
        } else if self.is_mutation_keyword() {
            // Parse as compound mutation
            let mut mutations = Vec::new();
            while self.is_mutation_keyword() {
                mutations.push(self.parse_mutation_action()?);
            }

            let span = self.span_from(start);

            Ok(Stmt::MatchMutate(MatchMutateStmt {
                pattern,
                where_clause,
                mutations,
                span,
            }))
        } else if self.check(&TokenKind::Walk) {
            // Parse as compound walk
            let walk = self.parse_walk()?;

            let span = self.span_from(start);

            Ok(Stmt::MatchWalk(MatchWalkStmt {
                pattern,
                where_clause,
                walk,
                span,
            }))
        } else {
            Err(crate::ParseError::unexpected_token(
                self.peek().span,
                "RETURN",
                self.peek().kind.name(),
            ))
        }
    }

    /// Parse zero or more OPTIONAL MATCH clauses.
    fn parse_optional_matches(&mut self) -> ParseResult<Vec<OptionalMatch>> {
        let mut optional_matches = Vec::new();
        while self.check(&TokenKind::Optional) {
            optional_matches.push(self.parse_optional_match()?);
        }
        Ok(optional_matches)
    }

    /// Parse a single OPTIONAL MATCH clause.
    fn parse_optional_match(&mut self) -> ParseResult<OptionalMatch> {
        let start = self.expect(&TokenKind::Optional)?.span;
        self.expect(&TokenKind::Match)?;

        let pattern = self.parse_pattern()?;

        let where_clause = if self.check(&TokenKind::Where) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = self.span_from(start);

        Ok(OptionalMatch {
            pattern,
            where_clause,
            span,
        })
    }

    /// Parse multiple statements until end of input.
    pub fn parse_stmts(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // ==================== MATCH ====================

    /// Parse a MATCH statement. Returns either a MatchStmt (with RETURN) or
    /// a MatchMutateStmt (with mutations like LINK, SET, KILL, UNLINK).
    pub(crate) fn parse_match(&mut self) -> ParseResult<MatchStmt> {
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

        // Parse OPTIONAL MATCH clauses
        let optional_matches = self.parse_optional_matches()?;

        // Parse RETURN (required for query)
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
            optional_matches,
            return_clause,
            order_by,
            limit,
            offset,
            span,
        })
    }

    /// Check if current token is a mutation keyword.
    fn is_mutation_keyword(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Link | TokenKind::Set | TokenKind::Kill | TokenKind::Unlink
        )
    }

    /// Parse a mutation action within a compound statement.
    fn parse_mutation_action(&mut self) -> ParseResult<MutationAction> {
        match &self.peek().kind {
            TokenKind::Link => Ok(MutationAction::Link(self.parse_link()?)),
            TokenKind::Set => Ok(MutationAction::Set(self.parse_set()?)),
            TokenKind::Kill => Ok(MutationAction::Kill(self.parse_kill()?)),
            TokenKind::Unlink => Ok(MutationAction::Unlink(self.parse_unlink()?)),
            _ => Err(crate::ParseError::unexpected_token(
                self.peek().span,
                "mutation (LINK, SET, KILL, UNLINK)",
                self.peek().kind.name(),
            )),
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

    pub(crate) fn parse_projections(&mut self) -> ParseResult<Vec<Projection>> {
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

    pub(crate) fn parse_attr_block(&mut self) -> ParseResult<Vec<AttrAssignment>> {
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
            let ident = self.expect_ident()?;
            // Check if this is an edge pattern: edge_type(targets)
            if self.check(&TokenKind::LParen) {
                self.advance();
                let mut targets = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    targets.push(self.expect_ident()?);
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        targets.push(self.expect_ident()?);
                    }
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Target::EdgePattern {
                    edge_type: ident,
                    targets,
                })
            } else {
                Ok(Target::Var(ident))
            }
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
            return Err(crate::ParseError::unexpected_token(
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

        let ret_type = if self.check(&TokenKind::Path) {
            self.advance();
            let alias = self.parse_optional_as_alias()?;
            WalkReturnType::Path { alias }
        } else if self.check(&TokenKind::Nodes) {
            self.advance();
            let alias = self.parse_optional_as_alias()?;
            WalkReturnType::Nodes { alias }
        } else if self.check(&TokenKind::Edges) {
            self.advance();
            let alias = self.parse_optional_as_alias()?;
            WalkReturnType::Edges { alias }
        } else if self.check(&TokenKind::Terminal) {
            self.advance();
            let alias = self.parse_optional_as_alias()?;
            WalkReturnType::Terminal { alias }
        } else {
            let projections = self.parse_projections()?;
            WalkReturnType::Projections(projections)
        };

        Ok(ret_type)
    }

    /// Parse optional AS alias.
    fn parse_optional_as_alias(&mut self) -> ParseResult<Option<String>> {
        if self.check(&TokenKind::As) {
            self.advance();
            Ok(Some(self.expect_ident()?))
        } else {
            Ok(None)
        }
    }

    // ==================== INSPECT ====================

    fn parse_inspect(&mut self) -> ParseResult<InspectStmt> {
        let start = self.expect(&TokenKind::Inspect)?.span;

        // Expect #id (can be identifier or integer)
        self.expect(&TokenKind::Hash)?;
        let id = match self.peek().kind.clone() {
            TokenKind::Ident(name) => {
                self.advance();
                name
            }
            TokenKind::Int(n) => {
                self.advance();
                n.to_string()
            }
            _ => {
                let token = self.peek();
                return Err(crate::ParseError::unexpected_token(
                    token.span,
                    "identifier or integer",
                    token.kind.name(),
                ));
            }
        };

        // Optional RETURN projections
        let projections = if self.check(&TokenKind::Return) {
            self.advance();
            Some(self.parse_projections()?)
        } else {
            None
        };

        let span = self.span_from(start);

        Ok(InspectStmt {
            id,
            projections,
            span,
        })
    }
}
