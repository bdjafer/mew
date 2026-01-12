//! Ontology definition parsing.
//!
//! Handles parsing of DSL definitions:
//! - Type aliases
//! - Node type definitions
//! - Edge type definitions
//! - Constraint definitions
//! - Rule definitions

use super::Parser;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::TokenKind;

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

    // ==================== TYPE ALIAS ====================

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

    // ==================== NODE TYPE ====================

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

    // ==================== EDGE TYPE ====================

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

    // ==================== CONSTRAINT ====================

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

    // ==================== RULE ====================

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
