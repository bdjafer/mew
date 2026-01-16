```markdown
# MEW Engine: Hybrid CPU/GPU Storage Architecture

**Version:** 1.0  
**Status:** Foundation  
**Scope:** Storage model, execution model, and data representation

---

## Part I: Context and Motivation

### 1.1 The Problem

MEW is a higher-order hypergraph database where:
- Nodes have types with inheritance hierarchies
- Edges are n-ary (hyperedges) with fixed arity per type
- Edges can connect to other edges (higher-order)
- Schema is declared as an ontology and compiled before use
- Worlds (nested scopes) provide isolation with their own local ontologies
- Queries involve pattern matching, traversals, and tensor-like operations

The challenge: design a storage architecture that serves both CPU and GPU execution efficiently, handles the full feature set (time, space, policies, meta-operations, federation), and remains conceptually coherent.

### 1.2 The Key Insight

**Compiled ontology transforms the problem.**

Unlike generic graph databases where any node can connect to any node with any edge type, MEW knows at compile time:
- All node types and their inheritance relationships
- All edge types, their arities, and their target type constraints
- All constraints and rules as patterns

This foreknowledge collapses an arbitrary hypergraph problem into a structured tensor collection problem with known shapes.

### 1.3 The Core Tension Resolved

A naive approach suggests two storage representations (CPU-optimized and GPU-optimized) with synchronization between them. This creates consistency complexity.

The resolution: **one logical model, one physical layout, partitioned by type structure**. CPU and GPU access the same data. CPU owns mutation. GPU owns bulk query. Synchronization is batched at transaction boundaries.

---

## Part II: Core Principles

### 2.1 Principle: Everything is an Entity

At the conceptual level, nodes and edges are both entities:

```
Entity
├── id: GlobalId           -- unique, immutable
├── type_tag: TypeId       -- concrete type
├── alive: Bool            -- existence flag
├── attributes: [Values]   -- columnar
└── targets: [GlobalId]?   -- if present, this is an edge
```

An edge is simply an entity with targets. This unification enables:
- Higher-order edges (edge targets are just GlobalIds of other edges)
- Uniform ID space for cross-references
- Consistent mutation semantics

### 2.2 Principle: Physical Storage Follows Type Structure

While the logical model is unified, physical storage is partitioned:

**Nodes:** Grouped by inheritance family. All types sharing a root ancestor are stored together in one family table. Type discrimination is a column, not a separate table.

**Edges:** Grouped by edge type. Each edge type has its own tensor. Edge types have no inheritance, so each tensor is homogeneous.

This partitioning reflects the ontology structure:
- Node inheritance exists → family tables with type tags
- Edge inheritance does not exist → per-type tensors

### 2.3 Principle: Columnar Layout

All storage is columnar:
- Each attribute is a dense array
- Subtype-specific attributes are sparse (nulls for other types)
- Edge targets are dense arrays of GlobalIds
- Bitmaps track entity liveness

Columnar layout enables:
- Coalesced GPU memory access
- CPU cache efficiency and SIMD vectorization
- Efficient projection (select subset of columns)

### 2.4 Principle: CPU Mutates, GPU Queries

Mutation (SPAWN, KILL, LINK, UNLINK, SET) is CPU-side:
- Validates against schema
- Appends to delta buffer
- Updates CPU-side state immediately (read-your-writes)

Query execution is GPU-side:
- Pattern matching as tensor operations
- Traversals as sparse matrix operations
- Filtering as parallel predicate evaluation

Synchronization happens at transaction/tick boundaries:
- Deltas batched and uploaded to GPU
- Parallel scatter-writes by type
- Indexes rebuilt when overflow exceeds threshold

### 2.5 Principle: Edges Stay Inside

Cross-scope references (between worlds/interiors) use ID attributes, not edges. A projection is an interior node that holds a `represents: GlobalId` attribute pointing to an external entity.

No edge crosses a world boundary. This maintains:
- Clean isolation semantics
- Independent schema per world
- Locality for query execution

---

## Part III: Data Model

### 3.1 GlobalId Space

Every entity (node or edge) across all worlds has a unique GlobalId. The ID space is partitioned:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GLOBAL ID SPACE                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   GlobalId: 64-bit integer                                          │
│                                                                      │
│   ┌──────────────┬──────────────┬──────────────────────┐            │
│   │   World ID   │  Table ID    │     Local Index      │            │
│   │   (16 bits)  │  (16 bits)   │      (32 bits)       │            │
│   └──────────────┴──────────────┴──────────────────────┘            │
│                                                                      │
│   This encoding enables:                                            │
│   • O(1) lookup: GlobalId → (World, Table, Index)                   │
│   • No hash table for resolution                                    │
│   • Efficient range queries per table                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Node Storage: Family Tables

Nodes are stored in family tables. One table per inheritance root.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         FAMILY TABLE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Family: Event (contains Event, BusinessEvent, PersonalEvent)      │
│                                                                      │
│   ┌─────────┬───────────┬───────────┬──────────┬─────────────┐      │
│   │  alive  │ type_tag  │ timestamp │   name   │ contract_id │      │
│   │ (bitmap)│  (enum)   │  (i64)    │ (string) │  (string?)  │      │
│   ├─────────┼───────────┼───────────┼──────────┼─────────────┤      │
│   │    1    │   Event   │  1700000  │  "init"  │    null     │      │
│   │    1    │  Business │  1700001  │  "sign"  │   "C-42"    │      │
│   │    0    │   Event   │  1700002  │  "void"  │    null     │  ←dead│
│   │    1    │  Personal │  1700003  │  "call"  │    null     │      │
│   └─────────┴───────────┴───────────┴──────────┴─────────────┘      │
│       ↑          ↑           ↑           ↑           ↑              │
│    bitmap    dense col   dense col  dense col  sparse col           │
│                                                                      │
│   • Root type attributes: dense (all rows have values)              │
│   • Subtype attributes: sparse (null for other types)               │
│   • type_tag: concrete type for filtering                           │
│   • alive: bitmap for existence                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Query compilation:**

```
MATCH e: Event WHERE e.timestamp > X
```

Compiles to: scan Event family table, no type filter (all rows are Events or subtypes).

```
MATCH e: BusinessEvent WHERE e.contract_id = "C-42"
```

Compiles to: scan Event family table, filter `type_tag = BusinessEvent`, filter `contract_id = "C-42"`.

### 3.3 Edge Storage: Type Tensors

Each edge type has its own tensor. No inheritance means homogeneous structure.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         EDGE TENSOR                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Edge Type: causes(Event, Event) with attribute 'mechanism'        │
│   Arity: 2                                                          │
│                                                                      │
│   ┌─────────┬───────────┬───────────┬───────────┐                   │
│   │  alive  │ target[0] │ target[1] │ mechanism │                   │
│   │ (bitmap)│ (GlobalId)│ (GlobalId)│  (string) │                   │
│   ├─────────┼───────────┼───────────┼───────────┤                   │
│   │    1    │   1001    │   1002    │ "trigger" │                   │
│   │    1    │   1002    │   1003    │ "cascade" │                   │
│   │    1    │   1001    │   1003    │ "direct"  │                   │
│   └─────────┴───────────┴───────────┴───────────┘                   │
│                                                                      │
│                                                                      │
│   Edge Type: via(LandmarkRef, LandmarkRef, LandmarkRef)             │
│   Arity: 3 (hyperedge)                                              │
│                                                                      │
│   ┌─────────┬───────────┬───────────┬───────────┬───────┐           │
│   │  alive  │ target[0] │ target[1] │ target[2] │ order │           │
│   ├─────────┼───────────┼───────────┼───────────┼───────┤           │
│   │    1    │   2001    │   2002    │   2003    │   1   │           │
│   │    1    │   2002    │   2003    │   2004    │   2   │           │
│   └─────────┴───────────┴───────────┴───────────┴───────┘           │
│                                                                      │
│                                                                      │
│   Edge Type: route_confidence(edge<via>) -- higher-order            │
│   Arity: 1 (target is an edge ID)                                   │
│                                                                      │
│   ┌─────────┬───────────┬───────┐                                   │
│   │  alive  │ target[0] │ level │   target[0] is a via edge's ID   │
│   ├─────────┼───────────┼───────┤                                   │
│   │    1    │   6001    │  0.8  │                                   │
│   │    1    │   6002    │  0.6  │                                   │
│   └─────────┴───────────┴───────┘                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.4 Edge Tensor as Sparse Matrix

A binary edge type is mathematically a sparse adjacency matrix:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TENSOR VIEW OF EDGES                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   causes(Event, Event) as adjacency matrix:                         │
│                                                                      │
│              1001  1002  1003  1004                                  │
│         1001  [ 0    1     1     0  ]                                │
│         1002  [ 0    0     1     0  ]                                │
│         1003  [ 0    0     0     0  ]                                │
│         1004  [ 0    0     0     0  ]                                │
│                                                                      │
│   Stored as COO: [(1001,1002), (1001,1003), (1002,1003)]            │
│                                                                      │
│   Also indexed as CSR (for forward traversal):                      │
│     row_ptr: [0, 2, 3, 3, 3]                                        │
│     col_idx: [1002, 1003, 1003]                                     │
│                                                                      │
│   And CSC (for reverse traversal):                                  │
│     col_ptr: [0, 0, 1, 3, 3]                                        │
│     row_idx: [1001, 1001, 1002]                                     │
│                                                                      │
│                                                                      │
│   N-ary edge types are N-dimensional sparse tensors in COO format.  │
│                                                                      │
│   Operations become tensor algebra:                                 │
│   • Traversal: SpMV (sparse matrix-vector multiply)                 │
│   • Multi-hop: SpMM or repeated SpMV                                │
│   • Pattern join: tensor contraction                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.5 Higher-Order Edge Resolution

Higher-order edges target other edges by GlobalId. Resolution is direct array lookup.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    HIGHER-ORDER PATTERN                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Pattern: MATCH via(a,b,c) AS v, route_confidence(v) AS rc         │
│                                                                      │
│   Execution:                                                        │
│   1. Iterate route_confidence tensor                                │
│   2. For each row, target[0] is a GlobalId of a via edge            │
│   3. Decode GlobalId → (world, via_table, local_index)              │
│   4. Lookup via tensor at that index → get (a, b, c)                │
│   5. Emit match: (a, b, c, v, rc)                                   │
│                                                                      │
│   ┌──────────────────┐         ┌──────────────────┐                 │
│   │ route_confidence │         │       via        │                 │
│   ├──────────────────┤         ├──────────────────┤                 │
│   │ target[0]: 6001 ─┼────────▶│ idx 0: (a,b,c)   │                 │
│   │ level: 0.8       │         │ edge_id: 6001    │                 │
│   └──────────────────┘         └──────────────────┘                 │
│                                                                      │
│   GlobalId encoding makes this a direct index calculation,          │
│   not a hash lookup.                                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part IV: World Structure

### 4.1 World as Scoped Storage

Each world (including ROOT) contains:
- A compiled ontology (types, constraints, rules)
- Storage tables (family tables + edge tensors)
- Time state (local tick counter)
- Space definitions (position columns + spatial indexes)
- Policy set
- Child worlds

```
┌─────────────────────────────────────────────────────────────────────┐
│                         WORLD STRUCTURE                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   World                                                             │
│   ├── id: WorldId                                                   │
│   ├── parent: World?                  -- null for ROOT              │
│   ├── exterior_id: GlobalId?          -- this world's ID in parent  │
│   │                                                                  │
│   ├── schema: CompiledOntology                                      │
│   │   ├── node_types: [NodeTypeSchema]                              │
│   │   ├── edge_types: [EdgeTypeSchema]                              │
│   │   ├── constraints: [CompiledConstraint]                         │
│   │   └── rules: [CompiledRule]                                     │
│   │                                                                  │
│   ├── storage: WorldStorage                                         │
│   │   ├── node_tables: Map<FamilyId, FamilyTable>                   │
│   │   ├── edge_tables: Map<EdgeTypeId, EdgeTensor>                  │
│   │   └── pending_deltas: [Delta]                                   │
│   │                                                                  │
│   ├── time: TimeState                                               │
│   │   ├── local_tick: u64                                           │
│   │   ├── mode: shared | independent | ratio(N)                     │
│   │   └── ratio: u32?                                               │
│   │                                                                  │
│   ├── spaces: Map<SpaceId, Space>                                   │
│   ├── policies: [Policy]                                            │
│   └── children: Map<GlobalId, World>                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 World Nesting

Worlds nest arbitrarily. Each interior is a complete, isolated database.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         WORLD NESTING                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT                                                              │
│   ├── schema: (Landmark, Event, causes, ...)                        │
│   ├── storage: (family tables, edge tensors)                        │
│   │                                                                  │
│   ├── Navigator #nav1 [has_interior]                                │
│   │   ├── exterior: node in ROOT with attributes                    │
│   │   └── interior: World                                           │
│   │       ├── schema: (LandmarkRef, Route, via, ...)                │
│   │       ├── storage: (own family tables, edge tensors)            │
│   │       │                                                          │
│   │       └── PlanningContext #ctx1 [has_interior]                  │
│   │           ├── exterior: node in nav1 interior                   │
│   │           └── interior: World                                   │
│   │               ├── schema: (Hypothesis, Evaluation, ...)         │
│   │               └── storage: (own tables)                         │
│   │                                                                  │
│   └── Navigator #nav2 [has_interior]                                │
│       └── interior: World (independent from nav1)                   │
│                                                                      │
│   Each world has independent:                                       │
│   • Schema (can shadow parent type names)                           │
│   • Storage (own tensors)                                           │
│   • Time (own tick, configurable relationship to parent)            │
│   • Rules and constraints                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.3 Cross-World References: Projections

Worlds reference external entities through projections, not edges.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         PROJECTION PATTERN                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT                               NAVIGATOR INTERIOR             │
│   ────                               ──────────────────             │
│                                                                      │
│   ┌────────────────┐                 ┌────────────────┐             │
│   │ Landmark #1001 │                 │ LandmarkRef    │             │
│   │ name: "Tower"  │◀ ─ ─ ─ ─ ─ ─ ─ ─│ represents:1001│             │
│   │ pos: [10,20,0] │   (ID, not edge)│ local_name:"T" │             │
│   └────────────────┘                 │ confidence:0.8 │             │
│                                      └────────────────┘             │
│                                             │                        │
│   No edge crosses the boundary.             │ edge (interior only)  │
│   LandmarkRef.represents is an              ▼                        │
│   attribute holding a GlobalId.      ┌────────────────┐             │
│                                      │ Route          │             │
│   The projection can be stale.       │ name: "North"  │             │
│   This models real perception.       └────────────────┘             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part V: Mutation Model

### 5.1 Delta Buffer

All mutations append to a delta buffer. The buffer is processed at sync boundaries.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DELTA STRUCTURE                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Delta                                                             │
│   ├── op: SPAWN | KILL | LINK | UNLINK | SET                        │
│   ├── entity_id: GlobalId                                           │
│   ├── type_id: TypeId | EdgeTypeId                                  │
│   ├── local_idx: u32                  -- slot in table              │
│   ├── targets: [GlobalId]?            -- for LINK                   │
│   ├── attr: AttrId?                   -- for SET                    │
│   └── value: Value?                   -- for SET, SPAWN attrs       │
│                                                                      │
│                                                                      │
│   Mutation Flow                                                     │
│   ─────────────                                                     │
│                                                                      │
│   1. Validate against compiled schema                               │
│   2. Allocate GlobalId and local slot                               │
│   3. Append Delta to pending buffer                                 │
│   4. Update CPU-side state (for read-your-writes)                   │
│   5. At sync boundary: batch upload to GPU                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Entity Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                       ENTITY LIFECYCLE                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   SPAWN                                                             │
│   ─────                                                             │
│   • Validate: type not abstract, required attrs present             │
│   • Allocate: slot from free list or grow table                     │
│   • Assign: GlobalId from world's ID range                          │
│   • Write: type_tag, attributes, set alive bit                      │
│   • For edges: validate target types, store target GlobalIds        │
│                                                                      │
│   KILL                                                              │
│   ────                                                              │
│   • Validate: no edges reference this entity (or cascade)           │
│   • Clear: alive bit                                                │
│   • Reclaim: add slot to free list                                  │
│   • Unregister: remove from ID mapping                              │
│                                                                      │
│   SET                                                               │
│   ───                                                               │
│   • Validate: attribute exists, type matches                        │
│   • If unique: check constraint, update index                       │
│   • Write: new value to column                                      │
│                                                                      │
│   Deleted entities leave tombstones (alive=0). Periodic             │
│   compaction reclaims space and rebuilds indexes.                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.3 Edge Tensor Mutation

Edge tensors require index maintenance. The overflow buffer pattern defers expensive rebuilds.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EDGE TENSOR MUTATION                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   EdgeTensor                                                        │
│   ├── base: indexed region                                          │
│   │   ├── targets: [GlobalId × arity]     -- sorted by target[0]   │
│   │   ├── attrs: [columns]                                          │
│   │   ├── alive: bitmap                                             │
│   │   └── csr_index: (row_ptr, col_idx)   -- for traversal         │
│   │                                                                  │
│   └── overflow: unindexed region                                    │
│       ├── targets: [GlobalId × arity]     -- append-only           │
│       ├── attrs: [columns]                                          │
│       └── alive: bitmap                                             │
│                                                                      │
│                                                                      │
│   LINK: append to overflow                                          │
│   UNLINK: clear alive bit (in base or overflow)                     │
│                                                                      │
│   Query: use CSR index on base, linear scan overflow                │
│                                                                      │
│   When overflow exceeds threshold:                                  │
│   • Merge base + overflow                                           │
│   • Filter dead entries                                             │
│   • Rebuild CSR index                                               │
│   • Reset overflow to empty                                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part VI: Query Model

### 6.1 Pattern Compilation

Patterns compile to tensor operations at ontology compile time.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PATTERN COMPILATION                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Source Pattern:                                                   │
│   MATCH a: Event, b: Event, causes(a, b) WHERE a.timestamp < b.timestamp
│                                                                      │
│   Compiled Plan:                                                    │
│   1. SCAN causes_tensor                    -- iterate edges         │
│   2. RESOLVE target[0] → a_idx             -- decode GlobalId       │
│   3. RESOLVE target[1] → b_idx             -- decode GlobalId       │
│   4. LOAD Event_table.timestamp[a_idx]     -- attribute access      │
│   5. LOAD Event_table.timestamp[b_idx]                              │
│   6. FILTER timestamp_a < timestamp_b      -- predicate             │
│   7. EMIT (a, b, edge_id)                  -- output                │
│                                                                      │
│   Type inheritance compiles to type_tag set membership:             │
│   MATCH e: Event  →  type_tag ∈ {Event, BusinessEvent, ...}        │
│                                                                      │
│   Higher-order compiles to join on edge IDs:                        │
│   MATCH via(a,b,c) AS v, confidence(v)  →  join via.id = conf.target│
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 Traversal as SpMV

Multi-hop traversals become sparse matrix operations.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TRAVERSAL AS TENSOR ALGEBRA                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Query: MATCH a: Event -[causes*2..4]-> b: Event                   │
│                                                                      │
│   Let A = adjacency matrix of causes edges                          │
│   Let v = one-hot vector for starting node(s)                       │
│                                                                      │
│   Hop 1: v₁ = A × v                                                 │
│   Hop 2: v₂ = A × v₁ = A² × v                                       │
│   Hop 3: v₃ = A × v₂ = A³ × v                                       │
│   Hop 4: v₄ = A × v₃ = A⁴ × v                                       │
│                                                                      │
│   Result: v₂ ∪ v₃ ∪ v₄ (reachable in 2-4 hops)                      │
│                                                                      │
│   This is standard GPU-accelerated SpMV using cuSPARSE.             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.3 Batch Query Execution

Queries of the same shape batch into single kernel launches.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    BATCH EXECUTION                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   100 queries: MATCH causes($start_i, b) RETURN b                   │
│                                                                      │
│   Naive: 100 kernel launches                                        │
│   Batched: 1 kernel launch, 100 thread blocks                       │
│                                                                      │
│   Each thread block:                                                │
│   • Loads its start_id from batch input array                       │
│   • Performs CSR lookup for that start                              │
│   • Writes results to its output region                             │
│                                                                      │
│   Batching amortizes kernel launch overhead and maximizes           │
│   GPU occupancy.                                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part VII: Time Model

### 7.1 Logical Time per World

Each world has an independent logical clock.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TIME MODEL                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   TimeState                                                         │
│   ├── local_tick: u64             -- this world's tick count        │
│   ├── mode: TimeMode                                                │
│   │   ├── shared                  -- tick with parent               │
│   │   ├── independent             -- tick only when explicit        │
│   │   └── ratio(N)                -- tick once per N parent ticks   │
│   └── ratio: u32?                                                   │
│                                                                      │
│                                                                      │
│   Tick Propagation                                                  │
│   ────────────────                                                  │
│                                                                      │
│   ROOT ticks                                                        │
│     │                                                                │
│     ├── World A [shared]         →  ticks                           │
│     │     └── World A1 [ratio(2)]  →  ticks every 2nd A tick        │
│     │                                                                │
│     ├── World B [independent]    →  does not tick                   │
│     │                                                                │
│     └── World C [ratio(10)]      →  ticks every 10th ROOT tick      │
│                                                                      │
│                                                                      │
│   Functions                                                         │
│   ─────────                                                         │
│   logical_time()        →  current world's tick                     │
│   logical_time(PARENT)  →  parent world's tick                      │
│   logical_time(ROOT)    →  root tick                                │
│   wall_time()           →  system clock (global)                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 Sync at Tick Boundary

GPU synchronization aligns with tick boundaries:

```
tick(world):
    1. Flush pending deltas to GPU
    2. Check constraints (pattern match on GPU)
    3. Execute rules (pattern match + production)
    4. Increment local_tick
    5. Process subscriptions (diff + notify)
    6. Propagate tick to children (based on time mode)
```

---

## Part VIII: Space Model

### 8.1 Spatial Positioning

Entities can be positioned in named spaces. Position is stored as additional columns.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         SPACE MODEL                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Space                                                             │
│   ├── id: SpaceId                                                   │
│   ├── name: String                                                  │
│   ├── dimensions: u32                                               │
│   └── metric: euclidean | cosine | manhattan | custom               │
│                                                                      │
│                                                                      │
│   Position Storage (extends FamilyTable)                            │
│   ──────────────────────────────────────                            │
│                                                                      │
│   FamilyTable<Landmark>                                             │
│   ├── ... standard columns ...                                      │
│   ├── pos_Physical: [f32 × 3]         -- position in Physical space │
│   ├── has_pos_Physical: bitmap        -- which entities have pos    │
│   ├── pos_Semantic: [f32 × 128]       -- position in Semantic space │
│   └── has_pos_Semantic: bitmap                                      │
│                                                                      │
│                                                                      │
│   Spatial Index                                                     │
│   ─────────────                                                     │
│   For distance queries: grid-based or tree structure                │
│   For nearest-neighbor: approximate (LSH) or exact (brute force)    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 8.2 Spatial Queries

```
distance(a, b, Physical) < 100    →  compute Euclidean distance
nearest(target, k, Semantic)      →  k-NN in Semantic space
```

These compile to GPU kernels operating on position columns.

---

## Part IX: Constraint and Rule Execution

### 9.1 Constraint Model

Constraints are compiled patterns with conditions. Checked at commit.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CONSTRAINT EXECUTION                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CompiledConstraint                                                │
│   ├── pattern: CompiledPattern                                      │
│   ├── condition: CompiledExpr         -- must be true               │
│   ├── hard: bool                      -- hard=reject, soft=warn     │
│   └── affected_types: [TypeId]        -- for incremental checking   │
│                                                                      │
│                                                                      │
│   Execution at Commit                                               │
│   ───────────────────                                               │
│                                                                      │
│   1. Identify constraints affected by pending deltas                │
│   2. For each affected constraint:                                  │
│      a. Execute pattern match on GPU → matches                      │
│      b. Evaluate condition for each match on GPU → bools            │
│      c. If any bool is false and hard=true → reject transaction     │
│      d. If any bool is false and hard=false → emit warning          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 9.2 Rule Model

Rules are compiled patterns with productions. Execute until quiescence.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    RULE EXECUTION                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CompiledRule                                                      │
│   ├── pattern: CompiledPattern                                      │
│   ├── production: CompiledProduction  -- actions to execute         │
│   ├── priority: i32                   -- higher fires first         │
│   └── auto: bool                      -- auto-fire on match         │
│                                                                      │
│                                                                      │
│   Execution at Tick                                                 │
│   ─────────────────                                                 │
│                                                                      │
│   1. Collect auto-firing rules sorted by priority                   │
│   2. Track fired (rule, match) pairs for cycle detection            │
│   3. Loop until no progress:                                        │
│      a. For each rule:                                              │
│         i.   Execute pattern match on GPU → matches                 │
│         ii.  For each new match (not in fired set):                 │
│              - Execute production (generates deltas)                │
│              - Add (rule, match) to fired set                       │
│              - Mark progress                                        │
│      b. If progress, continue loop                                  │
│   4. If same (rule, match) encountered twice → cycle detected       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part X: CPU/GPU Execution Split

### 10.1 Responsibility Division

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EXECUTION SPLIT                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CPU RESPONSIBILITIES                                              │
│   ────────────────────                                              │
│   • Schema validation                                               │
│   • Mutation processing (SPAWN, KILL, LINK, UNLINK, SET)            │
│   • Delta buffering                                                 │
│   • Transaction management                                          │
│   • ID allocation                                                   │
│   • Constraint/rule orchestration                                   │
│   • Policy evaluation (for mutations)                               │
│   • Query planning                                                  │
│   • Result marshaling                                               │
│                                                                      │
│   GPU RESPONSIBILITIES                                              │
│   ────────────────────                                              │
│   • Bulk pattern matching                                           │
│   • Traversal (SpMV, SpMM)                                          │
│   • Predicate filtering                                             │
│   • Aggregation                                                     │
│   • Spatial queries                                                 │
│   • Batch delta application                                         │
│   • Index rebuild (CSR construction)                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 10.2 Synchronization

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SYNC PROTOCOL                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   VERSION TRACKING                                                  │
│   ────────────────                                                  │
│   cpu_version: increments on each mutation                          │
│   gpu_version: increments on each sync                              │
│                                                                      │
│                                                                      │
│   SYNC TRIGGER                                                      │
│   ────────────                                                      │
│   At tick/commit boundary:                                          │
│   if cpu_version > gpu_version:                                     │
│       sync_deltas_to_gpu()                                          │
│       gpu_version = cpu_version                                     │
│                                                                      │
│                                                                      │
│   DELTA APPLICATION                                                 │
│   ─────────────────                                                 │
│   1. Group deltas by target table                                   │
│   2. For each table (parallel streams):                             │
│      a. Upload delta batch                                          │
│      b. Scatter-write to columns                                    │
│      c. Update alive bitmaps                                        │
│   3. For edge tables with overflow threshold exceeded:              │
│      a. Merge and rebuild CSR index                                 │
│   4. Synchronize all streams                                        │
│                                                                      │
│                                                                      │
│   CONSISTENCY GUARANTEE                                             │
│   ─────────────────────                                             │
│   Within a transaction: read-your-writes (from CPU state)           │
│   After commit: GPU state reflects all committed mutations          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part XI: Policy Model

### 11.1 Policy Structure

Policies are predicates that filter observations or gate mutations.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    POLICY MODEL                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Policy                                                            │
│   ├── operations: [MATCH | SPAWN | KILL | LINK | UNLINK | SET]      │
│   ├── scope_filter: ScopeSpec?        -- which scopes policy applies│
│   ├── condition: CompiledExpr         -- when policy applies        │
│   ├── action: ALLOW | DENY                                          │
│   └── priority: i32                   -- evaluation order           │
│                                                                      │
│                                                                      │
│   Evaluation                                                        │
│   ──────────                                                        │
│                                                                      │
│   For observations (MATCH):                                         │
│   • Policies compile to additional filter predicates                │
│   • Results filtered, not gated (empty result, not error)           │
│                                                                      │
│   For mutations (SPAWN, KILL, etc):                                 │
│   • Policies evaluated before applying delta                        │
│   • First matching DENY → reject                                    │
│   • First matching ALLOW → permit                                   │
│   • No match → default deny                                         │
│                                                                      │
│                                                                      │
│   Context Functions                                                 │
│   ─────────────────                                                 │
│   current_scope()      →  world executing the operation             │
│   current_actor()      →  identity performing the operation         │
│   target_scope()       →  world being accessed                      │
│   parent_of(scope)     →  parent world                              │
│   owner_of(scope)      →  exterior node of a world                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part XII: Meta Operations

### 12.1 Dynamic Schema Modification

META operations modify the compiled ontology at runtime.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    META OPERATIONS                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   META CREATE NODE                                                  │
│   ────────────────                                                  │
│   1. Parse and validate type declaration                            │
│   2. Determine family (new root or existing parent)                 │
│   3. Add type to schema registry                                    │
│   4. If new family: allocate new FamilyTable                        │
│   5. If existing family: add columns for new attributes             │
│   6. Allocate GPU memory for new columns                            │
│   7. Recompile affected patterns (constraints, rules)               │
│                                                                      │
│   META CREATE EDGE                                                  │
│   ────────────────                                                  │
│   1. Parse and validate edge declaration                            │
│   2. Add edge type to schema registry                               │
│   3. Allocate new EdgeTensor                                        │
│   4. Allocate GPU memory                                            │
│   5. Recompile affected patterns                                    │
│                                                                      │
│                                                                      │
│   Recompilation Scope                                               │
│   ───────────────────                                               │
│   META in world W recompiles only W's patterns.                     │
│   Other worlds are unaffected.                                      │
│   This enables independent evolution of interiors.                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part XIII: Subscriptions

### 13.1 Subscription as Perception

Subscriptions implement cross-scope perception (sensors).

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SUBSCRIPTION MODEL                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Subscription                                                      │
│   ├── subscriber: World               -- who is watching            │
│   ├── target: World                   -- what is being watched      │
│   ├── pattern: CompiledPattern        -- what to watch for          │
│   ├── on_create: CompiledProduction   -- handler for new matches    │
│   ├── on_update: CompiledProduction   -- handler for changed matches│
│   ├── on_delete: CompiledProduction   -- handler for removed matches│
│   └── last_matches: Set<MatchId>      -- state for diff computation │
│                                                                      │
│                                                                      │
│   Execution at Tick                                                 │
│   ─────────────────                                                 │
│   1. Re-execute pattern on target world → current_matches           │
│   2. Compute diff:                                                  │
│      • created = current_matches - last_matches                     │
│      • deleted = last_matches - current_matches                     │
│      • updated = changed attributes in intersection                 │
│   3. Execute handlers in subscriber's context                       │
│   4. Update last_matches                                            │
│                                                                      │
│   Handlers typically create/update/delete projections.              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part XIV: Versioning

### 14.1 Snapshot and Checkout

Versioning enables time-travel queries and branching.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    VERSION MODEL                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Snapshot                                                          │
│   ├── id: VersionId                                                 │
│   ├── created_at: Timestamp                                         │
│   ├── parent: VersionId?              -- for delta chains           │
│   └── storage: frozen or delta                                      │
│                                                                      │
│                                                                      │
│   Operations                                                        │
│   ──────────                                                        │
│                                                                      │
│   snapshot()                                                        │
│   • Freeze current state                                            │
│   • Return VersionId                                                │
│                                                                      │
│   checkout(version)                                                 │
│   • Load historical state (read-only)                               │
│   • Either full copy or reconstruct from delta chain                │
│                                                                      │
│   diff(v1, v2)                                                      │
│   • Return set of deltas between versions                           │
│                                                                      │
│   branch(name)                                                      │
│   • Create new branch from current state                            │
│                                                                      │
│   merge(branch)                                                     │
│   • Find common ancestor                                            │
│   • Compute deltas                                                  │
│   • Detect conflicts                                                │
│   • Apply if no conflicts                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part XV: Federation

### 15.1 Cross-Kernel References

Federation connects independent MEW instances (kernels).

```
┌─────────────────────────────────────────────────────────────────────┐
│                    FEDERATION MODEL                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Each kernel is an independent ROOT.                               │
│   Worlds exist within a kernel.                                     │
│   Federation bridges kernels.                                       │
│                                                                      │
│   ┌──────────────────┐         ┌──────────────────┐                 │
│   │    KERNEL A      │         │    KERNEL B      │                 │
│   │                  │         │                  │                 │
│   │   ROOT_A         │◄───────►│   ROOT_B         │                 │
│   │   ├── World 1    │ bridge  │   ├── World 3    │                 │
│   │   └── World 2    │         │   └── World 4    │                 │
│   │                  │         │                  │                 │
│   └──────────────────┘         └──────────────────┘                 │
│                                                                      │
│                                                                      │
│   Remote Reference                                                  │
│   ────────────────                                                  │
│   Cross-kernel references are attributes, not edges:                │
│   • remote_kernel: String                                           │
│   • remote_id: GlobalId                                             │
│   • sync_id: String (for federation protocol)                       │
│                                                                      │
│   Federation subscriptions enable cross-kernel perception.          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part XVI: Summary

### 16.1 Core Abstractions

| Abstraction | Definition |
|-------------|------------|
| **Entity** | Anything with identity: nodes and edges unified |
| **GlobalId** | Universal identifier encoding world, table, index |
| **FamilyTable** | Columnar storage for node inheritance hierarchy |
| **EdgeTensor** | Sparse tensor storage per edge type |
| **World** | Scoped database with own schema and storage |
| **Projection** | Interior entity referencing exterior by ID attribute |
| **Delta** | Atomic mutation record |
| **CompiledPattern** | Tensor operation plan from query/constraint/rule |

### 16.2 Invariants

1. **Edges stay inside.** No edge crosses world boundaries.
2. **Schema is compiled.** All types known before instance operations.
3. **Nodes inherit, edges do not.** Family tables for nodes, per-type tensors for edges.
4. **CPU mutates, GPU queries.** Clear responsibility division.
5. **Sync at boundaries.** Deltas batched, applied atomically.
6. **Projections can be stale.** This models real perception.
7. **GlobalId encodes location.** No hash lookup for resolution.

### 16.3 Design Qualities

**Unified:** One conceptual model (Entity) regardless of node/edge distinction.

**Partitioned:** Physical storage follows type structure for efficiency.

**Columnar:** All attributes stored as dense arrays for memory coalescing.

**Tensor-native:** Edge operations are sparse matrix operations.

**Incrementally mutable:** Overflow buffers defer expensive index rebuilds.

**Hierarchically scoped:** Worlds nest with independent schemas and storage.

**Temporally aware:** Per-world logical time with configurable relationships.

**Spatially aware:** Position columns with spatial indexes per space.

---

*This architecture enables MEW to execute as a high-performance hybrid CPU/GPU database while maintaining the full expressiveness of higher-order hypergraphs with nested scopes, temporal dynamics, spatial positioning, and dynamic schema evolution.*
```