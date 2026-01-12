# Implementation Spec Meta-Plan

**Version:** 3.0
**Status:** Aligned with Architecture v1.0
**Purpose:** Define what specs are, provide template, list components, establish implementation order

---

## 1. What a Spec Is

A spec defines **contracts between components**. It answers:
- What does this component promise to do?
- What does it refuse to do?
- What does it need from others?
- What do others need from it?
- How do we know when it's done?

A spec is **stable**. If you're rewriting specs frequently, they contain the wrong things.

---

## 2. What a Spec Is Not

A spec is **not**:
- Interface signatures (discover by coding)
- Algorithm details (the code IS the algorithm)
- Error types (emerge from edge cases)
- Internal data structures (implementation detail)
- Performance optimizations (premature)

If it will change when you refactor, it doesn't belong in a spec.

---

## 3. Spec Template

```
COMPONENT: [Name]

PURPOSE
  One sentence. What problem does this solve?

RESPONSIBILITIES
  - What it owns and must do
  
NON-RESPONSIBILITIES  
  - What seems related but belongs elsewhere

DEPENDS ON
  - Component: what it needs

DEPENDED ON BY
  - Component: what it provides

INVARIANTS
  - Properties that must always hold

ACCEPTANCE CRITERIA
  - Observable behaviors that mean "done"
  
NOTES
  - Clarifications, edge cases, design rationale
```

One page maximum. If longer, you're over-specifying.

---

## 4. Shared Data Types

These are **data structures**, not components. No specs needed — just Rust structs.

| Type | Purpose |
|------|---------|
| **Value** | Scalar attribute values (Null, Bool, Int, Float, String, Timestamp, Duration, NodeRef, EdgeRef) |
| **Node** | Node structure (id, type_id, version, attributes) |
| **Edge** | Edge structure (id, type_id, targets, version, attributes) |
| **EntityId** | NodeId \| EdgeId — unified ID space |
| **Error** | Error hierarchy (ParseError, AnalysisError, ConstraintError, etc.) |

---

## 5. Component List (13 Components)

### Tier 1: Storage

| Component | Purpose | Absorbs |
|-----------|---------|---------|
| **Graph** | Store nodes/edges, indexed access, snapshots | GraphStore + TypeIndex + AttrIndex + EdgeIndex + AdjacencyIndex + HigherOrderIndex |
| **Journal** | Write-ahead log, crash recovery | WAL + Recovery |

### Tier 2: Language

| Component | Purpose | Absorbs |
|-----------|---------|---------|
| **Parser** | Tokenize and parse source text | Lexer + StatementParser + OntologyParser |
| **Analyzer** | Name resolution, type checking | — |

### Tier 3: Schema

| Component | Purpose | Absorbs |
|-----------|---------|---------|
| **Registry** | Runtime schema: types, edges, constraints, rules | TypeRegistry + EdgeTypeRegistry + ConstraintRegistry + RuleRegistry |
| **Compiler** | Ontology → Registry + Layer 0 | SugarExpander + Validator + L0Generator + RegistryBuilder |

### Tier 4: Pattern

| Component | Purpose | Absorbs |
|-----------|---------|---------|
| **Pattern** | Compile patterns, match against graph, evaluate expressions | PatternCompiler + PatternMatcher + ExpressionEvaluator |

### Tier 5: Execution

| Component | Purpose | Absorbs |
|-----------|---------|---------|
| **Query** | Plan and execute MATCH/WALK/INSPECT | QueryPlanner + QueryExecutor |
| **Mutation** | Execute SPAWN/KILL/LINK/UNLINK/SET | — |
| **Constraint** | Validate mutations against constraints | — |
| **Rule** | Trigger and execute rules to quiescence | RuleMatcher + RuleExecutor |
| **Transaction** | ACID, orchestrate mutation→rule→constraint | TransactionManager + TransactionBuffer |

### Tier 6: Interface

| Component | Purpose | Absorbs |
|-----------|---------|---------|
| **Session** | External interface (REPL, HTTP, embedded) | SessionManager + Router + REPL + HTTPHandler |

---

## 6. Dependency Graph

```
                         Session
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
           Parser ───► Analyzer ◄─── Registry
              │             │             ▲
              │             │             │
              │             ▼             │
              └───────► Compiler ────────┘
                            │
                            ▼
         ┌──────────────────┴──────────────────┐
         │                                     │
         ▼                                     ▼
       Query ──────────────┬─────────────► Mutation
         │                 │                   │
         │                 ▼                   │
         │             Pattern                 │
         │                 │                   │
         │      ┌──────────┼──────────┐       │
         │      │          │          │       │
         │      ▼          ▼          ▼       │
         │  Constraint   Rule    Transaction ◄┘
         │      │          │          │
         │      └──────────┼──────────┘
         │                 │
         └────────────────►│
                           ▼
                        Graph
                           │
                           ▼
                       Journal
```

**Reading the graph:**
- Arrow A → B means "A depends on B"
- Transaction orchestrates Mutation, Rule, Constraint
- Query and Mutation both use Pattern
- Everything ultimately depends on Graph
- Journal is the durability layer under Graph

---

## 7. Implementation Phases

Implementation proceeds bottom-up through dependencies.

### Phase 1: Foundation
**Components:** Graph (in-memory, no indexes yet)

**Acceptance:** 
- Create nodes and edges by ID
- Retrieve nodes and edges by ID
- Delete nodes and edges
- Set attributes

**Deliverable:** Working in-memory graph store.

---

### Phase 2: Parsing
**Components:** Parser (statements only, not ontology yet)

**Acceptance:**
- Parse MATCH statements into AST
- Parse SPAWN/KILL/LINK/UNLINK/SET statements into AST
- Parse expressions (arithmetic, comparison, attribute access)
- Meaningful error messages with line/column

**Deliverable:** Can parse all statement types.

---

### Phase 3: Types (Hardcoded)
**Components:** Registry (hardcoded), Analyzer

**Acceptance:**
- Hardcoded Task, Person, Project types for testing
- Analyzer resolves type names to TypeId
- Analyzer resolves attribute names to AttrId
- Analyzer reports unknown type/attribute errors

**Deliverable:** Type-checked AST with hardcoded schema.

---

### Phase 4: Basic Execution
**Components:** Mutation, Query (simple), Transaction (minimal)

**Acceptance:**
- SPAWN creates nodes with type validation
- LINK creates edges with signature validation
- KILL deletes nodes (no cascade yet)
- UNLINK deletes edges
- SET modifies attributes
- MATCH single type returns results
- Auto-commit mode works

**Deliverable:** Basic CRUD operations work end-to-end.

---

### Phase 5: Ontology
**Components:** Parser (ontology), Compiler

**Acceptance:**
- Parse ontology DSL (node, edge, constraint, rule definitions)
- Expand modifiers ([required], [unique], [acyclic], etc.)
- Validate ontology (no cycles, references resolve)
- Generate Layer 0 nodes/edges
- Build Registry from ontology
- LOAD statement works

**Deliverable:** Can load .mew ontology files.

---

### Phase 6: Indexes
**Components:** Graph (add indexes)

**Acceptance:**
- Type index: find all nodes of type
- Adjacency index: find edges from/to node
- Attribute index: find nodes by attribute value/range
- Higher-order index: find edges about edges
- Query uses indexes instead of full scans

**Deliverable:** Indexed queries.

---

### Phase 7: Pattern Matching
**Components:** Pattern

**Acceptance:**
- Compile multi-variable patterns
- Match patterns with edges
- Evaluate WHERE conditions
- Transitive closure (edge+, edge*)
- NOT EXISTS patterns

**Deliverable:** Complex pattern matching works.

---

### Phase 8: Constraints
**Components:** Constraint

**Acceptance:**
- Immediate constraints checked after mutation
- Deferred constraints checked at commit
- Hard constraints abort transaction
- Soft constraints produce warnings
- Constraint error messages include context

**Deliverable:** Constraints enforce invariants.

---

### Phase 9: Rules
**Components:** Rule

**Acceptance:**
- Auto rules fire on matching mutations
- Rules execute in priority order
- Rules fire to quiescence
- Cycle detection prevents infinite loops
- Depth/action limits enforced

**Deliverable:** Reactive rule execution.

---

### Phase 10: Transactions
**Components:** Transaction (full)

**Acceptance:**
- BEGIN/COMMIT/ROLLBACK work
- Read-your-writes within transaction
- Constraint violation aborts entire transaction
- Deferred constraints checked at commit
- Savepoints work

**Deliverable:** Full ACID transactions.

---

### Phase 11: Durability
**Components:** Journal

**Acceptance:**
- WAL written before commit
- fsync ensures durability
- Recovery replays committed transactions
- Uncommitted transactions discarded on recovery
- Survives kill -9

**Deliverable:** Crash-safe persistence.

---

### Phase 12: Interface
**Components:** Session

**Acceptance:**
- REPL works interactively
- HTTP API accepts statements
- Session tracks transaction state
- Multiple concurrent sessions
- Graceful error handling

**Deliverable:** Production-ready interface.

---

## 8. Milestones

Milestones are **observable system behaviors**, not internal completions.

| Milestone | Behavior | After Phase |
|-----------|----------|-------------|
| **M1: Memory Store** | Create/read nodes and edges in memory | 1 |
| **M2: Parse** | Parse MEW statements to AST | 2 |
| **M3: Typed CRUD** | SPAWN/MATCH with hardcoded types | 4 |
| **M4: Load Ontology** | Load .mew file, custom types work | 5 |
| **M5: Indexed** | Queries use indexes | 6 |
| **M6: Patterns** | Multi-hop patterns, transitive closure | 7 |
| **M7: Constrained** | Constraints reject invalid mutations | 8 |
| **M8: Reactive** | Rules fire automatically | 9 |
| **M9: Transactional** | ACID transactions work | 10 |
| **M10: Durable** | Survives crash | 11 |
| **M11: Interactive** | REPL works end-to-end | 12 |

---

## 9. Spec Writing Process

1. **Write specs for Phase N components** (~1 page each)
2. **Verify dependencies** — each component's DEPENDS ON should already have specs
3. **Implement Phase N** — specs inform boundaries, code informs internals
4. **Update specs only if contracts change** — internal refactors don't touch specs
5. **Repeat for Phase N+1**

Parallel work: While implementing Phase N, specs for Phase N+1 can be drafted.

---

## 10. What We're NOT Specifying

- **Rust signatures** — Discover by coding
- **Algorithm pseudo-code** — The code is the algorithm
- **Error variant lists** — Emerge from edge cases
- **Performance targets** — Make it work, then make it fast
- **Internal data structures** — Implementation detail
- **v2 features** — META mode, branching, provenance are out of scope

---

## 11. Open Questions (Resolve During Implementation)

| Question | Resolve In Phase |
|----------|------------------|
| Best join order heuristics for pattern matching | 7 |
| WAL segment size and rotation policy | 11 |
| Transaction isolation level semantics | 10 |
| Cycle detection approach for acyclic constraints | 8 |
| Index selection heuristics in query planner | 6-7 |
| Higher-order cascade deletion semantics | 4 |
| Rule execution order when same priority | 9 |

We don't need answers before starting. We need to start to find answers.

---

## 12. Component Spec Order

Write specs in dependency order (bottom-up):

| Order | Component | Depends On |
|-------|-----------|------------|
| 1 | Graph | (none) |
| 2 | Journal | Graph |
| 3 | Parser | (none) |
| 4 | Registry | (none, data structure) |
| 5 | Analyzer | Parser, Registry |
| 6 | Compiler | Parser, Registry, Graph |
| 7 | Pattern | Graph, Registry |
| 8 | Mutation | Graph, Registry, Pattern |
| 9 | Query | Graph, Registry, Pattern |
| 10 | Constraint | Pattern, Registry |
| 11 | Rule | Pattern, Registry, Mutation |
| 12 | Transaction | Graph, Mutation, Constraint, Rule, Journal |
| 13 | Session | Parser, Analyzer, Query, Mutation, Transaction |

---