# MEW System Architecture

**Version:** 1.0
**Status:** Design Specification
**Purpose:** Minimal Executable World — a self-describing higher-order hypergraph database

---

# Part I: Foundations

## 1. Design Philosophy

MEW is a **typed higher-order hypergraph database** with:

- **Self-describing schema**: The schema is stored in the graph itself (Layer 0)
- **Higher-order edges**: Edges can target other edges, enabling meta-relationships
- **Declarative constraints**: Invariants expressed as patterns that must hold
- **Reactive rules**: Automatic transformations triggered by mutations
- **ACID transactions**: All mutations are atomic and consistent

### 1.1 Core Principles

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DESIGN PRINCIPLES                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. SELF-SPECIFICATION                                              │
│     The schema is data. Layer 0 nodes describe what nodes/edges     │
│     can exist. The system describes itself.                         │
│                                                                      │
│  2. RELATIONS ARE PRIMITIVE                                         │
│     Edges are first-class. Higher-order edges (edges about edges)   │
│     are native. No artificial node/edge asymmetry.                  │
│                                                                      │
│  3. CONSTRAINTS ARE DECLARATIVE                                     │
│     "What must be true" not "how to enforce it."                    │
│     Pattern + condition = constraint.                               │
│                                                                      │
│  4. RULES ARE REACTIVE                                              │
│     Mutations trigger rules. Rules fire to quiescence.              │
│     The system reaches stable states automatically.                 │
│                                                                      │
│  5. MINIMAL CORE                                                    │
│     13 components. Each has a distinct contract.                    │
│     Implementation details hidden inside components.                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 What MEW Is Good For

MEW is optimized for **symbolic reasoning over structured relationships**:

| Use Case | Why MEW Fits |
|----------|--------------|
| Knowledge graphs | Native higher-order for provenance, confidence |
| Constraint systems | Declarative constraints with automatic checking |
| Rule-based inference | Reactive rules with pattern matching |
| Schema-driven data | Self-describing, evolvable schema |
| Causal modeling | Directed edges, acyclicity constraints |
| Self-referential systems | Layer 0 is queryable graph structure |

### 1.3 What MEW Is Not

| Limitation | Cause | Alternative |
|------------|-------|-------------|
| Continuous dynamics | Discrete graph model | Discretize or external physics engine |
| Numerical computation | No tensor primitives | External numerical engine (PyTorch) |
| True randomness | Deterministic rules | External RNG or branching (v2) |
| Real-time streaming | Batch transaction model | Event bridge layer |

These are fundamental to discrete symbolic systems, not flaws.

---

## 2. Architecture Overview

### 2.1 Component Map (13 Components)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         MEW SYSTEM                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                      SESSION (External Interface)              │  │
│  │  • REPL / HTTP / Embedded API                                 │  │
│  │  • Connection management, result encoding                     │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
│                                │                                     │
│  ┌─────────────────────────────▼─────────────────────────────────┐  │
│  │                      PARSER                                    │  │
│  │  • Tokenize source text (statements + ontology)               │  │
│  │  • Build AST                                                  │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
│                                │                                     │
│  ┌─────────────────────────────▼─────────────────────────────────┐  │
│  │                      ANALYZER                                  │  │
│  │  • Name resolution against Registry                           │  │
│  │  • Type checking                                              │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
│                                │                                     │
│       ┌────────────────────────┼────────────────────────┐           │
│       │                        │                        │           │
│       ▼                        ▼                        ▼           │
│  ┌─────────┐            ┌─────────────┐          ┌───────────┐     │
│  │ COMPILER│            │    QUERY    │          │  MUTATION │     │
│  │         │            │             │          │           │     │
│  │Ontology │            │Plan + Exec  │          │SPAWN/KILL │     │
│  │→Registry│            │MATCH/WALK   │          │LINK/UNLINK│     │
│  │→Layer 0 │            │             │          │SET        │     │
│  └────┬────┘            └──────┬──────┘          └─────┬─────┘     │
│       │                        │                       │           │
│       ▼                        │                       │           │
│  ┌─────────┐                   │                       │           │
│  │REGISTRY │◀──────────────────┤                       │           │
│  │         │                   │                       │           │
│  │Types    │                   ▼                       ▼           │
│  │Edges    │            ┌─────────────────────────────────────┐   │
│  │Constr.  │            │              PATTERN                 │   │
│  │Rules    │            │  • Compile patterns                  │   │
│  └─────────┘            │  • Match against graph               │   │
│                         │  • Evaluate expressions              │   │
│                         │  • Transitive closure                │   │
│                         └──────────────┬──────────────────────┘   │
│                                        │                           │
│                         ┌──────────────┼──────────────┐           │
│                         │              │              │           │
│                         ▼              ▼              ▼           │
│                   ┌──────────┐  ┌──────────┐  ┌────────────┐     │
│                   │CONSTRAINT│  │   RULE   │  │TRANSACTION │     │
│                   │          │  │          │  │            │     │
│                   │Validate  │  │Trigger   │  │ACID        │     │
│                   │Hard/Soft │  │Execute   │  │Orchestrate │     │
│                   │Immediate/│  │Quiescence│  │Commit/     │     │
│                   │Deferred  │  │          │  │Rollback    │     │
│                   └──────────┘  └──────────┘  └─────┬──────┘     │
│                                                     │             │
│  ┌──────────────────────────────────────────────────▼───────────┐ │
│  │                         GRAPH                                 │ │
│  │  • Store nodes and edges                                     │ │
│  │  • Indexes: Type, Attribute, Edge, Adjacency, Higher-Order   │ │
│  │  • ID allocation                                             │ │
│  └──────────────────────────────────────────────────┬───────────┘ │
│                                                     │             │
│  ┌──────────────────────────────────────────────────▼───────────┐ │
│  │                        JOURNAL                                │ │
│  │  • Write-ahead log                                           │ │
│  │  • Crash recovery                                            │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Responsibilities

| Component | Purpose | Absorbs (Implementation Details) |
|-----------|---------|----------------------------------|
| **Graph** | Store nodes/edges, indexed access | GraphStore + TypeIndex + AttrIndex + EdgeIndex + AdjacencyIndex + HigherOrderIndex |
| **Parser** | Tokenize and parse all source text | Lexer + StatementParser + OntologyParser |
| **Analyzer** | Name resolution, type checking | — |
| **Registry** | Runtime schema lookup | TypeRegistry + EdgeTypeRegistry + ConstraintRegistry + RuleRegistry |
| **Compiler** | Ontology → Registry + Layer 0 | SugarExpander + Validator + L0Generator + RegistryBuilder |
| **Pattern** | Match patterns, evaluate expressions | PatternCompiler + PatternMatcher + ExpressionEvaluator |
| **Constraint** | Validate mutations against constraints | — |
| **Rule** | Trigger and execute rules to quiescence | RuleMatcher + RuleExecutor |
| **Query** | Plan and execute MATCH/WALK/INSPECT | QueryPlanner + QueryExecutor |
| **Mutation** | Execute SPAWN/KILL/LINK/UNLINK/SET | — |
| **Transaction** | ACID, orchestrate mutation→rule→constraint | TransactionManager + TransactionBuffer |
| **Journal** | WAL + recovery | WAL + Recovery |
| **Session** | External interface | SessionManager + REPL + HTTPHandler |

### 2.3 Dependency Graph

```
                    Session
                       │
                       ▼
                    Parser
                       │
                       ▼
                   Analyzer ◄──────────┐
                       │               │
          ┌────────────┼────────────┐  │
          │            │            │  │
          ▼            ▼            ▼  │
      Compiler      Query       Mutation
          │            │            │
          ▼            │            │
      Registry ◄───────┤            │
                       │            │
                       ▼            ▼
                    Pattern
                       │
          ┌────────────┼────────────┐
          │            │            │
          ▼            ▼            ▼
     Constraint      Rule      Transaction
          │            │            │
          └────────────┼────────────┘
                       │
                       ▼
                     Graph
                       │
                       ▼
                    Journal
```

---

## 3. Data Model

### 3.1 Entity Representation

```
┌─────────────────────────────────────────────────────────────────────┐
│                         NODE STRUCTURE                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Node {                                                             │
│    id: NodeId           // 64-bit, globally unique, immutable       │
│    type_id: TypeId      // Reference to Registry                    │
│    version: u64         // MVCC version                             │
│    attributes: Map<AttrId, Value>                                   │
│  }                                                                  │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                         EDGE STRUCTURE                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Edge {                                                             │
│    id: EdgeId           // 64-bit, globally unique, immutable       │
│    type_id: EdgeTypeId  // Reference to Registry                    │
│    targets: Vec<EntityId>  // Ordered list: NodeId | EdgeId        │
│    version: u64         // MVCC version                             │
│    attributes: Map<AttrId, Value>                                   │
│  }                                                                  │
│                                                                      │
│  EntityId = NodeId | EdgeId  // Unified ID space                    │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                      HIGHER-ORDER EDGES                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Example: causes(e1, e2) with confidence                            │
│                                                                      │
│  edge_1: causes                                                     │
│    targets: [NodeId(e1), NodeId(e2)]                               │
│                                                                      │
│  edge_2: confidence                                                 │
│    targets: [EdgeId(edge_1)]    ← Higher-order: targets an edge    │
│    attributes: { level: 0.85 }                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Value Types

```
┌─────────────────────────────────────────────────────────────────────┐
│                         VALUE TYPES                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Value = Null                                                       │
│        | Bool(bool)                                                 │
│        | Int(i64)                                                   │
│        | Float(f64)                                                 │
│        | String(String)                                             │
│        | Timestamp(i64)      // millis since epoch                  │
│        | Duration(i64)       // millis                              │
│        | NodeRef(NodeId)                                            │
│        | EdgeRef(EdgeId)                                            │
│                                                                      │
│  Type expressions in ontology:                                      │
│    • Scalar: String, Int, Float, Bool, Timestamp, Duration         │
│    • Optional: T? (nullable)                                        │
│    • Reference: NodeType, edge<EdgeType>                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 Layer 0 (Self-Describing Schema)

Layer 0 is the **schema stored as graph structure**. When you define:

```
node Task { title: String [required] }
edge belongs_to(task: Task, project: Project)
```

The compiler creates:

```
Nodes:
  _NodeType { name: "Task", abstract: false, ... }
  _AttributeDef { name: "title", required: true, ... }
  _EdgeType { name: "belongs_to", arity: 2, ... }
  _VarDef { name: "task", position: 0, ... }
  _VarDef { name: "project", position: 1, ... }

Edges:
  _type_has_attribute(_NodeType:Task, _AttributeDef:title)
  _attr_has_type(_AttributeDef:title, _ScalarTypeExpr:String)
  _edge_has_position(_EdgeType:belongs_to, _VarDef:task) { position: 0 }
  _edge_has_position(_EdgeType:belongs_to, _VarDef:project) { position: 1 }
  _var_has_type(_VarDef:task, _NamedTypeRef:Task)
  _var_has_type(_VarDef:project, _NamedTypeRef:Project)
```

**Key insight**: The schema is queryable. You can write:

```sql
MATCH t: _NodeType, a: _AttributeDef, _type_has_attribute(t, a)
WHERE t.name = "Task"
RETURN a.name, a.required
```

---

## 4. Execution Model

### 4.1 Reactive Execution (v1)

MEW v1 uses **synchronous reactive execution**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    REACTIVE EXECUTION MODEL                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   External mutation                                                  │
│         │                                                            │
│         ▼                                                            │
│   ┌───────────┐                                                     │
│   │   Apply   │                                                     │
│   │  mutation │                                                     │
│   └─────┬─────┘                                                     │
│         │                                                            │
│         ▼                                                            │
│   ┌───────────┐     ┌───────────┐                                   │
│   │   Rules   │────▶│  More     │──┐                                │
│   │   fire    │     │ mutations │  │                                │
│   └───────────┘     └───────────┘  │                                │
│         ▲                          │                                 │
│         └──────────────────────────┘                                │
│              (repeat until quiescent)                                │
│         │                                                            │
│         ▼                                                            │
│   ┌───────────┐                                                     │
│   │  Check    │                                                     │
│   │constraints│                                                     │
│   └─────┬─────┘                                                     │
│         │                                                            │
│    ┌────┴────┐                                                      │
│    │         │                                                       │
│    ▼         ▼                                                       │
│  Pass      Fail                                                      │
│    │         │                                                       │
│    ▼         ▼                                                       │
│  COMMIT   ROLLBACK                                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Semantics:**
1. External mutation enters transaction
2. Rules fire repeatedly until no new matches (quiescence)
3. Constraints check final state
4. Hard constraint violation → abort entire transaction
5. Success → commit

**Limits to prevent infinite loops:**
- `MAX_RULE_DEPTH = 100` — Nested rule trigger depth
- `MAX_ACTIONS = 10,000` — Total actions per transaction
- Same `(rule_id, bindings_hash)` executes at most once

### 4.2 Transaction Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TRANSACTION STATE MACHINE                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌────────┐   BEGIN    ┌────────┐                                  │
│   │  NONE  │ ─────────▶ │ ACTIVE │                                  │
│   └────────┘            └────┬───┘                                  │
│                              │                                       │
│               ┌──────────────┼──────────────┐                       │
│               │              │              │                        │
│               │ ROLLBACK     │ COMMIT       │ error                  │
│               ▼              ▼              ▼                        │
│         ┌─────────┐   ┌───────────┐   ┌─────────┐                   │
│         │ ABORTED │   │COMMITTING │   │ ABORTED │                   │
│         └─────────┘   └─────┬─────┘   └─────────┘                   │
│                             │                                        │
│                  ┌──────────┴──────────┐                            │
│                  │                     │                             │
│                  │ success             │ failure                     │
│                  ▼                     ▼                             │
│            ┌───────────┐         ┌─────────┐                        │
│            │ COMMITTED │         │ ABORTED │                        │
│            └───────────┘         └─────────┘                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Transaction buffer** tracks uncommitted changes:
- `created_nodes: Map<NodeId, Node>`
- `created_edges: Map<EdgeId, Edge>`
- `deleted_nodes: Set<NodeId>`
- `deleted_edges: Set<EdgeId>`
- `modified_attrs: Map<(EntityId, AttrId), (Old, New)>`

**Queries within transaction** see uncommitted changes (read-your-writes).

### 4.3 Mutation Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      MUTATION EXECUTION FLOW                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SPAWN t: Task { title = "New", priority = 8 }                      │
│                          │                                           │
│                          ▼                                           │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 1. TYPE VALIDATION                                             │  │
│  │    • Task.abstract = false? ✓                                  │  │
│  │    • title: String, "New" is String ✓                          │  │
│  │    • priority: Int, 8 is Int ✓                                 │  │
│  │    • Apply defaults for missing optional attrs                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                          │                                           │
│                          ▼                                           │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 2. APPLY TO TRANSACTION BUFFER                                 │  │
│  │    • Allocate NodeId                                           │  │
│  │    • Add to created_nodes                                      │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                          │                                           │
│                          ▼                                           │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 3. RULE ENGINE (repeat until quiescent)                        │  │
│  │    • Find rules affected by mutation                           │  │
│  │    • For each rule, find new matches                           │  │
│  │    • Execute productions (may create more mutations)           │  │
│  │    • Track executed (rule_id, bindings_hash) to avoid loops   │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                          │                                           │
│                          ▼                                           │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 4. CONSTRAINT CHECK                                            │  │
│  │    • Find constraints affected by all mutations                │  │
│  │    • Evaluate each constraint                                  │  │
│  │    • Hard fail → ABORT                                         │  │
│  │    • Soft fail → WARN                                          │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                          │                                           │
│                          ▼                                           │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 5. DEFERRED CONSTRAINTS (at commit)                            │  │
│  │    • Cardinality constraints                                   │  │
│  │    • Existence constraints                                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                          │                                           │
│                          ▼                                           │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 6. COMMIT                                                      │  │
│  │    • Write WAL                                                 │  │
│  │    • Apply to Graph                                            │  │
│  │    • Update indexes                                            │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part II: Component Specifications

## 5. Graph Component

**Purpose**: Store and retrieve nodes/edges with indexed access.

### 5.1 Contract

```rust
trait Graph {
    // Node operations
    fn create_node(type_id: TypeId, attrs: Attributes) -> NodeId;
    fn get_node(id: NodeId) -> Option<Node>;
    fn delete_node(id: NodeId) -> Result<()>;
    fn set_attr(id: NodeId, attr: AttrId, value: Value) -> Result<()>;
    
    // Edge operations
    fn create_edge(type_id: EdgeTypeId, targets: Vec<EntityId>, attrs: Attributes) -> EdgeId;
    fn get_edge(id: EdgeId) -> Option<Edge>;
    fn delete_edge(id: EdgeId) -> Result<()>;
    
    // Queries (index-backed)
    fn nodes_by_type(type_id: TypeId) -> Iterator<NodeId>;
    fn nodes_by_attr(type_id: TypeId, attr: AttrId, value: Value) -> Iterator<NodeId>;
    fn nodes_by_attr_range(type_id: TypeId, attr: AttrId, min: Value, max: Value) -> Iterator<NodeId>;
    fn edges_by_type(type_id: EdgeTypeId) -> Iterator<EdgeId>;
    fn edges_from(node: NodeId, edge_type: Option<EdgeTypeId>) -> Iterator<EdgeId>;
    fn edges_to(node: NodeId, edge_type: Option<EdgeTypeId>) -> Iterator<EdgeId>;
    fn edges_about(edge: EdgeId) -> Iterator<EdgeId>;  // Higher-order index
    
    // Snapshot
    fn snapshot() -> GraphSnapshot;
}
```

### 5.2 Internal Indexes

| Index | Structure | Purpose |
|-------|-----------|---------|
| **Type Index** | `TypeId → Set<NodeId>` | Find all nodes of a type |
| **Attribute Index** | `(TypeId, AttrId, Value) → Set<NodeId>` | Find nodes by attribute value/range |
| **Unique Index** | `(TypeId, AttrId, Value) → NodeId` | Enforce uniqueness, fast lookup |
| **Edge Index** | `(EdgeTypeId, Position, EntityId) → Set<EdgeId>` | Find edges by type and target |
| **Adjacency Index** | `NodeId → { outbound: Map<EdgeTypeId, Set<EdgeId>>, inbound: ... }` | Fast neighbor lookup |
| **Higher-Order Index** | `EdgeId → Set<EdgeId>` | Edges that target this edge |

### 5.3 Implementation Notes

- v1 starts **in-memory** (no BufferPool/FileManager)
- Indexes are maintained synchronously on mutation
- Higher-order index enables cascade deletion of meta-edges

---

## 6. Parser Component

**Purpose**: Tokenize and parse source text into AST.

### 6.1 Contract

```rust
trait Parser {
    fn parse_statement(source: &str) -> Result<Statement, ParseError>;
    fn parse_ontology(source: &str) -> Result<OntologyAST, ParseError>;
}

enum Statement {
    Match(MatchStmt),
    Walk(WalkStmt),
    Inspect(InspectStmt),
    Spawn(SpawnStmt),
    Kill(KillStmt),
    Link(LinkStmt),
    Unlink(UnlinkStmt),
    Set(SetStmt),
    Begin,
    Commit,
    Rollback,
    Load(LoadStmt),
    // ... etc
}
```

### 6.2 Grammar Summary

```ebnf
Statement     = MatchStmt | WalkStmt | MutationStmt | TxnStmt | AdminStmt

MatchStmt     = "match" Pattern ("where" Expr)? ReturnClause?
WalkStmt      = "walk" "from" Expr FollowClause+ ReturnWalkClause

MutationStmt  = SpawnStmt | KillStmt | LinkStmt | UnlinkStmt | SetStmt
SpawnStmt     = "spawn" VarDecl AttrBlock?
KillStmt      = "kill" VarRef
LinkStmt      = "link" EdgePattern AttrBlock?
UnlinkStmt    = "unlink" VarRef
SetStmt       = "set" AttrAccess "=" Expr

Pattern       = PatternElem ("," PatternElem)*
PatternElem   = NodePattern | EdgePattern
NodePattern   = Identifier ":" TypeRef
EdgePattern   = EdgeType "(" TargetList ")" ("as" Identifier)?
```

---

## 7. Analyzer Component

**Purpose**: Name resolution and type checking against Registry.

### 7.1 Contract

```rust
trait Analyzer {
    fn analyze(&self, stmt: Statement, registry: &Registry) -> Result<AnalyzedStmt, AnalysisError>;
}

struct AnalyzedStmt {
    stmt: Statement,
    bindings: Map<String, Binding>,      // Variable → type info
    type_info: Map<ExprId, TypeInfo>,    // Expression → result type
}
```

### 7.2 Analysis Passes

1. **Name Resolution**
   - Type names → TypeId via Registry
   - Variable references → declaration site
   - Attribute names → AttrId via Registry

2. **Type Checking**
   - Expression types computed bottom-up
   - Operator type rules enforced
   - Attribute access validated against type

3. **Scope Analysis**
   - Variable visibility rules
   - EXISTS inner scope handling

---

## 8. Registry Component

**Purpose**: Runtime schema lookup. Single source of truth for types, edges, constraints, rules.

### 8.1 Contract

```rust
trait Registry {
    // Type lookup
    fn get_type(&self, name: &str) -> Option<&TypeDef>;
    fn get_type_by_id(&self, id: TypeId) -> &TypeDef;
    fn is_subtype(&self, child: TypeId, parent: TypeId) -> bool;
    fn all_subtypes(&self, id: TypeId) -> &Set<TypeId>;
    
    // Edge type lookup
    fn get_edge_type(&self, name: &str) -> Option<&EdgeTypeDef>;
    fn get_edge_type_by_id(&self, id: EdgeTypeId) -> &EdgeTypeDef;
    
    // Constraint lookup
    fn constraints_for_type(&self, id: TypeId) -> &[ConstraintDef];
    fn constraints_for_edge(&self, id: EdgeTypeId) -> &[ConstraintDef];
    fn deferred_constraints(&self) -> &[ConstraintDef];
    
    // Rule lookup
    fn rules_for_type(&self, id: TypeId) -> &[RuleDef];  // sorted by priority
    fn rules_for_edge(&self, id: EdgeTypeId) -> &[RuleDef];
    fn get_rule(&self, name: &str) -> Option<&RuleDef>;
}
```

### 8.2 Data Structures

```rust
struct TypeDef {
    id: TypeId,
    name: String,
    parent_ids: Vec<TypeId>,
    attributes: Vec<AttrDef>,
    abstract_: bool,
    sealed: bool,
    l0_node_id: NodeId,  // Reference to Layer 0
}

struct EdgeTypeDef {
    id: EdgeTypeId,
    name: String,
    arity: u8,
    signature: Vec<SignatureParam>,  // (position, name, type_expr)
    attributes: Vec<AttrDef>,
    symmetric: bool,
    reflexive_allowed: bool,
    l0_node_id: NodeId,
}

struct ConstraintDef {
    id: ConstraintId,
    name: String,
    hard: bool,
    pattern: CompiledPattern,
    condition: CompiledExpr,
    affected_types: Set<TypeId>,
    affected_edges: Set<EdgeTypeId>,
    deferred: bool,
    message: String,
}

struct RuleDef {
    id: RuleId,
    name: String,
    priority: i32,
    auto: bool,
    pattern: CompiledPattern,
    production: CompiledProduction,
    affected_types: Set<TypeId>,
    affected_edges: Set<EdgeTypeId>,
}
```

---

## 9. Compiler Component

**Purpose**: Transform ontology source into populated Registry + Layer 0 graph.

### 9.1 Contract

```rust
trait Compiler {
    fn compile(&self, ontology: OntologyAST, graph: &mut Graph) -> Result<Registry, CompileError>;
}
```

### 9.2 Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COMPILATION PIPELINE                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Ontology Source                                                    │
│        │                                                             │
│        ▼                                                             │
│  ┌───────────┐                                                      │
│  │  PARSE    │  → OntologyAST                                       │
│  └─────┬─────┘                                                      │
│        │                                                             │
│        ▼                                                             │
│  ┌───────────┐                                                      │
│  │  EXPAND   │  Modifiers → explicit constraints/rules              │
│  │  SUGAR    │  [required] → constraint                             │
│  │           │  [unique] → constraint + index                       │
│  │           │  [acyclic] → cycle check constraint                  │
│  └─────┬─────┘                                                      │
│        │                                                             │
│        ▼                                                             │
│  ┌───────────┐                                                      │
│  │ VALIDATE  │  • No duplicate names                                │
│  │           │  • No inheritance cycles                             │
│  │           │  • All references resolve                            │
│  │           │  • Type expressions valid                            │
│  └─────┬─────┘                                                      │
│        │                                                             │
│        ▼                                                             │
│  ┌───────────┐                                                      │
│  │ GENERATE  │  Create Layer 0 nodes/edges in Graph                 │
│  │ LAYER 0   │  _NodeType, _EdgeType, _AttributeDef, etc.          │
│  └─────┬─────┘                                                      │
│        │                                                             │
│        ▼                                                             │
│  ┌───────────┐                                                      │
│  │  BUILD    │  Populate Registry from Layer 0                      │
│  │ REGISTRY  │  Precompute: subtypes, constraint index, rule index │
│  └─────┬─────┘                                                      │
│        │                                                             │
│        ▼                                                             │
│  Registry + Layer 0 Graph                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 10. Pattern Component

**Purpose**: Compile patterns, match against graph, evaluate expressions.

### 10.1 Contract

```rust
trait Pattern {
    fn compile(&self, pattern: PatternAST, registry: &Registry) -> Result<CompiledPattern, PatternError>;
    fn match_(&self, pattern: &CompiledPattern, graph: &Graph, initial: Bindings) -> Iterator<Bindings>;
    fn evaluate(&self, expr: &CompiledExpr, bindings: &Bindings, graph: &Graph) -> Result<Value, EvalError>;
}

struct CompiledPattern {
    node_vars: Vec<NodeVar>,      // Variables bound to nodes
    edge_patterns: Vec<EdgePat>,  // Edge patterns to match
    conditions: Vec<CompiledExpr>, // WHERE conditions
    
    // Execution hints
    join_order: Vec<VarId>,       // Order to bind variables
    index_hints: Vec<IndexHint>,  // Suggested indexes
}

struct Bindings {
    values: Map<VarId, EntityId>,
}
```

### 10.2 Matching Algorithm

```
match(pattern, initial_bindings) → Iterator<Bindings>:

  1. ORDER variables by selectivity (most constrained first)
  
  2. For first unbound variable V:
     
     If V is node variable:
       candidates = get_candidates(V.type, conditions_on_V)
     
     If V is connected via edge to bound variable B:
       candidates = traverse_edge(B, edge_type, position)
     
     If V is edge variable:
       candidates = find_matching_edges(edge_pattern)
  
  3. For each candidate C:
     
     bindings' = bindings ∪ {V → C}
     
     Check conditions involving V
       If fail → continue to next candidate
     
     If all variables bound:
       yield bindings'
     
     Else:
       recurse: match(remaining_pattern, bindings')
```

### 10.3 Transitive Closure

For `edge+(A, B)` or `edge*(A, B)`:

```
match_transitive(start, edge_type, mode, max_depth):
  visited = Set()
  frontier = [(start, 0)]
  
  while frontier not empty:
    (current, depth) = frontier.pop()
    
    if current in visited: continue
    visited.add(current)
    
    if depth > 0 or mode == Star:  // Star includes depth=0
      yield current
    
    if depth < max_depth:
      for neighbor in get_neighbors(current, edge_type):
        frontier.push((neighbor, depth + 1))
```

---

## 11. Constraint Component

**Purpose**: Validate mutations against constraints.

### 11.1 Contract

```rust
trait Constraint {
    fn check(
        &self,
        mutation: &Mutation,
        graph: &Graph,
        registry: &Registry,
        pattern: &Pattern,
    ) -> Result<Vec<Warning>, ConstraintViolation>;
    
    fn check_deferred(
        &self,
        graph: &Graph,
        registry: &Registry,
        pattern: &Pattern,
    ) -> Result<Vec<Warning>, ConstraintViolation>;
}
```

### 11.2 Constraint Categories

| Category | When Checked | Examples |
|----------|--------------|----------|
| **Immediate** | After each mutation | Value constraints (`>= 0`), required attrs, `no_self`, `acyclic` |
| **Deferred** | At commit time | Cardinality (`task -> 1`), existence (`=> EXISTS(...)`) |

### 11.3 Check Algorithm

```
check(mutation, graph, registry, pattern):
  
  affected = registry.constraints_for_mutation(mutation)
  
  for constraint in affected:
    
    // Seed pattern match from mutated entity
    initial = seed_from_mutation(constraint.pattern, mutation)
    matches = pattern.match(constraint.pattern, graph, initial)
    
    for bindings in matches:
      result = pattern.evaluate(constraint.condition, bindings, graph)
      
      if result == false:
        if constraint.hard:
          return Error(ConstraintViolation(constraint, bindings))
        else:
          warnings.push(Warning(constraint, bindings))
  
  return Ok(warnings)
```

---

## 12. Rule Component

**Purpose**: Trigger and execute rules to quiescence.

### 12.1 Contract

```rust
trait Rule {
    fn process(
        &self,
        mutation: &Mutation,
        txn: &mut Transaction,
        graph: &Graph,
        registry: &Registry,
        pattern: &Pattern,
        mutation_executor: &Mutation,
    ) -> Result<(), RuleError>;
}
```

### 12.2 Execution Algorithm

```
process(mutation, txn, graph, registry, pattern, mutation_executor):
  
  executed = Set()  // (rule_id, bindings_hash)
  
  loop:
    triggered = registry.rules_for_mutation(mutation)
    triggered.sort_by(priority, descending)
    
    new_matches = []
    
    for rule in triggered:
      for bindings in pattern.match(rule.pattern, graph):
        key = (rule.id, hash(bindings))
        if key not in executed:
          new_matches.push((rule, bindings))
    
    if new_matches.empty():
      break  // Quiescent
    
    for (rule, bindings) in new_matches:
      
      if txn.depth >= MAX_DEPTH:
        return Error(DepthLimitExceeded)
      if txn.action_count >= MAX_ACTIONS:
        return Error(ActionLimitExceeded)
      
      txn.depth += 1
      
      for action in rule.production.actions:
        execute_action(action, bindings, txn, mutation_executor)
        txn.action_count += 1
      
      txn.depth -= 1
      executed.add((rule.id, hash(bindings)))
  
  return Ok(())
```

---

## 13. Query Component

**Purpose**: Plan and execute MATCH/WALK/INSPECT statements.

### 13.1 Contract

```rust
trait Query {
    fn execute(
        &self,
        stmt: AnalyzedMatchStmt,
        graph: &Graph,
        registry: &Registry,
        pattern: &Pattern,
    ) -> Result<ResultSet, QueryError>;
}
```

### 13.2 Query Plan Operators

| Operator | Purpose |
|----------|---------|
| `TypeScan(type_id)` | Scan all nodes of type |
| `IndexScan(index, range)` | Scan index for range |
| `UniqueIndexLookup(index, value)` | Single-row lookup |
| `EdgeScan(edge_type)` | Scan all edges of type |
| `AdjacencyLookup(node, edge_type, direction)` | Find edges from/to node |
| `NestedLoopJoin(outer, inner, condition)` | Join via nested iteration |
| `HashJoin(left, right, keys)` | Join via hash table |
| `Filter(child, condition)` | Filter rows |
| `Project(child, projections)` | Select/compute columns |
| `Sort(child, keys)` | Sort rows |
| `Limit(child, count, offset)` | Limit rows |
| `Aggregate(child, group_keys, aggregations)` | Group and aggregate |
| `TransitiveClosure(start, edge_type, depth)` | BFS/DFS traversal |

### 13.3 WALK Execution

WALK is syntactic sugar over MATCH with transitive closure:

```sql
WALK FROM #start FOLLOW causes [depth: 1..5] RETURN PATH
```

Compiles to pattern with `causes+(start, end)` and depth constraints.

### 13.4 INSPECT Execution

INSPECT queries Layer 0:

```sql
INSPECT TYPES WHERE name LIKE "Task%"
```

Compiles to:

```sql
MATCH t: _NodeType WHERE t.name LIKE "Task%" RETURN t
```

---

## 14. Mutation Component

**Purpose**: Execute SPAWN/KILL/LINK/UNLINK/SET operations.

### 14.1 Contract

```rust
trait Mutation {
    fn spawn(&self, type_id: TypeId, attrs: Attributes, txn: &mut Transaction) -> Result<NodeId, MutationError>;
    fn kill(&self, node_id: NodeId, txn: &mut Transaction) -> Result<(), MutationError>;
    fn link(&self, edge_type: EdgeTypeId, targets: Vec<EntityId>, attrs: Attributes, txn: &mut Transaction) -> Result<EdgeId, MutationError>;
    fn unlink(&self, edge_id: EdgeId, txn: &mut Transaction) -> Result<(), MutationError>;
    fn set(&self, entity_id: EntityId, attr_id: AttrId, value: Value, txn: &mut Transaction) -> Result<(), MutationError>;
}
```

### 14.2 Validation

Each mutation validates:
- Type exists and is concrete (not abstract)
- Required attributes present
- Attribute types match
- Edge targets match signature types
- Entity exists (for KILL/UNLINK/SET)

### 14.3 Cascade Handling

**KILL node**: 
- Find all edges involving node (via adjacency index)
- Unlink each edge first
- Then delete node

**UNLINK edge**:
- Find all higher-order edges targeting this edge (via higher-order index)
- Unlink each meta-edge first
- Then delete edge

---

## 15. Transaction Component

**Purpose**: ACID transaction management, orchestrate mutation→rule→constraint flow.

### 15.1 Contract

```rust
trait Transaction {
    fn begin(&mut self, isolation: IsolationLevel) -> TxnId;
    fn commit(&mut self, txn_id: TxnId) -> Result<(), TxnError>;
    fn rollback(&mut self, txn_id: TxnId) -> Result<(), TxnError>;
    
    fn execute_mutation(
        &mut self,
        mutation: MutationStmt,
        graph: &mut Graph,
        registry: &Registry,
        pattern: &Pattern,
        constraint: &Constraint,
        rule: &Rule,
        mutation_exec: &Mutation,
    ) -> Result<MutationResult, TxnError>;
}
```

### 15.2 Commit Sequence

```
commit(txn_id):
  
  1. CHECK DEFERRED CONSTRAINTS
     • Cardinality constraints
     • Existence constraints
     • If fail → ABORT
  
  2. WRITE WAL
     • Append all mutations
     • Append COMMIT record
     • fsync()
  
  3. APPLY TO GRAPH
     • Move from txn buffer to graph
     • Update indexes
  
  4. RETURN SUCCESS
```

---

## 16. Journal Component

**Purpose**: Write-ahead log and crash recovery.

### 16.1 Contract

```rust
trait Journal {
    fn append(&mut self, entry: WALEntry) -> LSN;
    fn sync(&mut self) -> Result<(), IOError>;
    fn recover(&mut self, graph: &mut Graph) -> Result<RecoveryStats, RecoveryError>;
}

enum WALEntry {
    Begin { txn_id: TxnId },
    Commit { txn_id: TxnId },
    Abort { txn_id: TxnId },
    SpawnNode { txn_id: TxnId, node_id: NodeId, type_id: TypeId, attrs: Attributes },
    KillNode { txn_id: TxnId, node_id: NodeId, prev_state: Node },
    LinkEdge { txn_id: TxnId, edge_id: EdgeId, type_id: EdgeTypeId, targets: Vec<EntityId>, attrs: Attributes },
    UnlinkEdge { txn_id: TxnId, edge_id: EdgeId, prev_state: Edge },
    SetAttr { txn_id: TxnId, entity_id: EntityId, attr_id: AttrId, old: Value, new: Value },
}
```

### 16.2 Recovery Algorithm

```
recover(graph):
  
  1. Read WAL from last checkpoint
  
  2. Group entries by transaction
  
  3. Identify committed vs uncommitted transactions
  
  4. REDO: Replay all committed transactions in LSN order
  
  5. (Uncommitted transactions are simply ignored - 
      they were never applied to durable storage)
  
  6. System ready
```

---

## 17. Session Component

**Purpose**: External interface (REPL, HTTP, embedded).

### 17.1 Contract

```rust
trait Session {
    fn execute(&mut self, statement: &str) -> Result<Response, SessionError>;
    fn close(&mut self);
}

struct SessionState {
    id: SessionId,
    current_txn: Option<TxnId>,
    auto_commit: bool,
    current_branch: BranchId,  // For versioning (v1.x)
}
```

### 17.2 Statement Routing

```
execute(statement):
  
  ast = parser.parse_statement(statement)
  analyzed = analyzer.analyze(ast, registry)
  
  match analyzed:
    
    Query(match_stmt) →
      query.execute(match_stmt, graph, registry, pattern)
    
    Mutation(mut_stmt) →
      ensure_transaction()
      transaction.execute_mutation(mut_stmt, ...)
    
    Begin →
      transaction.begin(default_isolation)
    
    Commit →
      transaction.commit(current_txn)
    
    Rollback →
      transaction.rollback(current_txn)
    
    Load(ontology) →
      compiler.compile(ontology, graph)
```

---

# Part III: Implementation Plan

## 18. v1 Scope

### 18.1 In Scope (v1.0)

| Category | Features |
|----------|----------|
| **Data Model** | Nodes, edges, higher-order edges, attributes |
| **Schema** | Layer 0, type inheritance, edge signatures |
| **Queries** | MATCH, WALK, INSPECT, expressions, aggregates |
| **Mutations** | SPAWN, KILL, LINK, UNLINK, SET |
| **Constraints** | Hard/soft, immediate/deferred, modifiers |
| **Rules** | Auto-fire, priority, quiescence |
| **Transactions** | BEGIN/COMMIT/ROLLBACK, ACID |
| **Durability** | WAL, crash recovery |
| **Interface** | REPL, basic HTTP API |

### 18.2 Deferred (v1.x / v2)

| Feature | Version | Notes |
|---------|---------|-------|
| Versioning (snapshot, branch, diff, merge) | v1.x | Data versioning |
| META mode (reflection, schema evolution) | v2 | Runtime schema modification |
| Branching execution | v2 | Probabilistic/quantum simulation |
| Complex amplitudes | v2 | With branching |
| Provenance tracking | v1.x or v2 | Automatic causal trace |
| Automatic reification | v2 | Edge → node shadow |
| GPU acceleration | v2+ | Tensor-backed storage |
| Distribution | v2+ | Multi-node deployment |

### 18.3 Implementation Order

```
Phase 1: Foundation
════════════════════
  □ Core data structures (Node, Edge, Value)
  □ In-memory Graph (no indexes yet)
  □ Parser (statements only)
  □ Basic REPL

Phase 2: Type System
════════════════════
  □ Registry (hardcoded for testing)
  □ Analyzer (name resolution, type checking)
  □ Type validation in mutations

Phase 3: Ontology
════════════════════
  □ Ontology parser
  □ Sugar expander
  □ Layer 0 generator
  □ Registry builder
  □ LOAD statement

Phase 4: Pattern Matching
════════════════════
  □ Pattern compiler
  □ Pattern matcher (basic)
  □ Expression evaluator
  □ MATCH execution

Phase 5: Constraints & Rules
════════════════════
  □ Constraint checker
  □ Rule engine
  □ Transaction integration

Phase 6: Indexes
════════════════════
  □ Type index
  □ Attribute index
  □ Edge index
  □ Adjacency index
  □ Higher-order index
  □ Query planner (index selection)

Phase 7: Durability
════════════════════
  □ WAL
  □ Recovery
  □ Checkpointing

Phase 8: Polish
════════════════════
  □ WALK execution
  □ INSPECT execution
  □ HTTP API
  □ Error messages
  □ Performance tuning
```

---

## 19. Testing Strategy

### 19.1 Unit Tests (per component)

| Component | Test Focus |
|-----------|------------|
| Parser | Grammar coverage, error messages |
| Analyzer | Type errors, name resolution |
| Pattern | Match correctness, transitive closure |
| Constraint | Violation detection, hard vs soft |
| Rule | Quiescence, priority order, cycle prevention |
| Graph | Index correctness, CRUD operations |

### 19.2 Integration Tests

| Scenario | Coverage |
|----------|----------|
| End-to-end query | Parse → Analyze → Plan → Execute |
| End-to-end mutation | Mutation → Rules → Constraints → Commit |
| Transaction rollback | Constraint violation → abort → clean state |
| Recovery | Crash simulation → WAL replay → consistent state |

### 19.3 Property Tests

| Property | Description |
|----------|-------------|
| Schema invariants | Layer 0 matches Registry |
| Constraint soundness | No committed state violates hard constraints |
| Rule determinism | Same input → same output (modulo ID allocation) |
| Transaction isolation | Uncommitted changes invisible to other sessions |

---

## 20. Performance Targets (v1)

| Operation | Target | Notes |
|-----------|--------|-------|
| Point query by ID | < 1μs | Hash lookup |
| Type scan (1K nodes) | < 1ms | Index scan |
| Pattern match (simple) | < 10ms | 2-3 variables, indexed |
| Pattern match (complex) | < 100ms | 5+ variables, joins |
| Mutation (single) | < 1ms | Excluding rules |
| Rule execution (quiescence) | < 100ms | Typical case |
| Transaction commit | < 10ms | Excluding fsync |
| WAL fsync | < 50ms | Disk-dependent |

---

# Part IV: Future Extensions (v2+)

## 21. META Mode (v2)

**Runtime schema reflection and evolution.**

See separate META Mode specification. Summary:

- `META MATCH` — Query with unknown types (`edge<any>`)
- `META CREATE` — Runtime type/constraint/rule creation
- `META ENABLE/DISABLE` — Toggle constraints/rules
- Permission levels: READ, WRITE, ADMIN
- Layer 0 invariants protected

## 22. Branching Execution (v2)

**Multiple simultaneous execution branches for simulation.**

```sql
rule decay:
  p: Particle
  =>
  BRANCH [weight: 0.7]: SET p.state = "alpha"
  BRANCH [weight: 0.3]: SET p.state = "beta"
```

Both branches execute. Weights track density. Interference possible with complex amplitudes.

## 23. Provenance Tracking (v1.x or v2)

**Automatic causal trace of rule execution.**

Every mutation creates causation edge:

```
edge _caused_by(effect: any, cause: any) {
  rule_name: String,
  timestamp: Timestamp,
}
```

Execution trace becomes queryable graph structure.

## 24. GPU Acceleration (v2+)

**Tensor-backed storage for parallel pattern matching.**

- Batch candidate evaluation
- Parallel constraint checking
- Sparse matrix graph representation

Design contracts to accept batches from start (even if v1 processes sequentially).

---

# Appendix A: Layer 0 Schema

```
// Core meta-types (protected, immutable)

node _NodeType {
  name: String [required, unique]
  abstract: Bool = false
  sealed: Bool = false
  doc: String?
}

node _EdgeType {
  name: String [required, unique]
  arity: Int [required, >= 1]
  symmetric: Bool = false
  reflexive_allowed: Bool = true
  doc: String?
}

node _AttributeDef {
  name: String [required]
  required: Bool = false
  unique: Bool = false
  indexed: Bool = false
  doc: String?
}

node _ConstraintDef {
  name: String [required, unique]
  hard: Bool = true
  deferred: Bool = false
  message: String?
  doc: String?
}

node _RuleDef {
  name: String [required, unique]
  priority: Int = 0
  auto: Bool = true
  doc: String?
}

// Type expressions
node _ScalarTypeExpr { scalar_type: String }
node _NamedTypeRef { type_name: String }
node _EdgeTypeRef { edge_type_name: String }
node _OptionalTypeExpr { }
node _ListTypeExpr { }

// Pattern structure
node _PatternDef { }
node _VarDef { name: String, is_edge_var: Bool = false }
node _EdgePatternDef { negated: Bool = false }

// Expression structure (simplified)
node _Expr { }
node _LiteralExpr { value: String }
node _VarRefExpr { var_name: String }
node _AttrAccessExpr { attr_name: String }
node _BinaryOpExpr { operator: String }
node _UnaryOpExpr { operator: String }
node _FunctionCallExpr { function_name: String }

// Production structure
node _ProductionDef { }
node _SpawnAction { var_name: String }
node _KillAction { }
node _LinkAction { }
node _UnlinkAction { }
node _SetAction { attr_name: String }

// Schema relationships
edge _type_inherits(child: _NodeType, parent: _NodeType) [acyclic]
edge _type_has_attribute(type: _NodeType, attr: _AttributeDef)
edge _attr_has_type(attr: _AttributeDef, type_expr: _Expr)
edge _edge_has_position(edge: _EdgeType, var: _VarDef) { position: Int }
edge _var_has_type(var: _VarDef, type_expr: _Expr)
edge _constraint_has_pattern(constraint: _ConstraintDef, pattern: _PatternDef)
edge _constraint_has_condition(constraint: _ConstraintDef, condition: _Expr)
edge _rule_has_pattern(rule: _RuleDef, pattern: _PatternDef)
edge _rule_has_production(rule: _RuleDef, production: _ProductionDef)
edge _pattern_has_node_var(pattern: _PatternDef, var: _VarDef)
edge _pattern_has_edge_pattern(pattern: _PatternDef, edge_pattern: _EdgePatternDef)
edge _edge_pattern_has_target(edge_pattern: _EdgePatternDef, var: _VarDef) { position: Int }
```

---

# Appendix B: Error Codes

| Code | Category | Description |
|------|----------|-------------|
| E1xxx | Parse | Syntax errors |
| E2xxx | Analysis | Name resolution, type checking |
| E3xxx | Constraint | Constraint violations |
| E4xxx | Rule | Rule execution errors |
| E5xxx | Transaction | Transaction errors |
| E6xxx | Storage | I/O errors |

Examples:
- `E1001` — Unexpected token
- `E2001` — Unknown type
- `E2002` — Unknown attribute
- `E2003` — Type mismatch
- `E3001` — Hard constraint violated
- `E4001` — Rule depth limit exceeded
- `E5001` — Transaction already active
- `E6001` — WAL write failed

---

*End of MEW Architecture v1.0*