//! Abstract Syntax Tree types for MEW.

use std::fmt;

/// Source location for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// A statement in the MEW language.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Match(MatchStmt),
    MatchMutate(MatchMutateStmt),
    MatchWalk(MatchWalkStmt),
    Spawn(SpawnStmt),
    Kill(KillStmt),
    Link(LinkStmt),
    Unlink(UnlinkStmt),
    Set(SetStmt),
    Walk(WalkStmt),
    Inspect(InspectStmt),
    Txn(TxnStmt),
}

// ==================== MATCH ====================

/// MATCH statement for pattern matching.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStmt {
    pub pattern: Vec<PatternElem>,
    pub where_clause: Option<Expr>,
    /// OPTIONAL MATCH clauses (left outer joins)
    pub optional_matches: Vec<OptionalMatch>,
    pub return_clause: ReturnClause,
    pub order_by: Option<Vec<OrderTerm>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub span: Span,
}

/// An OPTIONAL MATCH clause (left outer join).
#[derive(Debug, Clone, PartialEq)]
pub struct OptionalMatch {
    pub pattern: Vec<PatternElem>,
    pub where_clause: Option<Expr>,
    pub span: Span,
}

/// MATCH followed by mutations (compound statement).
/// E.g., MATCH a: T, b: U WHERE ... LINK edge(a, b)
#[derive(Debug, Clone, PartialEq)]
pub struct MatchMutateStmt {
    pub pattern: Vec<PatternElem>,
    pub where_clause: Option<Expr>,
    pub mutations: Vec<MutationAction>,
    pub span: Span,
}

/// MATCH followed by WALK (compound statement).
/// E.g., MATCH e: Employee WHERE ... WALK FROM e FOLLOW ...
#[derive(Debug, Clone, PartialEq)]
pub struct MatchWalkStmt {
    pub pattern: Vec<PatternElem>,
    pub where_clause: Option<Expr>,
    pub walk: WalkStmt,
    pub span: Span,
}

/// A mutation action within a compound statement.
#[derive(Debug, Clone, PartialEq)]
pub enum MutationAction {
    Link(LinkStmt),
    Set(SetStmt),
    Kill(KillStmt),
    Unlink(UnlinkStmt),
}

/// An element in a pattern (node or edge pattern).
#[derive(Debug, Clone, PartialEq)]
pub enum PatternElem {
    Node(NodePattern),
    Edge(EdgePattern),
}

/// Node pattern: var: Type
#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub var: String,
    pub type_name: String,
    pub span: Span,
}

/// Edge pattern: edge_type(targets) AS alias
/// Or transitive: edge_type+(targets), edge_type*(targets)
#[derive(Debug, Clone, PartialEq)]
pub struct EdgePattern {
    pub edge_type: String,
    pub targets: Vec<String>,
    pub alias: Option<String>,
    /// Transitive modifier: None, Plus (+, one or more), Star (*, zero or more)
    pub transitive: Option<TransitiveKind>,
    pub span: Span,
}

/// Transitive edge pattern modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitiveKind {
    Plus, // + (one or more hops)
    Star, // * (zero or more hops)
}

/// A pattern with pattern elements and an optional WHERE clause.
/// Used in constraints and rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub elements: Vec<PatternElem>,
    pub where_clause: Option<Expr>,
    pub span: Span,
}

/// RETURN clause.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub distinct: bool,
    pub projections: Vec<Projection>,
    pub span: Span,
}

/// A projection in RETURN.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub expr: Expr,
    pub alias: Option<String>,
    pub span: Span,
}

/// ORDER BY term.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderTerm {
    pub expr: Expr,
    pub direction: OrderDirection,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}

// ==================== SPAWN ====================

/// SPAWN statement for creating nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnStmt {
    pub var: String,
    pub type_name: String,
    pub attrs: Vec<AttrAssignment>,
    pub returning: Option<ReturningClause>,
    pub span: Span,
}

/// Attribute assignment: name = value
#[derive(Debug, Clone, PartialEq)]
pub struct AttrAssignment {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

/// RETURNING clause.
#[derive(Debug, Clone, PartialEq)]
pub enum ReturningClause {
    Id,
    All,
    Fields(Vec<String>),
}

// ==================== KILL ====================

/// KILL statement for deleting nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct KillStmt {
    pub target: Target,
    pub cascade: Option<bool>,
    pub returning: Option<ReturningClause>,
    pub span: Span,
}

/// Target for KILL/SET/UNLINK operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Var(String),
    Id(String),
    Pattern(Box<MatchStmt>),
    /// Edge pattern: edge_type(targets) - used for UNLINK
    EdgePattern {
        edge_type: String,
        targets: Vec<String>,
    },
}

// ==================== LINK ====================

/// LINK statement for creating edges.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkStmt {
    pub var: Option<String>,
    pub edge_type: String,
    pub targets: Vec<TargetRef>,
    pub attrs: Vec<AttrAssignment>,
    pub returning: Option<ReturningClause>,
    pub span: Span,
}

/// Target reference for LINK.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetRef {
    Var(String),
    Id(String),
    Pattern(Box<MatchStmt>),
}

// ==================== UNLINK ====================

/// UNLINK statement for deleting edges.
#[derive(Debug, Clone, PartialEq)]
pub struct UnlinkStmt {
    pub target: Target,
    pub returning: Option<ReturningClause>,
    pub span: Span,
}

// ==================== SET ====================

/// SET statement for modifying attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct SetStmt {
    pub target: Target,
    pub assignments: Vec<AttrAssignment>,
    pub returning: Option<ReturningClause>,
    pub span: Span,
}

// ==================== WALK ====================

/// WALK statement for path traversal.
#[derive(Debug, Clone, PartialEq)]
pub struct WalkStmt {
    pub from: Expr,
    pub follow: Vec<FollowClause>,
    pub until: Option<Expr>,
    pub return_type: WalkReturnType,
    pub span: Span,
}

/// FOLLOW clause in WALK.
#[derive(Debug, Clone, PartialEq)]
pub struct FollowClause {
    pub edge_types: Vec<String>,
    pub direction: WalkDirection,
    pub min_depth: Option<i64>,
    pub max_depth: Option<i64>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WalkDirection {
    #[default]
    Outbound,
    Inbound,
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalkReturnType {
    /// RETURN PATH [AS alias]
    Path { alias: Option<String> },
    /// RETURN NODES [AS alias]
    Nodes { alias: Option<String> },
    /// RETURN EDGES [AS alias]
    Edges { alias: Option<String> },
    /// RETURN TERMINAL [AS alias]
    Terminal { alias: Option<String> },
    /// RETURN projections (already has alias in Projection)
    Projections(Vec<Projection>),
}

// ==================== INSPECT ====================

/// INSPECT statement for direct entity lookup by ID.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectStmt {
    /// The ID to look up (as a string without the # prefix)
    pub id: String,
    /// Optional projections to return (defaults to all attributes)
    pub projections: Option<Vec<Projection>>,
    pub span: Span,
}

// ==================== TRANSACTION ====================

/// Transaction statement.
#[derive(Debug, Clone, PartialEq)]
pub enum TxnStmt {
    Begin { isolation: Option<IsolationLevel> },
    Commit,
    Rollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadCommitted,
    Serializable,
}

// ==================== EXPRESSIONS ====================

/// Expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Var(String, Span),
    /// Attribute access: var.attr
    AttrAccess(Box<Expr>, String, Span),
    /// Binary operation
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>, Span),
    /// Unary operation
    UnaryOp(UnaryOp, Box<Expr>, Span),
    /// Function call
    FnCall(FnCall),
    /// ID reference: #id
    IdRef(String, Span),
    /// Parameter reference: $param
    Param(String, Span),
    /// EXISTS subpattern
    Exists(Vec<PatternElem>, Option<Box<Expr>>, Span),
    /// NOT EXISTS subpattern
    NotExists(Vec<PatternElem>, Option<Box<Expr>>, Span),
    /// List literal: [a, b, c]
    List(Vec<Expr>, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(lit) => lit.span,
            Expr::Var(_, span) => *span,
            Expr::AttrAccess(_, _, span) => *span,
            Expr::BinaryOp(_, _, _, span) => *span,
            Expr::UnaryOp(_, _, span) => *span,
            Expr::FnCall(fc) => fc.span,
            Expr::IdRef(_, span) => *span,
            Expr::Param(_, span) => *span,
            Expr::Exists(_, _, span) => *span,
            Expr::NotExists(_, _, span) => *span,
            Expr::List(_, span) => *span,
        }
    }
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    /// Duration in milliseconds
    Duration(i64),
    /// Timestamp as milliseconds since Unix epoch
    Timestamp(i64),
}

impl fmt::Display for LiteralKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralKind::Null => write!(f, "null"),
            LiteralKind::Bool(b) => write!(f, "{}", b),
            LiteralKind::Int(i) => write!(f, "{}", i),
            LiteralKind::Float(fl) => write!(f, "{}", fl),
            LiteralKind::String(s) => write!(f, "\"{}\"", s),
            LiteralKind::Duration(ms) => write!(f, "{}ms", ms),
            LiteralKind::Timestamp(ms) => write!(f, "@{}", ms),
        }
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    // Logical
    And,
    Or,
    // String
    Concat,
    // Null coalescing
    NullCoalesce,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "="),
            BinaryOp::NotEq => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::LtEq => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::GtEq => write!(f, ">="),
            BinaryOp::And => write!(f, "AND"),
            BinaryOp::Or => write!(f, "OR"),
            BinaryOp::Concat => write!(f, "++"),
            BinaryOp::NullCoalesce => write!(f, "??"),
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Function call.
#[derive(Debug, Clone, PartialEq)]
pub struct FnCall {
    pub name: String,
    pub args: Vec<Expr>,
    pub distinct: bool, // For count(DISTINCT ...) style calls
    pub span: Span,
}

// ==================== ONTOLOGY ====================

/// Ontology definition.
#[derive(Debug, Clone, PartialEq)]
pub enum OntologyDef {
    TypeAlias(TypeAliasDef),
    Node(NodeTypeDef),
    Edge(EdgeTypeDef),
    Constraint(ConstraintDef),
    Rule(RuleDef),
}

/// Type alias definition: type Name = BaseType [modifiers]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDef {
    pub name: String,
    pub base_type: String,
    pub modifiers: Vec<AttrModifier>,
    pub span: Span,
}

/// Node type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeTypeDef {
    pub name: String,
    pub parents: Vec<String>,
    pub attrs: Vec<AttrDef>,
    pub span: Span,
}

/// Attribute definition.
#[derive(Debug, Clone, PartialEq)]
pub struct AttrDef {
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
    pub modifiers: Vec<AttrModifier>,
    pub default_value: Option<Expr>,
    pub span: Span,
}

/// Attribute modifier.
#[derive(Debug, Clone, PartialEq)]
pub enum AttrModifier {
    Required,
    Unique,
    Default(Expr),
    Range {
        min: Option<Expr>,
        max: Option<Expr>,
    },
    /// in: ["a", "b", "c"] - allowed values
    InValues(Vec<Expr>),
    /// match: "regex" - regex pattern for validation
    Match(String),
    /// length: N..M - string length constraint
    Length { min: i64, max: i64 },
    /// format: email, url, uuid, etc. - built-in format validation
    Format(String),
}

/// Edge type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeDef {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub attrs: Vec<AttrDef>,
    pub modifiers: Vec<EdgeModifier>,
    pub span: Span,
}

/// Edge modifier.
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeModifier {
    Acyclic,
    Unique,
    NoSelf,
    Symmetric,
    Indexed,
    OnKillSource(ReferentialAction),
    OnKillTarget(ReferentialAction),
    /// Cardinality constraint: param_name -> min..max
    Cardinality {
        param: String,
        min: i64,
        max: CardinalityMax,
    },
}

/// Referential action for edge kill
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferentialAction {
    Cascade,
    Unlink,
    Prevent,
}

/// Maximum cardinality value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalityMax {
    Value(i64),
    Unbounded, // * (unlimited)
}

/// Constraint definition.
/// Format: constraint Name [modifiers]: Pattern => Condition
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintDef {
    pub name: String,
    pub pattern: Pattern,
    pub condition: Expr,
    pub modifiers: ConstraintModifiers,
    pub span: Span,
}

/// Constraint modifiers
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConstraintModifiers {
    pub soft: bool, // soft vs hard (default: hard)
    pub message: Option<String>,
}

/// Rule definition.
/// Format: rule Name [modifiers]: Pattern => Production
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDef {
    pub name: String,
    pub pattern: Pattern,
    pub auto: bool, // true = auto (default), false = manual
    pub priority: Option<i64>,
    pub production: Vec<RuleAction>,
    pub span: Span,
}

/// Rule production actions
#[derive(Debug, Clone, PartialEq)]
pub enum RuleAction {
    /// SPAWN var: Type { attrs }
    Spawn {
        var: String,
        type_name: String,
        attrs: Vec<AttrAssignment>,
        span: Span,
    },
    /// KILL var
    Kill { var: String, span: Span },
    /// LINK edge_type(targets) AS alias { attrs }
    Link {
        edge_type: String,
        targets: Vec<String>,
        alias: Option<String>,
        attrs: Vec<AttrAssignment>,
        span: Span,
    },
    /// UNLINK var
    Unlink { var: String, span: Span },
    /// SET var.attr = value
    Set {
        target: String,
        attr: String,
        value: Expr,
        span: Span,
    },
}
