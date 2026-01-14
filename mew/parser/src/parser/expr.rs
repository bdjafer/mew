//! Expression parsing.
//!
//! Handles operator precedence parsing for MEW expressions:
//! - Logical: OR, AND, NOT
//! - Comparison: =, !=, <, <=, >, >=
//! - Additive: +, -, ++
//! - Multiplicative: *, /, %
//! - Unary: -, NOT
//! - Postfix: attribute access (.)
//! - Primary: literals, variables, function calls, EXISTS

use super::Parser;
use crate::ast::{*, CollectLimit};
use crate::error::ParseResult;
use crate::lexer::TokenKind;

impl Parser {
    /// Parse an expression.
    pub(crate) fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_null_coalesce()?;

        while self.check(&TokenKind::Or) {
            let start = left.span();
            self.advance();
            let right = self.parse_null_coalesce()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(BinaryOp::Or, Box::new(left), Box::new(right), span);
        }

        Ok(left)
    }

    fn parse_null_coalesce(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and()?;

        while self.check(&TokenKind::NullCoalesce) {
            let start = left.span();
            self.advance();
            let right = self.parse_and()?;
            let span = self.span_from(start);
            left = Expr::BinaryOp(BinaryOp::NullCoalesce, Box::new(left), Box::new(right), span);
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

        // Handle string operators as contextual keywords.
        // These parse to function calls: `a STARTS WITH b` -> starts_with(a, b)
        let string_op = if self.check_ident("STARTS") {
            self.advance();
            if !self.check_ident("WITH") {
                return Err(crate::ParseError::unexpected_token(
                    self.peek().span,
                    "WITH",
                    self.peek().kind.name(),
                ));
            }
            self.advance();
            Some("starts_with")
        } else if self.check_ident("ENDS") {
            self.advance();
            if !self.check_ident("WITH") {
                return Err(crate::ParseError::unexpected_token(
                    self.peek().span,
                    "WITH",
                    self.peek().kind.name(),
                ));
            }
            self.advance();
            Some("ends_with")
        } else if self.check_ident("CONTAINS") {
            self.advance();
            Some("contains")
        } else if self.check(&TokenKind::In) {
            self.advance();
            Some("in")
        } else {
            None
        };

        if let Some(fn_name) = string_op {
            let start = left.span();
            let right = self.parse_additive()?;
            let span = self.span_from(start);
            left = Expr::FnCall(FnCall {
                name: fn_name.to_string(),
                args: vec![left, right],
                distinct: false,
                limit: None,
                span,
            });
        }

        // Handle IS [NOT] NULL
        if self.check(&TokenKind::Is) {
            let start = left.span();
            self.advance(); // consume IS

            let is_not = if self.check(&TokenKind::Not) {
                self.advance(); // consume NOT
                true
            } else {
                false
            };

            self.expect(&TokenKind::Null)?;
            let span = self.span_from(start);

            // Transform to comparison: expr = null or expr != null
            let null_expr = Expr::Literal(Literal {
                kind: LiteralKind::Null,
                span,
            });

            let op = if is_not { BinaryOp::NotEq } else { BinaryOp::Eq };
            left = Expr::BinaryOp(op, Box::new(left), Box::new(null_expr), span);
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

        // Handle type check: expr:Type
        // Must be checked before attribute access to handle node:Type syntax
        if self.check(&TokenKind::Colon) {
            let start = expr.span();
            self.advance();
            let type_name = self.expect_ident()?;
            let span = self.span_from(start);
            return Ok(Expr::TypeCheck(Box::new(expr), type_name, span));
        }

        // Handle attribute access: expr.attr
        // And duration literals: 30.seconds, 5.minutes, etc.
        while self.check(&TokenKind::Dot) {
            let start = expr.span();
            self.advance();
            let attr = self.expect_ident()?;

            // Check if this is a duration literal (number.unit)
            if let Some(duration_ms) = self.try_parse_duration_unit(&expr, &attr) {
                let span = self.span_from(start);
                expr = Expr::Literal(Literal {
                    kind: LiteralKind::Duration(duration_ms),
                    span,
                });
            } else {
                let span = self.span_from(start);
                expr = Expr::AttrAccess(Box::new(expr), attr, span);
            }
        }

        Ok(expr)
    }

    /// Try to parse a duration literal from a numeric expression and unit name.
    /// Returns Some(milliseconds) if successful, None otherwise.
    fn try_parse_duration_unit(&self, expr: &Expr, unit: &str) -> Option<i64> {
        // Extract numeric value from the expression
        let value = match expr {
            Expr::Literal(Literal {
                kind: LiteralKind::Int(n),
                ..
            }) => *n as f64,
            Expr::Literal(Literal {
                kind: LiteralKind::Float(f),
                ..
            }) => *f,
            _ => return None,
        };

        // Convert unit to milliseconds multiplier (case-insensitive without allocation)
        let multiplier: f64 = if unit.eq_ignore_ascii_case("millisecond")
            || unit.eq_ignore_ascii_case("milliseconds")
            || unit.eq_ignore_ascii_case("ms")
        {
            1.0
        } else if unit.eq_ignore_ascii_case("second")
            || unit.eq_ignore_ascii_case("seconds")
            || unit.eq_ignore_ascii_case("s")
        {
            1_000.0
        } else if unit.eq_ignore_ascii_case("minute")
            || unit.eq_ignore_ascii_case("minutes")
            || unit.eq_ignore_ascii_case("min")
        {
            60_000.0
        } else if unit.eq_ignore_ascii_case("hour")
            || unit.eq_ignore_ascii_case("hours")
            || unit.eq_ignore_ascii_case("h")
        {
            3_600_000.0
        } else if unit.eq_ignore_ascii_case("day") || unit.eq_ignore_ascii_case("days") {
            86_400_000.0
        } else if unit.eq_ignore_ascii_case("week") || unit.eq_ignore_ascii_case("weeks") {
            604_800_000.0
        } else {
            return None;
        };

        Some((value * multiplier) as i64)
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
            TokenKind::Timestamp(ts) => {
                let ts = *ts;
                self.advance();
                Ok(Expr::Literal(Literal {
                    kind: LiteralKind::Timestamp(ts),
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

            // List literal: [a, b, c]
            TokenKind::LBracket => {
                let start = self.advance().span;
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    elements.push(self.parse_expr()?);
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::RBracket) {
                            break; // trailing comma
                        }
                        elements.push(self.parse_expr()?);
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                let span = self.span_from(start);
                Ok(Expr::List(elements, span))
            }

            // Handle 'node' and 'edge' keywords as special built-in variables
            // These are used in WALK UNTIL clauses: UNTIL node.status = "done"
            TokenKind::Node => {
                self.advance();
                Ok(Expr::Var("node".to_string(), token.span))
            }
            TokenKind::Edge => {
                self.advance();
                Ok(Expr::Var("edge".to_string(), token.span))
            }

            // Identifier (variable or function call)
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for function call
                if self.check(&TokenKind::LParen) {
                    self.advance();

                    // Check for DISTINCT modifier (e.g., count(DISTINCT x))
                    let distinct = self.check(&TokenKind::Distinct);
                    if distinct {
                        self.advance();
                    }

                    let mut args = Vec::new();
                    // Handle count(*) - treat * as "count all rows" (empty args)
                    if self.check(&TokenKind::Star) {
                        self.advance(); // consume the *
                        // args remains empty - represents count(*)
                    } else if !self.check(&TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while self.check(&TokenKind::Comma) {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&TokenKind::RParen)?;

                    // Check for [limit: N] syntax (used with COLLECT)
                    let limit = if self.check(&TokenKind::LBracket) {
                        self.advance();
                        // "limit" can be either the Limit keyword or an identifier
                        if self.check(&TokenKind::Limit) {
                            self.advance();
                        } else if self.check_ident("limit") {
                            self.advance();
                        } else {
                            let token = self.peek();
                            return Err(crate::ParseError::unexpected_token(
                                token.span,
                                "limit",
                                token.kind.name(),
                            ));
                        }
                        self.expect(&TokenKind::Colon)?;
                        let limit_value = if self.check_ident("none") {
                            self.advance();
                            CollectLimit::None
                        } else {
                            CollectLimit::Value(self.expect_int()?)
                        };
                        self.expect(&TokenKind::RBracket)?;
                        Some(limit_value)
                    } else {
                        None
                    };

                    let span = self.span_from(token.span);
                    Ok(Expr::FnCall(FnCall {
                        name,
                        args,
                        distinct,
                        limit,
                        span,
                    }))
                } else {
                    Ok(Expr::Var(name, token.span))
                }
            }

            _ => Err(crate::ParseError::unexpected_token(
                token.span,
                "expression",
                token.kind.name(),
            )),
        }
    }
}
