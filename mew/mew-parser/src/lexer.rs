//! Lexer (tokenizer) for MEW source text.

use crate::{ParseError, ParseResult, Span};

/// Token types.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords (case-insensitive)
    Match,
    Where,
    Return,
    Order,
    By,
    Asc,
    Desc,
    Limit,
    Offset,
    Distinct,
    Spawn,
    Kill,
    Link,
    Unlink,
    Set,
    Walk,
    From,
    Follow,
    Until,
    Outbound,
    Inbound,
    Begin,
    Commit,
    Rollback,
    As,
    And,
    Or,
    Not,
    Exists,
    Null,
    True,
    False,
    Cascade,
    No,
    Returning,
    Node,
    Edge,
    Constraint,
    Rule,
    On,
    Read,
    Serializable,
    Committed,
    Optional,
    Any,
    Path,
    Nodes,
    Edges,
    Terminal,
    Load,
    Ontology,

    // Literals
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),

    // Symbols
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Dot,      // .
    Eq,       // =
    NotEq,    // !=
    Lt,       // <
    LtEq,     // <=
    Gt,       // >
    GtEq,     // >=
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %
    Pipe,     // |
    Hash,     // #
    Dollar,   // $
    Concat,   // ++
    Range,    // ..
    Question, // ?

    // End of file
    Eof,
}

impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::Match => "MATCH",
            TokenKind::Where => "WHERE",
            TokenKind::Return => "RETURN",
            TokenKind::Order => "ORDER",
            TokenKind::By => "BY",
            TokenKind::Asc => "ASC",
            TokenKind::Desc => "DESC",
            TokenKind::Limit => "LIMIT",
            TokenKind::Offset => "OFFSET",
            TokenKind::Distinct => "DISTINCT",
            TokenKind::Spawn => "SPAWN",
            TokenKind::Kill => "KILL",
            TokenKind::Link => "LINK",
            TokenKind::Unlink => "UNLINK",
            TokenKind::Set => "SET",
            TokenKind::Walk => "WALK",
            TokenKind::From => "FROM",
            TokenKind::Follow => "FOLLOW",
            TokenKind::Until => "UNTIL",
            TokenKind::Outbound => "OUTBOUND",
            TokenKind::Inbound => "INBOUND",
            TokenKind::Begin => "BEGIN",
            TokenKind::Commit => "COMMIT",
            TokenKind::Rollback => "ROLLBACK",
            TokenKind::As => "AS",
            TokenKind::And => "AND",
            TokenKind::Or => "OR",
            TokenKind::Not => "NOT",
            TokenKind::Exists => "EXISTS",
            TokenKind::Null => "null",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::Cascade => "CASCADE",
            TokenKind::No => "NO",
            TokenKind::Returning => "RETURNING",
            TokenKind::Node => "node",
            TokenKind::Edge => "edge",
            TokenKind::Constraint => "constraint",
            TokenKind::Rule => "rule",
            TokenKind::On => "ON",
            TokenKind::Read => "READ",
            TokenKind::Serializable => "SERIALIZABLE",
            TokenKind::Committed => "COMMITTED",
            TokenKind::Optional => "OPTIONAL",
            TokenKind::Any => "ANY",
            TokenKind::Path => "PATH",
            TokenKind::Nodes => "NODES",
            TokenKind::Edges => "EDGES",
            TokenKind::Terminal => "TERMINAL",
            TokenKind::Load => "LOAD",
            TokenKind::Ontology => "ONTOLOGY",
            TokenKind::Ident(_) => "identifier",
            TokenKind::Int(_) => "integer",
            TokenKind::Float(_) => "float",
            TokenKind::String(_) => "string",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Colon => ":",
            TokenKind::Dot => ".",
            TokenKind::Eq => "=",
            TokenKind::NotEq => "!=",
            TokenKind::Lt => "<",
            TokenKind::LtEq => "<=",
            TokenKind::Gt => ">",
            TokenKind::GtEq => ">=",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Pipe => "|",
            TokenKind::Hash => "#",
            TokenKind::Dollar => "$",
            TokenKind::Concat => "++",
            TokenKind::Range => "..",
            TokenKind::Question => "?",
            TokenKind::Eof => "end of input",
        }
    }
}

/// A token with its span.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn eof(pos: usize, line: usize, column: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            span: Span::new(pos, pos, line, column),
        }
    }
}

/// Lexer state.
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize all input into a vector of tokens.
    pub fn tokenize(mut self) -> ParseResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn current_span(&self) -> Span {
        Span::new(self.pos, self.pos, self.line, self.column)
    }

    fn span_from(&self, start: usize, start_line: usize, start_col: usize) -> Span {
        Span::new(start, self.pos, start_line, start_col)
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn next_char(&mut self) -> Option<char> {
        if let Some((pos, c)) = self.chars.next() {
            self.pos = pos + c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(c) = self.peek_char() {
                if c.is_whitespace() {
                    self.next_char();
                } else {
                    break;
                }
            }

            // Check for comment
            if self.peek_char() == Some('-') {
                let pos = self.pos;
                self.next_char();
                if self.peek_char() == Some('-') {
                    // Line comment
                    self.next_char();
                    while let Some(c) = self.peek_char() {
                        if c == '\n' {
                            break;
                        }
                        self.next_char();
                    }
                } else {
                    // Not a comment, restore position
                    // We can't easily restore, so we need to handle this differently
                    // Actually we've already consumed the '-', so we need to handle it in next_token
                    // For simplicity, let's not restore and handle '-' specially
                    self.pos = pos;
                    // Re-create the iterator from current position
                    self.chars = self.input[pos..].char_indices().peekable();
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> ParseResult<Token> {
        self.skip_whitespace_and_comments();

        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let Some(c) = self.next_char() else {
            return Ok(Token::eof(self.pos, self.line, self.column));
        };

        let kind = match c {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '.' => {
                if self.peek_char() == Some('.') {
                    self.next_char();
                    TokenKind::Range
                } else {
                    TokenKind::Dot
                }
            }
            '=' => TokenKind::Eq,
            '<' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    TokenKind::NotEq
                } else {
                    return Err(ParseError::new(
                        "unexpected character '!'",
                        self.span_from(start, start_line, start_col),
                    ));
                }
            }
            '+' => {
                if self.peek_char() == Some('+') {
                    self.next_char();
                    TokenKind::Concat
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                // Check for comment
                if self.peek_char() == Some('-') {
                    self.next_char();
                    // Skip to end of line
                    while let Some(c) = self.peek_char() {
                        if c == '\n' {
                            break;
                        }
                        self.next_char();
                    }
                    return self.next_token();
                }
                TokenKind::Minus
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '|' => TokenKind::Pipe,
            '#' => TokenKind::Hash,
            '$' => TokenKind::Dollar,
            '?' => TokenKind::Question,
            '"' => self.scan_string(start, start_line, start_col)?,
            '_' | 'a'..='z' | 'A'..='Z' => {
                self.scan_ident_or_keyword(c, start, start_line, start_col)
            }
            '0'..='9' => self.scan_number(c, start, start_line, start_col)?,
            _ => {
                return Err(ParseError::new(
                    format!("unexpected character '{}'", c),
                    self.span_from(start, start_line, start_col),
                ));
            }
        };

        Ok(Token::new(
            kind,
            self.span_from(start, start_line, start_col),
        ))
    }

    fn scan_string(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> ParseResult<TokenKind> {
        let mut value = String::new();

        loop {
            match self.next_char() {
                None => {
                    return Err(ParseError::new(
                        "unterminated string literal",
                        self.span_from(start, start_line, start_col),
                    ));
                }
                Some('"') => break,
                Some('\\') => {
                    let escaped = match self.next_char() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('\\') => '\\',
                        Some('"') => '"',
                        Some(c) => {
                            return Err(ParseError::new(
                                format!("invalid escape sequence '\\{}'", c),
                                self.current_span(),
                            ));
                        }
                        None => {
                            return Err(ParseError::new(
                                "unterminated escape sequence",
                                self.current_span(),
                            ));
                        }
                    };
                    value.push(escaped);
                }
                Some(c) => value.push(c),
            }
        }

        Ok(TokenKind::String(value))
    }

    fn scan_ident_or_keyword(
        &mut self,
        first: char,
        _start: usize,
        _start_line: usize,
        _start_col: usize,
    ) -> TokenKind {
        let mut ident = String::new();
        ident.push(first);

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        // Check for keywords (case-insensitive)
        match ident.to_uppercase().as_str() {
            "MATCH" => TokenKind::Match,
            "WHERE" => TokenKind::Where,
            "RETURN" => TokenKind::Return,
            "ORDER" => TokenKind::Order,
            "BY" => TokenKind::By,
            "ASC" => TokenKind::Asc,
            "DESC" => TokenKind::Desc,
            "LIMIT" => TokenKind::Limit,
            "OFFSET" => TokenKind::Offset,
            "DISTINCT" => TokenKind::Distinct,
            "SPAWN" => TokenKind::Spawn,
            "KILL" => TokenKind::Kill,
            "LINK" => TokenKind::Link,
            "UNLINK" => TokenKind::Unlink,
            "SET" => TokenKind::Set,
            "WALK" => TokenKind::Walk,
            "FROM" => TokenKind::From,
            "FOLLOW" => TokenKind::Follow,
            "UNTIL" => TokenKind::Until,
            "OUTBOUND" => TokenKind::Outbound,
            "INBOUND" => TokenKind::Inbound,
            "BEGIN" => TokenKind::Begin,
            "COMMIT" => TokenKind::Commit,
            "ROLLBACK" => TokenKind::Rollback,
            "AS" => TokenKind::As,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "NOT" => TokenKind::Not,
            "EXISTS" => TokenKind::Exists,
            "NULL" => TokenKind::Null,
            "TRUE" => TokenKind::True,
            "FALSE" => TokenKind::False,
            "CASCADE" => TokenKind::Cascade,
            "NO" => TokenKind::No,
            "RETURNING" => TokenKind::Returning,
            "NODE" => TokenKind::Node,
            "EDGE" => TokenKind::Edge,
            "CONSTRAINT" => TokenKind::Constraint,
            "RULE" => TokenKind::Rule,
            "ON" => TokenKind::On,
            // AUTO, PRIORITY, REQUIRED, UNIQUE, DEFAULT are context-specific
            // and handled as identifiers to avoid conflicts with attribute names
            "READ" => TokenKind::Read,
            "SERIALIZABLE" => TokenKind::Serializable,
            "COMMITTED" => TokenKind::Committed,
            "OPTIONAL" => TokenKind::Optional,
            "ANY" => TokenKind::Any,
            "PATH" => TokenKind::Path,
            "NODES" => TokenKind::Nodes,
            "EDGES" => TokenKind::Edges,
            "TERMINAL" => TokenKind::Terminal,
            // DEPTH is context-specific (WALK) and handled as identifier
            "LOAD" => TokenKind::Load,
            "ONTOLOGY" => TokenKind::Ontology,
            _ => TokenKind::Ident(ident),
        }
    }

    fn scan_number(
        &mut self,
        first: char,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> ParseResult<TokenKind> {
        let mut number = String::new();
        number.push(first);

        // Integer part
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                number.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        // Check for decimal point
        if self.peek_char() == Some('.') {
            // Look ahead to ensure it's not '..' (range)
            let mut lookahead = self.chars.clone();
            lookahead.next(); // consume '.'
            if let Some((_, next_c)) = lookahead.peek() {
                if *next_c == '.' {
                    // It's a range, don't consume the first '.'
                    let value: i64 = number.parse().map_err(|_| {
                        ParseError::new(
                            format!("invalid integer literal '{}'", number),
                            self.span_from(start, start_line, start_col),
                        )
                    })?;
                    return Ok(TokenKind::Int(value));
                }
            }

            number.push('.');
            self.next_char();

            // Fractional part
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    number.push(c);
                    self.next_char();
                } else {
                    break;
                }
            }

            // Parse as float
            let value: f64 = number.parse().map_err(|_| {
                ParseError::new(
                    format!("invalid float literal '{}'", number),
                    self.span_from(start, start_line, start_col),
                )
            })?;
            Ok(TokenKind::Float(value))
        } else {
            // Parse as integer
            let value: i64 = number.parse().map_err(|_| {
                ParseError::new(
                    format!("invalid integer literal '{}'", number),
                    self.span_from(start, start_line, start_col),
                )
            })?;
            Ok(TokenKind::Int(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<TokenKind> {
        Lexer::new(input)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn test_keywords() {
        let kinds = tokenize("MATCH WHERE RETURN");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Match,
                TokenKind::Where,
                TokenKind::Return,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_keywords_case_insensitive() {
        let kinds = tokenize("match Match MATCH");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Match,
                TokenKind::Match,
                TokenKind::Match,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let kinds = tokenize("foo Bar _baz");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("foo".into()),
                TokenKind::Ident("Bar".into()),
                TokenKind::Ident("_baz".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let kinds = tokenize("123 45.67");
        assert_eq!(
            kinds,
            vec![TokenKind::Int(123), TokenKind::Float(45.67), TokenKind::Eof]
        );
    }

    #[test]
    fn test_strings() {
        let kinds = tokenize(r#""hello" "world\n""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::String("hello".into()),
                TokenKind::String("world\n".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_symbols() {
        let kinds = tokenize("() {} [] , : . = != < <= > >= + - * / % | # $ ++ ..");
        assert_eq!(
            kinds,
            vec![
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Comma,
                TokenKind::Colon,
                TokenKind::Dot,
                TokenKind::Eq,
                TokenKind::NotEq,
                TokenKind::Lt,
                TokenKind::LtEq,
                TokenKind::Gt,
                TokenKind::GtEq,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Pipe,
                TokenKind::Hash,
                TokenKind::Dollar,
                TokenKind::Concat,
                TokenKind::Range,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_comments() {
        let kinds = tokenize("MATCH -- this is a comment\nRETURN");
        assert_eq!(
            kinds,
            vec![TokenKind::Match, TokenKind::Return, TokenKind::Eof]
        );
    }

    #[test]
    fn test_match_statement_tokens() {
        let kinds = tokenize("MATCH t: Task RETURN t");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Match,
                TokenKind::Ident("t".into()),
                TokenKind::Colon,
                TokenKind::Ident("Task".into()),
                TokenKind::Return,
                TokenKind::Ident("t".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_spawn_statement_tokens() {
        let kinds = tokenize(r#"SPAWN t: Task { title = "Hello" }"#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Spawn,
                TokenKind::Ident("t".into()),
                TokenKind::Colon,
                TokenKind::Ident("Task".into()),
                TokenKind::LBrace,
                TokenKind::Ident("title".into()),
                TokenKind::Eq,
                TokenKind::String("Hello".into()),
                TokenKind::RBrace,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_span_tracking() {
        let tokens = Lexer::new("MATCH\nt: Task").tokenize().unwrap();
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.column, 1);
    }

    #[test]
    fn test_nullable_type_syntax() {
        let kinds = tokenize("name: String?");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("name".into()),
                TokenKind::Colon,
                TokenKind::Ident("String".into()),
                TokenKind::Question,
                TokenKind::Eof
            ]
        );
    }
}
