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
use crate::ast::*;
use crate::error::ParseResult;
use crate::lexer::TokenKind;

impl Parser {
    /// Parse an expression.
    pub(crate) fn parse_expr(&mut self) -> ParseResult<Expr> {
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

            _ => Err(crate::ParseError::unexpected_token(
                token.span,
                "expression",
                token.kind.name(),
            )),
        }
    }
}
