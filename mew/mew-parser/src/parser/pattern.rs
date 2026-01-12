//! Pattern parsing.
//!
//! Handles parsing of node and edge patterns used in MATCH statements
//! and ontology constraints/rules.

use super::Parser;
use crate::ast::*;
use crate::error::ParseResult;
use crate::lexer::TokenKind;

impl Parser {
    /// Parse a pattern: comma-separated list of pattern elements.
    pub(crate) fn parse_pattern(&mut self) -> ParseResult<Vec<PatternElem>> {
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
    pub(crate) fn parse_pattern_elem(&mut self) -> ParseResult<PatternElem> {
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
            Err(crate::ParseError::unexpected_token(
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

    /// Parse an ontology pattern for constraints/rules: PatternElement ("," PatternElement)* WhereClause?
    pub(crate) fn parse_ontology_pattern(&mut self) -> ParseResult<Pattern> {
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
}
