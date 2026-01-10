# 

## Overview

**Timeline**: ~12 months
**Language**: TypeScript (moving to Rust for performance-critical paths later)
**Approach**: Iterative, test-driven, working software at each milestone

---

## Project Structure

```
hohg/
├── packages/
│   ├── core/           # Data structures, operations, Layer 0
│   ├── parser/         # Ontology language lexer & parser
│   ├── compiler/       # Ontology AST → graph structure
│   ├── query/          # Query language & execution
│   ├── rules/          # Rewrite rule engine
│   ├── storage/        # Persistence layer
│   └── cli/            # Command-line interface
├── ontologies/
│   ├── test-a-tasks/   # Test ontology A
│   ├── test-b-causal/  # Test ontology B
│   └── test-c-meta/    # Test ontology C
├── tests/
│   ├── unit/
│   ├── integration/
│   └── benchmarks/
└── docs/
    ├── spec/           # Layer 0 spec, grammar, etc.
    └── guides/

```

---

## Phase 1: Representation

### Sprint 1.0: Foundation (Week 1-2)

**Goal**: Project skeleton, tooling, basic types

**Tasks**:

```
□ Initialize monorepo (pnpm workspaces or turborepo)
□ Configure TypeScript (strict mode)
□ Set up testing framework (vitest)
□ Set up linting (eslint, prettier)
□ CI pipeline (GitHub Actions)
□ Define core type interfaces (no implementation yet):
    □ ID type
    □ Node interface
    □ Edge interface
    □ Graph interface
    □ Scalar types

```

**Deliverable**: Empty project that builds and runs tests

---

### Sprint 1.1: Core Data Structures (Week 3-4)

**Goal**: In-memory representation of HOHG

**Tasks**:

```
□ ID generation
    □ Choose strategy (UUID v7 for time-ordering, or sequential + random)
    □ Implement ID type with equality, hashing
    □ Test uniqueness guarantees

□ Node implementation
    □ NodeData: { id, typeTag, attributes }
    □ Attribute storage (Map<string, ScalarValue>)
    □ Type tag as string (type name)
    □ Immutable vs mutable decision (start mutable, optimize later)

□ Edge implementation
    □ EdgeData: { id, typeTag, targets: ID[], attributes }
    □ Targets are ordered list of IDs
    □ Targets can be node IDs OR edge IDs (higher-order)
    □ Validation: targets.length must match arity (deferred to type-checking)

□ Graph container
    □ GraphStore class
    □ nodes: Map<ID, NodeData>
    □ edges: Map<ID, EdgeData>
    □ Basic indexes:
        □ nodesByType: Map<TypeName, Set<ID>>
        □ edgesByType: Map<TypeName, Set<ID>>
        □ edgesBySource: Map<ID, Set<ID>> (edges where target[0] = ID)
        □ edgesByTarget: Map<ID, Set<ID>> (edges where any target = ID)

□ Tests
    □ Create nodes, verify storage
    □ Create edges, verify connectivity
    □ Create higher-order edges (edge pointing to edge)
    □ Basic retrieval by ID
    □ Index correctness

```

**Deliverable**: Can create/store/retrieve nodes and edges in memory

---

### Sprint 1.2: Basic Operations (Week 5-6)

**Goal**: CRUD operations with referential integrity

**Tasks**:

```
□ Node operations
    □ createNode(typeTag: string, attrs: Record<string, Scalar>) → ID
    □ getNode(id: ID) → NodeData | null
    □ deleteNode(id: ID) → boolean
        □ Check: no edges reference this node
        □ Option: cascade delete (delete referencing edges first)
    □ setNodeAttribute(id: ID, key: string, value: Scalar) → boolean
    □ getNodeAttribute(id: ID, key: string) → Scalar | null

□ Edge operations
    □ createEdge(typeTag: string, targets: ID[], attrs) → ID
        □ Validate: all targets exist (nodes or edges)
        □ Update indexes
    □ getEdge(id: ID) → EdgeData | null
    □ deleteEdge(id: ID) → boolean
        □ Check: no higher-order edges reference this edge
        □ Update indexes
    □ setEdgeAttribute(id: ID, key: string, value: Scalar) → boolean

□ Query primitives
    □ getNodesByType(typeTag: string) → Iterator<NodeData>
    □ getEdgesByType(typeTag: string) → Iterator<EdgeData>
    □ getEdgesFrom(nodeId: ID) → Iterator<EdgeData>
    □ getEdgesTo(nodeId: ID) → Iterator<EdgeData>
    □ getEdgesInvolving(id: ID) → Iterator<EdgeData> (any position)

□ Higher-order support
    □ isEdge(id: ID) → boolean
    □ isNode(id: ID) → boolean
    □ getEdgesAbout(edgeId: ID) → Iterator<EdgeData> (edges targeting this edge)

□ Tests
    □ CRUD operations work correctly
    □ Referential integrity enforced
    □ Indexes stay consistent through mutations
    □ Higher-order edges work
    □ Edge cases: delete node with edges, delete edge with higher-order refs

```

**Deliverable**: Working graph store with all basic operations

---

### Sprint 1.3: Layer 0 Type System (Week 7-8)

**Goal**: Hardcoded Layer 0, type checking, subtyping

**Tasks**:

```
□ Layer 0 type definitions (hardcoded)
    □ Define all 27 node types as constants
    □ Define all 34 edge types as constants
    □ Include: name, attributes, arity (for edges), inheritance

□ Type registry
    □ TypeRegistry class
    □ registerNodeType(def: NodeTypeDef)
    □ registerEdgeType(def: EdgeTypeDef)
    □ getNodeType(name: string) → NodeTypeDef | null
    □ getEdgeType(name: string) → EdgeTypeDef | null
    □ Preload Layer 0 types at construction

□ Subtyping
    □ isSubtype(child: string, parent: string) → boolean
    □ Handle inheritance chains
    □ Handle union types (T | U)
    □ Handle optional types (T?)
    □ Handle edge<E> references

□ Type-checked operations
    □ Wrap createNode to validate:
        □ Type exists
        □ Type is not abstract
        □ Required attributes present
        □ Attribute types match
    □ Wrap createEdge to validate:
        □ Type exists
        □ Correct number of targets
        □ Target types match signature (with subtyping)
    □ Wrap setAttribute to validate type

□ Attribute inheritance
    □ When checking attributes, walk inheritance chain
    □ Collect all attributes from type and ancestors

□ Tests
    □ Layer 0 types loaded correctly
    □ Subtyping works for inheritance
    □ Subtyping works for union/optional
    □ Type validation rejects bad inputs
    □ Attribute inheritance works

```

**Deliverable**: Type-safe graph operations, Layer 0 complete

---

### Sprint 1.4: Pattern Matching (Week 9-10)

**Goal**: Match patterns against graph

**Tasks**:

```
□ Pattern representation
    □ PatternDef interface (matching Layer 0 _PatternDef)
    □ NodeVar: { name, typeExpr, attrConstraints }
    □ EdgePattern: { edgeType, targets: VarName[], alias?, negated }
    □ Condition: expression tree

□ Expression representation
    □ Expr union type
    □ LiteralExpr, VarRefExpr, AttrAccessExpr
    □ BinaryOpExpr, UnaryOpExpr
    □ Expression evaluator: eval(expr, bindings) → Value

□ Pattern matching algorithm
    □ matchPattern(pattern: PatternDef, graph: GraphStore) → Iterator<Match>
    □ Match = Map<VarName, ID>
    □ Algorithm:
        1. Order variables by selectivity (use type cardinality)
        2. For each node var: iterate candidates, filter by type & attrs
        3. For each edge pattern: lookup edges, filter by signature
        4. For negative patterns: verify non-existence
        5. Evaluate conditions on complete bindings
        6. Yield valid matches

□ Optimization hooks
    □ Use type indexes to reduce candidate sets
    □ Use edge indexes when source/target is bound
    □ Short-circuit on first failed constraint

□ Tests
    □ Simple pattern: single node var
    □ Pattern with edge: two nodes connected
    □ Pattern with attribute constraints
    □ Pattern with conditions
    □ Negative patterns (not exists)
    □ Higher-order patterns (edge about edge)
    □ Empty results when no match
    □ Multiple matches returned correctly

```

**Deliverable**: Can match patterns against graph, returns bindings

---

### Sprint 1.5: Constraint Checking (Week 11-12)

**Goal**: Enforce constraints on mutations

**Tasks**:

```
□ Constraint representation
    □ ConstraintDef interface
    □ pattern: PatternDef
    □ condition: Expr (the implication RHS)
    □ hard: boolean

□ Constraint registry
    □ ConstraintRegistry class
    □ registerConstraint(def: ConstraintDef)
    □ getConstraintsForType(typeName: string) → ConstraintDef[]
    □ Preload Layer 0 constraints

□ Constraint checking
    □ checkConstraints(graph: GraphStore) → ConstraintViolation[]
    □ For each constraint:
        □ Find all pattern matches
        □ For each match, evaluate condition
        □ If condition false: record violation

□ Mutation hooks
    □ Wrap all mutations in constraint check
    □ Before committing mutation:
        □ Apply mutation to temporary state
        □ Check affected constraints
        □ If hard constraint violated: reject, rollback
        □ If soft constraint violated: warn, proceed

□ Affected constraint detection
    □ Given a mutation, which constraints could be affected?
    □ Build constraintsByType index at compile time:
        □ For each constraint, extract type names from pattern
        □ Map<TypeName, Set<ConstraintDef>>
    □ On mutation: lookup type + ancestors in index
    □ edge<any> constraints checked on ALL edge mutations

□ Tests
    □ Layer 0 constraints enforced on ontology structure
    □ User constraints enforced on instance data
    □ Hard constraints reject mutations
    □ Soft constraints allow with warning
    □ Multiple constraints checked
    □ Only affected constraints checked (performance)

```

**Deliverable**: Constraints enforced, invalid mutations rejected

---

### Sprint 1.6: Ontology Parser (Week 13-14)

**Goal**: Parse ontology DSL to AST

**Tasks**:

```
□ Lexer
    □ Token types: keywords, identifiers, symbols, literals
    □ Tokenize(source: string) → Token[]
    □ Handle comments (--)
    □ Track line/column for error messages

□ AST definitions
    □ OntologyAST: { declarations: Declaration[] }
    □ NodeDeclAST: { name, parents, attributes }
    □ EdgeDeclAST: { name, signature, modifiers, attributes }
    □ ConstraintDeclAST: { name?, pattern, condition }
    □ RuleDeclAST: { name?, pattern, production }
    □ PatternAST, ExprAST, etc.

□ Parser
    □ Recursive descent parser
    □ parse(tokens: Token[]) → OntologyAST
    □ Match grammar from spec
    □ Error recovery: report multiple errors, don't stop at first

□ Error reporting
    □ ParseError: { message, line, column, source snippet }
    □ Collect errors, return all at end
    □ Helpful messages ("expected '}', found 'node'")

□ Tests
    □ Parse valid ontologies (all three test ontologies)
    □ Reject invalid syntax with clear errors
    □ Handle edge cases: empty ontology, trailing commas, etc.
    □ Comments ignored correctly
    □ Line/column tracking accurate

```

**Deliverable**: Can parse ontology source to AST

---

### Sprint 1.7: Ontology Compiler (Week 15-16)

**Goal**: Compile AST to Layer 0 graph structure

**Tasks**:

```
□ Compiler pipeline
    □ compile(ast: OntologyAST, graph: GraphStore) → CompileResult
    □ CompileResult: { success, errors, ontologyId }

□ Type compilation
    □ For each NodeDeclAST:
        □ Create _NodeType node
        □ Create _AttributeDef nodes
        □ Create _type_has_attribute edges
        □ Create _type_inherits edges (for parents)
    □ For each EdgeDeclAST:
        □ Create _EdgeType node
        □ Create _VarDef for each signature position
        □ Create _edge_has_position edges
        □ Create _var_has_type edges
        □ Create _AttributeDef nodes for edge attrs

□ Constraint compilation
    □ For each ConstraintDeclAST:
        □ Create _ConstraintDef node
        □ Compile pattern to _PatternDef structure
        □ Compile condition to _Expr structure
        □ Create connecting edges

□ Rule compilation
    □ For each RuleDeclAST:
        □ Create _RuleDef node
        □ Compile pattern to _PatternDef structure
        □ Compile production to _ProductionDef structure
        □ Create connecting edges

□ Pattern compilation
    □ patternToGraph(ast: PatternAST, graph) → ID (of _PatternDef)
    □ Create _VarDef for each variable
    □ Create _EdgePattern for each edge constraint
    □ Create _ConditionExpr for where clause

□ Expression compilation
    □ exprToGraph(ast: ExprAST, graph) → ID (of _Expr subtype)
    □ Recursive structure building

□ Validation
    □ After compilation, check all Layer 0 constraints
    □ Report errors for invalid ontology structure

□ Type registry update
    □ After successful compilation, register types in TypeRegistry
    □ Register constraints in ConstraintRegistry

□ Tests
    □ Compile test ontology A, verify graph structure
    □ Compile test ontology B, verify higher-order types
    □ Compile test ontology C, verify self-reference
    □ Invalid ontologies produce clear errors
    □ Types available for instance creation after compile

```

**Deliverable**: Can load ontology from source, use it for type-checked operations

---

### Sprint 1.8: Integration & Test Ontology A (Week 17-18)

**Goal**: End-to-end working system, first ontology complete

**Tasks**:

```
□ Integration test harness
    □ Load ontology from file
    □ Create instances
    □ Run queries
    □ Assert results

□ Test Ontology A: Task Dependencies
    □ Write ontology file (already specified)
    □ Load and compile
    □ Create sample data:
        □ 5-10 tasks
        □ 3-4 people
        □ Dependencies between tasks
        □ Assignments
    □ Test constraint: no self-dependency
    □ Test constraint: no short cycles
    □ Query: all tasks blocking task X
    □ Query: all tasks assigned to person P
    □ Query: all unblocked tasks (no incomplete upstream)

□ API refinement
    □ Clean up public API based on usage
    □ Document main entry points
    □ Error message improvement

□ Performance baseline
    □ Benchmark: insert N nodes
    □ Benchmark: insert N edges
    □ Benchmark: pattern match on graph of size N
    □ Establish baseline for future comparison

□ Documentation
    □ README with quickstart
    □ API documentation (TSDoc)
    □ Example usage

□ Bug fixes
    □ Fix issues discovered during integration
    □ Edge cases in type checking
    □ Edge cases in pattern matching

```

**Deliverable**: Phase 1 complete. Working in-memory HOHG with ontology support.

---

## Phase 1 Checkpoint

At end of Phase 1 (Week 18 / ~4.5 months):

| Component | Status |
| --- | --- |
| Core data structures | ✓ Complete |
| CRUD operations | ✓ Complete |
| Layer 0 type system | ✓ Complete |
| Pattern matching | ✓ Complete |
| Constraint checking | ✓ Complete |
| Ontology parser | ✓ Complete |
| Ontology compiler | ✓ Complete |
| Test Ontology A | ✓ Passing |

**What you can do**:

- Define ontologies in DSL
- Load and compile them
- Create typed instances
- Enforce constraints
- Query with pattern matching

**What's missing**:

- Persistence (in-memory only)
- Versioning
- Full query language
- Rewrite rule execution
- Higher-order edge tests (Ontology B)

---

## Phase 2: Persistence

### Sprint 2.1: Serialization (Week 19-20)

**Goal**: Convert graph to/from bytes

**Tasks**:

```
□ Serialization format design
    □ Choose format: JSON (readable) vs MessagePack (compact) vs custom
    □ Recommendation: JSON for v1 (debuggable), binary later
    □ Schema:
        {
          "version": 1,
          "nodes": { [id]: { type, attrs } },
          "edges": { [id]: { type, targets, attrs } }
        }

□ Serializer
    □ serialize(graph: GraphStore) → Uint8Array
    □ serializeNode(node: NodeData) → object
    □ serializeEdge(edge: EdgeData) → object
    □ Handle all scalar types correctly

□ Deserializer
    □ deserialize(bytes: Uint8Array) → GraphStore
    □ Validate structure
    □ Rebuild indexes
    □ Type check on load (optional, can defer to usage)

□ Incremental serialization
    □ serializeNode(id: ID) → Uint8Array (single node)
    □ serializeEdge(id: ID) → Uint8Array (single edge)
    □ For append-only log

□ Tests
    □ Roundtrip: serialize → deserialize = identity
    □ All scalar types preserved
    □ Higher-order edges preserved
    □ Large graphs (10K+ nodes)
    □ Invalid input rejected with clear error

```

**Deliverable**: Can save/load graph to bytes

---

### Sprint 2.2: Storage Engine (Week 21-23)

**Goal**: Durable storage with crash recovery

**Tasks**:

```
□ Storage architecture
    □ Write-ahead log (WAL) for durability
    □ Periodic snapshots for fast recovery
    □ Single-file or directory-based (choose: directory for simplicity)

□ File layout
    /db/
      manifest.json      # current snapshot version, WAL position
      snapshots/
        v001.json        # full graph state
        v002.json
      wal/
        000001.log       # append-only mutation log
        000002.log

□ Write-ahead log
    □ WALEntry: { seq, op: 'create_node'|'delete_node'|..., data }
    □ appendToWAL(entry: WALEntry) → Promise<void>
    □ Flush to disk before returning
    □ Rotate log files at size threshold

□ Snapshot
    □ createSnapshot() → Promise<void>
    □ Serialize entire graph
    □ Write atomically (write to temp, rename)
    □ Update manifest
    □ Optionally truncate old WAL entries

□ Recovery
    □ recover(dbPath: string) → GraphStore
    □ Load latest snapshot
    □ Replay WAL entries after snapshot
    □ Validate consistency

□ Storage API
    □ PersistentGraphStore extends GraphStore
    □ Constructor takes dbPath
    □ All mutations automatically logged to WAL
    □ Periodic auto-snapshot (configurable)

□ Tests
    □ Create, close, reopen → data preserved
    □ Crash simulation: kill after WAL write, before snapshot → recovers
    □ Large WAL replay performance
    □ Concurrent read during write (single writer, multiple readers)

```

**Deliverable**: Durable graph store that survives restarts

---

### Sprint 2.3: Indexing (Week 24-25)

**Goal**: Fast queries via indexes

**Tasks**:

```
□ Index types
    □ TypeIndex: Map<TypeName, Set<ID>> (already exists)
    □ EdgeSourceIndex: Map<ID, Set<EdgeID>> (edges by position 0)
    □ EdgeTargetIndex: Map<ID, Set<EdgeID>> (edges by any position)
    □ AttributeIndex: Map<TypeName, Map<AttrName, Map<Value, Set<ID>>>>

□ Index maintenance
    □ Update indexes on every mutation
    □ Indexes persisted (rebuild on load is acceptable for v1)
    □ Incremental update, not full rebuild

□ Ontology-aware indexing
    □ Given ontology, determine which indexes to build
    □ Index attributes mentioned in constraints/patterns
    □ Skip indexes for rarely-queried attributes

□ Query planner hook
    □ IndexHints: suggest which index to use for a pattern
    □ estimateCardinality(typeName) → number
    □ estimateSelectivity(attrName, value) → number

□ Tests
    □ Index speeds up type queries
    □ Index speeds up edge traversal
    □ Index speeds up attribute lookup
    □ Index stays consistent through mutations
    □ Benchmark: with vs without index

```

**Deliverable**: Fast indexed queries

---

### Sprint 2.4: Versioning (Week 26-28)

**Goal**: Git-like version control for graph

**Tasks**:

```
□ Version model
    □ VersionID: content hash of graph state
    □ Version metadata: { id, parent(s), timestamp, message? }
    □ Versions form a DAG (branches and merges)

□ Snapshot management
    □ createVersion(message?: string) → VersionID
    □ Compute hash of current state
    □ Store snapshot if not exists
    □ Record parent → child relationship

□ Version storage
    /db/
      versions/
        index.json       # version DAG
        {versionId}.json # snapshot (or delta?)

□ Checkout
    □ checkout(versionId: VersionID) → ReadOnlyGraphStore
    □ Load historical snapshot
    □ Read-only operations only
    □ Or: switch working state to historical version

□ Branching
    □ createBranch(name: string) → BranchID
    □ Branches are named pointers to versions
    □ switchBranch(name: string) → void

□ Diff
    □ diff(v1: VersionID, v2: VersionID) → Diff
    □ Diff: { addedNodes, deletedNodes, modifiedNodes, addedEdges, ... }
    □ Efficient diff using content hashes

□ Merge (basic)
    □ merge(sourceBranch: string) → MergeResult
    □ Three-way merge: base, source, target
    □ Conflict detection (same node modified differently)
    □ For v1: reject on conflict, require manual resolution

□ Tests
    □ Create versions, verify DAG structure
    □ Checkout old version, verify read-only
    □ Branch and switch
    □ Diff shows correct changes
    □ Simple merge succeeds
    □ Conflicting merge detected

```

**Deliverable**: Version-controlled graph with branches

---

### Sprint 2.5: Query Language (Week 29-31)

**Goal**: Declarative query language

**Tasks**:

```
□ Query syntax design
    MATCH pattern
    WHERE condition
    RETURN projection
    ORDER BY expr
    LIMIT n

    Example:
    MATCH t1: Task, t2: Task, depends_on(t1, t2)
    WHERE t1.completed = false
    RETURN t1.title, t2.title

□ Query AST
    □ QueryAST: { match: PatternAST, where?: ExprAST, return: ProjectionAST, ... }
    □ ProjectionAST: list of expressions with optional aliases

□ Query parser
    □ Extend lexer with query keywords
    □ parseQuery(source: string) → QueryAST
    □ Error reporting

□ Query planner
    □ plan(query: QueryAST, indexes: IndexHints) → QueryPlan
    □ QueryPlan: sequence of operations
    □ Operations: ScanType, ScanIndex, NestedLoop, Filter, Project
    □ Basic optimization: push filters down, use indexes

□ Query executor
    □ execute(plan: QueryPlan, graph: GraphStore) → Iterator<Row>
    □ Row: Map<string, Value>
    □ Streaming execution (don't materialize all results)

□ Aggregation (optional for v1)
    □ COUNT, SUM, COLLECT
    □ GROUP BY

□ Query API
    □ graph.query(queryString) → Iterator<Row>
    □ graph.queryOne(queryString) → Row | null

□ Tests
    □ Simple pattern query
    □ Query with conditions
    □ Query with projection
    □ Query with ordering and limit
    □ Higher-order query (edges about edges)
    □ Empty results
    □ Syntax errors reported clearly

```

**Deliverable**: Can query graph with SQL-like language

---

### Sprint 2.6: Transactions (Week 32-33)

**Goal**: ACID transactions

**Tasks**:

```
□ Transaction model
    □ Single-writer, multiple-reader
    □ Writers acquire exclusive lock
    □ Readers see consistent snapshot

□ Transaction API
    □ tx = graph.beginTransaction()
    □ tx.createNode(...), tx.createEdge(...), etc.
    □ tx.commit() → validates constraints, persists
    □ tx.rollback() → discards changes

□ Isolation
    □ MVCC: readers see snapshot at transaction start
    □ Writers see their own uncommitted changes
    □ On commit: check for conflicts with concurrent commits

□ Constraint checking in transactions
    □ Defer constraint checking to commit time
    □ Check all affected constraints
    □ Reject commit if any hard constraint violated

□ Write-ahead log integration
    □ Transaction writes go to WAL
    □ Commit = flush WAL
    □ Rollback = discard WAL entries

□ Deadlock prevention
    □ Single-writer model prevents deadlock
    □ Timeout on lock acquisition

□ Tests
    □ Transaction commit persists changes
    □ Transaction rollback discards changes
    □ Constraint violation rejects commit
    □ Concurrent readers see consistent state
    □ Writer blocks other writers

```

**Deliverable**: ACID transactions with constraint checking

---

### Sprint 2.7: Rule Engine (Week 34-36)

**Goal**: Execute rewrite rules

**Tasks**:

```
□ Rule representation
    □ Load _RuleDef from graph
    □ Extract pattern and production
    □ Compile to efficient representation

□ Rule matching
    □ findRuleMatches(rule: RuleDef, graph) → Iterator<Match>
    □ Reuse pattern matching infrastructure
    □ Track which matches are "new" (for auto-firing)

□ Production execution
    □ executeProduction(production: ProductionDef, match: Match, graph)
    □ Create nodes/edges as specified
    □ Delete nodes/edges as specified
    □ Set attributes as specified
    □ Bind new entities to variables

□ Rule scheduler
    □ RuleScheduler class
    □ On mutation:
        1. Identify rules that might match
        2. Find new matches
        3. Execute productions (highest priority first)
        4. Repeat until quiescence or cycle

□ Cycle detection
    □ Track (rule, match) pairs executed
    □ If repeat: cycle detected
    □ Options: stop, rollback, continue with limit

□ Manual vs auto rules
    □ auto=true: fire automatically on pattern match
    □ auto=false: fire only when explicitly invoked
    □ API: graph.fireRule(ruleName, bindings?)

□ Tests
    □ Simple rule fires on pattern match
    □ Rule creates new nodes/edges
    □ Rule deletes nodes/edges
    □ Transitive closure rule (causes transitivity)
    □ Cycle detection stops infinite loop
    □ Priority ordering respected
    □ Manual rules don't auto-fire

```

**Deliverable**: Rewrite rules execute automatically

---

### Sprint 2.8: Test Ontology B (Week 37-38)

**Goal**: Validate higher-order and rules

**Tasks**:

```
□ Test Ontology B: Causal Model
    □ Write complete ontology file
    □ Load and compile
    □ Create sample data:
        □ 10-20 events
        □ Causal relationships with mechanisms
        □ Confidence edges (higher-order)
        □ Evidence nodes

□ Test higher-order edges
    □ Create confidence(causes_edge, 0.8)
    □ Query edges by confidence level
    □ Query: "all causes with confidence > 0.5"

□ Test constraints
    □ Temporal constraint: causes must respect time
    □ Confidence range: 0.0 to 1.0
    □ Verify violations rejected

□ Test rules
    □ Transitive causation rule fires
    □ Confidence is multiplied correctly
    □ No duplicate transitive edges created

□ Performance test
    □ 1000 events, 5000 causal edges
    □ Measure rule execution time
    □ Measure query time

□ Bug fixes
    □ Issues discovered during testing

```

**Deliverable**: Higher-order edges and rules working correctly

---

### Sprint 2.9: Performance & Optimization (Week 39-41)

**Goal**: Production-ready performance

**Tasks**:

```
□ Benchmarking framework
    □ Standardized benchmarks
    □ Automated benchmark runs
    □ Track performance over time

□ Benchmarks
    □ Insert 100K nodes: target < 10s
    □ Insert 100K edges: target < 10s
    □ Pattern match (simple) on 100K graph: target < 100ms
    □ Pattern match (complex) on 100K graph: target < 1s
    □ Rule execution (1000 matches): target < 1s

□ Profiling
    □ Identify hot paths
    □ Memory allocation analysis
    □ Index effectiveness

□ Optimizations
    □ Pattern matching: better join ordering
    □ Index: B-tree for range queries
    □ Memory: object pooling for hot paths
    □ Serialization: binary format option

□ Rust port (critical paths)
    □ Identify performance-critical code
    □ Consider Rust rewrite for:
        □ Pattern matching
        □ Index structures
        □ Serialization
    □ WASM or native binding

□ Caching
    □ Query result caching
    □ Pattern match result caching
    □ Invalidation on mutation

□ Tests
    □ Benchmarks pass targets
    □ No memory leaks
    □ Large graph handling (1M nodes)

```

**Deliverable**: Performance suitable for production use

---

### Sprint 2.10: Test Ontology C & Polish (Week 42-44)

**Goal**: Self-describing ontology works, system polished

**Tasks**:

```
□ Test Ontology C: Self-Describing Knowledge Base
    □ Write complete ontology file
    □ Load and compile
    □ Create sample data:
        □ Concepts and instances
        □ Propositions with truth values
        □ Provenance edges (higher-order)
        □ Confidence edges

□ Test self-reference
    □ OntologyRef node pointing to own ontology
    □ Query: "what types does this ontology define?"
    □ Meta-reasoning about own structure

□ Test complex rules
    □ Confidence decay rule
    □ Symmetric relation rule
    □ Verify correct execution

□ Test contradiction constraint
    □ Create contradicting propositions
    □ Verify constraint prevents both being true

□ CLI tool
    □ hohg init <path> - create new database
    □ hohg load <ontology> - load ontology
    □ hohg query <query> - run query
    □ hohg stats - show database statistics
    □ hohg export <path> - export to JSON

□ Documentation
    □ Complete API documentation
    □ Ontology language reference
    □ Query language reference
    □ Tutorial: building your first ontology
    □ Architecture overview

□ Error messages
    □ Review all error messages
    □ Make them actionable
    □ Include context and suggestions

□ Final testing
    □ All three test ontologies pass
    □ Edge cases covered
    □ Performance benchmarks pass

```

**Deliverable**: Phase 2 complete. Production-ready HOHG database.

---

## Phase 2 Checkpoint

At end of Phase 2 (Week 44 / ~11 months):

| Component | Status |
| --- | --- |
| Serialization | ✓ Complete |
| Storage engine | ✓ Complete |
| Indexing | ✓ Complete |
| Versioning | ✓ Complete |
| Query language | ✓ Complete |
| Transactions | ✓ Complete |
| Rule engine | ✓ Complete |
| Test Ontology A | ✓ Passing |
| Test Ontology B | ✓ Passing |
| Test Ontology C | ✓ Passing |
| CLI tool | ✓ Complete |
| Documentation | ✓ Complete |

**What you can do**:

- Define and load ontologies
- Store large graphs durably
- Query with full language
- Execute rewrite rules
- Version and branch
- Transactional mutations
- Self-describing ontologies

**Milestone achieved**: Production-ready HOHG database engine.

---

## Summary: Week-by-Week

| Week | Sprint | Deliverable |
| --- | --- | --- |
| 1-2 | 1.0 | Project foundation |
| 3-4 | 1.1 | Core data structures |
| 5-6 | 1.2 | Basic operations |
| 7-8 | 1.3 | Layer 0 type system |
| 9-10 | 1.4 | Pattern matching |
| 11-12 | 1.5 | Constraint checking |
| 13-14 | 1.6 | Ontology parser |
| 15-16 | 1.7 | Ontology compiler |
| 17-18 | 1.8 | Integration, Test A |
| 19-20 | 2.1 | Serialization |
| 21-23 | 2.2 | Storage engine |
| 24-25 | 2.3 | Indexing |
| 26-28 | 2.4 | Versioning |
| 29-31 | 2.5 | Query language |
| 32-33 | 2.6 | Transactions |
| 34-36 | 2.7 | Rule engine |
| 37-38 | 2.8 | Test Ontology B |
| 39-41 | 2.9 | Performance |
| 42-44 | 2.10 | Test C, Polish |

---

## Risk Mitigation

| Risk | Mitigation |
| --- | --- |
| Pattern matching too slow | Early benchmarking, index optimization, Rust port |
| Type system too complex | Start minimal, extend only when needed |
| Storage bugs lose data | Extensive testing, WAL verification |
| Scope creep | Strict sprint goals, defer nice-to-haves |
| Single developer bottleneck | Document everything, clean code |

---
