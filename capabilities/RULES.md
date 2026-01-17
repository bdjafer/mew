# MEW Rules Capability

**Version:** 1.0
**Status:** Draft
**Scope:** Reactive transformations, declarative production rules, self-maintaining graphs

---

# Part I: Motivation

## 1.1 The Core Problem

Constraints validate. They answer: "Is this graph state valid?"

But validation alone is insufficient. Consider:

| Scenario | Constraint Alone |
|----------|------------------|
| Task needs created_at timestamp | Reject task without timestamp |
| Completed task needs completed_at | Reject completion without timestamp |
| Deleting project should delete tasks | Reject deletion (orphan constraint) |
| Manager change should update reports | Reject change (stale derived data) |

Constraints force users to anticipate and handle every derived consequence manually. This is:
- **Error-prone**: Users forget edge cases
- **Repetitive**: Same derivation logic scattered across clients
- **Inconsistent**: Different clients derive differently

**The problem:** Graphs need to maintain themselves.

## 1.2 The Core Insight

**Separate validation from transformation.**

```
┌─────────────────────────────────────────────────────────────────────┐
│              CONSTRAINTS vs RULES                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CONSTRAINTS                        RULES                          │
│   ───────────                        ─────                          │
│                                                                      │
│   Validation                         Transformation                 │
│   "Is this valid?"                   "Make this valid."             │
│   Reject bad states                  Derive good states             │
│   Passive (checked)                  Active (executed)              │
│   Answer: yes/no                     Action: change graph           │
│                                                                      │
│   Constraints define WHAT            Rules define HOW               │
│   must be true.                      to make it true.               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

Rules are **declarative transformations**: when a pattern matches, produce changes.

This is the production rule model from expert systems, applied to graphs.

## 1.3 The Analogy

A spreadsheet cell with a formula:
- The formula declares a relationship: `C1 = A1 + B1`
- When A1 or B1 changes, C1 updates automatically
- The user doesn't manually recalculate

MEW rules work the same way:
- A rule declares a transformation: `Task without timestamp → set timestamp`
- When a matching Task appears, the rule fires automatically
- The user doesn't manually derive consequences

The graph becomes a **reactive dataflow system** where derived data maintains itself.

## 1.4 Why Rules Are Not Turing Complete

This is a **design choice**, not a limitation.

| Turing Complete | MEW Rules |
|-----------------|-----------|
| Arbitrary loops | Pattern matching (finite) |
| Arbitrary recursion | Quiescence (terminates) |
| Unbounded computation | Bounded actions |
| May not halt | Always halts |
| Hard to analyze | Statically analyzable |
| Sequential execution | Parallelizable |

**The tradeoff:** Rules cannot express arbitrary algorithms. They can only express pattern-driven transformations.

**The benefit:** Rules are:
- **Guaranteed to terminate** — No infinite loops
- **Analyzable** — Can detect conflicts, redundancy, cycles
- **Parallelizable** — Independent matches can fire concurrently
- **Predictable** — Same input always produces same output

For arbitrary computation, use the compute plane (see COMPUTE.md). Rules and invocations complement each other.

## 1.5 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Declarative** | Rules state what should happen, not how |
| **Pattern-driven** | All rule activation flows from pattern matches |
| **Graph-only** | Rules affect only the graph, no I/O |
| **Terminating** | Rules always reach quiescence |
| **Transactional** | Rule effects are atomic with triggering mutation |

---

# Part II: Core Model

## 2.1 Rule Structure

A rule has two parts:

```
rule <name> [modifiers]:
  <pattern>
  =>
  <production>
```

- **Pattern**: Graph pattern that triggers the rule (same syntax as MATCH)
- **Production**: Actions to execute when pattern matches

The arrow `=>` separates condition from consequence.

## 2.2 Pattern (Condition)

The pattern is a standard graph pattern:

```
rule example:
  t: Task, p: Person,
  assigned_to(t, p)
  WHERE t.status = "done"
  =>
  ...
```

Patterns can include:
- Node bindings: `t: Task`
- Edge bindings: `assigned_to(t, p) AS a`
- Conditions: `WHERE t.priority > 5`
- Negative existence: `WHERE NOT EXISTS(n: Notification, notifies(n, t))`

The pattern defines **when** the rule fires.

## 2.3 Production (Consequence)

The production is a sequence of actions:

| Action | Effect | Example |
|--------|--------|---------|
| `SPAWN` | Create node | `SPAWN n: Notification { message = "..." }` |
| `KILL` | Delete node | `KILL t` |
| `LINK` | Create edge | `LINK assigned_to(task, person)` |
| `UNLINK` | Delete edge | `UNLINK a` (edge variable) |
| `SET` | Modify attribute | `SET t.completed_at = now()` |

Actions execute in sequence. Variables bound by SPAWN are available to subsequent actions.

```
rule on_complete:
  t: Task WHERE t.status = "done" AND t.completed_at = null
  =>
  SET t.completed_at = now(),
  SPAWN n: Notification { message = "Task done: " ++ t.title },
  LINK notifies(n, t)
```

## 2.4 Variable Scoping

Variables flow forward through productions:

```
rule example:
  a: TypeA                           -- a bound from pattern
  =>
  SPAWN b: TypeB { ref = a.name },   -- a available, b now bound
  LINK connects(a, b),               -- a, b available
  SET b.processed = true             -- a, b available
```

**Invariant:** Cannot reference a variable before it's bound.

## 2.5 Rule Modifiers

| Modifier | Default | Meaning |
|----------|---------|---------|
| `priority: N` | 0 | Execution order (higher first) |
| `auto` | yes | Fire automatically when pattern matches |
| `manual` | no | Only fire when explicitly triggered |

```
rule high_priority [priority: 100]:
  ...

rule background_cleanup [priority: -10]:
  ...

rule archive_old [manual]:
  ...
```

Priority determines order when multiple rules match. Higher priority rules execute first.

## 2.6 Auto vs Manual Rules

**Auto rules** (default):
- Fire automatically when pattern matches
- Part of transaction execution
- Always evaluated

**Manual rules**:
- Never fire automatically
- Triggered explicitly: `TRIGGER rule_name`
- Useful for batch operations, maintenance tasks

```
-- Auto: always fires
rule auto_timestamp:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()

-- Manual: only when triggered
rule archive_completed [manual]:
  t: Task WHERE t.status = "done" AND t.archived = false
  => SET t.archived = true
```

---

# Part III: Execution Model

## 3.1 When Rules Fire

Rules fire **within transactions**, after user mutations but before constraint checking:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TRANSACTION LIFECYCLE                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   1. USER MUTATIONS                                                 │
│      Client applies SPAWN, KILL, LINK, UNLINK, SET                 │
│                                                                      │
│   2. RULE EXECUTION (repeat until quiescent)                        │
│      • Find rules whose patterns now match                         │
│      • Execute productions in priority order                       │
│      • Productions may create new matches                          │
│                                                                      │
│   3. CONSTRAINT CHECKING                                            │
│      • Evaluate all affected constraints                           │
│      • Hard constraint violation → ROLLBACK                        │
│      • Soft constraint violation → WARN                            │
│                                                                      │
│   4. COMMIT or ROLLBACK                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

Rules see the graph state after user mutations. Their effects are part of the same transaction.

## 3.2 Quiescence

Rules execute until **quiescence**: the state where no rule's pattern matches new bindings.

```
                    ┌──────────────────┐
                    │  User Mutations  │
                    └────────┬─────────┘
                             │
                             ▼
              ┌─────────────────────────────┐
              │  Find matching rule patterns │◄────┐
              └──────────────┬──────────────┘     │
                             │                     │
                    ┌────────┴────────┐           │
                    │  Any new matches? │          │
                    └────────┬────────┘           │
                             │                     │
                   Yes       │        No          │
                    ┌────────┴────────┐           │
                    ▼                 ▼           │
         ┌──────────────────┐  ┌───────────────┐ │
         │ Execute highest  │  │  QUIESCENT    │ │
         │ priority rule    │  │  (done)       │ │
         └────────┬─────────┘  └───────────────┘ │
                  │                               │
                  └───────────────────────────────┘
```

**Key insight:** A rule firing may create new graph state that matches other rules (or even the same rule with different bindings). The process continues until nothing new matches.

## 3.3 Priority Ordering

When multiple rules have matching patterns:

1. **Highest priority** rule fires first
2. **Same priority** rules fire in declaration order
3. After each rule fires, re-evaluate matches (new matches may appear)

```
rule high [priority: 100]:    -- fires first
  ...

rule medium [priority: 50]:   -- fires second
  ...

rule low [priority: 0]:       -- fires last
  ...
```

**Why priority matters:** Some rules should run before others. Timestamp rules (priority: 100) should fire before notification rules (priority: 0) so notifications see correct timestamps.

## 3.4 Once-Per-Binding Semantics

A rule fires **at most once per unique binding** within a transaction.

```
rule example:
  t: Task, p: Person, assigned_to(t, p)
  => ...
```

If Task t1 is assigned to Person p1, the binding (t1, p1) triggers the rule once. Even if subsequent rule actions don't change that binding, it won't fire again for (t1, p1) in this transaction.

**Why this matters:** Prevents infinite loops where a rule keeps re-matching its own output.

## 3.5 Termination Guarantees

Rules are **guaranteed to terminate** because:

1. **Finite matches**: Pattern matching produces a finite set of bindings
2. **Once-per-binding**: Each binding fires at most once
3. **Finite graph**: The graph has finite nodes and edges
4. **No external input**: Rules cannot wait for or fetch external data

**Safety limits** (configurable):

| Limit | Purpose |
|-------|---------|
| Max actions per transaction | Prevent runaway rule chains |
| Max rule depth | Prevent deep cascades |

If limits are exceeded, the transaction fails and rolls back.

## 3.6 Introspection

Rules are stored as graph structure (Layer 0). This enables:

- **Querying rules**: Find all rules that affect a type
- **Analyzing dependencies**: Which rules trigger other rules?
- **Conflict detection**: Do two rules produce contradictory effects?
- **Impact analysis**: What happens if this rule changes?

```
-- Find all rules that affect Task nodes
MATCH r: _RuleDef, p: _PatternDef, n: _NodePattern,
      _rule_has_pattern(r, p), _pattern_has_node(p, n)
WHERE n.type_name = "Task"
RETURN r.name
```

The rule system is **self-describing**: rules are data that can be queried like any other data.

## 3.7 Rule-Constraint Interaction

Rules can **fix** what constraints would reject:

```
-- Constraint: Tasks must have timestamps
constraint task_has_timestamp:
  t: Task => t.created_at != null

-- Rule: Automatically set timestamp
rule auto_timestamp [priority: 100]:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()
```

**Flow:**
1. User spawns Task without `created_at`
2. Rule fires, sets `created_at = now()`
3. Constraint checks → passes
4. Transaction commits

Rules and constraints work together: rules derive, constraints validate the final state.

## 3.8 Referential Actions as Rules

Referential actions on edges compile to high-priority rules:

```
edge belongs_to(task: Task, project: Project) [
  on_kill_target: cascade
]
```

Compiles to:

```
rule _belongs_to_cascade_target [priority: 1000]:
  t: Task, p: Project,
  belongs_to(t, p),
  _pending_kill(p)
  =>
  KILL t
```

This makes cascading deletes, nullification, and other referential behaviors part of the rule system.

---

# Part IV: Rules and Computation

## 4.1 The Boundary

Rules handle **declarative, graph-only transformations**. They cannot:

| Cannot Do | Why | Alternative |
|-----------|-----|-------------|
| Call external APIs | No I/O | Use invocations (COMPUTE.md) |
| Wait for user input | Synchronous execution | Use invocations |
| Run arbitrary algorithms | Must terminate | Use invocations |
| Access file system | No side effects | Use invocations |
| Send notifications | External effect | Rule creates invocation → runtime sends |

**The pattern:** Rules detect conditions and create invocations. Runtimes execute invocations.

## 4.2 Rules Creating Invocations

Rules can trigger external computation by spawning invocation nodes:

```
rule request_ml_classification:
  doc: Document
  WHERE doc.status = "uploaded"
    AND NOT EXISTS(inv: Invocation, input(inv, doc), invokes(inv, #classifier))
  =>
  SPAWN inv: Invocation { status = "pending" },
  LINK invokes(inv, #classifier),
  LINK input(inv, doc)
```

**Flow:**
1. Document uploaded
2. Rule detects unprocessed document
3. Rule spawns pending invocation
4. External runtime executes classifier
5. Runtime updates graph with results

Rules stay declarative. Computation happens externally.

## 4.3 Choosing Between Rules and Invocations

| Use Rules | Use Invocations |
|-----------|-----------------|
| Derive timestamps | Call external APIs |
| Cascade deletes | ML inference |
| Maintain counters | Document processing |
| Create derived edges | Cryptographic operations |
| Validate and fix state | Long-running tasks |

**Rule of thumb:** If it only touches the graph and always terminates, use a rule. Otherwise, use an invocation.

---

# Part V: Examples

## 5.1 Automatic Timestamps

```
rule auto_created_at [priority: 100]:
  e: Entity WHERE e.created_at = null
  => SET e.created_at = now()
```

Every entity gets `created_at` automatically. Users never forget.

## 5.2 Cascading Status Changes

```
rule cascade_project_archive:
  p: Project, t: Task,
  belongs_to(t, p)
  WHERE p.archived = true AND t.archived = false
  =>
  SET t.archived = true
```

Archiving a project archives all its tasks. One action, automatic propagation.

## 5.3 Derived Edges (Transitive Closure)

```
rule transitive_reports_to:
  a: Person, b: Person, c: Person,
  reports_to(a, b),
  reports_to(b, c)
  WHERE NOT EXISTS(indirectly_reports_to(a, c))
  =>
  LINK indirectly_reports_to(a, c)
```

The org chart maintains its transitive closure automatically.

## 5.4 Notification System

```
rule notify_on_assignment:
  t: Task, p: Person,
  assigned_to(t, p) AS a
  WHERE a.notified = false
  =>
  SPAWN n: Notification {
    message = "You were assigned: " ++ t.title,
    recipient_id = p._id
  },
  LINK notification_for(n, t),
  SET a.notified = true
```

Assignments automatically create notifications. The `notified` flag prevents duplicate notifications.

## 5.5 Idempotent Derived State

```
rule ensure_project_has_owner:
  p: Project, creator: Person,
  created_by(p, creator)
  WHERE NOT EXISTS(o: Person, owns(p, o))
  =>
  LINK owns(p, creator)
```

Projects always have an owner. If no explicit owner, the creator becomes owner.

## 5.6 Triggering External Processing

```
rule classify_new_document:
  doc: Document
  WHERE doc.classification = null
    AND NOT EXISTS(inv: Invocation, input(inv, doc))
  =>
  SPAWN inv: Invocation { status = "pending" },
  LINK invokes(inv, #document_classifier),
  LINK input(inv, doc)
```

Rule detects unclassified documents and queues them for ML processing. The actual classification runs externally.

---

# Part VI: Summary

## 6.1 What Rules Provide

| Capability | Benefit |
|------------|---------|
| Declarative transformations | Readable, maintainable |
| Pattern-driven activation | React to graph changes |
| Automatic quiescence | No manual coordination |
| Transactional execution | Atomic with user mutations |
| Constraint integration | Rules fix, constraints validate |

## 6.2 What Rules Do Not Provide

| Not Provided | Alternative |
|--------------|-------------|
| Arbitrary computation | Invocations (COMPUTE.md) |
| External I/O | Invocations |
| User interaction | Application layer |
| Complex control flow | Multiple rules with patterns |

## 6.3 Core Invariants

| Invariant | Meaning |
|-----------|---------|
| **Termination** | Rules always reach quiescence |
| **Determinism** | Same input → same output |
| **Atomicity** | Rule effects commit or rollback with transaction |
| **Graph-only** | Rules cannot have side effects outside the graph |

## 6.4 The Pattern

```
┌─────────────────────────────────────────────────────────────────────┐
│                         THE PATTERN                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Constraints validate: "Is this state valid?"                      │
│   Rules transform: "How do I make it valid / derived?"              │
│                                                                      │
│   Rules are:                                                        │
│   • Declarative (pattern → production)                              │
│   • Bounded (guaranteed termination)                                │
│   • Parallelizable (independent matches)                            │
│   • Graph-only (no external effects)                                │
│                                                                      │
│   For arbitrary computation: use invocations.                       │
│   Rules and invocations complement each other.                      │
│                                                                      │
│   The graph maintains itself.                                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of MEW Rules Capability*
