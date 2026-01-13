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
    Inspect,
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
    Type,
    In,
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
    /// Timestamp literal (milliseconds since epoch)
    Timestamp(i64),

    // Symbols
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Colon,      // :
    Dot,        // .
    Eq,         // =
    NotEq,      // !=
    Lt,         // <
    LtEq,       // <=
    Gt,         // >
    GtEq,       // >=
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Pipe,       // |
    Hash,       // #
    Dollar,     // $
    Concat,     // ++
    Range,      // ..
    Question,      // ?
    NullCoalesce,  // ??
    Arrow,         // =>
    RightArrow,    // ->

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
            TokenKind::Inspect => "INSPECT",
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
            TokenKind::Type => "type",
            TokenKind::In => "in",
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
            TokenKind::Timestamp(_) => "timestamp",
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
            TokenKind::NullCoalesce => "??",
            TokenKind::Arrow => "=>",
            TokenKind::RightArrow => "->",
            TokenKind::Eof => "end of input",
        }
    }

    /// Returns true if this token is a keyword (not an identifier, literal, or punctuation).
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Match
                | TokenKind::Where
                | TokenKind::Return
                | TokenKind::Order
                | TokenKind::By
                | TokenKind::Asc
                | TokenKind::Desc
                | TokenKind::Limit
                | TokenKind::Offset
                | TokenKind::Distinct
                | TokenKind::Spawn
                | TokenKind::Kill
                | TokenKind::Link
                | TokenKind::Unlink
                | TokenKind::Set
                | TokenKind::Walk
                | TokenKind::From
                | TokenKind::Follow
                | TokenKind::Until
                | TokenKind::Outbound
                | TokenKind::Inbound
                | TokenKind::Begin
                | TokenKind::Commit
                | TokenKind::Rollback
                | TokenKind::As
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Not
                | TokenKind::Exists
                | TokenKind::Null
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Cascade
                | TokenKind::No
                | TokenKind::Returning
                | TokenKind::Node
                | TokenKind::Edge
                | TokenKind::Constraint
                | TokenKind::Rule
                | TokenKind::Type
                | TokenKind::In
                | TokenKind::On
                | TokenKind::Read
                | TokenKind::Serializable
                | TokenKind::Committed
                | TokenKind::Optional
                | TokenKind::Any
                | TokenKind::Path
                | TokenKind::Nodes
                | TokenKind::Edges
                | TokenKind::Terminal
                | TokenKind::Load
                | TokenKind::Ontology
        )
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
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
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

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> ParseResult<Token> {
        self.skip_whitespace();

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
            '=' => {
                if self.peek_char() == Some('>') {
                    self.next_char();
                    TokenKind::Arrow
                } else {
                    TokenKind::Eq
                }
            }
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
                // Check for comment or right arrow
                match self.peek_char() {
                    Some('-') => {
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
                    Some('>') => {
                        self.next_char();
                        TokenKind::RightArrow
                    }
                    _ => TokenKind::Minus,
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '|' => TokenKind::Pipe,
            '#' => TokenKind::Hash,
            '$' => TokenKind::Dollar,
            '?' => {
                if self.peek_char() == Some('?') {
                    self.next_char();
                    TokenKind::NullCoalesce
                } else {
                    TokenKind::Question
                }
            }
            '@' => self.scan_timestamp(start, start_line, start_col)?,
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
            "INSPECT" => TokenKind::Inspect,
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
            "TYPE" => TokenKind::Type,
            "IN" => TokenKind::In,
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
        let has_decimal = if self.peek_char() == Some('.') {
            // Look ahead to see what follows the '.'
            let mut lookahead = self.chars.clone();
            lookahead.next(); // consume '.'
            match lookahead.peek() {
                // Range '..', or no digit after '.', or end of input: don't consume '.'
                Some((_, next_c)) if *next_c == '.' || !next_c.is_ascii_digit() => false,
                None => false,
                // Digit follows '.': consume it and the fractional part
                Some(_) => {
                    number.push('.');
                    self.next_char();
                    while let Some(c) = self.peek_char() {
                        if c.is_ascii_digit() {
                            number.push(c);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    true
                }
            }
        } else {
            false
        };

        // Check for exponent (scientific notation)
        let has_exponent = matches!(self.peek_char(), Some('e' | 'E'));
        if has_exponent {
            self.scan_exponent(&mut number)?;
        }

        if has_decimal || has_exponent {
            let value: f64 = number.parse().map_err(|_| {
                ParseError::new(
                    format!("invalid float literal '{}'", number),
                    self.span_from(start, start_line, start_col),
                )
            })?;
            Ok(TokenKind::Float(value))
        } else {
            let value: i64 = number.parse().map_err(|_| {
                ParseError::new(
                    format!("invalid integer literal '{}'", number),
                    self.span_from(start, start_line, start_col),
                )
            })?;
            Ok(TokenKind::Int(value))
        }
    }

    /// Scan the exponent part of a number (e.g., e10, E-5, e+3)
    fn scan_exponent(&mut self, number: &mut String) -> ParseResult<()> {
        // Consume 'e' or 'E'
        if let Some(c) = self.peek_char() {
            if c == 'e' || c == 'E' {
                number.push(c);
                self.next_char();
            } else {
                return Ok(());
            }
        }

        // Optional sign
        if let Some(c) = self.peek_char() {
            if c == '+' || c == '-' {
                number.push(c);
                self.next_char();
            }
        }

        // Exponent digits (at least one required)
        let mut has_digits = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                number.push(c);
                self.next_char();
                has_digits = true;
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(ParseError::new(
                format!("invalid exponent in number literal '{}'", number),
                self.current_span(),
            ));
        }

        Ok(())
    }

    /// Scan a timestamp literal (e.g., @2024-01-15 or @2024-01-15T10:30:00Z)
    fn scan_timestamp(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> ParseResult<TokenKind> {
        let mut ts_str = String::new();

        // Scan characters that are valid in ISO 8601 timestamps
        // (digits, dashes, colons, T, Z, plus, period)
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit()
                || c == '-'
                || c == ':'
                || c == 'T'
                || c == 'Z'
                || c == '+'
                || c == '.'
            {
                ts_str.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        if ts_str.is_empty() {
            return Err(ParseError::new(
                "expected timestamp after '@'",
                self.span_from(start, start_line, start_col),
            ));
        }

        // Parse the timestamp string to milliseconds since epoch
        let millis = self.parse_timestamp_to_millis(&ts_str).map_err(|e| {
            ParseError::new(
                format!("invalid timestamp '{}': {}", ts_str, e),
                self.span_from(start, start_line, start_col),
            )
        })?;

        Ok(TokenKind::Timestamp(millis))
    }

    /// Parse an ISO 8601 timestamp string to milliseconds since Unix epoch.
    fn parse_timestamp_to_millis(&self, s: &str) -> Result<i64, String> {
        // Parse formats:
        // @2024-01-15                    - date only (midnight UTC)
        // @2024-01-15T10:30:00Z          - full timestamp UTC
        // @2024-01-15T10:30:00+05:30     - with timezone offset
        // @2024-01-15T10:30:00.500Z      - with milliseconds

        let (datetime_part, tz_offset_ms) = if let Some(pos) = s.rfind('+') {
            // Has positive timezone offset
            let (dt, tz) = s.split_at(pos);
            let offset = self.parse_tz_offset(tz)?;
            (dt, offset)
        } else if s.ends_with('Z') {
            // UTC
            (&s[..s.len() - 1], 0i64)
        } else if let Some(pos) = s.rfind('-') {
            // Could be negative offset or just date separator
            // Check if it's after 'T' (time section)
            if s.contains('T') && pos > s.find('T').unwrap() {
                let (dt, tz) = s.split_at(pos);
                let offset = self.parse_tz_offset(tz)?;
                (dt, offset)
            } else {
                (s, 0i64)
            }
        } else {
            (s, 0i64)
        };

        // Parse the datetime part
        let parts: Vec<&str> = datetime_part.split('T').collect();
        let date_str = parts[0];
        let time_str = parts.get(1).copied();

        // Parse date: YYYY-MM-DD
        let date_parts: Vec<&str> = date_str.split('-').collect();
        if date_parts.len() != 3 {
            return Err(format!("invalid date format: {}", date_str));
        }

        let year: i32 = date_parts[0]
            .parse()
            .map_err(|_| format!("invalid year: {}", date_parts[0]))?;
        let month: u32 = date_parts[1]
            .parse()
            .map_err(|_| format!("invalid month: {}", date_parts[1]))?;
        let day: u32 = date_parts[2]
            .parse()
            .map_err(|_| format!("invalid day: {}", date_parts[2]))?;

        if month < 1 || month > 12 {
            return Err(format!("month out of range: {}", month));
        }
        if day < 1 || day > 31 {
            return Err(format!("day out of range: {}", day));
        }

        // Parse time if present: HH:MM:SS or HH:MM:SS.mmm
        let (hour, minute, second, millis) = if let Some(time) = time_str {
            let (time_main, millis) = if let Some(dot_pos) = time.find('.') {
                let ms_str = &time[dot_pos + 1..];
                let ms: i64 = ms_str
                    .parse()
                    .map_err(|_| format!("invalid milliseconds: {}", ms_str))?;
                (&time[..dot_pos], ms)
            } else {
                (time, 0i64)
            };

            let time_parts: Vec<&str> = time_main.split(':').collect();
            if time_parts.len() < 2 {
                return Err(format!("invalid time format: {}", time));
            }

            let hour: u32 = time_parts[0]
                .parse()
                .map_err(|_| format!("invalid hour: {}", time_parts[0]))?;
            let minute: u32 = time_parts[1]
                .parse()
                .map_err(|_| format!("invalid minute: {}", time_parts[1]))?;
            let second: u32 = time_parts
                .get(2)
                .map(|s| s.parse().unwrap_or(0))
                .unwrap_or(0);

            (hour, minute, second, millis)
        } else {
            (0, 0, 0, 0)
        };

        // Calculate days since Unix epoch (1970-01-01)
        // This is a simplified calculation - doesn't handle all edge cases
        let days = self.days_since_epoch(year, month, day);

        // Convert to milliseconds
        let total_ms = (days as i64) * 86_400_000 // days to ms
            + (hour as i64) * 3_600_000          // hours to ms
            + (minute as i64) * 60_000           // minutes to ms
            + (second as i64) * 1_000            // seconds to ms
            + millis                              // milliseconds
            - tz_offset_ms;                       // subtract timezone offset

        Ok(total_ms)
    }

    /// Parse timezone offset like "+05:30" or "-08:00" to milliseconds
    fn parse_tz_offset(&self, tz: &str) -> Result<i64, String> {
        if tz.is_empty() {
            return Ok(0);
        }

        let sign = if tz.starts_with('-') { -1i64 } else { 1i64 };
        let tz_clean = tz.trim_start_matches('+').trim_start_matches('-');

        let parts: Vec<&str> = tz_clean.split(':').collect();
        if parts.is_empty() {
            return Err(format!("invalid timezone: {}", tz));
        }

        let hours: i64 = parts[0]
            .parse()
            .map_err(|_| format!("invalid tz hours: {}", parts[0]))?;
        let minutes: i64 = parts.get(1).map(|s| s.parse().unwrap_or(0)).unwrap_or(0);

        Ok(sign * (hours * 3_600_000 + minutes * 60_000))
    }

    /// Calculate days since Unix epoch (1970-01-01)
    fn days_since_epoch(&self, year: i32, month: u32, day: u32) -> i64 {
        // Days in months (non-leap year)
        let days_before_month: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

        // Calculate years since 1970
        let y = year as i64;
        let mut days = (y - 1970) * 365;

        // Add leap days: every 4 years, minus every 100, plus every 400
        // Leap years between 1970 and year (exclusive)
        let leap_years_before = |yr: i64| -> i64 {
            let yr = yr - 1; // Years up to but not including yr
            yr / 4 - yr / 100 + yr / 400
        };

        days += leap_years_before(y) - leap_years_before(1970);

        // Add days from months
        days += days_before_month[(month - 1) as usize];

        // Add leap day if after Feb in a leap year
        if month > 2 && self.is_leap_year(year) {
            days += 1;
        }

        // Add days
        days += (day - 1) as i64;

        days
    }

    /// Check if a year is a leap year
    fn is_leap_year(&self, year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
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

    #[test]
    fn test_arrow_tokens() {
        let kinds = tokenize("=> -> = >");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Arrow,
                TokenKind::RightArrow,
                TokenKind::Eq,
                TokenKind::Gt,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_constraint_syntax_tokens() {
        let kinds = tokenize("constraint foo: e: Event => e.x > 0");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Constraint,
                TokenKind::Ident("foo".into()),
                TokenKind::Colon,
                TokenKind::Ident("e".into()),
                TokenKind::Colon,
                TokenKind::Ident("Event".into()),
                TokenKind::Arrow,
                TokenKind::Ident("e".into()),
                TokenKind::Dot,
                TokenKind::Ident("x".into()),
                TokenKind::Gt,
                TokenKind::Int(0),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_cardinality_tokens() {
        let kinds = tokenize("[task -> 1]");
        assert_eq!(
            kinds,
            vec![
                TokenKind::LBracket,
                TokenKind::Ident("task".into()),
                TokenKind::RightArrow,
                TokenKind::Int(1),
                TokenKind::RBracket,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_scientific_notation() {
        // Basic scientific notation
        let kinds = tokenize("1.23e10");
        assert_eq!(kinds, vec![TokenKind::Float(1.23e10), TokenKind::Eof]);

        // With negative exponent
        let kinds = tokenize("1.5e-10");
        assert_eq!(kinds, vec![TokenKind::Float(1.5e-10), TokenKind::Eof]);

        // Integer with exponent becomes float
        let kinds = tokenize("1e10");
        assert_eq!(kinds, vec![TokenKind::Float(1e10), TokenKind::Eof]);

        // With positive sign in exponent
        let kinds = tokenize("2.5E+3");
        assert_eq!(kinds, vec![TokenKind::Float(2.5e3), TokenKind::Eof]);

        // Uppercase E
        let kinds = tokenize("3.14E10");
        assert_eq!(kinds, vec![TokenKind::Float(3.14e10), TokenKind::Eof]);
    }
}
