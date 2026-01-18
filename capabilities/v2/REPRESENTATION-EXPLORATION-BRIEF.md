# MEW Representation Architecture: Exploration Brief

**Version:** 1.0  
**Status:** Design Exploration Phase  
**Audience:** AI Agent Engineering Team  
**Objective:** Iterate on REPRESENTATION.md to produce optimal architecture before implementation

---

## Executive Summary

MEW (Minimum Executable World) is a typed higher-order hypergraph database with declarative constraints and reactive rules. The current `REPRESENTATION.md` proposes a hybrid CPU/GPU architecture using columnar storage and sparse tensor representations.

**Scale Target**: Trillion-scale (10^12) edges per regional root world. The deployment model assumes:
- Multiple regional kernels (US, EU, ASIA) each with trillion-scale capacity
- Many small worlds, some medium, one very large root world per region
- Federation between regions (eventually consistent)
- Single global world is an anti-pattern (latency, consistency, availability)

**Critical Gap**: The current REPRESENTATION.md was designed for single-GPU scale. Trillion-scale requires a fundamentally different distributed architecture. This is the highest priority issue.

**Your mission**: 
1. Design a trillion-scale distributed architecture from first principles
2. Critically evaluate the current REPRESENTATION.md against this scale
3. Iterate to produce an optimal specification before implementation
4. Do not accept the current design as given — it is insufficient for target scale

---

## Part I: System Context

### 1.1 What is MEW?

MEW is a **self-describing higher-order hypergraph database** where:

- **Nodes** have types with inheritance hierarchies
- **Edges** are n-ary (hyperedges) connecting multiple nodes
- **Higher-order edges** can target other edges (meta-relationships)
- **Schema** is declared as an ontology and compiled before use
- **Constraints** are declarative patterns that must hold
- **Rules** fire automatically when patterns match, executing until quiescence
- **Worlds** provide nested scopes with independent schemas

```mew
ontology Example {
  node Task { title: String [required], status: String = "todo" }
  node Person { name: String [required] }
  
  edge assigned(task: Task, person: Person)
  edge confidence(about: edge<assigned>) { level: Float [>= 0.0, <= 1.0] }
  
  constraint done_needs_timestamp:
    t: Task WHERE t.status = "done" AND t.completed_at = null
    => FAIL "Completed tasks need completed_at"
  
  rule auto_archive:
    t: Task WHERE t.status = "done" => SET t.archived = true
}
```

### 1.2 Design Goals (Non-Negotiable)

| Goal | Rationale |
|------|-----------|
| **Higher-order edges** | Meta-relationships (confidence, provenance, annotations) are first-class |
| **Typed schema** | Compile-time knowledge enables optimization |
| **Pattern matching** | Core operation for queries, constraints, rules |
| **Declarative constraints** | "What must be true" not "how to enforce" |
| **Reactive rules** | Automatic transformations, fire to quiescence |
| **ACID transactions** | Correctness guarantees |
| **Nested worlds** | Scoped isolation for multi-agent systems |

### 1.3 Target Workloads

| Workload | Priority | Notes |
|----------|----------|-------|
| **Symbolic reasoning** | Primary | Knowledge graphs, causal modeling |
| **Simulation** | Primary | Rule-based world evolution |
| **Pattern analytics** | Primary | Complex pattern queries |
| **Real-time streaming** | Secondary | May require architectural changes |
| **High write throughput** | Secondary | Current design optimizes reads |
| **Cross-world transactions** | Deferred | Not in v1 scope |

### 1.4 Scale Targets

**Per-World Scale** (before recommending shard to multiple worlds):

| Metric | Target | Stretch | Hard Limit |
|--------|--------|---------|------------|
| Nodes | 10B | 100B | 1T |
| Edges | 100B | 1T | 10T |
| Higher-order edges | 10B | 100B | 1T |
| Pattern queries/sec | 100K | 1M | 10M |
| Point queries/sec | 10M | 100M | 1B |
| Mutations/sec | 1M | 10M | 100M |

**System-Wide Scale** (across all worlds):

| Metric | Target | Notes |
|--------|--------|-------|
| Worlds per kernel | 1M | Many small, few large |
| Small worlds | < 1M glyphs | Majority of worlds |
| Medium worlds | 1M - 1B glyphs | Common working size |
| Large worlds | 1B - 100B glyphs | Shared regional roots |
| Root worlds | 100B - 1T glyphs | Continental scale |

### 1.5 Deployment Model

**Expected World Distribution Pattern**:
```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TYPICAL DEPLOYMENT                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Regional Kernel (US-WEST)                                             │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                                                                  │   │
│   │   ROOT WORLD (Trillion-scale)                                   │   │
│   │   ├── Shared knowledge base                                     │   │
│   │   ├── Common ontologies                                         │   │
│   │   ├── Global entities (companies, places, concepts)            │   │
│   │   └── Multi-tenant, read-heavy                                  │   │
│   │                                                                  │   │
│   │   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐           │   │
│   │   │ Medium World │ │ Medium World │ │ Medium World │  ...      │   │
│   │   │ (Org A)      │ │ (Org B)      │ │ (Org C)      │           │   │
│   │   │ 1B glyphs    │ │ 500M glyphs  │ │ 2B glyphs    │           │   │
│   │   └──────────────┘ └──────────────┘ └──────────────┘           │   │
│   │         │                │                │                      │   │
│   │         ▼                ▼                ▼                      │   │
│   │   ┌─────┐┌─────┐  ┌─────┐┌─────┐  ┌─────┐┌─────┐┌─────┐       │   │
│   │   │Small││Small│  │Small││Small│  │Small││Small││Small│ ...   │   │
│   │   │World││World│  │World││World│  │World││World││World│       │   │
│   │   │(usr)││(usr)│  │(usr)││(usr)│  │(usr)││(usr)││(usr)│       │   │
│   │   └─────┘└─────┘  └─────┘└─────┘  └─────┘└─────┘└─────┘       │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘

Similar deployments in: US-EAST, EU-WEST, EU-CENTRAL, ASIA-PACIFIC, etc.
Federation layer connects regional kernels (eventually consistent).
```

**Anti-Pattern**: Single globally shared world across all regions
- Latency: Speed of light limits cross-continental round trips
- Consistency: Distributed transactions at global scale don't work
- Availability: Single point of failure

**Recommended Pattern**: Regional root worlds with federation
- Each continent/region has its own trillion-scale root
- Federation syncs between regions (eventually consistent)
- Cross-region queries route through federation layer
- Local reads are fast, cross-region writes are async

### 1.6 World Placement and Co-Location

**Principle**: Worlds are logical scopes. Physical placement is a scheduling decision.

**Typical Pattern**: Small/nested worlds co-locate with their parent for locality.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    WORLD PLACEMENT MODEL                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   PARIS DATACENTER                                                      │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                                                                  │   │
│   │   NODE 1 (8 GPUs, 640GB)                                        │   │
│   │   ┌───────────────────────────────────────────────────────────┐ │   │
│   │   │ ROOT WORLD (EU-WEST) - Partition Shard 0                  │ │   │
│   │   │ 5B edges of regional root                                  │ │   │
│   │   │                                                            │ │   │
│   │   │ CO-LOCATED SMALL WORLDS (same process, isolated memory):  │ │   │
│   │   │ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │ │   │
│   │   │ │ User    │ │ User    │ │ Org     │ │ Agent   │  ...     │ │   │
│   │   │ │ World   │ │ World   │ │ World   │ │ World   │ (1000s)  │ │   │
│   │   │ │ 10K     │ │ 50K     │ │ 1M      │ │ 100K    │          │ │   │
│   │   │ │ glyphs  │ │ glyphs  │ │ glyphs  │ │ glyphs  │          │ │   │
│   │   │ └─────────┘ └─────────┘ └─────────┘ └─────────┘          │ │   │
│   │   └───────────────────────────────────────────────────────────┘ │   │
│   │                                                                  │   │
│   │   NODE 2-200: More root world shards + co-located small worlds  │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   WORLD PLACEMENT RULES:                                                │
│   • Small worlds (< 1M glyphs): Co-locate on parent's node             │
│   • Medium worlds (1M-1B): Dedicated node or small cluster              │
│   • Large worlds (> 1B): Multi-node with partitioning                   │
│   • Nested worlds: Default co-locate with exterior node's world        │
│   • Hot worlds: Replicate to multiple nodes for read scaling            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Co-Location Benefits**:
| Benefit | Explanation |
|---------|-------------|
| Locality | Parent-child WATCH has zero network hop |
| Efficiency | Small worlds share GPU memory pool |
| Simplicity | Thousands of worlds in single process |
| Isolation | Logical separation despite physical co-location |

**World Mobility** (worlds can move):
```
WORLD LIFECYCLE:
  
  1. CREATE: World spawned, placed on local node
  2. GROW: World exceeds threshold, migrated to dedicated resources
  3. MOVE: World relocated for load balancing or locality
  4. ARCHIVE: Inactive world moved to cold storage
  5. RESTORE: Archived world brought back on-demand

MIGRATION PROTOCOL:
  • Snapshot world state
  • Transfer to target node(s)
  • Update routing table
  • Redirect traffic
  • Cleanup source
```

**Scheduling Considerations**:
| Factor | Policy |
|--------|--------|
| Size | Small → co-locate, Large → dedicated |
| Access pattern | Hot → replicate, Cold → archive |
| Parent location | Nested → prefer parent's node |
| User affinity | User worlds → user's nearest region |
| Compliance | Data residency → restrict to region |

**World Registry** (tracks world placement):
```
WorldRegistry {
  world_id → {
    size_class: small | medium | large,
    placement: [node_id, ...],  // can be multiple for replication
    parent: world_id?,
    region: region_id,
    status: active | migrating | archived,
  }
}
```

**Open Questions for World Placement**:
1. How to handle world growth? (Auto-migrate when threshold exceeded?)
2. What's the co-location limit per node? (Memory-bound: maybe 10K small worlds?)
3. How to balance isolation vs efficiency? (Process per world vs shared process?)
4. How to handle cross-node parent-child? (WATCH latency increases)

---

## Part II: Current Design Summary

### 2.1 Core Representation (REPRESENTATION.md)

**Unified Glyph Model**:
```
Glyph
├── id: GlobalId (64-bit)
├── type_tag: TypeId
├── alive: Bool (bitmap)
├── attributes: [Values] (columnar)
└── targets: [GlobalId]? (if edge)
```

**GlobalId Encoding — 64-bit IS SUFFICIENT** (validated analysis):

```
KEY INSIGHT: Edges don't cross world boundaries.
             Therefore, edge tensors only store WORLD-LOCAL IDs.
             Cross-world references are rare (projections only).

WITHIN-WORLD ID (64-bit):
┌────────────────┬────────────────────────────────────────────┐
│   Table ID     │              Local Index                    │
│   (16 bits)    │              (48 bits)                      │
└────────────────┴────────────────────────────────────────────┘
     65K types         281 trillion per table

Capacity:
  • 65,536 node/edge types per world (16 bits)
  • 281 trillion entities per type (48 bits = 2^48)
  • 18 quintillion total addressable per world
  • MORE than sufficient for trillion-scale

CROSS-WORLD REFERENCE (compound, for projections only):
┌─────────────────────────┬─────────────────────────────────────┐
│       World ID          │           Local ID                   │
│   (variable/64-bit)     │           (64-bit)                   │
└─────────────────────────┴─────────────────────────────────────┘
  • Only used for projection attributes
  • Resolved at federation layer
  • Rare compared to edge targets
```

**Why 64-bit Works**:
| Concern | Resolution |
|---------|------------|
| "Need trillion edges" | 48-bit index = 281T per type ✓ |
| "Need cross-world refs" | Edges don't cross worlds; projections use compound ID |
| "Storage cost" | 64-bit = half the cost of 128-bit at trillion scale |

**Storage Savings at Trillion Scale**:
- 1T edges × 2 targets × 8 bytes = 16 TB (64-bit)
- 1T edges × 2 targets × 16 bytes = 32 TB (128-bit)
- **Savings: 16 TB per trillion edges**

**The "no cross-world edges" rule has a 2x storage efficiency payoff.**

**Node Storage**: Family tables grouped by inheritance root
- All subtypes in one table
- Type tag column for discrimination
- Dense columns for root attributes, sparse for subtype-specific

**Edge Storage**: Per-type tensors
- Each edge type has dedicated tensor
- Stored as COO (Coordinate), indexed as CSR/CSC
- Attributes as additional columns

**Higher-Order Resolution**: Direct index lookup
- Edge targets can be EdgeIds
- GlobalId decoding gives O(1) access to target edge

### 2.2 Execution Model

**CPU/GPU Split**:
- CPU: Schema validation, mutation processing, orchestration
- GPU: Pattern matching, traversals, filtering, aggregation

**Sync Protocol**:
- Mutations batch into delta buffer (CPU-side)
- At tick/commit boundary: upload deltas to GPU
- GPU applies deltas, rebuilds indexes if needed
- Queries execute on GPU

**Sparse Matrix Operations**:
- Traversal: SpMV (sparse matrix-vector multiply)
- Multi-hop: SpMM or repeated SpMV
- Pattern join: Tensor contraction

### 2.3 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Columnar storage | GPU coalesced access, SIMD vectorization |
| Compiled ontology | Known shapes enable optimization |
| Per-type edge tensors | Homogeneous structure, efficient filtering |
| Family tables for nodes | Polymorphic queries without union |
| Tick-based sync | Batch amortizes overhead |
| Overflow buffers | Defer expensive index rebuilds |
| Worlds isolate edges | Enables distribution without distributed transactions |

---

## Part III: Identified Issues and Gaps

### 3.1 Critical Issues (Must Address)

#### Issue 0: Trillion-Scale Architecture (NEW — Not in REPRESENTATION.md)

**Problem**: The current REPRESENTATION.md assumes single-GPU or loosely-coupled multi-GPU. Trillion-scale requires a fundamentally different architecture.

**Scale Reality Check**:
```
1 Trillion edges × 64 bytes avg = 64 TB
Single H100 = 80 GB
Required: 800+ GPUs just for edge storage
```

**Architectural Implications**:

| Aspect | Single-GPU Design | Trillion-Scale Requirement |
|--------|-------------------|---------------------------|
| Storage | Single tensor | Distributed tensor shards |
| Index | CSR in one GPU | Partitioned CSR across cluster |
| Query | Local execution | Distributed query planning |
| Sync | CPU→GPU transfer | Cross-node communication |
| Failure | Rebuild from CPU | Replicated, fault-tolerant |

**What's Missing from REPRESENTATION.md**:

1. **Partitioning Strategy**: How to split a trillion-edge tensor across 1000+ GPUs?
   - Vertex-cut (edges partitioned, vertices replicated)
   - Edge-cut (vertices partitioned, edges follow)
   - Hybrid (hot vertices replicated, cold partitioned)

2. **Distributed Pattern Matching**: Patterns may span GPUs
   - Local pattern then shuffle?
   - Push-based (send patterns to data)?
   - Pull-based (fetch data to patterns)?

3. **Communication Primitives**: What operations between nodes?
   - All-gather (for broadcast)
   - All-reduce (for aggregation)
   - Point-to-point (for shuffle)
   - RDMA for low latency?

4. **Consistency at Scale**: With 1000+ nodes, what consistency model?
   - Eventual (fast but complex reasoning)
   - Causal (good for most workloads)
   - Strong (expensive, limited throughput)

5. **Failure Handling**: With 1000+ nodes, failures are constant
   - Replication factor (2x? 3x?)
   - Recovery protocol
   - Consistent hashing for rebalancing

**Exploration Tasks Added**:
- T1: Design trillion-scale partitioning strategy
- T2: Prototype distributed pattern matching
- T3: Evaluate communication libraries (NCCL, UCX, MPI)
- T4: Define consistency model for distributed execution

**This is the highest priority gap.** Everything else assumes we solve this.

---

#### Issue 1: CPU→GPU Synchronization Bottleneck

**Problem**: PCIe bandwidth (64 GB/s) is 47x slower than GPU HBM bandwidth (3 TB/s). All mutations flow through this bottleneck.

**Impact**: Write-heavy workloads limited by PCIe, not compute.

**Current Mitigation**: Batching at tick boundaries.

**Open Questions**:
- What mutation rate saturates PCIe?
- Can we do GPU-resident mutations with atomics?
- What's the tradeoff vs. validation complexity?

**Exploration Paths**:
1. Quantify bottleneck with benchmarks
2. Research GPU-resident graph mutation (cuGraph, Gunrock)
3. Design hybrid: simple mutations on GPU, complex on CPU

#### Issue 2: Higher-Order Join Random Access

**Problem**: Joining higher-order edges requires random lookups.

```
route_confidence tensor → target[0] is EdgeId → random lookup into via tensor
```

Random access destroys GPU performance (no coalescing).

**Open Questions**:
- What's the actual performance impact?
- Can we sort to enable merge joins?
- Should common patterns be materialized?

**Exploration Paths**:
1. Benchmark higher-order patterns at scale
2. Design sort-merge join for edge tensors
3. Explore denormalization / materialized views
4. Consider edge-centric vs vertex-centric models

#### Issue 3: Tick-Based Latency vs Throughput

**Problem**: Current design batches at tick boundaries, optimizing throughput but sacrificing latency.

**Impact**: 
- Real-time queries must wait for tick
- Interactive workloads may feel sluggish

**Open Questions**:
- What's acceptable latency for target workloads?
- Can we support both batch and streaming modes?
- How does continuous sync affect consistency?

**Exploration Paths**:
1. Define latency requirements per workload
2. Design continuous sync mode (no tick batching)
3. Research differential dataflow for incremental computation

### 3.2 Significant Gaps (Should Address)

#### Gap 1: Spatial/Embedding Index Strategy

**Current State**: "Grid-based or tree structure... LSH or brute force" — too vague.

**Why It Matters**: AI/ML workloads need efficient k-NN on high-dimensional embeddings (128-1024 dims).

**Required Decisions**:
- Which algorithm? HNSW, IVF-PQ, others?
- GPU-native implementation?
- Incremental updates vs rebuild?

**Exploration Paths**:
1. Survey state-of-art GPU ANN libraries (FAISS, RAFT)
2. Benchmark on representative embedding data
3. Design integration with tensor storage

#### Gap 2: Compression Strategy

**Current State**: Not mentioned.

**Why It Matters**: Columnar data compresses 3-10x with dictionary, RLE, delta encoding. More data fits in GPU memory.

**Required Decisions**:
- Compress on GPU or CPU?
- Decompress on-the-fly or materialize?
- Which algorithms? (LZ4, ZSTD, nvCOMP)

**Exploration Paths**:
1. Profile compression ratios on representative data
2. Benchmark GPU decompression overhead
3. Design compressed tensor format

#### Gap 3: Multi-GPU Strategy

**Current State**: Single GPU assumed. Federation for multi-kernel, but not multi-GPU.

**Why It Matters**: Large graphs exceed single GPU memory (80GB). NVLink enables 600+ GB/s between GPUs.

**Required Decisions**:
- Partitioning strategy (vertex-cut vs edge-cut)?
- Communication patterns for multi-hop queries?
- Memory placement (which data on which GPU)?

**Exploration Paths**:
1. Research distributed graph partitioning (METIS, etc.)
2. Study cuGraph multi-GPU implementation
3. Design partition-aware query planning

#### Gap 4: Cost-Based Query Planning

**Current State**: Join order and index hints mentioned, but no cost model.

**Why It Matters**: Without cardinality estimates, query plans are guesses. Wrong join order can be 1000x slower.

**Required Decisions**:
- What statistics to maintain?
- How to estimate pattern selectivity?
- Adaptive re-optimization?

**Exploration Paths**:
1. Define statistics schema (counts, histograms, samples)
2. Research graph query optimization (Neo4j, Dgraph, etc.)
3. Design cardinality estimator for typed patterns

#### Gap 5: Failure Recovery for GPU State

**Current State**: WAL and CPU-side recovery specified. GPU state unaddressed.

**Why It Matters**: GPU memory is volatile. Crash loses all GPU-resident data.

**Required Decisions**:
- Rebuild from CPU state on restart?
- Checkpoint GPU state periodically?
- How long does rebuild take?

**Exploration Paths**:
1. Quantify GPU state rebuild time at scale
2. Design checkpoint strategy for tensors
3. Consider persistent GPU memory (future hardware)

### 3.3 Design Tradeoffs (Need Explicit Decision)

#### Tradeoff 1: Generality vs Optimization

| More General | More Optimized |
|--------------|----------------|
| Dynamic schema | Static compiled schema |
| Any edge arity | Fixed arity per type |
| Heterogeneous patterns | Homogeneous batches |

**Question**: How much runtime flexibility do we sacrifice for performance?

#### Tradeoff 2: Consistency vs Performance

| Stronger Consistency | Better Performance |
|---------------------|-------------------|
| Immediate constraint check | Deferred at commit |
| Synchronous rule firing | Batched rule execution |
| Strong cross-world | Eventual projections |

**Question**: What consistency guarantees are required for target workloads?

#### Tradeoff 3: Memory vs Compute

| More Memory | More Compute |
|-------------|--------------|
| Materialized views | Computed on query |
| Denormalized higher-order | Join at runtime |
| Pre-built indexes | Scan and filter |

**Question**: What's the memory budget? Can we trade memory for speed?

---

## Part IV: Alternative Approaches to Explore

### 4.0 Architecture: Regional Deployment with Federation

**This is the recommended deployment model, not an alternative.**

**Concept**: Each region (US, EU, ASIA) runs independent trillion-scale kernels. Federation layer synchronizes between regions.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         GLOBAL ARCHITECTURE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   US-WEST Kernel          US-EAST Kernel          EU-WEST Kernel           │
│   ┌──────────────┐        ┌──────────────┐        ┌──────────────┐         │
│   │ Root World   │        │ Root World   │        │ Root World   │         │
│   │ (1T glyphs)  │        │ (800B glyphs)│        │ (1.2T glyphs)│         │
│   │              │        │              │        │              │         │
│   │ 200 nodes    │        │ 160 nodes    │        │ 240 nodes    │         │
│   │ 1600 GPUs    │        │ 1280 GPUs    │        │ 1920 GPUs    │         │
│   └──────┬───────┘        └──────┬───────┘        └──────┬───────┘         │
│          │                       │                       │                  │
│          └───────────────────────┼───────────────────────┘                  │
│                                  │                                          │
│                    ┌─────────────▼─────────────┐                            │
│                    │     FEDERATION LAYER      │                            │
│                    │  • Async replication      │                            │
│                    │  • Conflict resolution    │                            │
│                    │  • Cross-region routing   │                            │
│                    │  • Eventually consistent  │                            │
│                    └───────────────────────────┘                            │
│                                                                              │
│   ASIA-PACIFIC Kernel     ASIA-EAST Kernel                                 │
│   ┌──────────────┐        ┌──────────────┐                                 │
│   │ Root World   │        │ Root World   │                                 │
│   │ (600B glyphs)│        │ (900B glyphs)│                                 │
│   └──────────────┘        └──────────────┘                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Within-Region Architecture** (trillion-scale single kernel):

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    REGIONAL KERNEL (e.g., US-WEST)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     COORDINATOR LAYER                                │   │
│   │  • Query routing                                                    │   │
│   │  • Transaction coordination                                         │   │
│   │  • Partition metadata                                               │   │
│   │  • Load balancing                                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│            ┌───────────────────────┼───────────────────────┐                │
│            ▼                       ▼                       ▼                │
│   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│   │  PARTITION 0    │    │  PARTITION 1    │    │  PARTITION N    │        │
│   │  (Node Group 0) │    │  (Node Group 1) │    │  (Node Group N) │        │
│   │                 │    │                 │    │                 │        │
│   │  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │        │
│   │  │ GPU 0-7   │  │    │  │ GPU 0-7   │  │    │  │ GPU 0-7   │  │        │
│   │  │ 640 GB    │  │    │  │ 640 GB    │  │    │  │ 640 GB    │  │        │
│   │  │           │  │    │  │           │  │    │  │           │  │        │
│   │  │ ~5B edges │  │    │  │ ~5B edges │  │    │  │ ~5B edges │  │        │
│   │  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │        │
│   │                 │    │                 │    │                 │        │
│   │  Local pattern  │    │  Local pattern  │    │  Local pattern  │        │
│   │  matching       │    │  matching       │    │  matching       │        │
│   └────────┬────────┘    └────────┬────────┘    └────────┬────────┘        │
│            │                      │                      │                  │
│            └──────────────────────┼──────────────────────┘                  │
│                                   │                                         │
│                    ┌──────────────▼──────────────┐                          │
│                    │     INTERCONNECT FABRIC     │                          │
│                    │  InfiniBand 400G / NVLink   │                          │
│                    │  For cross-partition comm   │                          │
│                    └─────────────────────────────┘                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key Design Decisions for Regional Deployment**:

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| Within-region consistency | Strong (linearizable) | Single datacenter, low latency |
| Cross-region consistency | Eventually consistent | Speed of light, availability |
| Partition strategy | Hybrid vertex-cut | Balance load, minimize communication |
| Replication factor | 3x within region | Standard fault tolerance |
| Cross-region replication | Async, conflict-free | CRDT-based for certain types |

**Open Questions for Federation**:
1. What's the conflict resolution strategy for concurrent cross-region writes?
2. How do cross-region queries work? Route to each region and merge?
3. What data is replicated vs federated? (Hot entities replicated, cold federated)
4. How do we handle region failures? (Failover to nearest region?)

---

### 4.1 Alternative: GPU-Resident Mutations

**Concept**: Instead of CPU mutations + GPU sync, do mutations directly on GPU.

**How It Works**:
- Allocate mutation queue in GPU memory
- GPU kernel processes queue, applies to tensors
- Use atomics for concurrent access
- CPU only validates schema, enqueues

**Pros**:
- Eliminates PCIe bottleneck for writes
- Lower latency mutations
- Better GPU utilization

**Cons**:
- Complex atomics for sparse structures
- Harder to debug
- Constraint checking on GPU is complex

**Exploration Task**: Prototype GPU-resident edge insertion, measure throughput vs current design.

### 4.2 Alternative: Differential Dataflow Integration

**Concept**: Track changes incrementally, propagate deltas through computation graph.

**How It Works**:
- Each operator (filter, join, project) is incremental
- Changes flow through, updating only affected results
- Rules become streaming operators

**Pros**:
- Sub-millisecond latency for incremental updates
- No tick boundary needed
- Natural fit for reactive rules

**Cons**:
- Complex implementation
- Memory overhead for change tracking
- May not parallelize as well on GPU

**Exploration Task**: Research Materialize, Noria, Differential Dataflow. Assess applicability to MEW.

### 4.3 Alternative: Adjacency List with GPU Batching

**Concept**: Store graph as adjacency lists, batch-convert to tensors for queries.

**How It Works**:
- Mutations update adjacency lists (fast, localized)
- Before query, convert hot subgraph to CSR tensor
- Query executes on tensor
- Result converts back

**Pros**:
- Fast mutations (no index rebuild)
- Only materialize what's queried
- Familiar model

**Cons**:
- Conversion overhead
- May not amortize for complex queries
- Memory for both representations

**Exploration Task**: Benchmark conversion overhead vs query benefit.

### 4.4 Alternative: Learned Indexes

**Concept**: Replace B-trees/hash indexes with ML models that predict location.

**How It Works**:
- Train model on data distribution
- Model predicts approximate location
- Small local search refines

**Pros**:
- Smaller memory footprint
- Can be GPU-accelerated
- Adapts to data distribution

**Cons**:
- Update cost (retraining)
- Accuracy vs lookup tradeoff
- Research-stage for graphs

**Exploration Task**: Survey learned index literature, assess applicability to typed hypergraphs.

---

## Part V: Exploration Tasks

### 5.0 Trillion-Scale Tasks (HIGHEST PRIORITY)

| ID | Task | Purpose | Deliverable |
|----|------|---------|-------------|
| T1 | Design trillion-scale partitioning strategy | Distribute data across 1000+ GPUs | Partitioning algorithm spec |
| T2 | Prototype distributed pattern matching | Patterns that span partitions | Working prototype + benchmarks |
| T3 | Evaluate communication libraries | Cross-node data movement | NCCL vs UCX vs MPI comparison |
| T4 | Define consistency model | Correctness guarantees | Formal consistency spec |
| T5 | Design failure/recovery protocol | Fault tolerance | Replication + recovery spec |
| T6 | Prototype trillion-edge synthetic dataset | Test at scale | Data generator + 1T edge test set |
| T7 | Benchmark cross-partition query overhead | Communication cost | Latency breakdown by pattern type |
| T8 | Design global index structure | Efficient routing | Distributed index spec |
| T9 | Design world placement/scheduling system | Co-location, migration | Placement algorithm spec |
| T10 | Prototype co-located small worlds | Many worlds per node | Memory/perf benchmarks |
| T11 | Design world migration protocol | Mobility | Migration state machine |

**These tasks block all other work.** A trillion-scale architecture is fundamentally different from single-node.

### 5.1 Benchmarking Tasks

| ID | Task | Purpose | Deliverable |
|----|------|---------|-------------|
| B1 | Generate synthetic hypergraph at scale targets | Baseline data | 10B node, 100B edge dataset |
| B2 | Measure CPU→GPU transfer throughput | Quantify sync bottleneck | Bandwidth at various batch sizes |
| B3 | Profile higher-order join patterns | Random access impact | Latency breakdown |
| B4 | Compare CSR vs COO vs dense for various sparsities | Format selection | Crossover points |
| B5 | Benchmark existing GPU graph libraries | Competitive baseline | cuGraph, Gunrock numbers |
| B6 | Benchmark InfiniBand vs Ethernet for graph workloads | Network selection | Throughput/latency comparison |
| B7 | Profile NCCL collective operations at scale | Multi-GPU communication | All-reduce, all-gather numbers |

### 5.2 Design Tasks

| ID | Task | Purpose | Deliverable |
|----|------|---------|-------------|
| D1 | Formalize cost model for pattern queries | Query optimization | Cost function + statistics schema |
| D2 | Design multi-GPU partitioning strategy | Scale beyond single GPU | Partition algorithm + placement policy |
| D3 | Specify compression format for tensors | Memory efficiency | Compressed tensor spec |
| D4 | Design spatial index integration | Embedding support | HNSW/IVF integration spec |
| D5 | Define GPU failure recovery protocol | Reliability | Recovery procedure + time bounds |

### 5.3 Research Tasks

| ID | Task | Purpose | Deliverable |
|----|------|---------|-------------|
| R1 | Survey GPU-resident graph mutation techniques | Write optimization | Literature review + applicability assessment |
| R2 | Study differential dataflow systems | Streaming mode | Architecture comparison |
| R3 | Analyze graph partitioning algorithms | Distribution | METIS, streaming partitioning review |
| R4 | Review GPU ANN libraries | Spatial indexes | FAISS, RAFT feature comparison |
| R5 | Investigate RDMA for multi-node | Network optimization | Feasibility assessment |

### 5.4 Prototype Tasks

| ID | Task | Purpose | Deliverable |
|----|------|---------|-------------|
| P1 | GPU-resident edge insertion kernel | Test alternative | Working CUDA code + benchmarks |
| P2 | Higher-order join with sort-merge | Test optimization | Implementation + comparison |
| P3 | Compressed COO format | Test compression | Format + decompression kernel |
| P4 | Simple pattern matcher on GPU | Validate tensor approach | End-to-end prototype |

---

## Part VI: Decision Framework

### 6.1 Evaluation Criteria

When comparing alternatives, evaluate on:

| Criterion | Weight | Metric |
|-----------|--------|--------|
| **Query throughput** | High | Patterns/second at scale targets |
| **Query latency** | High | P99 latency for interactive queries |
| **Write throughput** | Medium | Mutations/second |
| **Memory efficiency** | Medium | Bytes per glyph |
| **Implementation complexity** | Medium | Lines of code, maintainability |
| **Scalability** | High | Linear scaling to 100+ GPUs |
| **Failure recovery** | Medium | Time to recover, data loss risk |
| **Flexibility** | Low | Runtime schema changes |

### 6.2 Decision Gates

**Gate 0: Trillion-Scale Architecture Decision** (BLOCKING)
- Input: T1-T8 results, study of TAO/ByteGraph/Spanner
- Decision: 
  - Partitioning strategy (vertex-cut vs edge-cut vs hybrid)
  - Communication model (push vs pull vs hybrid)
  - Consistency model (strong vs eventual vs causal)
  - ID encoding (64-bit vs 128-bit vs hierarchical)
- Deadline: **Before any other design work**
- This gate blocks all others — trillion-scale changes everything

**Gate 1: Sync Model Decision**
- Input: B2, R1, P1 results
- Decision: CPU-mutate-GPU-query vs GPU-resident mutations vs hybrid
- Deadline: After Gate 0

**Gate 2: Storage Format Decision**
- Input: B4, D3, P3 results
- Decision: CSR vs COO vs adaptive, compression yes/no
- Deadline: After Gate 0

**Gate 3: Regional Deployment Decision**
- Input: T4, T5, federation research
- Decision: 
  - Cross-region consistency model
  - Federation protocol
  - Conflict resolution strategy
- Deadline: Before multi-region design

**Gate 4: Index Strategy Decision**
- Input: D1, D4, R4 results
- Decision: Cost model, spatial index algorithm
- Deadline: Before query optimizer implementation

**Gate 5: Failure Handling Decision**
- Input: T5, hardware reliability data
- Decision:
  - Replication factor
  - Recovery protocol
  - Checkpoint strategy
- Deadline: Before production deployment

---

## Part VII: Constraints and Assumptions

### 7.1 Hardware Assumptions

**Single Node** (baseline unit):

| Component | Value | Notes |
|-----------|-------|-------|
| GPUs per node | 8 (H100/B200) | NVLink-connected |
| GPU memory | 80GB per GPU, 640GB total | Or 192GB with B200 |
| GPU bandwidth | 3 TB/s per GPU | HBM3 |
| NVLink bandwidth | 900 GB/s | GPU-to-GPU |
| PCIe bandwidth | 64 GB/s (5.0 x16) | CPU-GPU |
| CPU cores | 64-128 | For orchestration |
| System memory | 1-2 TB | For overflow, indexes |
| NVMe storage | 30+ TB | For persistence |

**Cluster Scale** (for trillion-scale worlds):

| Metric | Calculation | Result |
|--------|-------------|--------|
| Memory per trillion glyphs | ~100 bytes/glyph avg | 100 TB |
| GPUs needed | 100 TB / 80 GB | 1,250 GPUs |
| Nodes needed | 1,250 / 8 | ~160 nodes |
| Network fabric | InfiniBand 400G | Required |
| Storage | 100 TB × 3 (replicas) | 300+ TB NVMe |

**Regional Deployment** (trillion-scale root world):

| Component | Quantity | Purpose |
|-----------|----------|---------|
| Compute nodes | 200+ | GPU compute |
| Storage nodes | 50+ | Persistent storage |
| Network switches | 20+ | InfiniBand fabric |
| Total GPUs | 1,600+ | Pattern matching |
| Total memory | 120+ TB GPU, 400+ TB CPU | Working set |

### 7.2 Software Constraints

| Constraint | Rationale |
|------------|-----------|
| Rust implementation | Project decision, performance + safety |
| CUDA for GPU | Industry standard, best tooling |
| No external DB dependency | MEW is the database |
| Open source libraries OK | cuSPARSE, Thrust, etc. |

### 7.3 Non-Goals (Out of Scope)

- Real-time guarantees (hard deadlines)
- SQL compatibility
- Distributed consensus (v2+)
- Dynamic schema modification (META mode v2+)
- Complex amplitude / quantum branching (v2+)

---

## Part VIII: Success Criteria

### 8.1 Phase 1: Trillion-Scale Foundation

- [ ] **T1-T8 tasks complete** — trillion-scale architecture is the prerequisite
- [ ] Gate 0 decision made: partitioning, communication, consistency, ID encoding
- [ ] Proof-of-concept: 100B edge distributed pattern matching
- [ ] Study complete: TAO, ByteGraph, Spanner architecture patterns
- [ ] Hardware requirements validated: GPU cluster sizing for trillion-scale

### 8.2 Phase 2: Regional Architecture

- [ ] Regional deployment model fully specified
- [ ] Federation protocol designed (cross-region sync)
- [ ] Failure handling and recovery protocols defined
- [ ] All benchmarking tasks (B1-B7) complete with data
- [ ] All design tasks (D1-D5) have draft specifications
- [ ] Gate 1-3 decisions made

### 8.3 Phase 3: Detailed Design Complete

- [ ] REPRESENTATION.md v2 addresses all critical issues
- [ ] All significant gaps have specified solutions
- [ ] All tradeoffs have explicit documented decisions
- [ ] Performance model predicts meeting scale targets:
  - [ ] 1T edges per regional root world
  - [ ] 1M pattern queries/sec
  - [ ] 10M mutations/sec
  - [ ] < 10ms P99 latency for local queries
- [ ] Architecture review by distributed systems experts
- [ ] Comparison with TAO/ByteGraph validates approach

### 8.4 Phase 4: Ready for Implementation

- [ ] Component interfaces fully specified
- [ ] Data structures and formats defined (128-bit IDs, partition scheme)
- [ ] Communication protocols specified (NCCL/UCX integration)
- [ ] Error handling and edge cases documented
- [ ] Test strategy defined (including chaos engineering for failures)
- [ ] Implementation order determined
- [ ] Prototype at 1B scale validates design

---

## Part IX: Reference Materials

### 9.1 Current Specifications

| Document | Location | Relevance |
|----------|----------|-----------|
| REPRESENTATION.md | `capabilities/v2/REPRESENTATION.md` | Primary design doc |
| architecture.md | `specs/architecture.md` | System architecture |
| DSL specs | `specs/core/3_DSL.md` | Query language |
| Layer 0 | `specs/core/2_LAYER0.md` | Schema representation |

### 9.2 External References

**GPU Graph Libraries**:

| Resource | Source | Relevance |
|----------|--------|-----------|
| cuGraph | NVIDIA RAPIDS | GPU graph library, multi-GPU support |
| Gunrock | UC Davis | GPU graph analytics |
| GraphBLAS | graphblas.org | Sparse linear algebra for graphs |
| FAISS | Meta Research | GPU ANN search |
| nvCOMP | NVIDIA | GPU compression |

**Distributed Computing Frameworks**:

| Resource | Purpose | Notes |
|----------|---------|-------|
| NCCL | Multi-GPU collective ops | Standard for GPU clusters |
| UCX | Unified communication | RDMA, GPU-direct |
| Ray | Distributed compute | Actor model, object store |
| Dask | Distributed arrays | Python, scales to clusters |
| Apache Arrow Flight | Data transfer | Zero-copy, columnar |

**Incremental/Streaming**:

| Resource | Purpose | Notes |
|----------|---------|-------|
| Differential Dataflow | Incremental computation | Timely dataflow model |
| Materialize | Streaming SQL | Built on DD |
| Flink | Stream processing | Exactly-once, stateful |
| Kafka | Event streaming | High-throughput messaging |

### 9.3 Academic Papers

| Paper | Topic |
|-------|-------|
| "Gunrock: A High-Performance Graph Processing Library on the GPU" | GPU graph processing |
| "GraphBLAS: Building Blocks for Graph Algorithms" | Sparse matrix graph ops |
| "HNSW: Efficient and Robust ANN" | Spatial indexing |
| "Differential Dataflow" (McSherry) | Incremental computation |
| "The Case for Learned Index Structures" | Learned indexes |

---

## Part X: Appendix

### A.0 Key Changes from Current REPRESENTATION.md

**The current spec is a starting point, not a final design.** Major revisions needed:

| Current Design | Required for Trillion-Scale |
|----------------|----------------------------|
| Single GPU assumed | 1000+ GPU distributed cluster |
| 32-bit local index (4B max) | 48-bit index (281T) — 64-bit total ✓ |
| CPU→GPU sync via PCIe | Cross-node sync via InfiniBand/NVLink |
| Single CSR index | Partitioned CSR across cluster |
| Local pattern matching | Distributed pattern matching |
| Single coordinator | Distributed coordination |
| WAL on local disk | Replicated, distributed WAL |
| No partitioning | Vertex-cut or edge-cut partitioning |
| Eventual consistency OK | Strong within-region, eventual cross-region |
| Implicit world placement | Explicit world scheduling/co-location |

**What to Keep** (validated for trillion-scale):
- Glyph abstraction (unified node/edge)
- Compiled ontology → known shapes
- Columnar storage principle
- Sparse tensor representation
- **World isolation (no cross-world edges)** — THIS ENABLES 64-BIT IDs
- Higher-order edge via ID reference
- **64-bit world-local IDs** — sufficient (48-bit index = 281T per type)

**What to Redesign**:
- ~~GlobalId encoding~~ → **64-bit is fine** (cross-world via compound ref)
- Storage layer (distributed tensors)
- Sync protocol (cross-node communication)
- Query execution (distributed pattern matching)
- Index structures (partitioned, replicated)
- Failure handling (replication, recovery)
- World placement and scheduling (NEW)

### A.1 Glossary

| Term | Definition |
|------|------------|
| **Glyph** | Unified abstraction for nodes and edges |
| **GlobalId** | 64-bit identifier encoding world, table, index |
| **Family Table** | Columnar storage for node type hierarchy |
| **Edge Tensor** | Sparse matrix storage for edge type |
| **Tick** | Logical time step, sync boundary |
| **World** | Scoped database with independent schema |
| **Projection** | Interior node referencing exterior by ID |
| **Quiescence** | State where no more rules fire |

### A.2 Current Design Diagrams

**Storage Architecture**:
```
┌─────────────────────────────────────────────────────────────┐
│                         WORLD                                │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │  Family Table   │  │  Family Table   │  ... (per root)  │
│  │  (Node Type A)  │  │  (Node Type B)  │                   │
│  │  ┌───┬───┬───┐  │  │  ┌───┬───┬───┐  │                   │
│  │  │id │tag│...│  │  │  │id │tag│...│  │                   │
│  │  └───┴───┴───┘  │  │  └───┴───┴───┘  │                   │
│  └─────────────────┘  └─────────────────┘                   │
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │  Edge Tensor    │  │  Edge Tensor    │  ... (per type)  │
│  │  (causes)       │  │  (assigned)     │                   │
│  │  ┌───┬───┬───┐  │  │  ┌───┬───┬───┐  │                   │
│  │  │src│dst│...│  │  │  │t0 │t1 │...│  │                   │
│  │  └───┴───┴───┘  │  │  └───┴───┴───┘  │                   │
│  │  + CSR index    │  │  + CSR index    │                   │
│  └─────────────────┘  └─────────────────┘                   │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                    Delta Buffer                          ││
│  │  [mutation1, mutation2, ...]  → sync to GPU at tick     ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

**Execution Flow**:
```
Statement → Parse → Analyze → [Query | Mutation]
                                   │         │
                                   │         ▼
                                   │    ┌─────────┐
                                   │    │  Delta  │
                                   │    │ Buffer  │
                                   │    └────┬────┘
                                   │         │ (at tick)
                                   ▼         ▼
                              ┌──────────────────┐
                              │   GPU Storage    │
                              │  (tensors, CSR)  │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌──────────────────┐
                              │ Pattern Matching │
                              │    (SpMV, etc)   │
                              └────────┬─────────┘
                                       │
                                       ▼
                                   Results
```

### A.3 Contact and Escalation

For questions about:
- **MEW language semantics**: See `specs/` directory
- **Current implementation**: See `mew/` Rust workspace
- **Project direction**: Escalate to project lead

---

*End of Exploration Brief v1.0*

---

**Next Steps for Agent Team**:

1. Read `REPRESENTATION.md` in full
2. Read `specs/architecture.md` for system context
3. Begin with benchmarking tasks (B1-B2) to establish baselines
4. Parallelize research tasks (R1-R5)
5. Report findings and recommendations at each decision gate
