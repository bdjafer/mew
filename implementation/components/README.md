# Implementation Spec Meta-Plan

**Version:** 2.0
**Status:** Revised
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

INPUTS
  - What crosses the boundary inward

OUTPUTS
  - What crosses the boundary outward

ACCEPTANCE CRITERIA
  - Observable behaviors that mean "done"
  
OPEN QUESTIONS
  - Unknowns to resolve during implementation
```

One page maximum. If longer, you're over-specifying.

---

## 4. Component List

### Layer 0: Foundation (no internal dependencies)

| Component | Purpose |
|-----------|---------|
| **Value** | Represent scalar attribute values |
| **Identity** | Generate and manage node/edge IDs |
| **Error** | Error type hierarchy |

### Layer 1: Core Structures

| Component | Purpose |
|-----------|---------|
| **Node** | In-memory node representation |
| **Edge** | In-memory edge representation (including higher-order) |
| **GraphStore** | Store and retrieve nodes/edges by ID |

### Layer 2: Language Frontend

| Component | Purpose |
|-----------|---------|
| **Lexer** | Tokenize HOHG source text |
| **Parser** | Build AST from tokens |
| **Analyzer** | Name resolution and type checking |

### Layer 3: Registries

| Component | Purpose |
|-----------|---------|
| **TypeRegistry** | Runtime lookup of node type definitions |
| **EdgeTypeRegistry** | Runtime lookup of edge type definitions |
| **ConstraintRegistry** | Index constraints by affected types |
| **RuleRegistry** | Index rules by trigger types, priority order |

### Layer 4: Ontology Compiler

| Component | Purpose |
|-----------|---------|
| **OntologyParser** | Parse ontology DSL into AST |
| **SugarExpander** | Expand modifiers into explicit constraints/rules |
| **Validator** | Check ontology consistency |
| **L0Generator** | Generate Layer 0 graph structure |
| **RegistryBuilder** | Populate runtime registries from ontology |

### Layer 5: Indexes

| Component | Purpose |
|-----------|---------|
| **TypeIndex** | Find nodes by type |
| **AdjacencyIndex** | Find edges from/to a node |
| **AttributeIndex** | Find nodes by attribute value (range queries) |
| **EdgeIndex** | Find edges by type and target position |
| **HigherOrderIndex** | Find edges that reference other edges |

### Layer 6: Pattern System

| Component | Purpose |
|-----------|---------|
| **PatternCompiler** | Transform pattern AST into executable form |
| **PatternMatcher** | Find all matches of a pattern in the graph |
| **ExpressionEvaluator** | Evaluate expressions given bindings |

### Layer 7: Constraint & Rule

| Component | Purpose |
|-----------|---------|
| **ConstraintChecker** | Validate mutations against constraints |
| **RuleEngine** | Trigger and execute rules on mutations |

### Layer 8: Execution

| Component | Purpose |
|-----------|---------|
| **QueryPlanner** | Generate execution plan from analyzed query |
| **QueryExecutor** | Execute query plan, stream results |
| **MutationExecutor** | Execute SPAWN/KILL/LINK/UNLINK/SET |

### Layer 9: Transaction

| Component | Purpose |
|-----------|---------|
| **Transaction** | Track pending changes within a transaction |
| **TransactionManager** | Orchestrate begin/commit/rollback |

### Layer 10: Durability

| Component | Purpose |
|-----------|---------|
| **WAL** | Append-only log for durability |
| **Recovery** | Restore state from WAL after crash |
| **BufferPool** | Cache pages in memory |
| **FileManager** | Read/write pages to disk |

### Layer 11: API

| Component | Purpose |
|-----------|---------|
| **Session** | Track client session state |
| **Router** | Dispatch statements to appropriate executor |
| **REPL** | Command-line interface |

### Layer 12: Versioning (defer to v1.x if needed)

| Component | Purpose |
|-----------|---------|
| **Snapshot** | Immutable point-in-time capture |
| **Branch** | Named mutable reference to snapshot |
| **Diff** | Compute changes between snapshots |
| **Merge** | Combine branches |

---

## 5. Dependency Graph

```
Layer 0:  Value    Identity    Error
             \        |        /
              \       |       /
Layer 1:       Node   Edge   GraphStore
                  \    |    /
                   \   |   /
Layer 2:         Lexer → Parser → Analyzer
                                     |
Layer 3:    TypeRegistry  EdgeTypeRegistry  ConstraintRegistry  RuleRegistry
                  \            |                   |              /
                   \           |                   |             /
Layer 4:            OntologyParser → SugarExpander → Validator
                                          |
                                    L0Generator → RegistryBuilder
                                          
Layer 5:    TypeIndex  AdjacencyIndex  AttributeIndex  EdgeIndex  HOIndex
                  \          |              |            |         /
                   \         |              |            |        /
Layer 6:            PatternCompiler → PatternMatcher ← ExprEvaluator
                                           |
                          +----------------+----------------+
                          |                                 |
Layer 7:          ConstraintChecker                    RuleEngine
                          |                                 |
                          +----------------+----------------+
                                           |
Layer 8:               QueryPlanner → QueryExecutor
                                           |
                                    MutationExecutor
                                           |
Layer 9:                Transaction ← TransactionManager
                                           |
Layer 10:                    WAL → Recovery → BufferPool → FileManager
                                           |
Layer 11:                      Session → Router → REPL
```

---

## 6. Implementation Order

Implementation proceeds bottom-up through the dependency graph.

### Phase 1: Foundation
**Components:** Value, Identity, Error, Node, Edge, GraphStore

**Acceptance:** Can create nodes and edges in memory, retrieve by ID.

### Phase 2: Parsing
**Components:** Lexer, Parser

**Acceptance:** Can parse MATCH/SPAWN/LINK statements into AST.

### Phase 3: Registries
**Components:** TypeRegistry, EdgeTypeRegistry (hardcoded for testing)

**Acceptance:** Can lookup type definitions at runtime.

### Phase 4: Basic Execution
**Components:** Analyzer (partial), MutationExecutor, QueryExecutor (simple)

**Acceptance:** Can SPAWN nodes, LINK edges, MATCH single type.

### Phase 5: Ontology Loading
**Components:** OntologyParser, SugarExpander, Validator, L0Generator, RegistryBuilder

**Acceptance:** Can load .hog ontology file and populate registries.

### Phase 6: Indexes
**Components:** TypeIndex, AdjacencyIndex

**Acceptance:** Queries use indexes instead of full scans.

### Phase 7: Pattern Matching
**Components:** PatternCompiler, PatternMatcher, ExpressionEvaluator

**Acceptance:** Multi-variable patterns with edges work.

### Phase 8: Constraints
**Components:** ConstraintRegistry, ConstraintChecker

**Acceptance:** Hard constraints reject invalid mutations.

### Phase 9: Rules
**Components:** RuleRegistry, RuleEngine

**Acceptance:** Auto rules fire on mutations, cascade works.

### Phase 10: Transactions
**Components:** Transaction, TransactionManager

**Acceptance:** BEGIN/COMMIT/ROLLBACK work, deferred constraints checked at commit.

### Phase 11: Durability
**Components:** WAL, Recovery, BufferPool, FileManager

**Acceptance:** Survives process crash, recovers from WAL.

### Phase 12: API Polish
**Components:** Session, Router, REPL

**Acceptance:** Interactive REPL session works end-to-end.

---

## 7. Milestones

Milestones are defined by **observable system behavior**, not internal completion.

| Milestone | Behavior | Phases |
|-----------|----------|--------|
| **M1: Memory Graph** | Create/read nodes and edges in memory | 1 |
| **M2: Parse** | Parse HOHG statements to AST | 1-2 |
| **M3: Hardcoded Types** | SPAWN/MATCH with hardcoded Task/Person types | 1-4 |
| **M4: Load Ontology** | Load .hog file, types available | 1-5 |
| **M5: Indexed Queries** | Queries use type index | 1-6 |
| **M6: Complex Patterns** | Multi-hop patterns, transitive closure | 1-7 |
| **M7: Validated** | Constraints reject bad mutations | 1-8 |
| **M8: Reactive** | Rules fire automatically | 1-9 |
| **M9: Transactional** | ACID transactions work | 1-10 |
| **M10: Durable** | Survives crash | 1-11 |
| **M11: Interactive** | REPL works end-to-end | 1-12 |

---

## 8. Spec Writing Process

1. **Write specs for Phase N components** (~1 page each)
2. **Verify dependencies** — each component's DEPENDS ON should already have specs
3. **Implement Phase N** — specs inform boundaries, code informs internals
4. **Update specs only if contracts change** — internal refactors don't touch specs
5. **Repeat for Phase N+1**

Parallel work: While implementing Phase N, specs for Phase N+1 can be drafted.

---

## 9. What We're NOT Doing

- **No Rust signatures in specs** — Write them in code
- **No algorithm pseudo-code** — Implement to discover
- **No error variant lists** — Emerge from edge cases
- **No performance targets yet** — Make it work, then make it fast
- **No distributed/GPU considerations** — v2+ concerns

---

## 10. Open Questions (Project Level)

These are unknowns that will be resolved during implementation:

- Best join order heuristics for pattern matching
- WAL segment size and rotation policy
- Buffer pool eviction strategy
- Transaction isolation level semantics
- Cycle detection approach in rule engine
- Index selection heuristics in planner

We don't need answers before starting. We need to start to find answers.

---

## 11. Next Step

Write specs for **Phase 1 components**: Value, Identity, Error, Node, Edge, GraphStore.

Each spec: one page, follows template, focuses on contracts not internals.