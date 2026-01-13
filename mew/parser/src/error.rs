//! Parser error types.

use crate::Span;
use std::fmt;

/// A parse error with location information.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub expected: Option<Vec<String>>,
    pub found: Option<String>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            expected: None,
            found: None,
        }
    }

    pub fn with_expected(mut self, expected: Vec<String>) -> Self {
        self.expected = Some(expected);
        self
    }

    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }

    pub fn unexpected_eof(span: Span, expected: &str) -> Self {
        Self {
            message: format!("unexpected end of input, expected {}", expected),
            span,
            expected: Some(vec![expected.to_string()]),
            found: Some("end of input".to_string()),
        }
    }

    pub fn unexpected_token(span: Span, expected: &str, found: &str) -> Self {
        Self {
            message: format!("expected {}, found {}", expected, found),
            span,
            expected: Some(vec![expected.to_string()]),
            found: Some(found.to_string()),
        }
    }

    pub fn line(&self) -> usize {
        self.span.line
    }

    pub fn column(&self) -> usize {
        self.span.column
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;
