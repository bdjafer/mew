# Branching Execution Model

**Version:** 0.1 (Future Extension)
**Status:** Design Draft
**Target:** v2.x
**Purpose:** Native support for branching structures and probabilistic execution

---

# Part I: Motivation

## 1.1 The Limitation of Single-World Execution

Current execution model:

```
State_0 ──mutation──► State_1 ──rules──► State_2 (quiescent)
```

One state at a time. Mutations transform it. Rules process to stability. Return control.

This works for:
- Databases (one consistent state)
- Business rules (deterministic outcomes)
- Knowledge graphs (facts are facts)

This fails for:
- Quantum systems (superposition is real)
- Probabilistic reasoning (uncertainty propagates)
- Counterfactual analysis (what-if requires alternatives)
- Decision theory (evaluate multiple futures)
- Simulation of physical systems (measurement branches)

## 1.2 The Cost of Explicit Branching

You can model branches explicitly in v1:

```
node WorldState { weight: Float }
edge in_world(entity: any, world: WorldState)
edge branches_from(child: WorldState, parent: WorldState)

-- Every entity duplicated per world
-- Every query filters by world
-- Manual bookkeeping everywhere
```

Problems:
- Extreme verbosity
- Error-prone duplication
- No execution optimization (all worlds in same graph)
- Queries are awkward
- No structural guarantees (easy to corrupt)

Native branching eliminates this overhead.

## 1.3 What Native Branching Enables

| Capability | Without Native | With Native |
|------------|----------------|-------------|
| Quantum simulation | Manual world duplication | Direct representation |
| Bayesian inference | External computation | Native propagation |
| Monte Carlo | External sampling | Branch enumeration |
| Counterfactuals | Can't really do | Query alternative branches |
| Decision analysis | External | Evaluate branch outcomes |
| Causal inference | Very difficult | Intervention = branching |

---

# Part II: Use Cases

## 2.1 Quantum Simulation

Model a quantum system directly:

```
node Particle {
  position: Float,
  momentum: Float,
  spin: String?  -- null = superposition
}

node Detector {
  reading: String?
}

edge interacts(particle: Particle, detector: Detector)

rule measurement:
  p: Particle, d: Detector,
  interacts(p, d)
  WHERE p.spin = null
  =>
  BRANCH [weight: 0.5]:
    SET p.spin = "up",
    SET d.reading = "up"
  BRANCH [weight: 0.5]:
    SET p.spin = "down",
    SET d.reading = "down"
```

After execution:
- Two branches exist
- Each branch has definite spin
- Superposition represented as branch structure
- Interference possible if branches reconverge

## 2.2 Probabilistic Reasoning

Bayesian network as graph:

```
node Variable {
  name: String,
  value: String?
}

edge influences(cause: Variable, effect: Variable) {
  probability_table: String  -- serialized CPT
}

rule propagate:
  cause: Variable, effect: Variable,
  influences(cause, effect) AS inf
  WHERE cause.value != null AND effect.value = null
  =>
  FOR (value, weight) IN parse_cpt(inf.probability_table, cause.value):
    BRANCH [weight: weight]:
      SET effect.value = value
```

Query: "What's the probability of disease given symptom?"

```
MATCH d: Variable, s: Variable
WHERE d.name = "disease" AND s.name = "symptom" AND s.value = "present"
MARGINALIZE d.value
-- Returns: { "yes": 0.3, "no": 0.7 }
```

## 2.3 Decision Analysis

Evaluate strategic decisions:

```
node Decision { choice: String? }
node Outcome { utility: Float? }

edge leads_to(decision: Decision, outcome: Outcome)

rule evaluate_options:
  d: Decision
  WHERE d.choice = null
  =>
  BRANCH [weight: 1.0]:
    SET d.choice = "option_A"
  BRANCH [weight: 1.0]:
    SET d.choice = "option_B"
  BRANCH [weight: 1.0]:
    SET d.choice = "option_C"
```

Then simulate consequences in each branch. Query expected utility:

```
MATCH d: Decision, o: Outcome, leads_to(d, o)
GROUP BY d.choice
AGGREGATE SUM(branch.weight * o.utility) AS expected_utility
ORDER BY expected_utility DESC
```

## 2.4 Counterfactual Analysis

"What would have happened if X?"

```
-- Find the branch point where X could have occurred
MATCH branch_point: State
WHERE could_have(branch_point, "X")

-- Fork from there with X forced
FORK FROM branch_point AS counterfactual
  WITH SET x.happened = true

-- Compare outcomes
MATCH actual: Outcome IN BRANCH main
MATCH counter: Outcome IN BRANCH counterfactual
RETURN diff(actual, counter)
```

## 2.5 Physical Simulation (Nuclear Reactor)

The motivating example from earlier discussion:

```
node Particle {
  type: String,
  energy: Float,
  position_x: Float,
  position_y: Float,
  position_z: Float
}

edge collides(p1: Particle, p2: Particle) {
  cross_section: Float
}

rule fission:
  neutron: Particle, nucleus: Particle,
  collides(neutron, nucleus)
  WHERE neutron.type = "neutron" 
    AND nucleus.type = "U235"
    AND neutron.energy > 0.025  -- thermal threshold
  =>
  BRANCH [weight: 0.85]:  -- fission occurs
    KILL nucleus,
    SPAWN n1: Particle { type = "neutron", energy = 1.0, ... },
    SPAWN n2: Particle { type = "neutron", energy = 1.0, ... },
    SPAWN fragment1: Particle { type = "fission_product", ... },
    SPAWN fragment2: Particle { type = "fission_product", ... }
  BRANCH [weight: 0.15]:  -- capture without fission
    KILL neutron,
    SET nucleus.type = "U236"
```

The full branching tree represents all possible reaction chains. Probability of criticality = branch weight sum for runaway branches.

---

# Part III: Execution Model

## 3.1 Core Concepts

| Concept | Definition |
|---------|------------|
| **Branch** | A distinct line of evolution through state space |
| **Weight** | Path density; how many microscopic paths this branch represents |
| **Fork** | Point where one branch becomes multiple |
| **Merge** | Point where branches reconverge (interference) |
| **Decohere** | Branches that can no longer merge |

## 3.2 Branch Identity

Each branch has:
- Unique identifier
- Parent branch (null for root)
- Fork point (state where it diverged)
- Weight (cumulative from root)
- Status: active | merged | decohered

```
Branch {
  id: BranchId,
  parent: BranchId?,
  fork_point: StateId,
  weight: Float,
  status: BranchStatus
}
```

## 3.3 Execution Semantics

### BRANCH Action

```
rule example:
  x: SomeType
  =>
  BRANCH [weight: W1]:
    actions_1
  BRANCH [weight: W2]:
    actions_2
```

Execution:
1. Current branch B with weight W
1. Create child branch B1 with weight W × W1
3. Create child branch B2 with weight W × W2
4. Execute actions_1 in context of B1
5. Execute actions_2 in context of B2
6. Both branches continue independently

### Quiescence in Branching Context

Quiescence is per-branch. A branch quiesces when no rules match in that branch.

Global quiescence: all branches quiescent.

Execution can:
- Run all branches to quiescence
- Run for N total steps across branches
- Run specific branch(es)
- Run until predicate on branch structure

### Weights Normalize?

Two options:

**Option A: Weights are absolute**
- Weights accumulate multiplicatively
- Total weight across branches may not equal 1
- Useful for: branch counting, interference calculations

**Option B: Weights are probabilities**
- Weights at each fork must sum to 1
- Total weight always conserved
- Useful for: probabilistic interpretation

Recommendation: **Option A** (absolute weights). Normalization can be computed when needed. More general, matches physics interpretation.

## 3.4 Querying Branches

### Single-Branch Query

```
MATCH p: Particle IN BRANCH b
WHERE b.id = "branch_123"
RETURN p.state
```

### Cross-Branch Query

```
MATCH p: Particle
ACROSS BRANCHES
WHERE p.type = "neutron"
RETURN branch.id, branch.weight, COUNT(p)
```

### Marginalization

```
MATCH p: Particle
WHERE p.type = "electron"
MARGINALIZE p.spin
-- Returns: { "up": 0.6, "down": 0.4 }
-- (weighted by branch weights)
```

### Interference Query

Find branches that could reconverge:

```
MATCH b1: Branch, b2: Branch
WHERE b1.fork_point = b1.fork_point
  AND can_interfere(b1, b2)
RETURN b1, b2, interference_term(b1, b2)
```

## 3.5 Merge and Decoherence

### Merge (Interference)

Two branches can merge if:
- Same fork point (shared ancestor)
- Identical structure at merge point (indistinguishable)

```
MERGE b1, b2 INTO b3
-- b3.weight = b1.weight + b2.weight (constructive)
-- or complex interference calculation
```

This is quantum interference: indistinguishable paths combine.

### Decoherence

Branches decohere when they become distinguishable to the environment.

```
rule decoherence:
  p: Particle, env: Environment,
  interacts(p, env)
  WHERE p.in_superposition
  =>
  DECOHERE branches_involving(p)
```

Decohered branches:
- Cannot merge
- Evolve independently
- Represent separate "worlds"

## 3.6 Branch Pruning

Without pruning, exponential explosion. Strategies:

**Weight threshold:**
```
SET branch.prune_threshold = 1e-10
-- Branches with weight below threshold are dropped
```

**Max branches:**
```
SET branch.max_count = 10000
-- Keep highest-weight branches, prune rest
```

**Importance sampling:**
```
SET branch.sampling = "importance"
-- Sample branches proportional to weight
-- Track sampling correction factor
```

**Coarse-graining:**
```
COARSEN BY (p.position / grid_size)
-- Merge branches that agree on coarse variables
```

---

# Part IV: High-Level Architecture

## 4.1 Component Changes

| Component | v1 | v2 (Branching) |
|-----------|-----|----------------|
| **Graph** | Single state | Branch-aware state tree |
| **Rule** | Fire once | Fire per branch |
| **Query** | Single result set | Branch-indexed results |
| **Transaction** | ACID on single state | ACID per branch or cross-branch |

## 4.2 New Components

### BranchManager

```
COMPONENT: BranchManager

PURPOSE
  Track branch structure, weights, and lifecycle.

RESPONSIBILITIES
  - Create branches on BRANCH action
  - Track parent-child relationships
  - Maintain branch weights
  - Handle merge and decoherence
  - Implement pruning strategies

DEPENDS ON
  - Graph: fork state at branch point

DEPENDED ON BY
  - Rule: create branches during execution
  - Query: filter/aggregate by branch
  - Transaction: branch-aware commits
```

### BranchExecutor

```
COMPONENT: BranchExecutor

PURPOSE
  Orchestrate execution across multiple branches.

RESPONSIBILITIES
  - Schedule rule evaluation across branches
  - Parallelize independent branches
  - Detect per-branch quiescence
  - Implement execution strategies (all, N steps, until)

DEPENDS ON
  - Rule: execute in branch context
  - BranchManager: branch structure
  - Graph: branch state access

DEPENDED ON BY
  - Transaction: multi-branch execution
  - Session: execution control
```

## 4.3 Graph Storage for Branches

### Option A: Full Copy

Each branch is complete copy of state.

```
Branch_0: { full graph state }
Branch_1: { full graph state }  -- copied from Branch_0
Branch_2: { full graph state }  -- copied from Branch_0
```

Simple. Expensive. O(state_size × branch_count) memory.

### Option B: Copy-on-Write

Branches share unchanged structure. Only differences stored.

```
Base state: { shared nodes and edges }
Branch_1 delta: { changed/added nodes and edges }
Branch_2 delta: { changed/added nodes and edges }
```

Efficient. Complex reads (must merge base + delta). Standard technique.

### Option C: Persistent Data Structures

Immutable structures with structural sharing.

```
State_0: persistent graph
State_1: shares most of State_0, different where changed
State_2: shares most of State_0, different where changed
```

Elegant. Requires persistent graph implementation. Good fit for functional style.

**Recommendation:** Option B (copy-on-write) for v2. Well-understood, reasonable complexity.

## 4.4 Parallel Execution

Independent branches can execute in parallel.

```
Branch_1: rules firing ──────►
Branch_2: rules firing ──────►  (parallel)
Branch_3: rules firing ──────►
```

Synchronization needed:
- Shared read of common ancestor
- Write to branch-local delta
- Merge operations coordinate

GPU potential:
- Batch rule matching across branches
- Parallel action execution per branch
- Tensor-backed branch state

---

# Part V: Syntax Summary

## 5.1 Rule Syntax

```
rule name:
  pattern
  =>
  BRANCH [weight: W]:
    actions
  BRANCH [weight: W]:
    actions
```

## 5.2 Query Syntax

```
-- In specific branch
MATCH pattern IN BRANCH branch_expr

-- Across all branches
MATCH pattern ACROSS BRANCHES

-- Marginalize (sum weights by value)
MATCH pattern
MARGINALIZE variable.attribute

-- Branch metadata
MATCH pattern
RETURN branch.id, branch.weight, branch.parent
```

## 5.3 Branch Control

```
-- Fork explicitly
FORK AS branch_name

-- Merge branches
MERGE b1, b2 INTO b3

-- Mark as decohered
DECOHERE b1, b2

-- Prune low-weight
PRUNE BRANCHES WHERE branch.weight < 1e-10
```

## 5.4 Execution Control

```
-- Run all branches to quiescence
RUN TO QUIESCENCE

-- Run N steps total across branches
RUN STEPS 1000

-- Run specific branch
RUN BRANCH b1 TO QUIESCENCE

-- Run until condition
RUN UNTIL (SELECT SUM(branch.weight) WHERE outcome = "critical") > 0.5
```

## Part VI Complex Amplitudes (Quantum Extension)

For full quantum structure, weights should be complex amplitudes:

**Current (real weights):**
```
BRANCH [weight: 0.7]: actions_A
BRANCH [weight: 0.3]: actions_B
```

**Extended (complex amplitudes):**
```
BRANCH [amplitude: 0.7 + 0.3i]: actions_A
BRANCH [amplitude: 0.5 - 0.4i]: actions_B
```

**Interference semantics:**

When branches reconverge (become indistinguishable):
- Real weights: P = w₁ + w₂ (no interference)
- Complex amplitudes: P = |α₁ + α₂|² (interference)

**Requirements:**
- Complex as primitive value type
- Branch stores amplitude, not just weight
- Merge operation sums amplitudes before squaring
- Born rule: P(branch) = |amplitude|²

**Use cases:**
- Quantum simulation
- Path integral formulations
- Interference phenomena

---

# Part VII: Migration Path

## 7.1 v1 Compatibility

v1 execution = single branch, no BRANCH actions.

A v1 ontology runs unchanged in v2:
- Implicit single branch with weight 1.0
- Queries return single-branch results
- No change in behavior

## 7.2 Gradual Adoption

1. Add BRANCH syntax to rules (opt-in per rule)
2. Queries default to "current branch" unless ACROSS BRANCHES
3. Explicit branch control for advanced use cases
4. Full branching for quantum/probabilistic applications

## 7.3 Performance Baseline

Without branching, v2 execution should match v1 performance:
- Single branch = no branching overhead
- Copy-on-write with no writes = no copy overhead
- Branch-aware queries with one branch = no aggregation overhead

Branching cost only when branching used.

---

# Part VIII: Open Questions

## 8.1 Interference Semantics

When can branches merge? Physical answer: when indistinguishable. But "indistinguishable" in graph terms means what exactly?

Candidates:
- Identical node/edge structure
- Isomorphic subgraph
- Same values for specified "observable" attributes
- User-defined equivalence relation

## 8.2 Constraint Checking

How do constraints apply to branches?

Option A: Per-branch
- Each branch satisfies constraints independently
- Branch can exist even if sibling violates

Option B: Global
- Constraints on branch structure itself
- "No branch where X" as constraint

Probably: Option A default, Option B as explicit branch constraints.

## 8.3 Transaction Semantics

What does COMMIT mean with branches?

Option A: Commit all branches
- Entire tree becomes durable

Option B: Commit specific branches
- Select which branches to persist
- Others discarded

Option C: Commit weighted
- Persist branches above weight threshold

## 8.4 Observation Problem

In physics: observation collapses superposition.

In HOHG: should queries collapse branches? Or just read without effect?

Probably: Queries read without effect. Explicit OBSERVE action collapses/decoheres.

---

# Part IX: Summary

## 9.1 What Branching Adds

| Capability | Description |
|------------|-------------|
| Native superposition | Multiple states coexist |
| Probabilistic reasoning | Weight-based inference |
| Counterfactuals | Query alternative branches |
| Quantum simulation | Direct representation |
| Decision analysis | Evaluate multiple futures |

## 9.2 What Branching Costs

| Cost | Mitigation |
|------|------------|
| Memory (branch explosion) | Copy-on-write, pruning |
| CPU (parallel branches) | Parallelize independent branches |
| Complexity (execution model) | Clean semantics, gradual adoption |
| Query complexity | Default single-branch, opt-in cross-branch |

---

*End of Branching Execution Model specification.*