# Test Ontologies Strategy

## 1. Complexity Dimensions

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      COMPLEXITY DIMENSIONS                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  STRUCTURAL COMPLEXITY                                                      │
│  ─────────────────────                                                      │
│    • Number of types                                                        │
│    • Inheritance depth                                                      │
│    • Attribute variety (scalars, required, unique, indexed)                │
│    • Edge arity (binary, ternary, hyperedges)                              │
│    • Higher-order edges (edges about edges)                                │
│                                                                              │
│  SEMANTIC COMPLEXITY                                                        │
│  ───────────────────                                                        │
│    • Constraint sophistication (simple → transitive → EXISTS)              │
│    • Rule complexity (simple SET → multi-action → recursive)               │
│    • Pattern complexity (single node → joins → transitive)                 │
│    • Cardinality constraints                                               │
│    • Referential actions (cascade, prevent)                                │
│                                                                              │
│  DOMAIN COMPLEXITY                                                          │
│  ─────────────────                                                          │
│    • Real-world modeling challenges                                        │
│    • Temporal reasoning                                                    │
│    • State machines / workflows                                            │
│    • Self-reference / meta-modeling                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 2. Level Definitions

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ LEVEL 1: FUNDAMENTALS                                                        │
│ ══════════════════════                                                       │
│                                                                              │
│ Features Used:                                                              │
│   ✓ Basic node types                                                        │
│   ✓ Scalar attributes (String, Int, Bool, Timestamp)                       │
│   ✓ Binary edges                                                            │
│   ✓ Required attributes                                                     │
│   ✓ Default values                                                          │
│   ✗ Inheritance                                                             │
│   ✗ Constraints (beyond required)                                           │
│   ✗ Rules                                                                   │
│   ✗ Higher-order                                                            │
│                                                                              │
│ Goal: Test basic parsing, type creation, simple CRUD operations            │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ LEVEL 2: STRUCTURE                                                           │
│ ══════════════════                                                           │
│                                                                              │
│ Features Used:                                                              │
│   ✓ Everything from Level 1                                                │
│   ✓ Type inheritance (single)                                              │
│   ✓ Attribute modifiers: unique, indexed, range, enum, match               │
│   ✓ Edge modifiers: no_self, unique                                        │
│   ✓ Simple constraints (value validation)                                  │
│   ✗ Complex constraints (patterns with multiple nodes)                     │
│   ✗ Rules                                                                   │
│   ✗ Higher-order                                                            │
│                                                                              │
│ Goal: Test inheritance, attribute validation, index creation               │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ LEVEL 3: DYNAMICS                                                            │
│ ═════════════════                                                            │
│                                                                              │
│ Features Used:                                                              │
│   ✓ Everything from Level 2                                                │
│   ✓ Multiple inheritance                                                   │
│   ✓ Complex constraints (multi-node patterns, EXISTS, NOT EXISTS)          │
│   ✓ Cardinality constraints                                                │
│   ✓ Transitive patterns (+ and *)                                          │
│   ✓ Auto rules (SPAWN, LINK, SET)                                          │
│   ✓ Referential actions (cascade, prevent)                                 │
│   ✓ Edge attributes                                                        │
│   ✗ Higher-order edges                                                      │
│   ✗ Manual rules                                                            │
│                                                                              │
│ Goal: Test constraint checking, rule execution, cascades                   │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ LEVEL 4: HIGHER-ORDER                                                        │
│ ═════════════════════                                                        │
│                                                                              │
│ Features Used:                                                              │
│   ✓ Everything from Level 3                                                │
│   ✓ Higher-order edges (edge<T>)                                           │
│   ✓ Edges about edges about edges (multi-level)                            │
│   ✓ Symmetric edges                                                        │
│   ✓ Hyperedges (arity > 2)                                                 │
│   ✓ Complex rule chains                                                    │
│   ✓ Manual rules                                                           │
│   ✓ Acyclic constraints                                                    │
│                                                                              │
│ Goal: Test higher-order storage, indexing, and traversal                   │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ LEVEL 5: AGI-READY                                                           │
│ ══════════════════                                                           │
│                                                                              │
│ Features Used:                                                              │
│   ✓ Everything from Level 4                                                │
│   ✓ Self-referential patterns (types that reference own type)             │
│   ✓ META-aware design (types for representing types)                       │
│   ✓ Confidence/uncertainty modeling                                        │
│   ✓ Provenance tracking                                                    │
│   ✓ Temporal reasoning                                                     │
│   ✓ Complex rule ecosystems                                                │
│   ✓ Soft constraints                                                       │
│                                                                              │
│ Goal: Test full system integration, self-modeling, AGI patterns            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 3. Size Definitions

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SIZE CATEGORIES                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  SHORT (S)                                                                  │
│  ──────────                                                                 │
│    Lines: ~30-60                                                            │
│    Types: 2-4 node types                                                    │
│    Edges: 2-4 edge types                                                    │
│    Constraints: 0-3                                                         │
│    Rules: 0-2                                                               │
│    Purpose: Focused test of specific features                              │
│                                                                              │
│  MEDIUM (M)                                                                 │
│  ───────────                                                                │
│    Lines: ~100-200                                                          │
│    Types: 5-8 node types                                                    │
│    Edges: 5-10 edge types                                                   │
│    Constraints: 3-8                                                         │
│    Rules: 2-5                                                               │
│    Purpose: Realistic domain model                                         │
│                                                                              │
│  LONG (L)                                                                   │
│  ──────────                                                                 │
│    Lines: ~250-400+                                                         │
│    Types: 10+ node types                                                    │
│    Edges: 10+ edge types                                                    │
│    Constraints: 8+                                                          │
│    Rules: 5+                                                                │
│    Purpose: Comprehensive real-world system                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 4. Test Ontology Matrix

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ONTOLOGY MATRIX                                      │
├─────────┬───────────────────┬───────────────────┬───────────────────────────┤
│  LEVEL  │      SHORT        │      MEDIUM       │         LONG              │
├─────────┼───────────────────┼───────────────────┼───────────────────────────┤
│         │                   │                   │                           │
│    1    │  1S: Bookmarks    │  1M: Library      │  1L: Contact Manager      │
│ FUNDA-  │  (URL, Tag,       │  (Book, Author,   │  (Person, Org, Address,   │
│ MENTAL  │   Folder)         │   Borrower, Loan) │   Phone, Email, Tag,      │
│         │                   │                   │   Relationship)           │
│         │                   │                   │                           │
├─────────┼───────────────────┼───────────────────┼───────────────────────────┤
│         │                   │                   │                           │
│    2    │  2S: Task List    │  2M: E-Commerce   │  2L: HR System            │
│ STRUC-  │  (Task, Tag       │  (Product, Cat,   │  (Employee, Department,   │
│ TURE    │   with inherit)   │   Review, Order)  │   Role, Skill, Office,    │
│         │                   │                   │   Certification)          │
│         │                   │                   │                           │
├─────────┼───────────────────┼───────────────────┼───────────────────────────┤
│         │                   │                   │                           │
│    3    │  3S: Event Chain  │  3M: Project Mgmt │  3L: Workflow Engine      │
│ DYNAM-  │  (Event, causes   │  (Project, Task,  │  (State, Transition,      │
│ ICS     │   with temporal)  │   Milestone,      │   Condition, Action,      │
│         │                   │   Resource)       │   WorkItem, History)      │
│         │                   │                   │                           │
├─────────┼───────────────────┼───────────────────┼───────────────────────────┤
│         │                   │                   │                           │
│    4    │  4S: Fact Base    │  4M: Scientific   │  4L: Argumentation        │
│ HIGHER  │  (Fact, Evidence, │  (Hypothesis,     │  (Claim, Argument,        │
│ ORDER   │   confidence)     │   Experiment,     │   Evidence, Rebuttal,     │
│         │                   │   supports)       │   Stance, Debate)         │
│         │                   │                   │                           │
├─────────┼───────────────────┼───────────────────┼───────────────────────────┤
│         │                   │                   │                           │
│    5    │  5S: Concept Net  │  5M: Belief-      │  5L: Cognitive Agent      │
│  AGI    │  (Concept,        │      Desire-      │  (Percept, Belief, Goal,  │
│ READY   │   learned_rel,    │      Intention    │   Plan, Action, Memory,   │
│         │   confidence)     │  (Agent, Belief,  │   Self-Model, Rule,       │
│         │                   │   Goal, Plan)     │   Confidence, Φ-metrics)  │
│         │                   │                   │                           │
└─────────┴───────────────────┴───────────────────┴───────────────────────────┘
```

## 5. Domain Rationale

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DOMAIN SELECTION RATIONALE                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  LEVEL 1: Familiar domains, immediate utility                               │
│  ───────────────────────────────────────────────                            │
│    • Bookmarks: Everyone understands, minimal concepts                     │
│    • Library: Classic database example, clear relationships                │
│    • Contacts: Real-world use case, demonstrates scale                     │
│                                                                              │
│  LEVEL 2: Business domains requiring validation                             │
│  ──────────────────────────────────────────────                             │
│    • Tasks: Simple inheritance (Task → SubTask)                            │
│    • E-Commerce: Product hierarchies, reviews, validation                  │
│    • HR: Deep inheritance, many constraints, real complexity               │
│                                                                              │
│  LEVEL 3: Systems with state and causation                                  │
│  ───────────────────────────────────────────                                │
│    • Events: Pure causation modeling, temporal constraints                 │
│    • Projects: Dependencies, deadlines, resource allocation               │
│    • Workflows: State machines, transitions, conditions                   │
│                                                                              │
│  LEVEL 4: Knowledge representation with uncertainty                         │
│  ──────────────────────────────────────────────────                         │
│    • Facts: Simple provenance and confidence                               │
│    • Science: Hypotheses, evidence, reproducibility                       │
│    • Arguments: Full argumentation theory, attacks/supports               │
│                                                                              │
│  LEVEL 5: AGI-relevant cognitive architectures                              │
│  ─────────────────────────────────────────────                              │
│    • Concepts: Learning new relations, concept formation                   │
│    • BDI: Classic agent architecture, goal-directed behavior              │
│    • Cognitive: Full self-modeling, meta-reasoning, Φ                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 6. Feature Coverage Matrix

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      FEATURE COVERAGE BY ONTOLOGY                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Feature                    │ 1S 1M 1L │ 2S 2M 2L │ 3S 3M 3L │ 4S 4M 4L │ 5S 5M 5L │
│  ───────────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────│
│  Basic node types           │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  String/Int/Bool attrs      │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Timestamp/Duration         │ ·  ✓  ✓  │ ·  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Binary edges               │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  [required]                 │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Default values             │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  ───────────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────│
│  Single inheritance         │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Multiple inheritance       │ ·  ·  ·  │ ·  ·  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ✓  ✓  ✓  │
│  [unique]                   │ ·  ·  ✓  │ ✓  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  [indexed]                  │ ·  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Range constraints          │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  [enum] / in:               │ ·  ·  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  [match] regex              │ ·  ·  ✓  │ ·  ✓  ✓  │ ·  ·  ✓  │ ·  ·  ✓  │ ·  ·  ✓  │
│  [no_self]                  │ ·  ·  ·  │ ·  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  ───────────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────│
│  Explicit constraints       │ ·  ·  ·  │ ·  ·  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  EXISTS in constraint       │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ✓  ✓  ✓  │
│  NOT EXISTS                 │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Cardinality                │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  Transitive (+/*)           │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  [acyclic]                  │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ·  ✓  │ ·  ✓  ✓  │
│  Auto rules                 │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Rule with SPAWN            │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Rule with KILL             │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ✓  │ ·  ·  ✓  │ ·  ✓  ✓  │
│  [cascade]                  │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  [prevent]                  │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ✓  │ ·  ·  ✓  │ ·  ·  ✓  │
│  Edge attributes            │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  ───────────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────│
│  edge<T> (higher-order)     │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Multi-level H.O.           │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  [symmetric]                │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ✓  ✓  ✓  │
│  Hyperedges (arity > 2)     │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  Manual rules               │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  Rule priority ordering     │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ✓  ✓  ✓  │
│  ───────────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────│
│  Self-referential types     │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ✓  │ ✓  ✓  ✓  │
│  Confidence modeling        │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ✓  ✓  ✓  │
│  Provenance tracking        │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│  Soft constraints           │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ·  │ ·  ·  ✓  │ ·  ✓  ✓  │
│  Temporal reasoning         │ ·  ·  ·  │ ·  ·  ·  │ ✓  ✓  ✓  │ ·  ✓  ✓  │ ✓  ✓  ✓  │
│  Type aliases               │ ·  ·  ·  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │ ·  ✓  ✓  │
│                                                                              │
│  Legend: ✓ = covered, · = not covered                                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 7. Naming Convention

```
Ontology Identifier: {Level}{Size}_{DomainName}

Examples:
  1S_Bookmarks
  1M_Library  
  1L_Contacts
  2S_Tasks
  2M_Ecommerce
  2L_HumanResources
  3S_EventChain
  3M_ProjectManagement
  3L_Workflow
  4S_FactBase
  4M_Scientific
  4L_Argumentation
  5S_ConceptNet
  5M_BDI
  5L_CognitiveAgent
```

## 8. Test Case Categories Per Ontology

Each ontology should include comments marking test scenarios:

```
-- @test:parse - Ontology should parse without errors
-- @test:compile - Ontology should compile to Layer 0
-- @test:spawn - Test node creation
-- @test:link - Test edge creation  
-- @test:constraint:pass - Valid data should pass
-- @test:constraint:fail - Invalid data should be rejected
-- @test:rule:trigger - Rule should fire on condition
-- @test:rule:chain - Multiple rules should cascade correctly
-- @test:query:simple - Basic MATCH should work
-- @test:query:pattern - Pattern matching should work
-- @test:query:transitive - Transitive queries should work
-- @test:higher-order - Higher-order edges should work
```
