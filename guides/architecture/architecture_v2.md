# HOHG System Architecture

**Version:** 1.0
**Status:** Design Specification
**Scope:** System architecture, component design, algorithms, and data structures

---

# Part I: System Overview

## 1. Architecture Philosophy

### 1.1 Design Principles

| Principle | Architectural Implication |
|-----------|--------------------------|
| **Graph-native** | All data represented as nodes and edges; no relational mapping |
| **Constraint-first** | Validation integrated into mutation path, not bolted on |
| **Compile-time optimization** | Ontology structure enables specialized indexes and query plans |
| **Inspectable** | All state observable through the same query mechanisms |
| **Atomic semantics** | Transactions include constraint checking and rule execution |

### 1.2 Non-Goals

- **Not a distributed system** (v1): Single-node deployment; clustering deferred
- **Not a streaming processor**: Batch-oriented queries; reactive extensions deferred
- **Not a full logic engine**: Decidable subset of reasoning; no arbitrary inference

---

## 2. Component Architecture

### 2.1 High-Level Component Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐│
│  │   CLI       │  │  HTTP API   │  │  Language   │  │  Embedded Library  ││
│  │   (REPL)    │  │  (REST/WS)  │  │  Bindings   │  │                     ││
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘│
└─────────┼────────────────┼────────────────┼─────────────────────┼───────────┘
          │                │                │                     │
          └────────────────┴────────────────┴─────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              API GATEWAY                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Statement Router │ Session Manager │ Connection Pool │ Auth (stub) │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              QUERY PROCESSOR                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │   Lexer    │  │   Parser   │  │  Analyzer  │  │   Query Planner      │  │
│  │            │──▶│            │──▶│            │──▶│                      │  │
│  └────────────┘  └────────────┘  └────────────┘  └──────────────────────┘  │
│                                                              │              │
│  ┌──────────────────────────────────────────────────────────▼────────────┐  │
│  │                        Query Executor                                  │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐ │  │
│  │  │  MATCH  │  │  WALK   │  │ TRANSFORM│  │  ADMIN  │  │  VERSION    │ │  │
│  │  │ Executor│  │ Executor│  │ Executor │  │ Executor│  │  Executor   │ │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CORE ENGINE                                     │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Transaction Manager                             │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────────────────┐│ │
│  │  │  Begin   │  │  Commit  │  │ Rollback │  │  Isolation Controller   ││ │
│  │  └──────────┘  └──────────┘  └──────────┘  └─────────────────────────┘│ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐ │
│  │  Pattern Matcher │  │Constraint Checker│  │     Rule Engine           │ │
│  │                  │  │                  │  │                           │ │
│  │  • Subgraph ISO  │  │  • Hard/Soft     │  │  • Trigger Detection      │ │
│  │  • Transitive    │  │  • Incremental   │  │  • Priority Ordering      │ │
│  │  • Negative      │  │  • Index-assisted│  │  • Action Execution       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘ │
│                                                                             │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Type System                                     │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐ │ │
│  │  │ Type Registry │  │ Subtype Graph│  │  Runtime Type Checker       │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              STORAGE LAYER                                   │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Graph Store                                     │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐ │ │
│  │  │  Node Store  │  │  Edge Store  │  │  Attribute Store             │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐ │
│  │  Index Manager  │  │ Version Manager │  │     WAL (Write-Ahead Log)  │ │
│  │                 │  │                 │  │                             │ │
│  │  • Type Index   │  │  • Snapshots    │  │  • Transaction Log         │ │
│  │  • Attr Index   │  │  • Branches     │  │  • Recovery                │ │
│  │  • Edge Index   │  │  • Diff Engine  │  │  • Checkpointing           │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘ │
│                                                                             │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Persistence                                     │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐ │ │
│  │  │  Page Cache  │  │  Buffer Pool │  │  File Manager                │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              COMPILER                                        │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  Ontology  │  │  Semantic  │  │  Constraint│  │   Registry Builder   │  │
│  │  Parser    │──▶│  Analyzer  │──▶│  Expander  │──▶│                      │  │
│  └────────────┘  └────────────┘  └────────────┘  └──────────────────────┘  │
│                                                              │              │
│                                                              ▼              │
│  ┌────────────────────────────────────────────────────────────────────────┐│
│  │                    Runtime Registries                                   ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐││
│  │  │  Types   │  │  Edges   │  │Constraints│  │  Rules   │  │  Indexes │││
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘││
│  └────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **API Gateway** | Protocol handling, connection management, authentication |
| **Query Processor** | Parse, analyze, plan, and execute statements |
| **Core Engine** | Graph operations, constraints, rules, transactions |
| **Storage Layer** | Persistence, indexing, versioning, recovery |
| **Compiler** | Ontology parsing, validation, registry generation |

---

## 3. Data Flow

### 3.1 Query Execution Flow

```
Statement Text
     │
     ▼
┌─────────────┐
│   Lexer     │ ──▶ Token Stream
└─────────────┘
     │
     ▼
┌─────────────┐
│   Parser    │ ──▶ AST
└─────────────┘
     │
     ▼
┌─────────────┐
│  Analyzer   │ ──▶ Annotated AST (types, references resolved)
└─────────────┘
     │
     ▼
┌─────────────┐
│  Planner    │ ──▶ Query Plan (optimized execution strategy)
└─────────────┘
     │
     ▼
┌─────────────┐
│  Executor   │ ──▶ Result Stream
└─────────────┘
```

### 3.2 Mutation Execution Flow

```
Mutation Statement
     │
     ▼
┌────────────────┐
│ Parse & Plan   │
└────────────────┘
     │
     ▼
┌────────────────┐
│ Begin Txn      │ (if not already in transaction)
└────────────────┘
     │
     ▼
┌────────────────┐
│ Type Check     │ ──▶ Error if type mismatch
└────────────────┘
     │
     ▼
┌────────────────┐
│ Apply Mutation │ ──▶ Tentative changes in transaction buffer
└────────────────┘
     │
     ▼
┌────────────────┐
│ Check          │ ──▶ Error if hard constraint fails (immediate)
│ Constraints    │
└────────────────┘
     │
     ▼
┌────────────────┐
│ Find Triggered │
│ Rules          │
└────────────────┘
     │
     ├──▶ If rules found: Execute rule actions ──▶ (loop back to Apply)
     │
     ▼
┌────────────────┐
│ Check Deferred │ ──▶ Error if cardinality constraint fails
│ Constraints    │
└────────────────┘
     │
     ▼
┌────────────────┐
│ Commit         │ ──▶ Persist to storage, update indexes
└────────────────┘
     │
     ▼
Result (IDs, counts)
```

### 3.3 Ontology Loading Flow

```
Ontology Source (.hog)
     │
     ▼
┌─────────────────┐
│ Ontology Lexer  │ ──▶ Tokens
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Ontology Parser │ ──▶ Ontology AST
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Name Resolution │ ──▶ AST with resolved type references
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Type Checking   │ ──▶ AST with type annotations
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Sugar Expansion │ ──▶ Modifiers expanded to constraints
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Validation      │ ──▶ Error if invalid ontology
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ L0 Generation   │ ──▶ Layer 0 nodes and edges
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Registry Build  │ ──▶ Runtime registries populated
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Index Creation  │ ──▶ Indexes built from declarations
└─────────────────┘
     │
     ▼
Ontology Active
```

---

## 4. Threading Model

### 4.1 Concurrency Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Thread Pools                              │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │  Connection Pool │  │   Query Workers  │  │ Background    │ │
│  │  (I/O threads)   │  │   (CPU threads)  │  │ (maintenance) │ │
│  │                  │  │                  │  │               │ │
│  │  Accept/Read/    │  │  Parse/Plan/     │  │ Checkpoint    │ │
│  │  Write           │  │  Execute         │  │ Index Maint   │ │
│  │                  │  │                  │  │ WAL Cleanup   │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
│           │                    │                    │           │
│           └────────────────────┼────────────────────┘           │
│                                │                                │
│                                ▼                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Work Queue                           │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐          │   │
│  │  │ Task │ │ Task │ │ Task │ │ Task │ │ ...  │          │   │
│  │  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘          │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Locking Strategy

| Resource | Lock Type | Granularity |
|----------|-----------|-------------|
| Graph Store | Read-Write Lock | Per-node / Per-edge |
| Index | Read-Write Lock | Per-index segment |
| Transaction Buffer | Exclusive | Per-transaction |
| Registries | Read-mostly | Global (copy-on-write) |
| WAL | Append-only | Sequential |

### 4.3 Transaction Isolation

| Level | Implementation |
|-------|----------------|
| **Read Committed** | Snapshot at statement start; see committed changes |
| **Serializable** | Snapshot at transaction start; conflict detection at commit |

---

# Part II: Core Data Structures

## 5. Graph Representation

### 5.1 Node Structure

```
Node {
    id: NodeId                          // Unique identifier (64-bit or UUID)
    type_id: TypeId                     // Reference to _NodeType
    attributes: AttributeMap            // Attribute name → value
    version: Version                    // For MVCC
    flags: NodeFlags                    // Deleted, locked, etc.
}

NodeId = opaque 64-bit or 128-bit identifier
TypeId = index into type registry

AttributeMap = one of:
    - Dense array (for fixed schema, known attribute count)
    - Hash map (for sparse/variable attributes)
    - Hybrid (dense for common, map for rare)

NodeFlags {
    deleted: bit
    pending_delete: bit
    has_higher_order_edges: bit         // Optimization hint
}
```

### 5.2 Edge Structure

```
Edge {
    id: EdgeId                          // Unique identifier
    type_id: EdgeTypeId                 // Reference to _EdgeType
    targets: TargetArray                // Ordered array of target IDs
    attributes: AttributeMap            // Edge attributes
    version: Version
    flags: EdgeFlags
}

TargetArray = fixed-size array based on edge type arity
    - Binary edges: [source_id, target_id]
    - Hyperedges: [target_0, target_1, ..., target_n]
    
Target = NodeId | EdgeId                // For higher-order edges

EdgeFlags {
    deleted: bit
    pending_delete: bit
    is_higher_order: bit                // True if any target is an edge
}
```

### 5.3 Higher-Order Edge Handling

```
Higher-order edge storage:

1. Same Edge table (no separate structure)
2. Target array can contain EdgeIds
3. Type system distinguishes node vs edge targets via signature

Example:
    causes(Event, Event) → targets = [NodeId, NodeId]
    confidence(edge<causes>, Float) → targets = [EdgeId] + attributes.level

Higher-order lookup:
    - Forward: Given edge E, find edges targeting E
    - Reverse: Given edge H, find edges H targets
    
Implementation:
    - Maintain higher_order_index: EdgeId → Set<EdgeId>
    - Updated on LINK/UNLINK of higher-order edges
    - Used for cascade deletion
```

### 5.4 Attribute Storage

```
AttributeValue = 
    | Null
    | Bool(bool)
    | Int(i64)
    | Float(f64)
    | String(string_ref)              // Interned or heap
    | Timestamp(i64)                  // Milliseconds since epoch
    | Duration(i64)                   // Milliseconds

String storage options:
    1. Inline (≤ 16 bytes): Store in AttributeValue directly
    2. Interned (common strings): Store reference to intern table
    3. Heap (large/unique): Store reference to string heap

AttributeMap layouts:
    
    Dense layout (for types with fixed attributes):
        [Value0, Value1, Value2, ...]
        - Index = attribute position in type definition
        - Fast access, no key lookup
        - Null represented by sentinel value
    
    Sparse layout (for variable attributes):
        HashMap<AttributeId, AttributeValue>
        - Flexible, supports inheritance
        - Higher overhead per attribute
```

---

## 6. Index Structures

### 6.1 Index Types

```
┌─────────────────────────────────────────────────────────────────┐
│                        Index Manager                             │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Type Index                            │   │
│  │  TypeId → Set<NodeId>                                   │   │
│  │  Supports: Find all nodes of type T                     │   │
│  │  Structure: Hash map with ordered sets                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Attribute Index                         │   │
│  │  (TypeId, AttrId, Value) → Set<NodeId>                  │   │
│  │  Supports: Find nodes where T.attr = value              │   │
│  │  Structure: B+ tree or LSM tree                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Edge Index                            │   │
│  │  (EdgeTypeId, position, TargetId) → Set<EdgeId>         │   │
│  │  Supports: Find edges of type E where target[i] = X     │   │
│  │  Structure: Composite B+ tree                           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 Adjacency Index                          │   │
│  │  NodeId → {outbound: Map<EdgeTypeId, Set<EdgeId>>,      │   │
│  │            inbound: Map<EdgeTypeId, Set<EdgeId>>}       │   │
│  │  Supports: Find edges from/to node N                    │   │
│  │  Structure: Hash map with nested maps                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │               Higher-Order Index                         │   │
│  │  EdgeId → Set<EdgeId>                                   │   │
│  │  Supports: Find edges referencing edge E                │   │
│  │  Structure: Hash map                                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 Unique Index                             │   │
│  │  (TypeId, AttrId, Value) → NodeId (exactly one)         │   │
│  │  Supports: Uniqueness constraint checking               │   │
│  │  Structure: Hash map                                    │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Index Selection

```
Query pattern → Index choice:

MATCH t: Task
    → Type Index: Task → all task IDs
    
MATCH t: Task WHERE t.status = "done"
    → Attribute Index: (Task, status, "done") → matching IDs
    → Fallback: Type Index + filter
    
MATCH t: Task, p: Person, assigned_to(t, p)
    → Start from smaller set
    → Edge Index: (assigned_to, 0, t) or (assigned_to, 1, p)
    
MATCH e1: Event, e2: Event, causes(e1, e2) AS c, confidence(c, _)
    → Edge Index for causes
    → Higher-Order Index for confidence on causes edges
```

### 6.3 Index Maintenance

```
On SPAWN node:
    1. Add to Type Index (node.type_id)
    2. Add to Type Index (all parent types)
    3. For each indexed attribute:
        - Add to Attribute Index
        - If unique: check + add to Unique Index

On KILL node:
    1. Remove from all Type Indexes
    2. Remove from all Attribute Indexes
    3. Remove from Unique Indexes
    4. Update Adjacency Index (edges become invalid)

On LINK edge:
    1. Add to Edge Index for edge type
    2. Update Adjacency Index for all targets
    3. If higher-order: add to Higher-Order Index

On UNLINK edge:
    1. Remove from Edge Index
    2. Update Adjacency Index
    3. Remove from Higher-Order Index
    4. Cascade: find and unlink higher-order edges

On SET attribute:
    1. If indexed: update Attribute Index (remove old, add new)
    2. If unique: check new value, update Unique Index
```

---

## 7. Registry Structures

### 7.1 Type Registry

```
TypeRegistry {
    types: Map<TypeName, TypeDef>
    type_ids: Map<TypeId, TypeDef>
    inheritance_graph: DirectedGraph<TypeId>    // Child → Parent edges
    
    operations:
        lookup(name) → TypeDef?
        lookup_id(id) → TypeDef?
        is_subtype(child, parent) → bool        // Uses inheritance_graph
        get_all_subtypes(type) → Set<TypeId>    // Cached, precomputed
        get_all_attributes(type) → List<AttrDef> // Inherited + own
}

TypeDef {
    id: TypeId
    name: String
    parent_ids: List<TypeId>
    own_attributes: List<AttrDef>
    all_attributes: List<AttrDef>               // Cached: inherited + own
    abstract: bool
    sealed: bool
    layer0_node_id: NodeId                      // Reference to _NodeType node
}

AttrDef {
    id: AttrId
    name: String
    type: ScalarType | NodeTypeRef
    required: bool
    unique: bool
    indexed: IndexOrder
    default_value: AttributeValue?
    layer0_node_id: NodeId                      // Reference to _AttributeDef
}
```

### 7.2 Edge Type Registry

```
EdgeTypeRegistry {
    edge_types: Map<EdgeTypeName, EdgeTypeDef>
    edge_type_ids: Map<EdgeTypeId, EdgeTypeDef>
    
    operations:
        lookup(name) → EdgeTypeDef?
        validate_targets(edge_type, targets) → Result<(), TypeError>
}

EdgeTypeDef {
    id: EdgeTypeId
    name: String
    arity: Int
    signature: List<SignatureParam>             // Position → type constraint
    attributes: List<AttrDef>
    symmetric: bool
    reflexive_allowed: bool
    layer0_node_id: NodeId
}

SignatureParam {
    position: Int
    name: String
    type: TypeExpr                              // Node type, edge<T>, union, any
    is_edge_ref: bool
}
```

### 7.3 Constraint Registry

```
ConstraintRegistry {
    constraints: List<ConstraintDef>
    by_type: Map<TypeId, List<ConstraintDef>>   // Constraints affecting type
    by_edge_type: Map<EdgeTypeId, List<ConstraintDef>>
    
    operations:
        get_affected(mutation) → List<ConstraintDef>
}

ConstraintDef {
    id: ConstraintId
    name: String
    hard: bool
    message: String?
    pattern: CompiledPattern
    condition: CompiledExpr
    affected_types: Set<TypeId>                 // Precomputed
    affected_edge_types: Set<EdgeTypeId>
    layer0_node_id: NodeId
}
```

### 7.4 Rule Registry

```
RuleRegistry {
    rules: List<RuleDef>
    auto_rules: List<RuleDef>                   // Sorted by priority (descending)
    manual_rules: Map<RuleName, RuleDef>
    by_type: Map<TypeId, List<RuleDef>>
    by_edge_type: Map<EdgeTypeId, List<RuleDef>>
    
    operations:
        get_triggered(mutation) → List<RuleDef>  // Only auto rules
        get_manual(name) → RuleDef?
}

RuleDef {
    id: RuleId
    name: String?
    priority: Int
    auto: bool
    pattern: CompiledPattern
    production: CompiledProduction
    affected_types: Set<TypeId>
    affected_edge_types: Set<EdgeTypeId>
    layer0_node_id: NodeId
}
```

---

## 8. Compiled Structures

### 8.1 Compiled Pattern

```
CompiledPattern {
    node_vars: List<CompiledNodeVar>
    edge_vars: List<CompiledEdgeVar>
    edge_patterns: List<CompiledEdgePattern>
    conditions: List<CompiledExpr>              // WHERE clause conditions
    
    // Execution hints
    estimated_cardinality: Int
    recommended_start: VarId                    // Best variable to start matching
    join_order: List<JoinStep>                  // Optimized join sequence
}

CompiledNodeVar {
    var_id: VarId
    name: String
    type_id: TypeId
    type_expr: TypeExpr                         // For unions, any
}

CompiledEdgeVar {
    var_id: VarId
    name: String
    edge_type_id: EdgeTypeId?                   // null for edge<any>
}

CompiledEdgePattern {
    edge_type_id: EdgeTypeId
    target_vars: List<VarId>                    // Position → variable
    alias_var: VarId?                           // If bound with AS
    negated: bool
    transitive: TransitiveMode                  // None, Plus, Star
    depth_limit: Int?
}

TransitiveMode = None | Plus | Star

JoinStep = 
    | ScanType(TypeId)
    | ScanIndex(IndexRef, Condition)
    | JoinEdge(EdgeTypeId, from_var, to_var)
    | Filter(CompiledExpr)
```

### 8.2 Compiled Expression

```
CompiledExpr =
    | Literal(AttributeValue)
    | VarRef(VarId)
    | AttrAccess(CompiledExpr, AttrId)
    | BinaryOp(Operator, CompiledExpr, CompiledExpr)
    | UnaryOp(Operator, CompiledExpr)
    | FunctionCall(FunctionId, List<CompiledExpr>)
    | Exists(CompiledPattern)
    | NotExists(CompiledPattern)
    | Aggregate(AggregateOp, CompiledExpr, CompiledPattern?)
    | If(CompiledExpr, CompiledExpr, CompiledExpr)
    | Case(CompiledExpr?, List<WhenClause>, CompiledExpr?)
    | Coalesce(List<CompiledExpr>)

Operator = 
    | Eq | Ne | Lt | Gt | Le | Ge
    | Add | Sub | Mul | Div | Mod
    | And | Or | Not | Neg
    | Concat

AggregateOp = Count | Sum | Avg | Min | Max | Collect
```

### 8.3 Compiled Production

```
CompiledProduction {
    actions: List<CompiledAction>               // Ordered by execution sequence
}

CompiledAction =
    | Spawn {
        var_name: String,
        type_id: TypeId,
        attributes: List<(AttrId, CompiledExpr)>
    }
    | Kill {
        target_var: VarId
    }
    | Link {
        edge_type_id: EdgeTypeId,
        target_vars: List<VarId>,
        alias_name: String?,
        attributes: List<(AttrId, CompiledExpr)>
    }
    | Unlink {
        target_var: VarId
    }
    | Set {
        target_var: VarId,
        attr_id: AttrId,
        value: CompiledExpr
    }
```

---

# Part III: Storage Architecture

## 9. Persistence Model

### 9.1 Storage Layout

```
Database Directory Structure:

/database_root/
├── data/
│   ├── nodes/
│   │   ├── segment_0001.dat          # Node data pages
│   │   ├── segment_0002.dat
│   │   └── ...
│   ├── edges/
│   │   ├── segment_0001.dat          # Edge data pages
│   │   └── ...
│   └── strings/
│       ├── heap_0001.dat             # String heap
│       └── intern_table.dat          # Interned strings
├── indexes/
│   ├── type_index.idx
│   ├── attr_*.idx                    # Per-attribute indexes
│   ├── edge_*.idx                    # Per-edge-type indexes
│   └── adjacency.idx
├── wal/
│   ├── wal_000001.log                # Write-ahead log segments
│   ├── wal_000002.log
│   └── ...
├── snapshots/
│   ├── snap_20240115_103000/         # Point-in-time snapshots
│   │   ├── metadata.json
│   │   ├── nodes.snap
│   │   ├── edges.snap
│   │   └── indexes.snap
│   └── ...
├── branches/
│   ├── main/                         # Main branch state
│   └── experiment/                   # Other branches
├── meta/
│   ├── ontology.bin                  # Compiled ontology
│   ├── registries.bin                # Runtime registries
│   └── config.json                   # Database configuration
└── lock                              # Database lock file
```

### 9.2 Page Structure

```
Page Layout (fixed size, e.g., 16KB):

┌─────────────────────────────────────────────────────────────┐
│ Page Header (64 bytes)                                      │
│   magic: u32                                                │
│   page_id: u64                                              │
│   page_type: u8 (Node, Edge, Index, Overflow)              │
│   item_count: u16                                           │
│   free_space_offset: u16                                    │
│   checksum: u32                                             │
│   lsn: u64 (Log Sequence Number)                           │
│   reserved: [u8; 32]                                        │
├─────────────────────────────────────────────────────────────┤
│ Item Pointers (variable, grows downward)                    │
│   [offset: u16, size: u16] × item_count                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                      Free Space                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Item Data (variable, grows upward)                          │
│   Actual node/edge/index entries                            │
└─────────────────────────────────────────────────────────────┘

Node Item:
    id: NodeId (8 bytes)
    type_id: TypeId (4 bytes)
    version: Version (8 bytes)
    flags: u8
    attr_count: u8
    [attr_id: u16, value: AttributeValue] × attr_count
    
    Variable-size values (strings) store offset to string heap

Edge Item:
    id: EdgeId (8 bytes)
    type_id: EdgeTypeId (4 bytes)
    version: Version (8 bytes)
    flags: u8
    arity: u8
    [target_id: u64] × arity
    attr_count: u8
    [attr_id: u16, value: AttributeValue] × attr_count
```

### 9.3 Write-Ahead Log (WAL)

```
WAL Entry:

┌─────────────────────────────────────────────────────────────┐
│ Entry Header                                                │
│   lsn: u64                      # Log Sequence Number       │
│   txn_id: u64                   # Transaction ID            │
│   timestamp: i64                # Wall clock time           │
│   entry_type: u8                # Begin, Commit, Abort, Op  │
│   entry_size: u32               # Size of data portion      │
│   prev_lsn: u64                 # Previous LSN in this txn  │
│   checksum: u32                                             │
├─────────────────────────────────────────────────────────────┤
│ Entry Data (depends on entry_type)                          │
│                                                             │
│ Begin:                                                      │
│   isolation_level: u8                                       │
│                                                             │
│ Commit:                                                     │
│   (empty)                                                   │
│                                                             │
│ Abort:                                                      │
│   (empty)                                                   │
│                                                             │
│ SpawnNode:                                                  │
│   node_id: NodeId                                           │
│   type_id: TypeId                                           │
│   attributes: serialized AttributeMap                       │
│                                                             │
│ KillNode:                                                   │
│   node_id: NodeId                                           │
│   prev_state: serialized Node (for rollback)               │
│                                                             │
│ LinkEdge:                                                   │
│   edge_id: EdgeId                                           │
│   type_id: EdgeTypeId                                       │
│   targets: serialized TargetArray                           │
│   attributes: serialized AttributeMap                       │
│                                                             │
│ UnlinkEdge:                                                 │
│   edge_id: EdgeId                                           │
│   prev_state: serialized Edge (for rollback)               │
│                                                             │
│ SetAttribute:                                               │
│   entity_id: NodeId | EdgeId                               │
│   attr_id: AttrId                                           │
│   old_value: AttributeValue                                 │
│   new_value: AttributeValue                                 │
└─────────────────────────────────────────────────────────────┘

WAL Operations:

append(entry) → LSN
    1. Serialize entry
    2. Compute checksum
    3. Append to current segment
    4. If segment full, rotate to new segment
    5. Return assigned LSN

sync() → void
    Force WAL to disk (fsync)

recover() → List<Transaction>
    1. Read all WAL segments
    2. Group entries by transaction
    3. For committed: replay operations
    4. For uncommitted: apply undo records

checkpoint() → void
    1. Flush all dirty pages to disk
    2. Write checkpoint record to WAL
    3. Delete WAL segments before checkpoint
```

---

## 10. Transaction Manager

### 10.1 Transaction State

```
Transaction {
    id: TxnId
    state: TxnState                             # Active, Committed, Aborted
    isolation_level: IsolationLevel
    start_timestamp: Timestamp
    
    // Write set
    created_nodes: Map<NodeId, Node>
    created_edges: Map<EdgeId, Edge>
    deleted_nodes: Set<NodeId>
    deleted_edges: Set<EdgeId>
    modified_attrs: Map<(EntityId, AttrId), (OldValue, NewValue)>
    
    // Read set (for serializable)
    read_nodes: Set<NodeId>
    read_edges: Set<EdgeId>
    predicate_locks: List<PredicateLock>
    
    // Savepoints
    savepoints: Map<SavepointName, SavepointState>
    
    // Rule execution tracking
    executed_rules: Set<(RuleId, BindingHash)>
    rule_depth: Int
    action_count: Int
}

TxnState = Active | Committing | Committed | Aborting | Aborted

SavepointState {
    created_nodes_checkpoint: Int
    created_edges_checkpoint: Int
    modified_attrs_checkpoint: Int
    executed_rules_checkpoint: Int
}
```

### 10.2 Transaction Operations

```
begin(isolation_level) → TxnId:
    1. Allocate transaction ID
    2. Record start timestamp
    3. Initialize empty write/read sets
    4. Return transaction handle

commit(txn) → Result<(), Error>:
    1. txn.state = Committing
    
    2. Acquire locks on write set
       - For each modified entity, acquire exclusive lock
       - If conflict detected: abort
    
    3. Validate read set (serializable only)
       - Check no concurrent modifications to read set
       - Check predicate locks not violated
       - If conflict: abort
    
    4. Check deferred constraints
       - Cardinality constraints
       - If violation: abort
    
    5. Write WAL records
       - One entry per operation
       - Final commit entry
       - sync() to ensure durability
    
    6. Apply changes to storage
       - Update node/edge pages
       - Update indexes
    
    7. Release locks
    
    8. txn.state = Committed
    
    9. Notify waiting transactions

rollback(txn) → void:
    1. txn.state = Aborting
    
    2. Write abort record to WAL
    
    3. Apply undo operations (reverse order)
       - Restore old attribute values
       - Delete created nodes/edges
       - Restore deleted nodes/edges
    
    4. Release locks
    
    5. txn.state = Aborted

savepoint(txn, name) → void:
    1. Record current write set sizes
    2. Store in txn.savepoints[name]

rollback_to(txn, name) → void:
    1. Get savepoint state
    2. Truncate write sets to checkpoint
    3. Truncate executed_rules
    4. Apply undo for truncated operations
```

### 10.3 Concurrency Control

```
Read Committed Implementation:

    read_node(txn, node_id) → Node:
        1. Check txn.created_nodes (return local if exists)
        2. Check txn.deleted_nodes (return null if deleted)
        3. Read committed version from storage
        4. Apply any local modifications
        5. Return node
    
    read_edge(txn, edge_id) → Edge:
        (similar to read_node)

Serializable Implementation:

    read_node(txn, node_id) → Node:
        1. Check local write set
        2. Read committed version from storage
        3. Add node_id to txn.read_nodes
        4. Return node
    
    validate_read_set(txn) → bool:
        For each node_id in txn.read_nodes:
            current_version = storage.get_version(node_id)
            if current_version > txn.start_timestamp:
                return false  // Conflict
        return true

Predicate Locking (for patterns):

    execute_pattern(txn, pattern) → ResultSet:
        1. Record predicate lock for pattern
        2. Execute pattern matching
        3. Add matched entities to read set
        4. Return results
    
    validate_predicate_locks(txn) → bool:
        For each predicate_lock in txn.predicate_locks:
            current_results = execute_pattern_snapshot(predicate_lock.pattern)
            if current_results != predicate_lock.original_results:
                return false  // Phantom detected
        return true
```

---

## 11. Version Manager

### 11.1 Snapshot Structure

```
Snapshot {
    id: SnapshotId
    label: String
    timestamp: Timestamp
    branch: BranchId
    parent_snapshot: SnapshotId?
    
    // Content references (copy-on-write)
    node_store_ref: StoreRef
    edge_store_ref: StoreRef
    index_refs: Map<IndexId, StoreRef>
    
    // Metadata
    node_count: Int
    edge_count: Int
    ontology_hash: Hash
}

StoreRef = 
    | FullCopy(path)                            // Independent copy
    | Delta(base_ref, delta_path)               // Base + changes
    | Shared(snapshot_id)                       // Share with another snapshot
```

### 11.2 Branch Structure

```
Branch {
    id: BranchId
    name: String
    head_snapshot: SnapshotId
    created_from: (BranchId, SnapshotId)
    created_at: Timestamp
}

BranchManager {
    branches: Map<BranchName, Branch>
    current_branch: BranchId
    
    operations:
        create_branch(name, from_snapshot?)
        switch_branch(name)
        delete_branch(name)
        list_branches()
        get_head(branch)
}
```

### 11.3 Snapshot Operations

```
create_snapshot(label) → SnapshotId:
    1. Begin atomic operation
    2. Copy-on-write: mark current pages as snapshot reference
    3. Create snapshot metadata
    4. Store snapshot
    5. Return snapshot ID

checkout(snapshot_id, mode) → void:
    mode = ReadOnly:
        1. Set current view to snapshot
        2. Block all mutations
        3. Queries return snapshot data
    
    mode = Restore:
        1. Verify user confirmation
        2. Create backup snapshot of current state
        3. Apply snapshot as new current state
        4. Update indexes
        5. Clear transaction state

diff(snapshot_a, snapshot_b) → Diff:
    1. Load both snapshot metadata
    2. Compare node stores:
       - Find added nodes (in B not in A)
       - Find removed nodes (in A not in B)
       - Find modified nodes (different version)
    3. Compare edge stores (similar)
    4. Return structured diff

merge(source_branch, target_branch) → MergeResult:
    1. Find common ancestor snapshot
    2. Compute diff: ancestor → source
    3. Compute diff: ancestor → target
    4. Detect conflicts:
       - Same node modified in both
       - Node deleted in one, modified in other
    5. If no conflicts:
       - Apply source changes to target
       - Create merge commit snapshot
    6. If conflicts:
       - Return conflict list for resolution
```

---

# Part IV: Query Processing

## 12. Parser Architecture

### 12.1 Lexer

```
Token Types:

    Keywords: MATCH, WHERE, RETURN, SPAWN, KILL, LINK, UNLINK, SET, ...
    Operators: +, -, *, /, =, !=, <, >, <=, >=, AND, OR, NOT, ++, ...
    Delimiters: (, ), {, }, [, ], ,, :, ., ->, =>, AS
    Literals: String, Int, Float, Bool, Null
    Identifiers: [a-zA-Z_][a-zA-Z0-9_]*
    IdRefs: #identifier, #"quoted-id"
    Comments: -- single line, /* multi-line */

Lexer State Machine:

    START
      │
      ├─ [a-zA-Z_] ──▶ IDENTIFIER ──▶ (check keyword table)
      │
      ├─ [0-9] ──▶ NUMBER ──▶ (check for . → FLOAT)
      │
      ├─ " ──▶ STRING ──▶ (handle escapes, find closing ")
      │
      ├─ # ──▶ ID_REF ──▶ (identifier or quoted)
      │
      ├─ - ──▶ MINUS_OR_COMMENT ──▶ (- → minus, -- → comment)
      │
      ├─ operator chars ──▶ OPERATOR ──▶ (longest match)
      │
      └─ whitespace ──▶ (skip)

Token {
    type: TokenType
    lexeme: String
    line: Int
    column: Int
}
```

### 12.2 Parser (Recursive Descent)

```
Parser Structure:

    parse_statement() → Statement:
        match peek():
            MATCH → parse_match_stmt()
            WALK → parse_walk_stmt()
            INSPECT → parse_inspect_stmt()
            SPAWN → parse_spawn_stmt()
            KILL → parse_kill_stmt()
            LINK → parse_link_stmt()
            UNLINK → parse_unlink_stmt()
            SET → parse_set_stmt()
            BEGIN → parse_begin_stmt()
            COMMIT → parse_commit_stmt()
            ROLLBACK → parse_rollback_stmt()
            EXPLAIN → parse_explain_stmt()
            PROFILE → parse_profile_stmt()
            ...

    parse_match_stmt() → MatchStmt:
        expect(MATCH)
        pattern = parse_pattern()
        optional_matches = []
        while peek() == OPTIONAL:
            expect(OPTIONAL)
            expect(MATCH)
            optional_matches.append(parse_pattern())
        where_clause = parse_optional_where()
        return_clause = parse_return()       // Required
        order_clause = parse_optional_order()
        limit_clause = parse_optional_limit()
        return MatchStmt(pattern, optional_matches, where_clause, 
                         return_clause, order_clause, limit_clause)

    parse_pattern() → Pattern:
        elements = [parse_pattern_element()]
        while peek() == COMMA:
            expect(COMMA)
            elements.append(parse_pattern_element())
        where_clause = parse_optional_where()
        return Pattern(elements, where_clause)

    parse_pattern_element() → PatternElement:
        if peek_ahead() == COLON:  // identifier : type
            return parse_node_pattern()
        else:
            return parse_edge_pattern()

    parse_expr() → Expr:
        return parse_or_expr()
    
    parse_or_expr() → Expr:
        left = parse_and_expr()
        while peek() == OR:
            expect(OR)
            right = parse_and_expr()
            left = BinaryOp(Or, left, right)
        return left
    
    // ... similar for other precedence levels

Error Recovery:
    On syntax error:
        1. Record error with location
        2. Skip to synchronization token (;, }, statement keyword)
        3. Continue parsing for more errors
        4. Return partial AST with errors
```

### 12.3 Ontology Parser

```
Ontology-Specific Productions:

    parse_ontology_file() → OntologyFile:
        ontology_decl = parse_optional_ontology_decl()
        declarations = []
        while not eof():
            declarations.append(parse_declaration())
        return OntologyFile(ontology_decl, declarations)

    parse_declaration() → Declaration:
        doc = parse_optional_doc_comment()
        match peek():
            TYPE → parse_type_alias(doc)
            NODE → parse_node_type(doc)
            EDGE → parse_edge_type(doc)
            CONSTRAINT → parse_constraint(doc)
            RULE → parse_rule(doc)

    parse_node_type(doc) → NodeTypeDecl:
        expect(NODE)
        name = expect(IDENTIFIER)
        parents = parse_optional_inheritance()
        expect(LBRACE)
        attributes = []
        while peek() != RBRACE:
            attributes.append(parse_attribute_decl())
        expect(RBRACE)
        return NodeTypeDecl(name, parents, attributes, doc)

    parse_attribute_modifiers() → AttributeModifiers:
        if peek() != LBRACKET:
            return default_modifiers()
        expect(LBRACKET)
        modifiers = AttributeModifiers()
        loop:
            match peek():
                REQUIRED → modifiers.required = true
                UNIQUE → modifiers.unique = true
                INDEXED → modifiers.indexed = parse_index_order()
                GE → modifiers.min = parse_literal()
                LE → modifiers.max = parse_literal()
                IN → modifiers.enum_values = parse_enum_list()
                MATCH → modifiers.match_pattern = parse_string()
                LENGTH → modifiers.length = parse_range()
                INT_LITERAL → modifiers.range = parse_range()
            if peek() != COMMA:
                break
            expect(COMMA)
        expect(RBRACKET)
        return modifiers
```

---

## 13. Semantic Analyzer

### 13.1 Analysis Phases

```
Semantic Analysis Pipeline:

    AST
     │
     ▼
┌─────────────────┐
│ Name Resolution │  Resolve type/edge/variable references
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Type Inference  │  Infer types for expressions
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Type Checking   │  Verify type compatibility
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Scope Analysis  │  Variable scoping, shadowing detection
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Constraint Check│  Verify pattern/expression well-formedness
└────────┬────────┘
         │
         ▼
    Annotated AST
```

### 13.2 Name Resolution

```
NameResolver {
    type_registry: TypeRegistry
    edge_type_registry: EdgeTypeRegistry
    scope_stack: List<Scope>
    
    resolve(ast) → ResolvedAST:
        walk ast:
            on TypeRef(name):
                resolved = type_registry.lookup(name)
                if not resolved:
                    error("Unknown type: " + name)
                annotate with resolved type_id
            
            on EdgeTypeRef(name):
                resolved = edge_type_registry.lookup(name)
                if not resolved:
                    error("Unknown edge type: " + name)
                annotate with resolved edge_type_id
            
            on VarRef(name):
                resolved = lookup_in_scopes(name)
                if not resolved:
                    error("Unknown variable: " + name)
                annotate with var_id and type
            
            on Pattern:
                push_scope()
                for each node_var:
                    declare_variable(node_var.name, node_var.type)
                for each edge_var:
                    declare_variable(edge_var.name, edge_var.type)
                resolve children
                pop_scope() or keep (depends on context)
}

Scope {
    variables: Map<String, VarInfo>
    parent: Scope?
    
    lookup(name) → VarInfo?:
        if name in variables:
            return variables[name]
        if parent:
            return parent.lookup(name)
        return null
    
    declare(name, type) → VarId:
        if name in variables:
            error("Duplicate variable: " + name)
        var_id = allocate_var_id()
        variables[name] = VarInfo(var_id, type)
        return var_id
}
```

### 13.3 Type Checking

```
TypeChecker {
    check(node) → Type:
        match node:
            Literal(value) → literal_type(value)
            
            VarRef(var_id) → var_id.declared_type
            
            AttrAccess(base, attr_name):
                base_type = check(base)
                attr_def = lookup_attribute(base_type, attr_name)
                if not attr_def:
                    error("Attribute not found: " + attr_name)
                return attr_def.type
            
            BinaryOp(op, left, right):
                left_type = check(left)
                right_type = check(right)
                return check_binary_op(op, left_type, right_type)
            
            FunctionCall(func, args):
                arg_types = [check(arg) for arg in args]
                return check_function(func, arg_types)
            
            Exists(pattern):
                check_pattern(pattern)
                return Bool
            
            ...
    
    check_pattern(pattern):
        for node_var in pattern.node_vars:
            validate_type_exists(node_var.type)
        
        for edge_pattern in pattern.edge_patterns:
            edge_type = lookup_edge_type(edge_pattern.type_name)
            if len(edge_pattern.targets) != edge_type.arity:
                error("Arity mismatch")
            for i, target_var in enumerate(edge_pattern.targets):
                expected_type = edge_type.signature[i].type
                actual_type = lookup_var_type(target_var)
                if not is_subtype(actual_type, expected_type):
                    error("Type mismatch at position " + i)
        
        for condition in pattern.conditions:
            cond_type = check(condition)
            if cond_type != Bool:
                error("Condition must be boolean")

    is_subtype(sub, super) → bool:
        if sub == super:
            return true
        if super is Any:
            return true
        if sub is union:
            return all(is_subtype(member, super) for member in sub.members)
        if super is union:
            return any(is_subtype(sub, member) for member in super.members)
        return type_registry.is_subtype(sub, super)
}
```

---

## 14. Query Planner

### 14.1 Plan Representation

```
QueryPlan = 
    | Scan(type_id, filter?)
    | IndexScan(index_ref, range, filter?)
    | EdgeTraversal(edge_type_id, direction, from_plan)
    | NestedLoop(outer_plan, inner_plan, join_condition)
    | HashJoin(left_plan, right_plan, join_keys)
    | Filter(plan, condition)
    | Project(plan, projections)
    | Sort(plan, sort_keys)
    | Limit(plan, count, offset)
    | Distinct(plan, keys)
    | Aggregate(plan, group_keys, aggregations)
    | Union(plans)
    | TransitiveClosure(edge_type_id, start_plan, direction, depth_limit)

PlanCost {
    estimated_rows: Float
    io_cost: Float
    cpu_cost: Float
    
    total() → Float:
        return io_cost + cpu_cost
}
```

### 14.2 Planning Algorithm

```
Planner {
    plan_match(match_stmt) → QueryPlan:
        // 1. Enumerate join orderings
        orderings = enumerate_join_orders(match_stmt.pattern)
        
        // 2. For each ordering, compute cost
        best_plan = null
        best_cost = infinity
        
        for ordering in orderings:
            plan = build_plan(ordering)
            cost = estimate_cost(plan)
            if cost < best_cost:
                best_plan = plan
                best_cost = cost
        
        // 3. Add projection, sort, limit
        plan = add_projection(best_plan, match_stmt.return_clause)
        plan = add_sort(plan, match_stmt.order_clause)
        plan = add_limit(plan, match_stmt.limit_clause)
        
        return plan

    enumerate_join_orders(pattern) → List<JoinOrder>:
        // For small patterns: enumerate all
        // For large patterns: use heuristics
        
        node_vars = pattern.node_vars
        edge_patterns = pattern.edge_patterns
        
        if len(node_vars) <= 6:
            return all_permutations(node_vars)
        else:
            return heuristic_orderings(node_vars, edge_patterns)

    build_plan(ordering) → QueryPlan:
        plan = null
        
        for var in ordering:
            if plan == null:
                // First variable: choose scan method
                plan = choose_initial_scan(var)
            else:
                // Subsequent: choose join method
                plan = choose_join(plan, var)
        
        // Add conditions not yet applied
        for condition in remaining_conditions:
            plan = Filter(plan, condition)
        
        return plan

    choose_initial_scan(var) → QueryPlan:
        type_id = var.type_id
        
        // Check for equality conditions we can use
        for condition in var.conditions:
            if condition is Equality(attr, literal):
                if has_index(type_id, attr):
                    return IndexScan(index, value=literal)
        
        // Check for range conditions
        for condition in var.conditions:
            if condition is Range(attr, min, max):
                if has_index(type_id, attr):
                    return IndexScan(index, range=(min, max))
        
        // Fall back to type scan
        return Scan(type_id)

    estimate_cost(plan) → PlanCost:
        match plan:
            Scan(type_id):
                rows = statistics.type_count(type_id)
                io_cost = rows / PAGE_SIZE
                cpu_cost = rows * FILTER_COST
                return PlanCost(rows, io_cost, cpu_cost)
            
            IndexScan(index, range):
                selectivity = estimate_selectivity(index, range)
                rows = statistics.type_count(index.type_id) * selectivity
                io_cost = log(rows) + rows / PAGE_SIZE
                cpu_cost = rows * INDEX_LOOKUP_COST
                return PlanCost(rows, io_cost, cpu_cost)
            
            NestedLoop(outer, inner, _):
                outer_cost = estimate_cost(outer)
                inner_cost = estimate_cost(inner)
                rows = outer_cost.rows * inner_cost.rows * JOIN_SELECTIVITY
                io_cost = outer_cost.io_cost + outer_cost.rows * inner_cost.io_cost
                cpu_cost = outer_cost.cpu_cost + outer_cost.rows * inner_cost.cpu_cost
                return PlanCost(rows, io_cost, cpu_cost)
            
            // ... other plan types
}
```

### 14.3 Index Selection

```
IndexSelector {
    select_index(type_id, conditions) → IndexChoice?:
        candidates = []
        
        for condition in conditions:
            if condition is Equality(attr, _):
                if has_unique_index(type_id, attr):
                    // Unique index is best for equality
                    return UniqueIndexChoice(attr)
                if has_attribute_index(type_id, attr):
                    candidates.append(AttributeIndexChoice(attr))
            
            if condition is Range(attr, _, _):
                if has_attribute_index(type_id, attr):
                    candidates.append(RangeIndexChoice(attr))
        
        if empty(candidates):
            return null
        
        // Choose index with best selectivity
        return min(candidates, key=estimate_selectivity)

    estimate_selectivity(index_choice, condition) → Float:
        match condition:
            Equality(_, value):
                // Use histogram if available
                if has_histogram(index_choice.attr):
                    return histogram_selectivity(value)
                // Default: assume uniform distribution
                return 1.0 / statistics.distinct_values(index_choice.attr)
            
            Range(_, min, max):
                // Estimate fraction of range covered
                attr_min, attr_max = statistics.value_range(index_choice.attr)
                range_fraction = (max - min) / (attr_max - attr_min)
                return clamp(range_fraction, 0.0, 1.0)
}
```

---

## 15. Query Executor

### 15.1 Execution Model

```
Executor uses iterator (Volcano) model:

    Operator {
        open() → void
        next() → Row?
        close() → void
    }

Each plan node becomes an operator:

    ScanOperator(type_id, filter) {
        cursor: TypeIndex.Cursor
        
        open():
            cursor = type_index.scan(type_id)
        
        next() → Row?:
            while cursor.has_next():
                node = cursor.next()
                if filter == null or evaluate(filter, node):
                    return Row(node)
            return null
        
        close():
            cursor.close()
    }

    IndexScanOperator(index, range, filter) {
        cursor: Index.Cursor
        
        open():
            cursor = index.range_scan(range)
        
        next() → Row?:
            while cursor.has_next():
                entity = cursor.next()
                if filter == null or evaluate(filter, entity):
                    return Row(entity)
            return null
        
        close():
            cursor.close()
    }

    NestedLoopJoinOperator(outer, inner, condition) {
        outer_row: Row?
        
        open():
            outer.open()
            outer_row = null
        
        next() → Row?:
            loop:
                // Get next inner row for current outer
                if outer_row != null:
                    inner_row = inner.next()
                    if inner_row != null:
                        combined = merge(outer_row, inner_row)
                        if evaluate(condition, combined):
                            return combined
                        continue
                
                // Move to next outer row
                outer_row = outer.next()
                if outer_row == null:
                    return null
                inner.close()
                inner.open()  // Reset inner
        
        close():
            outer.close()
            inner.close()
    }
```

### 15.2 Pattern Matching

```
PatternMatcher {
    match(pattern, bindings) → Iterator<Bindings>:
        // Build execution plan from pattern
        plan = build_pattern_plan(pattern, bindings)
        
        // Execute and yield bindings
        for row in execute(plan):
            yield extract_bindings(row, pattern)

    build_pattern_plan(pattern, initial_bindings) → QueryPlan:
        // Order variables by selectivity
        ordered_vars = order_by_selectivity(pattern.node_vars)
        
        plan = null
        bound_vars = set(initial_bindings.keys())
        
        for var in ordered_vars:
            if var.name in bound_vars:
                // Variable already bound: point lookup
                if plan == null:
                    plan = PointLookup(var)
                else:
                    plan = NestedLoop(plan, PointLookup(var), true_condition)
            else:
                // Unbound: scan or edge traversal
                connected_edges = find_edges_connecting(var, bound_vars)
                
                if empty(connected_edges):
                    // No connection: must scan
                    var_plan = choose_scan(var)
                else:
                    // Use edge to traverse
                    edge = choose_best_edge(connected_edges)
                    var_plan = EdgeTraversal(edge, from bound var)
                
                if plan == null:
                    plan = var_plan
                else:
                    plan = NestedLoop(plan, var_plan, edge_condition)
                
                bound_vars.add(var.name)
        
        // Add remaining conditions
        for condition in pattern.conditions:
            plan = Filter(plan, condition)
        
        return plan

    order_by_selectivity(vars) → List<Var>:
        // Prefer:
        // 1. Already-bound variables
        // 2. Variables with unique index conditions
        // 3. Variables with index conditions
        // 4. Variables with fewer instances
        // 5. Variables connected to already-ordered vars
        
        scored = []
        for var in vars:
            score = compute_selectivity_score(var)
            scored.append((score, var))
        
        return [var for (_, var) in sorted(scored)]
}
```

### 15.3 Transitive Closure

```
TransitiveClosureExecutor {
    execute(edge_type, start_nodes, direction, min_depth, max_depth) 
        → Iterator<(StartNode, EndNode, Depth)>:
        
        visited = Map<StartNode, Set<EndNode>>()
        
        for start in start_nodes:
            visited[start] = Set()
            
            // BFS with depth tracking
            frontier = [(start, 0)]  // (node, depth)
            
            while not empty(frontier):
                current, depth = frontier.pop_front()
                
                if depth > max_depth:
                    continue
                
                if current in visited[start]:
                    continue  // Cycle detection
                
                visited[start].add(current)
                
                if depth >= min_depth:
                    yield (start, current, depth)
                
                // Expand frontier
                neighbors = get_neighbors(current, edge_type, direction)
                for neighbor in neighbors:
                    if neighbor not in visited[start]:
                        frontier.append((neighbor, depth + 1))

    get_neighbors(node, edge_type, direction) → List<Node>:
        match direction:
            Outbound:
                edges = edge_index.lookup(edge_type, position=0, target=node)
                return [edge.targets[1] for edge in edges]
            Inbound:
                edges = edge_index.lookup(edge_type, position=1, target=node)
                return [edge.targets[0] for edge in edges]
            Any:
                return get_neighbors(node, Outbound) + get_neighbors(node, Inbound)
}
```

---

# Part V: Constraint and Rule Engine

## 16. Constraint Checker

### 16.1 Architecture

```
ConstraintChecker {
    registry: ConstraintRegistry
    pattern_matcher: PatternMatcher
    
    check_all(mutation) → List<ConstraintViolation>:
        violations = []
        
        // Get affected constraints
        affected = registry.get_affected(mutation)
        
        for constraint in affected:
            result = check_constraint(constraint, mutation)
            if result.violated:
                violations.append(result)
        
        return violations

    check_constraint(constraint, mutation) → CheckResult:
        // Find matches of the pattern that involve mutated entities
        relevant_bindings = find_relevant_matches(constraint.pattern, mutation)
        
        for bindings in relevant_bindings:
            condition_result = evaluate(constraint.condition, bindings)
            
            if not condition_result:
                return CheckResult(
                    violated = true,
                    constraint = constraint,
                    bindings = bindings,
                    message = format_message(constraint, bindings)
                )
        
        return CheckResult(violated = false)

    find_relevant_matches(pattern, mutation) → Iterator<Bindings>:
        // Optimization: Only check patterns involving mutated entities
        
        mutated_ids = get_mutated_entity_ids(mutation)
        
        // For each pattern variable, check if mutation could affect it
        for var in pattern.node_vars:
            if mutation_affects_type(mutation, var.type):
                // Seed pattern matching with mutated entity
                for id in mutated_ids:
                    if entity_matches_type(id, var.type):
                        initial = {var.name: id}
                        for match in pattern_matcher.match(pattern, initial):
                            yield match
}
```

### 16.2 Incremental Checking

```
IncrementalConstraintChecker {
    // Maintain index of constraint → relevant entities
    constraint_entity_index: Map<ConstraintId, Set<EntityId>>
    
    on_mutation(mutation):
        // Update index based on mutation
        match mutation:
            SpawnNode(node):
                for constraint in get_constraints_for_type(node.type):
                    if matches_pattern(constraint.pattern, node):
                        constraint_entity_index[constraint.id].add(node.id)
            
            KillNode(node_id):
                for constraint_id in find_constraints_involving(node_id):
                    constraint_entity_index[constraint_id].remove(node_id)
            
            // Similar for edges and attribute changes

    check_incremental(mutation) → List<ConstraintViolation>:
        // Only check constraints that could be affected
        
        affected_constraints = Set()
        
        match mutation:
            SpawnNode(node):
                // New node might violate constraints
                affected_constraints.update(get_constraints_for_type(node.type))
            
            SetAttribute(entity_id, attr, _):
                // Changed attribute might violate constraints mentioning it
                affected_constraints.update(get_constraints_for_attribute(attr))
            
            LinkEdge(edge):
                // New edge might complete patterns
                affected_constraints.update(get_constraints_for_edge_type(edge.type))
        
        violations = []
        for constraint in affected_constraints:
            // Check only bindings involving mutated entities
            for bindings in get_affected_bindings(constraint, mutation):
                if not evaluate(constraint.condition, bindings):
                    violations.append(create_violation(constraint, bindings))
        
        return violations
}
```

### 16.3 Constraint Categories

```
ConstraintCategory = 
    | Immediate      // Check immediately on mutation
    | Deferred       // Check at commit time

categorize_constraint(constraint) → ConstraintCategory:
    pattern = constraint.pattern
    
    // Cardinality constraints are deferred
    if is_cardinality_constraint(constraint):
        return Deferred
    
    // Existence constraints (=> EXISTS(...)) are deferred
    if condition_requires_existence(constraint.condition):
        return Deferred
    
    // Everything else is immediate
    return Immediate

Execution order:

    1. Apply mutation
    2. Check Immediate constraints → fail fast if violated
    3. Execute triggered rules
    4. At commit: Check Deferred constraints → fail if violated
```

---

## 17. Rule Engine

### 17.1 Architecture

```
RuleEngine {
    registry: RuleRegistry
    pattern_matcher: PatternMatcher
    executor: MutationExecutor
    
    // Execution limits
    max_depth: Int = 100
    max_actions: Int = 10000
    
    // Execution state
    executed: Set<(RuleId, BindingHash)>
    current_depth: Int
    action_count: Int
    
    process_mutation(mutation, txn) → Result<(), Error>:
        executed = Set()
        current_depth = 0
        action_count = 0
        
        // Process until quiescence
        loop:
            triggered_rules = find_triggered_rules(mutation)
            
            if empty(triggered_rules):
                break  // Quiescent
            
            // Execute in priority order
            sorted_rules = sort_by_priority(triggered_rules)
            
            for (rule, bindings) in sorted_rules:
                binding_hash = hash(bindings)
                
                // Check if already executed
                if (rule.id, binding_hash) in executed:
                    continue
                
                // Check limits
                if current_depth >= max_depth:
                    return Error("Rule depth limit exceeded")
                
                // Execute rule
                result = execute_rule(rule, bindings, txn)
                
                if result.is_error:
                    return result
                
                executed.add((rule.id, binding_hash))
        
        return Ok(())

    find_triggered_rules(mutation) → List<(RuleDef, Bindings)>:
        triggered = []
        
        auto_rules = registry.get_auto_rules_for_mutation(mutation)
        
        for rule in auto_rules:
            matches = pattern_matcher.match(rule.pattern)
            for bindings in matches:
                triggered.append((rule, bindings))
        
        return triggered

    execute_rule(rule, bindings, txn) → Result<(), Error>:
        current_depth += 1
        defer: current_depth -= 1
        
        // Execute each action in the production
        for action in rule.production.actions:
            if action_count >= max_actions:
                return Error("Rule action limit exceeded")
            
            result = execute_action(action, bindings, txn)
            
            if result.is_error:
                return result
            
            action_count += 1
            
            // Update bindings with spawned entities
            if action is SpawnAction:
                bindings[action.var_name] = result.entity_id
        
        return Ok(())

    execute_action(action, bindings, txn) → Result<EntityId?, Error>:
        match action:
            SpawnAction(var_name, type_id, attributes):
                attr_values = {}
                for (attr_id, expr) in attributes:
                    value = evaluate(expr, bindings)
                    attr_values[attr_id] = value
                return executor.spawn(type_id, attr_values, txn)
            
            KillAction(target_var):
                entity_id = bindings[target_var]
                return executor.kill(entity_id, txn)
            
            LinkAction(edge_type_id, target_vars, alias, attributes):
                targets = [bindings[var] for var in target_vars]
                attr_values = {}
                for (attr_id, expr) in attributes:
                    value = evaluate(expr, bindings)
                    attr_values[attr_id] = value
                result = executor.link(edge_type_id, targets, attr_values, txn)
                if alias:
                    bindings[alias] = result.edge_id
                return result
            
            UnlinkAction(target_var):
                edge_id = bindings[target_var]
                return executor.unlink(edge_id, txn)
            
            SetAction(target_var, attr_id, value_expr):
                entity_id = bindings[target_var]
                value = evaluate(value_expr, bindings)
                return executor.set_attribute(entity_id, attr_id, value, txn)
}
```

### 17.2 Trigger Detection

```
TriggerDetector {
    rule_patterns: Map<RuleId, CompiledPattern>
    type_to_rules: Map<TypeId, Set<RuleId>>
    edge_type_to_rules: Map<EdgeTypeId, Set<RuleId>>
    
    build_indexes():
        for (rule_id, pattern) in rule_patterns:
            for node_var in pattern.node_vars:
                type_to_rules[node_var.type_id].add(rule_id)
            for edge_pattern in pattern.edge_patterns:
                edge_type_to_rules[edge_pattern.edge_type_id].add(rule_id)

    find_potentially_triggered(mutation) → Set<RuleId>:
        match mutation:
            SpawnNode(type_id, _):
                return type_to_rules[type_id]
                       ∪ type_to_rules[parent_types(type_id)]
            
            KillNode(node_id):
                type_id = get_type(node_id)
                return type_to_rules[type_id]
                       ∪ type_to_rules[parent_types(type_id)]
            
            LinkEdge(edge_type_id, _, _):
                return edge_type_to_rules[edge_type_id]
                       ∪ type_to_rules[affected_node_types]
            
            UnlinkEdge(edge_id):
                edge = get_edge(edge_id)
                return edge_type_to_rules[edge.type_id]
            
            SetAttribute(entity_id, attr_id, _):
                type_id = get_type(entity_id)
                // Rules that reference this attribute
                return type_to_rules[type_id]
                       ∩ rules_mentioning_attribute(attr_id)
}
```

### 17.3 Cycle Detection

```
CycleDetector {
    execution_stack: List<(RuleId, BindingHash)>
    
    push(rule_id, binding_hash) → Result<(), CycleError>:
        entry = (rule_id, binding_hash)
        
        if entry in execution_stack:
            // Cycle detected
            cycle_path = extract_cycle(entry)
            return Error(CycleError(cycle_path))
        
        execution_stack.push(entry)
        return Ok(())
    
    pop():
        execution_stack.pop()
    
    extract_cycle(repeated_entry) → List<(RuleId, BindingHash)>:
        start_index = execution_stack.index(repeated_entry)
        return execution_stack[start_index:]
}
```

---

# Part VI: Compiler

## 18. Ontology Compiler

### 18.1 Compilation Pipeline

```
OntologyCompiler {
    compile(source: String) → Result<CompiledOntology, Errors>:
        // Phase 1: Parse
        tokens = lexer.tokenize(source)
        ast = parser.parse(tokens)
        if ast.has_errors:
            return Errors(ast.errors)
        
        // Phase 2: Resolve names
        resolved_ast = name_resolver.resolve(ast)
        if resolved_ast.has_errors:
            return Errors(resolved_ast.errors)
        
        // Phase 3: Type check
        typed_ast = type_checker.check(resolved_ast)
        if typed_ast.has_errors:
            return Errors(typed_ast.errors)
        
        // Phase 4: Expand sugar
        expanded_ast = sugar_expander.expand(typed_ast)
        
        // Phase 5: Validate
        validation_errors = validator.validate(expanded_ast)
        if not empty(validation_errors):
            return Errors(validation_errors)
        
        // Phase 6: Generate Layer 0 structure
        l0_structure = l0_generator.generate(expanded_ast)
        
        // Phase 7: Build registries
        registries = registry_builder.build(expanded_ast, l0_structure)
        
        // Phase 8: Create indexes
        indexes = index_builder.build(expanded_ast)
        
        return Ok(CompiledOntology(l0_structure, registries, indexes))
}
```

### 18.2 Sugar Expansion

```
SugarExpander {
    expand(ast) → ExpandedAST:
        expanded = clone(ast)
        
        // Expand type aliases (inline)
        expanded = expand_type_aliases(expanded)
        
        // Expand attribute modifiers to constraints
        for node_type in expanded.node_types:
            for attr in node_type.attributes:
                constraints = expand_attribute_modifiers(node_type, attr)
                expanded.constraints.extend(constraints)
        
        // Expand edge modifiers to constraints/rules
        for edge_type in expanded.edge_types:
            constraints, rules = expand_edge_modifiers(edge_type)
            expanded.constraints.extend(constraints)
            expanded.rules.extend(rules)
        
        return expanded

    expand_type_aliases(ast) → AST:
        // Build alias map
        aliases = {}
        for alias_decl in ast.type_aliases:
            aliases[alias_decl.name] = alias_decl
        
        // Expand all type references
        return ast.transform(node =>
            if node is TypeRef and node.name in aliases:
                return expand_alias(aliases[node.name])
            return node
        )

    expand_attribute_modifiers(node_type, attr) → List<Constraint>:
        constraints = []
        mods = attr.modifiers
        
        if mods.required:
            constraints.append(Constraint(
                name = "{node_type.name}_{attr.name}_required",
                pattern = Pattern([NodeVar("x", node_type.name)]),
                condition = NotNull(AttrAccess(VarRef("x"), attr.name))
            ))
        
        if mods.unique:
            constraints.append(Constraint(
                name = "{node_type.name}_{attr.name}_unique",
                pattern = Pattern([
                    NodeVar("x1", node_type.name),
                    NodeVar("x2", node_type.name)
                ], where = NotEqual(VarRef("x1"), VarRef("x2"))),
                condition = Or(
                    IsNull(AttrAccess(VarRef("x1"), attr.name)),
                    NotEqual(
                        AttrAccess(VarRef("x1"), attr.name),
                        AttrAccess(VarRef("x2"), attr.name)
                    )
                )
            ))
        
        if mods.min != null:
            constraints.append(Constraint(
                name = "{node_type.name}_{attr.name}_min",
                pattern = Pattern([NodeVar("x", node_type.name)],
                    where = NotNull(AttrAccess(VarRef("x"), attr.name))),
                condition = Gte(AttrAccess(VarRef("x"), attr.name), Literal(mods.min))
            ))
        
        // Similar for max, enum, match, length...
        
        return constraints

    expand_edge_modifiers(edge_type) → (List<Constraint>, List<Rule>):
        constraints = []
        rules = []
        mods = edge_type.modifiers
        
        if mods.no_self:
            // For binary edges
            constraints.append(Constraint(
                name = "{edge_type.name}_no_self",
                pattern = Pattern([
                    NodeVar("x", edge_type.signature[0].type),
                    EdgePattern(edge_type.name, ["x", "x"])
                ]),
                condition = Literal(false)
            ))
        
        if mods.acyclic:
            constraints.append(Constraint(
                name = "{edge_type.name}_acyclic",
                pattern = Pattern([
                    NodeVar("x", edge_type.signature[0].type),
                    EdgePattern(edge_type.name, ["x", "x"], transitive=Plus)
                ]),
                condition = Literal(false)
            ))
        
        for cardinality in mods.cardinalities:
            // Generate cardinality constraints
            constraints.extend(expand_cardinality(edge_type, cardinality))
        
        if mods.on_kill_source == Cascade:
            rules.append(Rule(
                name = "{edge_type.name}_cascade_source",
                priority = 1000,
                pattern = Pattern([
                    NodeVar("source", edge_type.signature[0].type),
                    NodeVar("target", edge_type.signature[1].type),
                    EdgePattern(edge_type.name, ["source", "target"]),
                    EdgePattern("_pending_kill", ["source"])
                ]),
                production = [KillAction("target")]
            ))
        
        // Similar for other modifiers...
        
        return (constraints, rules)
}
```

### 18.3 Layer 0 Generation

```
L0Generator {
    generate(ast) → L0Structure:
        l0 = L0Structure()
        
        // Generate ontology node
        ontology_node = spawn_node("_Ontology", {
            name = ast.ontology_name,
            version = ast.version
        })
        l0.add(ontology_node)
        
        // Generate node types
        for node_type in ast.node_types:
            type_node = generate_node_type(node_type, ontology_node)
            l0.add(type_node)
        
        // Generate edge types
        for edge_type in ast.edge_types:
            type_node = generate_edge_type(edge_type, ontology_node)
            l0.add(type_node)
        
        // Generate constraints
        for constraint in ast.constraints:
            constraint_structure = generate_constraint(constraint, ontology_node)
            l0.add(constraint_structure)
        
        // Generate rules
        for rule in ast.rules:
            rule_structure = generate_rule(rule, ontology_node)
            l0.add(rule_structure)
        
        return l0

    generate_node_type(node_type, ontology) → L0Nodes:
        nodes = []
        
        // Create _NodeType node
        type_node = spawn_node("_NodeType", {
            name = node_type.name,
            abstract = node_type.abstract,
            sealed = node_type.sealed,
            doc = node_type.doc
        })
        nodes.append(type_node)
        
        // Link to ontology
        nodes.append(link_edge("_ontology_declares_type", [ontology.id, type_node.id]))
        
        // Generate inheritance edges
        for parent in node_type.parents:
            parent_node = lookup_type_node(parent)
            nodes.append(link_edge("_type_inherits", [type_node.id, parent_node.id]))
        
        // Generate attributes
        for attr in node_type.attributes:
            attr_structure = generate_attribute(attr, type_node)
            nodes.extend(attr_structure)
        
        return nodes

    generate_attribute(attr, owner) → L0Nodes:
        nodes = []
        
        // Create _AttributeDef node
        attr_node = spawn_node("_AttributeDef", {
            name = attr.name,
            required = attr.modifiers.required,
            unique = attr.modifiers.unique,
            indexed = attr.modifiers.indexed,
            default_value = serialize_default(attr.default_value),
            doc = attr.doc
        })
        nodes.append(attr_node)
        
        // Link to owner
        nodes.append(link_edge("_type_has_attribute", [owner.id, attr_node.id]))
        
        // Generate type expression
        if is_scalar_type(attr.type):
            nodes.append(link_edge("_attr_has_scalar_type", [attr_node.id], {
                scalar_type = attr.type.name
            }))
        else:
            type_expr_nodes = generate_type_expr(attr.type)
            nodes.extend(type_expr_nodes)
            nodes.append(link_edge("_attr_has_type", 
                [attr_node.id, type_expr_nodes[0].id]))
        
        return nodes

    generate_constraint(constraint, ontology) → L0Nodes:
        nodes = []
        
        // Create _ConstraintDef node
        constraint_node = spawn_node("_ConstraintDef", {
            name = constraint.name,
            hard = constraint.hard,
            message = constraint.message,
            doc = constraint.doc
        })
        nodes.append(constraint_node)
        
        // Link to ontology
        nodes.append(link_edge("_ontology_declares_constraint", 
            [ontology.id, constraint_node.id]))
        
        // Generate pattern
        pattern_structure = generate_pattern(constraint.pattern)
        nodes.extend(pattern_structure)
        nodes.append(link_edge("_constraint_has_pattern", 
            [constraint_node.id, pattern_structure[0].id]))
        
        // Generate condition
        condition_structure = generate_expr(constraint.condition)
        nodes.extend(condition_structure)
        nodes.append(link_edge("_constraint_has_condition", 
            [constraint_node.id, condition_structure[0].id]))
        
        return nodes

    generate_pattern(pattern) → L0Nodes:
        nodes = []
        
        // Create _PatternDef node
        pattern_node = spawn_node("_PatternDef", {})
        nodes.append(pattern_node)
        
        // Generate node variables
        for node_var in pattern.node_vars:
            var_node = spawn_node("_VarDef", {
                name = node_var.name,
                is_edge_var = false
            })
            nodes.append(var_node)
            nodes.append(link_edge("_pattern_has_node_var", 
                [pattern_node.id, var_node.id]))
            
            // Generate type constraint
            type_expr = generate_type_expr(node_var.type)
            nodes.extend(type_expr)
            nodes.append(link_edge("_var_has_type", [var_node.id, type_expr[0].id]))
        
        // Generate edge patterns
        for edge_pattern in pattern.edge_patterns:
            ep_structure = generate_edge_pattern(edge_pattern)
            nodes.extend(ep_structure)
            nodes.append(link_edge("_pattern_has_edge_pattern", 
                [pattern_node.id, ep_structure[0].id]))
        
        // Generate conditions
        for condition in pattern.conditions:
            cond_structure = generate_expr(condition)
            nodes.extend(cond_structure)
            nodes.append(link_edge("_pattern_has_condition", 
                [pattern_node.id, cond_structure[0].id]))
        
        return nodes
}
```

---

## 19. Registry Builder

```
RegistryBuilder {
    build(ast, l0_structure) → Registries:
        type_registry = build_type_registry(ast.node_types, l0_structure)
        edge_registry = build_edge_registry(ast.edge_types, l0_structure)
        constraint_registry = build_constraint_registry(ast.constraints, l0_structure)
        rule_registry = build_rule_registry(ast.rules, l0_structure)
        
        return Registries(type_registry, edge_registry, constraint_registry, rule_registry)

    build_type_registry(node_types, l0) → TypeRegistry:
        registry = TypeRegistry()
        
        // First pass: create all TypeDefs
        for node_type in node_types:
            l0_node = l0.find_node("_NodeType", name = node_type.name)
            
            type_def = TypeDef(
                id = allocate_type_id(),
                name = node_type.name,
                parent_ids = [],  // filled in second pass
                own_attributes = [],
                all_attributes = [],  // filled in third pass
                abstract = node_type.abstract,
                sealed = node_type.sealed,
                layer0_node_id = l0_node.id
            )
            
            registry.register(type_def)
        
        // Second pass: resolve parent references
        for node_type in node_types:
            type_def = registry.lookup(node_type.name)
            for parent_name in node_type.parents:
                parent_def = registry.lookup(parent_name)
                type_def.parent_ids.append(parent_def.id)
        
        // Third pass: build inheritance graph and collect all attributes
        for type_def in registry.all_types():
            type_def.all_attributes = collect_all_attributes(type_def, registry)
        
        // Build subtype index for fast subtype checks
        registry.build_subtype_index()
        
        return registry

    build_edge_registry(edge_types, l0) → EdgeTypeRegistry:
        registry = EdgeTypeRegistry()
        
        for edge_type in edge_types:
            l0_node = l0.find_node("_EdgeType", name = edge_type.name)
            
            signature = []
            for i, param in enumerate(edge_type.signature):
                signature.append(SignatureParam(
                    position = i,
                    name = param.name,
                    type = compile_type_expr(param.type),
                    is_edge_ref = is_edge_ref_type(param.type)
                ))
            
            edge_def = EdgeTypeDef(
                id = allocate_edge_type_id(),
                name = edge_type.name,
                arity = len(signature),
                signature = signature,
                attributes = compile_attributes(edge_type.attributes),
                symmetric = edge_type.modifiers.symmetric,
                reflexive_allowed = not edge_type.modifiers.no_self,
                layer0_node_id = l0_node.id
            )
            
            registry.register(edge_def)
        
        return registry

    build_constraint_registry(constraints, l0) → ConstraintRegistry:
        registry = ConstraintRegistry()
        
        for constraint in constraints:
            l0_node = l0.find_node("_ConstraintDef", name = constraint.name)
            
            compiled_pattern = compile_pattern(constraint.pattern)
            compiled_condition = compile_expr(constraint.condition)
            
            affected_types = extract_affected_types(compiled_pattern)
            affected_edge_types = extract_affected_edge_types(compiled_pattern)
            
            constraint_def = ConstraintDef(
                id = allocate_constraint_id(),
                name = constraint.name,
                hard = constraint.hard,
                message = constraint.message,
                pattern = compiled_pattern,
                condition = compiled_condition,
                affected_types = affected_types,
                affected_edge_types = affected_edge_types,
                layer0_node_id = l0_node.id
            )
            
            registry.register(constraint_def)
        
        // Build type → constraints index
        registry.build_type_index()
        
        return registry

    build_rule_registry(rules, l0) → RuleRegistry:
        registry = RuleRegistry()
        
        for rule in rules:
            l0_node = l0.find_node("_RuleDef", name = rule.name)
            
            compiled_pattern = compile_pattern(rule.pattern)
            compiled_production = compile_production(rule.production)
            
            rule_def = RuleDef(
                id = allocate_rule_id(),
                name = rule.name,
                priority = rule.priority,
                auto = rule.auto,
                pattern = compiled_pattern,
                production = compiled_production,
                affected_types = extract_affected_types(compiled_pattern),
                affected_edge_types = extract_affected_edge_types(compiled_pattern),
                layer0_node_id = l0_node.id
            )
            
            registry.register(rule_def)
        
        // Sort auto rules by priority
        registry.sort_auto_rules()
        
        // Build type → rules index
        registry.build_type_index()
        
        return registry
}
```

---

# Part VII: API and Integration

## 20. Statement Executor

```
StatementExecutor {
    query_processor: QueryProcessor
    transaction_manager: TransactionManager
    core_engine: CoreEngine
    
    execute(statement, session) → Result:
        match statement.kind:
            // Queries
            "Match" → execute_match(statement, session)
            "Walk" → execute_walk(statement, session)
            "Inspect" → execute_inspect(statement, session)
            
            // Mutations
            "Spawn" → execute_spawn(statement, session)
            "Kill" → execute_kill(statement, session)
            "Link" → execute_link(statement, session)
            "Unlink" → execute_unlink(statement, session)
            "Set" → execute_set(statement, session)
            
            // Transactions
            "Begin" → execute_begin(statement, session)
            "Commit" → execute_commit(session)
            "Rollback" → execute_rollback(session)
            
            // Admin
            "Load" → execute_load(statement, session)
            "Extend" → execute_extend(statement, session)
            "Show" → execute_show(statement, session)
            "CreateIndex" → execute_create_index(statement, session)
            "DropIndex" → execute_drop_index(statement, session)
            
            // Versioning
            "Snapshot" → execute_snapshot(statement, session)
            "Checkout" → execute_checkout(statement, session)
            "Diff" → execute_diff(statement, session)
            "CreateBranch" → execute_create_branch(statement, session)
            "Merge" → execute_merge(statement, session)
            
            // Debug
            "Explain" → execute_explain(statement, session)
            "Profile" → execute_profile(statement, session)

    execute_match(stmt, session) → MatchResult:
        // Check timeout
        timeout = stmt.timeout ?? session.default_timeout
        deadline = now() + timeout
        
        // Parse and plan
        plan = query_processor.plan_match(stmt)
        
        // Execute with timeout
        results = []
        for row in query_processor.execute(plan, session.transaction):
            if now() > deadline:
                return Error("Query timeout")
            results.append(row)
            if len(results) > session.max_results:
                return Warning("Result limit reached", results)
        
        return MatchResult(results, stats)

    execute_spawn(stmt, session) → SpawnResult:
        // Get or create transaction
        txn = session.transaction ?? begin_auto_commit_txn()
        
        try:
            // Type check
            type_def = lookup_type(stmt.type)
            if type_def.abstract:
                return Error("Cannot instantiate abstract type")
            
            // Prepare attributes
            attrs = {}
            for (name, expr) in stmt.attributes:
                attr_def = type_def.get_attribute(name)
                if not attr_def:
                    return Error("Unknown attribute: " + name)
                
                value = evaluate(expr, {})
                if not type_compatible(value, attr_def.type):
                    return Error("Type mismatch for " + name)
                
                attrs[attr_def.id] = value
            
            // Apply defaults
            for attr_def in type_def.all_attributes:
                if attr_def.id not in attrs:
                    if attr_def.default_value:
                        attrs[attr_def.id] = evaluate_default(attr_def.default_value)
                    elif attr_def.required:
                        return Error("Required attribute missing: " + attr_def.name)
            
            // Create node
            node_id = core_engine.spawn_node(type_def.id, attrs, txn)
            
            // Check constraints
            violations = core_engine.check_constraints(SpawnMutation(node_id), txn)
            if not empty(violations):
                return Error(format_violations(violations))
            
            // Execute triggered rules
            rule_result = core_engine.execute_rules(SpawnMutation(node_id), txn)
            if rule_result.is_error:
                return rule_result
            
            // Auto-commit if not in explicit transaction
            if session.auto_commit:
                commit(txn)
            
            return SpawnResult(success=true, id=node_id)
        
        catch error:
            if session.auto_commit:
                rollback(txn)
            return Error(error)
}
```

---

## 21. Session Management

```
Session {
    id: SessionId
    user: UserId?
    transaction: Transaction?
    auto_commit: bool = true
    default_timeout: Duration
    max_results: Int
    current_branch: BranchId
    readonly_snapshot: SnapshotId?
    
    // Statement history (for REPL)
    history: List<Statement>
    
    // Prepared statements
    prepared: Map<String, PreparedStatement>
}

SessionManager {
    sessions: Map<SessionId, Session>
    
    create_session(config) → Session:
        session = Session(
            id = generate_session_id(),
            auto_commit = config.auto_commit ?? true,
            default_timeout = config.timeout ?? 30.seconds,
            max_results = config.max_results ?? 10000,
            current_branch = "main"
        )
        sessions[session.id] = session
        return session
    
    close_session(session_id):
        session = sessions[session_id]
        if session.transaction:
            rollback(session.transaction)
        sessions.remove(session_id)
    
    begin_transaction(session, isolation) → Transaction:
        if session.transaction:
            return Error("Transaction already active")
        if session.readonly_snapshot:
            return Error("Cannot begin transaction in readonly mode")
        
        txn = transaction_manager.begin(isolation)
        session.transaction = txn
        session.auto_commit = false
        return txn
    
    commit(session) → Result:
        if not session.transaction:
            return Error("No active transaction")
        
        result = transaction_manager.commit(session.transaction)
        session.transaction = null
        session.auto_commit = true
        return result
    
    rollback(session) → Result:
        if not session.transaction:
            return Error("No active transaction")
        
        result = transaction_manager.rollback(session.transaction)
        session.transaction = null
        session.auto_commit = true
        return result
}
```

---

## 22. Result Streaming

```
ResultStream {
    // For large result sets, stream results instead of buffering
    
    plan: QueryPlan
    executor: Executor
    state: StreamState
    
    enum StreamState = NotStarted | Streaming | Exhausted | Error
    
    open():
        executor.open(plan)
        state = Streaming
    
    next() → Row?:
        if state != Streaming:
            return null
        
        row = executor.next()
        if row == null:
            state = Exhausted
        return row
    
    close():
        executor.close()
        state = Exhausted
    
    // Collect all (with limit)
    collect(limit) → List<Row>:
        open()
        results = []
        while len(results) < limit:
            row = next()
            if row == null:
                break
            results.append(row)
        close()
        return results
    
    // Async iteration support
    async_next() → Future<Row?>:
        return async { next() }
}

ResultEncoder {
    encode(result, format) → Bytes:
        match format:
            JSON → encode_json(result)
            MessagePack → encode_msgpack(result)
            CSV → encode_csv(result)
            Pretty → encode_pretty(result)  // Human-readable
    
    encode_json(result) → Bytes:
        return json.encode({
            columns: result.columns,
            rows: [encode_row(row) for row in result.rows],
            stats: result.stats
        })
    
    encode_row(row) → Object:
        obj = {}
        for (name, value) in row:
            obj[name] = encode_value(value)
        return obj
    
    encode_value(value) → JsonValue:
        match value:
            Null → null
            Bool(b) → b
            Int(i) → i
            Float(f) → f
            String(s) → s
            Timestamp(t) → { "$timestamp": t }
            NodeRef(id) → { "$node": id }
            EdgeRef(id) → { "$edge": id }
            List(items) → [encode_value(i) for i in items]
}
```

---

# Part VIII: Deployment Architecture

## 23. Single-Node Deployment

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HOHG Server Process                                │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Protocol Handlers                               │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────────┐   │ │
│  │  │  TCP Socket  │  │  HTTP/REST   │  │  WebSocket (subscriptions) │   │ │
│  │  │  (native)    │  │              │  │                            │   │ │
│  │  └──────────────┘  └──────────────┘  └────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Engine Core                                     │ │
│  │  (Query Processor, Transaction Manager, Constraint/Rule Engine)        │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         Storage Engine                                  │ │
│  │  (Graph Store, Index Manager, Version Manager, WAL)                    │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
└──────────────────────────────────────┼───────────────────────────────────────┘
                                       │
                                       ▼
                            ┌────────────────────┐
                            │    File System     │
                            │                    │
                            │  /data/...         │
                            │  /indexes/...      │
                            │  /wal/...          │
                            │  /snapshots/...    │
                            └────────────────────┘
```

### 23.1 Configuration

```
Configuration File (hohg.conf):

server:
    host: "0.0.0.0"
    port: 7474
    http_port: 7475
    max_connections: 1000

storage:
    data_dir: "/var/hohg/data"
    page_size: 16384               # 16KB pages
    buffer_pool_size: "4GB"
    wal_dir: "/var/hohg/wal"
    wal_sync_mode: "fsync"         # fsync, fdatasync, async
    checkpoint_interval: "5min"

query:
    default_timeout: "30s"
    max_result_rows: 100000
    max_pattern_depth: 100

rules:
    max_depth: 100
    max_actions: 10000
    cascade_depth_limit: 100
    cascade_count_limit: 10000

versioning:
    snapshot_retention: "30d"
    max_snapshots: 100
    auto_checkpoint: true

logging:
    level: "info"
    file: "/var/log/hohg/hohg.log"
    max_size: "100MB"
    max_files: 10
```

### 23.2 Embedded Mode

```
Embedded API (Language Binding):

    // Initialize embedded engine
    engine = HOHG.embedded({
        data_dir: "./data",
        buffer_pool_size: "1GB"
    })
    
    // Load ontology
    engine.load_ontology("""
        node Person { name: String [required] }
        edge knows(a: Person, b: Person)
    """)
    
    // Execute statements
    result = engine.execute("SPAWN p: Person { name = 'Alice' }")
    alice_id = result.id
    
    result = engine.execute("""
        MATCH p: Person WHERE p.name = 'Alice'
        RETURN p
    """)
    
    // Close
    engine.close()
```

---

## 24. Monitoring and Observability

### 24.1 Metrics

```
Metrics exported (Prometheus format):

# Query metrics
hohg_queries_total{type="match|walk|spawn|kill|..."}
hohg_query_duration_seconds{type="...", quantile="0.5|0.9|0.99"}
hohg_query_errors_total{type="...", error="..."}

# Transaction metrics
hohg_transactions_total{status="committed|aborted"}
hohg_transaction_duration_seconds{quantile="..."}
hohg_active_transactions

# Storage metrics
hohg_nodes_total{type="..."}
hohg_edges_total{type="..."}
hohg_storage_bytes{component="nodes|edges|indexes|wal"}
hohg_buffer_pool_hits_total
hohg_buffer_pool_misses_total
hohg_wal_writes_total
hohg_checkpoints_total

# Constraint/Rule metrics
hohg_constraint_violations_total{constraint="...", hard="true|false"}
hohg_rule_executions_total{rule="..."}
hohg_rule_duration_seconds{rule="...", quantile="..."}

# Session metrics
hohg_active_sessions
hohg_session_duration_seconds{quantile="..."}
```

### 24.2 Health Checks

```
Health Check Endpoints:

GET /health
    Returns 200 if server is running
    Response: { "status": "healthy" }

GET /health/ready
    Returns 200 if server can accept queries
    Checks: storage initialized, registries loaded
    Response: { 
        "status": "ready",
        "checks": {
            "storage": "ok",
            "registries": "ok",
            "wal": "ok"
        }
    }

GET /health/live
    Returns 200 if server is alive
    Basic liveness check
```

---

# Part IX: Error Handling

## 25. Error Categories

```
ErrorCode Structure: E<category><number>

    E1xxx: Syntax errors (parsing)
    E2xxx: Constraint violations
    E3xxx: Type errors
    E4xxx: Not found errors
    E5xxx: Limit/resource errors
    E6xxx: Transaction errors
    E7xxx: Storage errors
    E8xxx: Configuration errors
    E9xxx: Internal errors

Error {
    code: String                    // E.g., "E2003"
    category: ErrorCategory
    message: String
    location: SourceLocation?       // For syntax errors
    context: ErrorContext?          // Additional data
    hints: List<String>
    cause: Error?                   // Chained error
}

SourceLocation {
    line: Int
    column: Int
    snippet: String
}

ErrorContext =
    | ConstraintContext(constraint_name, bindings, expected, actual)
    | TypeContext(expected_type, actual_type, expression)
    | NotFoundContext(entity_type, identifier)
    | LimitContext(limit_name, limit_value, actual_value)
    | TransactionContext(txn_id, state)
```

### 25.1 Error Recovery

```
ErrorRecovery {
    // Parser recovery
    recover_parser_error(parser, error) → AST?:
        // Skip to synchronization point
        sync_tokens = [SEMICOLON, RBRACE, MATCH, SPAWN, ...]
        
        while not parser.at_end():
            if parser.current_token() in sync_tokens:
                break
            parser.advance()
        
        // Record error and continue
        parser.record_error(error)
        return parse_next_statement()
    
    // Transaction recovery
    recover_transaction_error(txn, error) → RecoveryAction:
        match error.code:
            // Constraint violation: rollback
            E2xxx → Rollback(txn)
            
            // Type error: rollback
            E3xxx → Rollback(txn)
            
            // Not found: depends on context
            E4xxx → if txn.in_rule: Rollback else: ReturnError
            
            // Resource limit: rollback
            E5xxx → Rollback(txn)
            
            // Storage error: rollback + alert
            E7xxx → Rollback(txn) then Alert
    
    // WAL recovery
    recover_from_wal() → RecoveryResult:
        // Read WAL from last checkpoint
        entries = read_wal_entries(last_checkpoint)
        
        committed_txns = Set()
        uncommitted_txns = Set()
        
        for entry in entries:
            match entry.type:
                Begin → uncommitted_txns.add(entry.txn_id)
                Commit → 
                    uncommitted_txns.remove(entry.txn_id)
                    committed_txns.add(entry.txn_id)
                Abort → uncommitted_txns.remove(entry.txn_id)
        
        // Replay committed transactions
        for entry in entries:
            if entry.txn_id in committed_txns:
                apply_entry(entry)
        
        // Undo uncommitted (already not applied, just log)
        log("Discarding uncommitted transactions: " + uncommitted_txns)
        
        return RecoveryResult(
            committed = len(committed_txns),
            discarded = len(uncommitted_txns)
        )
}
```

---

# Part X: Testing Architecture

## 26. Test Categories

```
Test Pyramid:

    ┌───────────────────┐
    │   Integration     │  End-to-end scenarios
    │   Tests           │  
    ├───────────────────┤
    │   Component       │  Engine, storage, compiler
    │   Tests           │  
    ├───────────────────┤
    │   Unit            │  Individual functions
    │   Tests           │  
    └───────────────────┘

Test Categories:

1. Unit Tests
   - Parser tests (each grammar production)
   - Analyzer tests (type checking, name resolution)
   - Planner tests (plan generation, cost estimation)
   - Executor tests (operator behavior)
   - Index tests (operations, consistency)

2. Component Tests
   - Storage tests (CRUD, transactions, recovery)
   - Compiler tests (full compilation pipeline)
   - Rule engine tests (trigger, execute, cycle)
   - Constraint tests (check, incremental)

3. Integration Tests
   - Full query execution
   - Transaction scenarios
   - Concurrent operations
   - Versioning workflows
   - API endpoint tests

4. Property-Based Tests
   - Arbitrary graphs remain consistent
   - Transactions are atomic
   - Constraints are never violated
   - Rules terminate

5. Performance Tests
   - Benchmark suite for common patterns
   - Scalability tests (increasing data size)
   - Concurrency tests (many parallel queries)
```

### 26.1 Test Ontology

```
Standard Test Ontology:

node TestNode {
    name: String [required],
    value: Int = 0,
    timestamp: Timestamp = now()
}

node TypeA : TestNode {
    a_field: String?
}

node TypeB : TestNode {
    b_field: Int?
}

edge test_edge(from: TestNode, to: TestNode) {
    weight: Float = 1.0
}

edge typed_edge(from: TypeA, to: TypeB) {}

constraint test_value_positive:
    n: TestNode WHERE n.value != null
    => n.value >= 0

rule test_auto_increment [priority: 10]:
    n: TestNode WHERE n.value = 0
    =>
    SET n.value = 1
```

### 26.2 Test Utilities

```
TestHarness {
    engine: Engine
    
    setup():
        engine = create_test_engine()
        engine.load_test_ontology()
    
    teardown():
        engine.close()
        cleanup_test_data()
    
    // Helpers
    spawn_test_nodes(count, type = "TestNode") → List<NodeId>:
        ids = []
        for i in 0..count:
            id = execute("SPAWN n: {type} {{ name = 'test_{i}' }}")
            ids.append(id)
        return ids
    
    spawn_test_graph(node_count, edge_count) → TestGraph:
        nodes = spawn_test_nodes(node_count)
        for i in 0..edge_count:
            from_idx = random(node_count)
            to_idx = random(node_count)
            execute("LINK test_edge(#{nodes[from_idx]}, #{nodes[to_idx]})")
        return TestGraph(nodes)
    
    assert_node_exists(id):
        result = execute("INSPECT #{id}")
        assert result.found
    
    assert_constraint_violated(constraint_name, action):
        try:
            action()
            fail("Expected constraint violation")
        catch ConstraintViolation e:
            assert e.constraint == constraint_name
}
```

---

# Appendix A: Algorithm Complexity

| Operation | Average Case | Worst Case | Notes |
|-----------|--------------|------------|-------|
| SPAWN node | O(1) + O(A) | O(A) | A = attributes |
| KILL node | O(E) | O(E) | E = connected edges |
| LINK edge | O(1) + O(S) | O(S) | S = signature positions |
| UNLINK edge | O(H) | O(H) | H = higher-order refs |
| SET attribute | O(1) | O(1) | Direct update |
| Type scan | O(N) | O(N) | N = nodes of type |
| Index scan | O(log N + K) | O(log N + K) | K = matching rows |
| Pattern match | O(N₁ × N₂ × ... × E) | Exponential | Depends on pattern |
| Transitive closure | O(V + E) | O(V × E) | V = vertices, E = edges |
| Constraint check | O(M × P) | O(M × P) | M = matches, P = pattern size |
| Rule execution | O(R × M × A) | Unbounded | Limited by depth/action |

---

# Appendix B: Storage Sizes

| Entity Type | Fixed Overhead | Variable Cost |
|-------------|----------------|---------------|
| Node | 32 bytes | + 8 bytes/attr + string sizes |
| Edge | 32 bytes | + 8 bytes/target + 8 bytes/attr |
| Index entry | 24 bytes | + key size |
| WAL entry | 32 bytes | + operation data |
| Type registration | 128 bytes | + attribute defs |
| Constraint registration | 256 bytes | + pattern size |
| Rule registration | 256 bytes | + pattern + production |

---

# Appendix C: Configuration Defaults

| Parameter | Default | Min | Max |
|-----------|---------|-----|-----|
| `buffer_pool_size` | 1GB | 64MB | System RAM |
| `page_size` | 16KB | 4KB | 64KB |
| `max_connections` | 100 | 1 | 10000 |
| `query_timeout` | 30s | 1s | 1h |
| `max_result_rows` | 100000 | 100 | 10M |
| `rule_max_depth` | 100 | 10 | 1000 |
| `rule_max_actions` | 10000 | 100 | 1M |
| `cascade_depth_limit` | 100 | 10 | 1000 |
| `transitive_depth_default` | 100 | 10 | 10000 |
| `wal_segment_size` | 64MB | 1MB | 1GB |
| `checkpoint_interval` | 5min | 1min | 1h |

---

*End of HOHG System Architecture*