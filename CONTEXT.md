# HOHG: Higher-Order Hypergraph System

## Context Document

**Version:** 1.0
**Status:** Foundation
**Purpose:** Entry point to the HOHG project — vision, philosophy, and orientation

---

# Part I: Vision

## 1.1 The Problem

Current approaches to artificial intelligence fall into two failure modes:

**Symbolic systems** (knowledge graphs, ontologies, rule engines) are:
- Brittle: They break when encountering situations outside their designed scope
- Static: They cannot learn or adapt their own structure
- Labor-intensive: Humans must hand-engineer all knowledge

**Neural systems** (deep learning, LLMs) are:
- Opaque: No stable, inspectable representation of knowledge
- Ungrounded: Symbols have no fixed meaning, leading to drift
- Inefficient: Require massive compute for simple reasoning
- Unreliable: Cannot guarantee constraints or invariants

Neither approach alone produces systems that can:
- Learn new concepts and relationships
- Maintain stable, inspectable knowledge
- Guarantee behavioral constraints
- Reason reliably across domains
- Improve themselves without catastrophic drift

**The gap:** We lack a substrate that combines the learnability of neural systems with the stability and inspectability of symbolic systems.

---

## 1.2 The Hypothesis

**Core hypothesis:** Reality is structured as a higher-order hypergraph.

This means:
- **Entities exist** (nodes)
- **Relations connect entities** (hyperedges — connecting any number of nodes)
- **Relations can relate to relations** (higher-order — edges about edges)
- **Structure transforms according to rules** (rewriting — the dynamics of change)

If this hypothesis is correct, then a system whose internal representation matches this structure should be able to:
- Represent any domain
- Learn new structure within the representation
- Maintain semantic stability through explicit constraint
- Reason by graph transformation
- Model itself (self-representation)

**The bet:** Build the right substrate, and intelligence becomes a matter of learning the right structure within it.

---

## 1.3 The Goal

**Build an AGI system grounded in higher-order hypergraphs.**

This requires:

1. **A database** that stores higher-order hypergraphs efficiently
2. **An ontology system** that constrains graph structure and enables optimization
3. **A rewrite engine** that transforms graphs according to rules
4. **A learning system** that proposes structural changes
5. **A selection mechanism** that evaluates and commits changes
6. **A grounding system** that connects symbols to reality

The end state is a system that:
- Learns ontologies from experience
- Reasons by graph transformation
- Maintains coherent, inspectable beliefs
- Improves itself within safe constraints
- Generalizes across domains

---

# Part II: Core Concepts

## 2.1 What is a Higher-Order Hypergraph

A **graph** connects nodes with edges. Each edge connects exactly two nodes.

A **hypergraph** generalizes this: edges (called hyperedges) can connect any number of nodes. An edge might connect three nodes, or five, or one.

A **higher-order hypergraph** goes further: edges can connect to other edges, not just nodes. This allows:

- **Confidence about claims:** An edge stating "A causes B" can have another edge attached stating "confidence: 0.8"
- **Provenance:** Any edge can have edges recording where it came from
- **Meta-reasoning:** Edges about the structure of reasoning itself

```
Standard Graph:
  A ──── B
  
Hypergraph:
  A ──┬── B
      │
      C
  (One edge connecting three nodes)

Higher-Order Hypergraph:
  A ───causes───► B
           │
           └──confidence: 0.8
  (An edge about the "causes" edge)
```

Higher-order structure is essential for:
- Uncertainty quantification
- Provenance tracking
- Meta-cognition
- Self-modeling

---

## 2.2 What is an Ontology

An **ontology** defines what can exist in a graph.

Without an ontology, a graph is unstructured: any node can connect to any other node with any edge. This is flexible but:
- Cannot be optimized (no structure to exploit)
- Cannot be validated (no constraints to check)
- Cannot be reasoned about (no types to guide inference)

An ontology declares:
- **Node types:** What kinds of entities exist (Person, Event, Concept)
- **Edge types:** What kinds of relationships exist, and what they connect (causes: Event → Event)
- **Constraints:** What must always be true (causes must respect temporal order)
- **Rules:** What transformations can occur (if A causes B and B causes C, then A causes C)

With an ontology, a graph becomes:
- **Optimizable:** Index by type, optimize queries by structure
- **Validatable:** Reject invalid data, enforce invariants
- **Reasonably:** Apply rules, derive new knowledge

**An ontology is physics for a graph.** It defines what's possible.

---

## 2.3 What is Layer 0

**Layer 0** is the meta-ontology: the ontology that defines what ontologies are.

It answers: What is a type? What is a constraint? What is a rule?

Layer 0 is:
- **Fixed:** It does not change (or changes extremely rarely via formal process)
- **Hardcoded:** It is built into the engine, not stored as data
- **Universal:** All ontologies are expressed in terms of Layer 0
- **Self-describing:** Ontologies are stored as Layer 0 structures in the graph itself

Layer 0 defines approximately:
- 26 node types (for describing ontology structure)
- 39 edge types (for connecting ontology components)
- 19 constraints (ensuring ontology validity)

When you write:

```
node Event {
  timestamp: Int
}
```

This compiles to Layer 0 structure:
- A `_NodeType` node with `name = "Event"`
- An `_AttributeDef` node with `name = "timestamp"`
- A `_type_has_attribute` edge connecting them

**Layer 0 is the foundation. Everything else is built on it.**

---

## 2.4 The Compilation Model

Ontologies are written in a domain-specific language (DSL) and **compiled** into the system.

**Compilation does:**
1. Parse ontology text into an AST
2. Validate the ontology (well-formed, consistent)
3. Generate Layer 0 graph structure
4. Build in-memory registries (for type checking, constraint checking)
5. Create optimized indexes (for observation performance)

**Compilation does not:**
- Execute at runtime (it runs once when loading)
- Create user data (only ontology structure)
- Modify the engine (it configures, not changes)

**Why compilation matters:**

| Without Compilation | With Compilation |
|---------------------|------------------|
| Generic indexes | Type-specific indexes |
| No validation | Full type checking |
| Runtime schema discovery | Compile-time optimization |
| Dynamic, slow | Static, fast |

The compiler is the **gatekeeper** for ontology structure. Direct manipulation of Layer 0 types is prohibited. This ensures:
- All ontologies are validated
- No invalid structure can exist
- Clear audit trail for changes

---

## 2.5 The Execution Model

At runtime, the system:

1. **Stores** nodes and edges in the graph
2. **Validates** mutations against constraints
3. **Indexes** by type and structure for fast queries
4. **Triggers** rules when patterns match
5. **Versions** changes for history and rollback

**Constraint enforcement:**
- Before any mutation commits, check affected constraints
- If hard constraint fails: reject mutation, rollback
- If soft constraint fails: warn, proceed

**Rule execution:**
- After mutation commits, find matching rule patterns
- Execute rule productions (SPAWN, KILL, LINK, UNLINK, SET)
- Repeat until no new matches (quiescence) or cycle detected
- All changes atomic: if any step fails, everything rolls back

**Transactions:**
- All mutations within a transaction are atomic
- Includes triggered rules
- Either everything commits or nothing does

---

# Part III: Design Philosophy

## 3.1 Principles

### Explicit over implicit

Structure should be visible in the graph, not hidden in code. If something exists, it should be a node or edge. If a relationship holds, it should be an explicit edge, not an implicit computation.

### Constrained over arbitrary

The power of the system comes from constraints, not freedom. A graph that can contain anything is a graph that can guarantee nothing. Ontologies constrain what's possible so that what exists is meaningful.

### Stable over fluid

Meaning must be stable. A symbol that meant X yesterday and Y today is useless for reasoning. Layer 0 provides the stable foundation. User ontologies provide stable domains. Only instance data changes freely.

### Inspectable over opaque

Any belief, relationship, or reasoning step should be observable. You can ask: Why does this edge exist? What supports this claim? What rules produced this conclusion? Transparency is not optional.

### Validated over trusted

All mutations pass through constraint checking. All type signatures are verified. Invalid structure is rejected, not accepted and hoped for the best.

---

## 3.2 What We Optimize For

**Semantic integrity:** The graph is always in a valid state. Constraints hold. Types match. No corruption.

**Observation performance:** Common patterns are fast. Indexes exist for declared types. The ontology guides optimization.

**Reasoning transparency:** Every conclusion can be traced. Provenance is native. Meta-queries are supported.

**Safe evolution:** The system can change without breaking. Additive changes work. Breaking changes have migration paths.

**Self-modeling:** The system can represent itself. Ontologies are data. Introspection is natural.

---

## 3.3 What We Sacrifice

**Maximum flexibility:** You cannot store arbitrary untyped data. Structure must be declared. This is intentional.

**Write performance:** Constraint checking adds overhead to every mutation. We accept this for integrity.

**Simplicity:** The system has many concepts (types, edges, constraints, rules, patterns, expressions). This complexity serves expressiveness.

**Immediate gratification:** You must design an ontology before storing data. The upfront investment pays off in observation performance and data quality.

---

## 3.4 Key Trade-offs

| Trade-off | Our Choice | Rationale |
|-----------|------------|-----------|
| Flexibility vs Safety | Safety | Invalid data is worse than rejected data |
| Speed vs Correctness | Correctness | Wrong answers fast are still wrong |
| Simplicity vs Power | Power | The problem domain is complex |
| Dynamic vs Static | Static (compile-time) | Optimization requires known structure |
| General vs Specific | Specific (per-ontology) | Specialization enables efficiency |

---

# Part IV: Architecture Overview

## 4.1 Component Map

```
┌─────────────────────────────────────────────────────────────────────┐
│                            USER                                      │
│         Ontologies    Queries    Mutations    Applications          │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          COMPILER                                    │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐    │
│   │   Parser    │  │  Validator  │  │    Code Generator       │    │
│   │             │  │             │  │  (Registries, Indexes)  │    │
│   └─────────────┘  └─────────────┘  └─────────────────────────┘    │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           ENGINE                                     │
│   ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐  │
│   │  Graph Store  │  │  Constraint   │  │    Rule Executor      │  │
│   │               │  │   Checker     │  │                       │  │
│   └───────────────┘  └───────────────┘  └───────────────────────┘  │
│   ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐  │
│   │Index Manager  │  │  Transaction  │  │  Observation Executor │  │
│   │               │  │   Manager     │  │                       │  │
│   └───────────────┘  └───────────────┘  └───────────────────────┘  │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                    Layer 0 (Hardcoded)                       │  │
│   │      Meta-types, Meta-edges, Meta-constraints                │  │
│   └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          STORAGE                                     │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐    │
│   │ Graph Data  │  │   Indexes   │  │   Version History       │    │
│   └─────────────┘  └─────────────┘  └─────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4.2 Data Flow

### Ontology Loading

```
Ontology File (.hog)
       │
       ▼
   [Parser] ──► AST
       │
       ▼
  [Validator] ──► Validated AST (or errors)
       │
       ▼
[Code Generator] ──► Layer 0 graph structure
       │             + Type Registry
       │             + Constraint Registry
       │             + Rule Registry
       │             + Indexes
       ▼
   [Engine] ◄── Registries loaded, ontology active
```

### Observation Execution

```
Observation Statement
    │
    ▼
[Parser] ──► Observation AST
    │
    ▼
[Planner] ──► Observation Plan (uses type info, indexes)
    │
    ▼
[Executor] ──► Iterate graph, match patterns, filter
    │
    ▼
Results (stream of matches)
```

### Mutation Execution

```
Mutation (SPAWN, KILL, LINK, UNLINK, SET)
    │
    ▼
[Type Checker] ──► Validate types match
    │
    ▼
[Apply to Graph] ──► Tentative change
    │
    ▼
[Constraint Checker] ──► Check affected constraints
    │                      │
    │                      ├── Fail: Rollback, return error
    │                      │
    │                      └── Pass: Continue
    ▼
[Rule Engine] ──► Find triggered rules
    │             Execute productions
    │             (recursive until quiescent)
    │              │
    │              ├── Fail: Rollback everything
    │              │
    │              └── Pass: Continue
    ▼
[Commit] ──► Persist to storage, update indexes
    │
    ▼
Success (return new IDs)
```

---

## 4.3 Compilation Pipeline

The compiler transforms ontology source into runtime artifacts:

| Stage | Input | Output |
|-------|-------|--------|
| **Lexing** | Source text | Token stream |
| **Parsing** | Tokens | AST |
| **Name Resolution** | AST | AST with resolved references |
| **Type Checking** | AST | AST with type annotations |
| **Validation** | AST | Validated AST or errors |
| **Structure Generation** | AST | Layer 0 nodes and edges |
| **Registry Building** | Layer 0 structure | In-memory registries |
| **Index Creation** | Registries | Optimized index structures |

---

## 4.4 Runtime Execution

The engine processes operations against the compiled ontology:

| Operation | Flow |
|-----------|------|
| **SPAWN** (node) | Type check → Apply → Constraint check → Rule trigger → Commit |
| **LINK** (edge) | Signature check → Apply → Constraint check → Rule trigger → Commit |
| **KILL** (node) | Reference check → Apply → Constraint check → Rule trigger → Commit |
| **UNLINK** (edge) | Reference check → Apply → Constraint check → Rule trigger → Commit |
| **SET** (attribute) | Type check → Apply → Constraint check → Rule trigger → Commit |
| **MATCH** (observation) | Parse → Plan → Execute → Stream results |
| **matchPattern** | Compile pattern → Execute → Stream matches |

---

# Part V: Design Decisions

## 5.1 Why Hypergraphs (not property graphs)

Property graphs (used by Neo4j, etc.) have edges connecting exactly two nodes. This forces awkward modeling for n-ary relationships:

**Property graph (awkward):**
```
To represent "Alice, Bob, and Carol collaborated on Project X":
- Create intermediate "Collaboration" node
- Connect Alice → Collaboration
- Connect Bob → Collaboration  
- Connect Carol → Collaboration
- Connect Collaboration → Project X
```

**Hypergraph (natural):**
```
collaborated(Alice, Bob, Carol, Project_X)
- Single edge, four targets
- Direct representation of the relationship
```

Hyperedges enable:
- Natural representation of n-ary relationships
- Fewer nodes (no intermediate nodes needed)
- Cleaner queries (match one edge, not a subgraph)

---

## 5.2 Why Higher-Order (edges about edges)

Standard graphs cannot make statements about relationships. Higher-order hypergraphs can:

**Standard graph limitation:**
```
How do you say "The claim that A causes B has confidence 0.8"?
- Can't attach data to edges
- Must create intermediate "Claim" node
- Lose the direct connection
```

**Higher-order hypergraph:**
```
causes(A, B)  ← edge e1
confidence(e1, 0.8)  ← edge about edge e1
```

Higher-order enables:
- **Uncertainty:** Confidence/probability on any claim
- **Provenance:** Track where any information came from
- **Meta-reasoning:** Reason about the structure of reasoning
- **Self-modeling:** Represent the system's own ontology

---

## 5.3 Why Compiled Ontologies (not schema-less)

Schema-less databases (like document stores) offer flexibility but sacrifice:

| Schema-less | Compiled Ontology |
|-------------|-------------------|
| Store anything | Store valid data only |
| Discover structure at runtime | Know structure at compile time |
| Generic indexes | Type-specific indexes |
| Validate in application code | Validate in database |
| Fast writes | Fast reads |

We choose compiled ontologies because:
- **Performance:** Observation optimization requires known structure
- **Correctness:** Constraint enforcement requires declared types
- **Reasoning:** Rules require typed patterns

The upfront cost of declaring an ontology pays dividends in runtime performance and data quality.

---

## 5.4 Why Pattern-Based Constraints

Constraints could be:
- **Procedural:** Code that runs on each mutation
- **Declarative:** Logic that states what must be true

We chose declarative, pattern-based constraints:

```
constraint temporal_causation:
  e1: Event, e2: Event, causes(e1, e2)
  => e1.timestamp <= e2.timestamp
```

Benefits:
- **Inspectable:** You can observe what constraints exist
- **Optimizable:** Engine can index for constraint checking
- **Composable:** Patterns reused in queries, rules, constraints
- **Self-documenting:** The constraint IS its specification

---

## 5.5 Why Compiler-Gated Types

Types could be created:
- **Directly:** By creating `_NodeType` nodes in the graph
- **Through compiler:** By submitting ontology text

We chose compiler-gating:

```typescript
// DISALLOWED
createNode("_NodeType", { name: "NewType" })  // Error

// ALLOWED
engine.extendOntology(`node NewType { ... }`)  // Goes through compiler
```

Benefits:
- **Validation:** Compiler checks ontology coherence
- **Safety:** Cannot create invalid type structures
- **Audit:** Clear path for type changes
- **Control:** Can add approval workflows

The compiler is the gatekeeper. This is a safety feature, not a limitation.

---

# Part VI: The Path to ASI

## 6.1 Phase Overview

### Phase 1-2: The Substrate

Build a production-ready HOHG database:
- Higher-order hypergraph storage
- Compiled ontologies
- Constraint enforcement
- Rule execution
- Observation optimization
- Versioning

**Milestone:** A database others can use, with documented ontologies.

### Phase 3: Dynamics

Make the graph compute:
- Rewrite rules create autonomous dynamics
- The graph evolves according to declared rules
- Pattern matching triggers transformations

**Milestone:** Graphs that compute, not just store.

### Phase 4: Evaluation

Add self-measurement:
- **Coherence (C):** Internal consistency, integration, fragility
- **Option-Space (Ω):** Available transformations, future possibilities
- **Friction (F):** Effort vs value, efficiency

The **Persistence Functional:** Φ = C + Ω - F

**Milestone:** The system can evaluate its own state.

### Phase 5: Self-Model

The graph contains itself:
- Layer 1 "cognitive physics" ontology
- Representation of own rules and constraints
- Meta-cognition as higher-order structure

**Milestone:** The system reasons about itself.

### Phase 6: Learning

Neural models propose changes:
- Observe graph state
- Propose: new nodes, edges, types, rules
- Evaluate proposals by ΔΦ
- Accept/reject based on impact

Learning is **constrained proposal**, not unconstrained gradient descent.

**Milestone:** The system improves through experience.

### Phase 7: Grounding

Connect to reality:
- Perception becomes graph structure
- Graph state becomes action
- Feedback loop through environment

Grounding is **Φ-relevance:** What affects persistence is meaningful.

**Milestone:** The system is situated in the world.

### Phase 8: Integration

Combine all components:
- Substrate holds knowledge
- Self-model enables reflection
- Learning improves structure
- Grounding connects to reality
- Φ guides behavior

**Milestone:** General intelligence that learns, reasons, and acts.

---

## 6.3 What Success Looks Like

**Technical metrics:**
- Outperforms LLMs on reasoning tasks (accuracy)
- Uses orders of magnitude less compute (efficiency)
- Maintains consistency over time (stability)
- Improves with experience (learning)
- Handles new domains without retraining (generalization)

**Behavioral metrics:**
- Can explain its reasoning (transparency)
- Maintains stated constraints (reliability)
- Recognizes own uncertainty (calibration)
- Defers on questions outside competence (humility)

**Safety metrics:**
- Values are inspectable (alignment verification)
- Behavior matches stated goals (goal stability)
- Modifications are auditable (change tracking)
- Constraints cannot be bypassed (integrity)

---

# Part VII: How to Read the Documentation

## 7.1 Document Map

```
┌─────────────────────────────────────────────────────────────────────┐
│  START HERE                                                          │
│  ═══════════                                                         │
│  Context Document (this document)                                   │
│    │                                                                 │
│    ├── Want to understand the technical foundation?                 │
│    │   └── Layer 0 Specification                                    │
│    │                                                                 │
│    ├── Want to write an ontology?                                   │
│    │   └── Ontology Language Reference                              │
│    │   └── Test Ontologies (examples)                               │
│    │                                                                 │
│    ├── Want to operate the database?                                  │
│    │   └── Language Reference                                       │
│    │                                                                 │
│    ├── Want to implement the system?                                │
│    │   └── Architecture Overview                                    │
│    │   └── Implementation Plan                                      │
│    │                                                                 │
│    ├── Want to propose changes to Layer 0?                          │
│    │   └── Revision Protocol                                        │
│    │                                                                 │
│    └── Confused by terminology?                                     │
│        └── Glossary                                                 │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.2 Reading Order

**For understanding (theory-first):**
1. Context Document (this)
2. Layer 0 Specification
3. Test Ontologies (examples)
4. Architecture Overview

**For contributing (technical-first):**
1. Context Document (Part II, IV only)
2. Architecture Overview
3. Implementation Plan
4. Layer 0 Specification (as reference)

**For using (user-first):**
1. Context Document (Part II only)
2. Ontology Language Reference
3. HOHG Language Reference
4. Test Ontologies (examples)

## 8.3 Quick Reference

| I want to... | Go to... |
|--------------|----------|
| Understand what HOHG is | Context Document, Part I-II |
| See the design philosophy | Context Document, Part III |
| Understand the architecture | Context Document, Part IV or Architecture Overview |
| Know why decisions were made | Context Document, Part V |
| See the roadmap | Context Document, Part VII |
| Read the formal specification | Layer 0 Specification |
| Write an ontology | Ontology Language Reference |
| See ontology examples | Test Ontologies (9 examples) |
| Write queries | HOHG Language Reference |
| Implement the system | Architecture Overview + Implementation Plan |
| Propose spec changes | Revision Protocol |
| Look up a term | Glossary |

---

# Appendix A: Historical Context

This document captures decisions made during the initial design phase. Key decision points:

**Hypergraph choice:** Standard graphs were rejected because binary edges cannot naturally represent n-ary relationships. Hypergraphs generalize cleanly.

**Higher-order choice:** The need to represent confidence, provenance, and meta-reasoning led to higher-order structure. Reification (the RDF approach) was rejected as too awkward.

**Compiled ontology choice:** Schema-less was rejected because observation optimization and constraint enforcement require known structure. The upfront cost is worth the runtime benefits.

**Pattern-based constraints:** Procedural constraints were rejected because they cannot be inspected, optimized, or reasoned about. Declarative patterns are transparent and composable.

**Compiler gating:** Direct type creation was rejected because it bypasses validation. The compiler ensures ontology coherence.

**Φ as training objective:** Traditional reward functions were rejected because they don't capture long-term flourishing. The Persistence Functional (Coherence + Option-Space - Friction) measures what matters for sustained existence.

---

# Appendix B: Related Theories

Systems, ideas and theories that relates with this design:

**Hypergraph rewriting (Wolfram Physics Project):** The idea that physics might be a hypergraph undergoing rewriting.

**Category theory:** The emphasis on compositionality, types, and structure-preserving maps. Layer 0 is implicitly categorical.

**Description logics (OWL):** The idea of formal ontologies with inference. We adopt simpler, decidable rules rather than full DL expressiveness.

**Global Workspace Theory:** The idea of a shared workspace for cognition. The hypergraph serves as explicit workspace.

**Type theory:** The importance of types for correctness and reasoning. Layer 0 is essentially a type system for graphs.

**Database theory:** Transaction semantics, constraint enforcement, observation optimization. Standard database wisdom applies.

---

# Closing

This document is the entry point to the HOHG project. It captures the vision, philosophy, and key decisions made during design.

The specification is in Layer 0. The process is in the Revision Protocol. The examples are in the Test Ontologies. The implementation guidance is in the Architecture Overview and Implementation Plan.

**The hypothesis:** Reality is a higher-order hypergraph.

**The goal:** Build ASI grounded in this structure.

**The path:** Substrate → Dynamics → Evaluation → Self-Model → Learning → Grounding → Integration.

**The principle:** Structure is explicit, selection is Φ-based, learning proposes but does not own.

Now, build.

---

*Document version: 1.0*
*Last updated: [Date]*
*Status: Foundation*