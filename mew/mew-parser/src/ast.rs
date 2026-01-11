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
    Spawn(SpawnStmt),
    Kill(KillStmt),
    Link(LinkStmt),
    Unlink(UnlinkStmt),
    Set(SetStmt),
    Walk(WalkStmt),
    Txn(TxnStmt),
}

// ==================== MATCH ====================

/// MATCH statement for pattern matching.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStmt {
    pub pattern: Vec<PatternElem>,
    pub where_clause: Option<Expr>,
    pub return_clause: ReturnClause,
    pub order_by: Option<Vec<OrderTerm>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub span: Span,
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
#[derive(Debug, Clone, PartialEq)]
pub struct EdgePattern {
    pub edge_type: String,
    pub targets: Vec<String>,
    pub alias: Option<String>,
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

/// Target for KILL/SET operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Var(String),
    Id(String),
    Pattern(Box<MatchStmt>),
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
    Path,
    Nodes,
    Edges,
    Terminal,
    Projections(Vec<Projection>),
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
}

impl fmt::Display for LiteralKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralKind::Null => write!(f, "null"),
            LiteralKind::Bool(b) => write!(f, "{}", b),
            LiteralKind::Int(i) => write!(f, "{}", i),
            LiteralKind::Float(fl) => write!(f, "{}", fl),
            LiteralKind::String(s) => write!(f, "\"{}\"", s),
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
    pub span: Span,
}

// ==================== ONTOLOGY ====================

/// Ontology definition.
#[derive(Debug, Clone, PartialEq)]
pub enum OntologyDef {
    Node(NodeTypeDef),
    Edge(EdgeTypeDef),
    Constraint(ConstraintDef),
    Rule(RuleDef),
}

/// Node type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeTypeDef {
    pub name: String,
    pub attrs: Vec<AttrDef>,
    pub span: Span,
}

/// Attribute definition.
#[derive(Debug, Clone, PartialEq)]
pub struct AttrDef {
    pub name: String,
    pub type_name: String,
    pub modifiers: Vec<AttrModifier>,
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
}

/// Edge type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeDef {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub modifiers: Vec<EdgeModifier>,
    pub span: Span,
}

/// Edge modifier.
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeModifier {
    Acyclic,
    Unique,
    OnKill(OnKillAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnKillAction {
    Cascade,
    Restrict,
    SetNull,
}

/// Constraint definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintDef {
    pub name: String,
    pub on_type: String,
    pub condition: Expr,
    pub span: Span,
}

/// Rule definition.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDef {
    pub name: String,
    pub on_type: String,
    pub auto: bool,
    pub priority: Option<i64>,
    pub production: Vec<Stmt>,
    pub span: Span,
}
