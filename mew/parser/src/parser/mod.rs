//! Parser for MEW source text.
//!
//! This module is organized into submodules by parsing category:
//! - `expr`: Expression parsing (operators, literals, function calls)
//! - `pattern`: Pattern parsing (node/edge patterns)
//! - `stmt`: Statement parsing (MATCH, SPAWN, KILL, etc.)
//! - `ontology`: Ontology definition parsing (node/edge types, constraints, rules)

mod expr;
mod ontology;
mod pattern;
mod stmt;

use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::{Lexer, Token, TokenKind};

// ==================== PARSER STATE ====================

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
}

// ==================== TOKEN HELPERS ====================

impl Parser {
    pub(crate) fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("tokens should always end with EOF")
        })
    }

    pub(crate) fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    pub(crate) fn check_ident(&self, name: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(s) if s.eq_ignore_ascii_case(name))
    }

    pub(crate) fn peek_is_ident(&self) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(_))
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> ParseResult<Token> {
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

    pub(crate) fn expect_ident(&mut self) -> ParseResult<String> {
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
    pub(crate) fn expect_name(&mut self) -> ParseResult<String> {
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

    pub(crate) fn expect_int(&mut self) -> ParseResult<i64> {
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

    pub(crate) fn expect_keyword(&mut self, name: &str) -> ParseResult<Token> {
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

    pub(crate) fn span_from(&self, start: Span) -> Span {
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

/// Parse multiple statements from source text.
pub fn parse_stmts(input: &str) -> ParseResult<Vec<Stmt>> {
    Parser::new(input)?.parse_stmts()
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

// ==================== TESTS ====================

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
                assert_eq!(s.var(), "t");
                assert_eq!(s.type_name(), "Task");
                assert_eq!(s.attrs().len(), 2);
                assert_eq!(s.attrs()[0].name, "title");
                assert_eq!(s.attrs()[1].name, "priority");
            }
            _ => panic!("Expected SPAWN"),
        }
    }

    #[test]
    fn test_parse_spawn_with_duration() {
        let stmt = parse_stmt(r#"SPAWN t: Timer { timeout = 30.seconds }"#).unwrap();

        match stmt {
            Stmt::Spawn(s) => {
                assert_eq!(s.var(), "t");
                assert_eq!(s.type_name(), "Timer");
                assert_eq!(s.attrs().len(), 1);
                assert_eq!(s.attrs()[0].name, "timeout");
                // Check that the value is a duration literal
                match &s.attrs()[0].value {
                    Expr::Literal(lit) => match &lit.kind {
                        LiteralKind::Duration(ms) => {
                            assert_eq!(*ms, 30_000); // 30 seconds = 30000 ms
                        }
                        _ => panic!("Expected Duration literal, got {:?}", lit.kind),
                    },
                    _ => panic!("Expected Literal, got {:?}", s.attrs()[0].value),
                }
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
    fn test_parse_duration_expression() {
        // Test duration in RETURN clause
        let stmt = parse_match("MATCH x: T RETURN 30.seconds").unwrap();

        let proj = &stmt.return_clause.projections[0].expr;
        match proj {
            Expr::Literal(lit) => match &lit.kind {
                LiteralKind::Duration(ms) => {
                    assert_eq!(*ms, 30_000);
                }
                _ => panic!("Expected Duration literal, got {:?}", lit.kind),
            },
            _ => panic!("Expected Literal, got {:?}", proj),
        }
    }

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
