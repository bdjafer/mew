# MEW Timing & Execution Model

**Version:** 1.0
**Status:** Capability
**Deferred to:** v2
**Scope:** Time representation, tick models, execution semantics, quiescence

---

# Part I: Context & Motivation

## 1.1 The Problem

MEW graphs evolve through mutations and rules. But *when* do things happen? Several questions arise:

- What does `now()` return?
- When do rules fire?
- What if rule execution takes too long?
- Can different parts of the graph evolve at different rates?
- How does external time relate to internal time?

Without clear answers, behavior is unpredictable and use cases are limited.

## 1.2 Requirements

Different applications need different timing models:

| Application | Timing Needs |
|-------------|--------------|
| Database | Wall-clock timestamps, immediate consistency |
| Game | Fixed tick rate (60 Hz), deterministic replay |
| Simulation | Logical time, faster/slower than real-time |
| Real-time system | Bounded latency, predictable execution |
| Event sourcing | Logical ordering, causal consistency |

A single hardcoded model cannot serve all these. MEW needs flexibility.

## 1.3 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Dual time** | Both logical and wall-clock time available |
| **Explicit advancement** | Time doesn't advance implicitly during execution |
| **Configurable execution** | Tick-driven, event-driven, or hybrid |
| **Bounded by default** | Execution has limits to prevent runaway |
| **Determinism when needed** | Logical time enables replay |

---

# Part II: Time Model

## 2.1 Two Kinds of Time

MEW provides two distinct time concepts:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TIME CONCEPTS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WALL TIME                           LOGICAL TIME                   │
│  ─────────                           ────────────                   │
│                                                                      │
│  wall_time() → Timestamp             logical_time() → Int           │
│                                                                      │
│  • Real-world clock                  • Abstract counter             │
│  • Monotonic                         • Monotonic                    │
│  • Advances continuously             • Advances explicitly          │
│  • Non-deterministic                 • Deterministic                │
│  • For: timestamps, TTLs,            • For: ordering, causality,    │
│         real-world deadlines               simulation, replay       │
│                                                                      │
│  Cannot control                      Controlled by tick             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Wall Time

```
wall_time() → Timestamp
```

Returns the current real-world time (milliseconds since epoch).

**Properties:**
- Always available
- Monotonic (never goes backward)
- Not controlled by MEW
- Same call in same transaction may return different values
- Non-deterministic (replaying same mutations yields different wall times)

**Use cases:**
- `created_at: Timestamp = wall_time()`
- `WHERE wall_time() > t.expires_at`
- Audit trails, logging

**Not suitable for:**
- Deterministic simulation
- Replay/debugging
- Ordering guarantees

## 2.3 Logical Time

```
logical_time() → Int
```

Returns the current logical tick counter.

**Properties:**
- Starts at 0 (or configured initial value)
- Only advances via explicit `TICK` or configured triggers
- Same within a tick (all operations in tick see same value)
- Deterministic (same inputs → same logical times)

**Use cases:**
- `turn: Int = logical_time()`
- `WHERE logical_time() - t.created_tick > 100`
- Game turns, simulation steps
- Causal ordering

## 2.4 The `now()` Function

For convenience, `now()` is an alias with configurable binding:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        now() CONFIGURATION                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SET time.now_source = "wall"     -- now() = wall_time()           │
│  SET time.now_source = "logical"  -- now() = logical_time()        │
│  SET time.now_source = "hybrid"   -- now() = (wall_time(),         │
│                                   --          logical_time())       │
│                                                                      │
│  Default: "wall"                                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Rationale:** Most users expect `now()` to mean wall clock. Simulations can rebind it to logical time for determinism.

## 2.5 Time in Different Contexts

| Context | `wall_time()` | `logical_time()` |
|---------|---------------|------------------|
| Attribute default | Evaluated at SPAWN/LINK | Evaluated at SPAWN/LINK |
| Rule condition (WHERE) | Evaluated at match time | Evaluated at match time |
| Rule action (SET) | Evaluated at execution | Evaluated at execution |
| Query (MATCH WHERE) | Evaluated at query time | Evaluated at query time |
| Constraint condition | **Not allowed** (non-deterministic) | Allowed |

**Constraint restriction:** Constraints must be deterministic. `wall_time()` in constraints would make validation non-repeatable.

```
-- COMPILE ERROR
constraint recent_only:
  t: Task
  => wall_time() - t.created_at < 86400000
-- Error: wall_time() not allowed in constraints

-- OK: use logical time
constraint not_too_old:
  t: Task
  => logical_time() - t.created_tick < 100
```

---

# Part III: Tick Model

## 3.1 What Is a Tick?

A **tick** is the fundamental unit of logical time advancement.

```
┌─────────────────────────────────────────────────────────────────────┐
│                          TICK SEMANTICS                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Before tick N:                                                     │
│    logical_time() = N                                               │
│    Graph in state S_N                                               │
│                                                                      │
│  During tick N:                                                     │
│    Process pending mutations                                        │
│    Fire rules (to quiescence or limit)                             │
│    Check constraints                                                │
│                                                                      │
│  After tick N:                                                      │
│    logical_time() = N + 1                                          │
│    Graph in state S_{N+1}                                          │
│                                                                      │
│  All operations within tick N see logical_time() = N               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.2 Tick Triggers

What causes a tick to occur?

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TICK TRIGGER MODES                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  A. MANUAL                                                          │
│     ──────                                                          │
│     Tick occurs only when explicitly requested                      │
│                                                                      │
│     TICK                         -- advance by 1                    │
│     TICK 10                      -- advance by 10                   │
│                                                                      │
│     Use: Simulations, turn-based games, testing                    │
│                                                                      │
│  B. PER-TRANSACTION                                                 │
│     ───────────────                                                 │
│     Each committed transaction advances the tick                    │
│                                                                      │
│     SET time.tick_on = "commit"                                    │
│                                                                      │
│     Use: Event sourcing, causal ordering                           │
│                                                                      │
│  C. PERIODIC                                                        │
│     ────────                                                        │
│     Tick occurs at fixed wall-clock intervals                      │
│                                                                      │
│     SET time.tick_interval = 16ms    -- ~60 Hz                     │
│     SET time.tick_interval = 100ms   -- 10 Hz                      │
│                                                                      │
│     Use: Games, real-time simulations, polling                     │
│                                                                      │
│  D. HYBRID                                                          │
│     ──────                                                          │
│     Combination: periodic baseline + manual advancement            │
│                                                                      │
│     SET time.tick_interval = 100ms                                 │
│     SET time.tick_on = "commit"      -- also tick on commit        │
│                                                                      │
│     Use: Real-time with event bursts                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.3 Tick Scope

By default, MEW uses a **global tick** — one tick counter for the entire graph.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GLOBAL TICK                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  One tick counter for entire graph.                                │
│  All rules fire on same tick.                                      │
│                                                                      │
│     logical_time() = 42 (everywhere)                               │
│                                                                      │
│  This is the default and simplest model.                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Scoped Timing (v2+):** For different parts of the graph to have independent tick rates, use **world interiors** from [INTERIOR.md](./INTERIOR.md). Each interior can be configured with its own time relationship to its parent:

```
node Agent [has_interior] {
  interior: ontology [time: shared] { ... }      -- ticks with parent
}

node Simulation [has_interior] {
  interior: ontology [time: independent] { ... } -- manual tick only
}

node SlowProcess [has_interior] {
  interior: ontology [time: ratio(10)] { ... }   -- ticks every 10th parent tick
}
```

This unifies timing and structural isolation: every interior IS a time scope. See INTERIOR.md Part VIII for full details on time configuration in worlds.

---

# Part IV: Execution Model

## 4.1 Execution Phases

```
┌─────────────────────────────────────────────────────────────────────┐
│                       EXECUTION PHASES                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐                                                    │
│  │   ACCEPT    │  Mutations enter the system                       │
│  │             │  (immediate or queued)                            │
│  └──────┬──────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────┐                                                    │
│  │   PROCESS   │  Apply mutations to graph                         │
│  │             │  Type check, authorization                        │
│  └──────┬──────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────┐                                                    │
│  │    REACT    │  Rules fire                                       │
│  │             │  (to quiescence or limit)                         │
│  └──────┬──────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────┐                                                    │
│  │  VALIDATE   │  Check constraints                                │
│  │             │  (hard fail → rollback)                           │
│  └──────┬──────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────┐                                                    │
│  │   COMMIT    │  Persist changes                                  │
│  │             │  Notify subscribers                               │
│  └─────────────┘                                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.2 Input Modes

How do mutations enter the system?

```
┌─────────────────────────────────────────────────────────────────────┐
│                         INPUT MODES                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  IMMEDIATE (default)                                                │
│  ───────────────────                                                │
│                                                                      │
│  Mutation processed synchronously.                                  │
│  Client blocks until complete.                                      │
│                                                                      │
│     Client ──SPAWN t: Task──▶ MEW ──#t returned──▶ Client          │
│                                                                      │
│  SET execution.input_mode = "immediate"                            │
│                                                                      │
│                                                                      │
│  QUEUED                                                             │
│  ──────                                                             │
│                                                                      │
│  Mutation added to queue, acknowledged.                            │
│  Processed later (on tick or batch).                               │
│                                                                      │
│     Client ──SPAWN t: Task──▶ Queue ──ack──▶ Client                │
│                                  │                                  │
│                               (later)                               │
│                                  │                                  │
│                                  ▼                                  │
│                              Process                                │
│                                                                      │
│  SET execution.input_mode = "queued"                               │
│  SET execution.queue_capacity = 10000                              │
│                                                                      │
│                                                                      │
│  BATCHED                                                            │
│  ───────                                                            │
│                                                                      │
│  Mutations accumulate until tick.                                  │
│  All processed together.                                           │
│                                                                      │
│     SET execution.input_mode = "batched"                           │
│     SET execution.batch_window = 16ms                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.3 Rule Execution Modes

When and how do rules fire?

```
┌─────────────────────────────────────────────────────────────────────┐
│                     RULE EXECUTION MODES                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  EAGER (default)                                                    │
│  ───────────────                                                    │
│                                                                      │
│  Rules fire immediately after each mutation.                       │
│  Recursive until quiescence (or limit).                            │
│  Part of the same transaction.                                     │
│                                                                      │
│     Mutation → Rules → Rules → ... → Quiescence → Commit           │
│                                                                      │
│  SET execution.rule_mode = "eager"                                 │
│                                                                      │
│                                                                      │
│  DEFERRED                                                           │
│  ────────                                                           │
│                                                                      │
│  Mutations commit without rules.                                   │
│  Rules fire on next tick.                                          │
│                                                                      │
│     Mutation → Commit                                              │
│     (tick)                                                          │
│     Rules → Rules → ... → Quiescence                               │
│                                                                      │
│  SET execution.rule_mode = "deferred"                              │
│                                                                      │
│                                                                      │
│  EXPLICIT                                                           │
│  ────────                                                           │
│                                                                      │
│  Rules only fire when explicitly invoked.                          │
│                                                                      │
│     Mutation → Commit                                              │
│     PROCESS RULES  -- explicit call                                │
│     Rules → Quiescence                                             │
│                                                                      │
│  SET execution.rule_mode = "explicit"                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.4 Quiescence

**Quiescence** = No rules can fire (no patterns match, or all matches already processed).

```
┌─────────────────────────────────────────────────────────────────────┐
│                     QUIESCENCE HANDLING                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  STRICT (require quiescence)                                        │
│  ───────────────────────────                                        │
│                                                                      │
│  Transaction waits for full quiescence.                            │
│  If limits hit before quiescence: rollback.                        │
│                                                                      │
│  SET execution.quiescence = "strict"                               │
│                                                                      │
│                                                                      │
│  BOUNDED (limit then commit)                                        │
│  ───────────────────────────                                        │
│                                                                      │
│  Process rules up to limit.                                        │
│  Commit whatever state exists.                                     │
│  Remaining rule matches deferred to next tick.                     │
│                                                                      │
│  SET execution.quiescence = "bounded"                              │
│                                                                      │
│                                                                      │
│  BEST-EFFORT (time-bounded)                                        │
│  ──────────────────────────                                         │
│                                                                      │
│  Process rules until time budget exhausted.                        │
│  Commit current state.                                             │
│  Continue on next tick.                                            │
│                                                                      │
│  SET execution.quiescence = "best_effort"                          │
│  SET execution.time_budget = 10ms                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.5 Execution Limits

Regardless of quiescence mode, limits prevent runaway:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      EXECUTION LIMITS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  LIMIT                │ DEFAULT  │ PURPOSE                          │
│  ─────────────────────┼──────────┼────────────────────────────────  │
│  max_rule_depth       │ 100      │ Nested rule trigger depth       │
│  max_rule_actions     │ 10,000   │ Total actions per tick          │
│  max_execution_time   │ 1000ms   │ Wall-clock budget               │
│  max_queue_size       │ 10,000   │ Pending mutations               │
│  max_batch_size       │ 1,000    │ Mutations per batch             │
│                                                                      │
│  Configuration:                                                     │
│                                                                      │
│  SET execution.max_rule_depth = 100                                │
│  SET execution.max_rule_actions = 10000                            │
│  SET execution.max_execution_time = 1000ms                         │
│                                                                      │
│  Behavior on limit:                                                 │
│                                                                      │
│  SET execution.on_limit = "rollback"   -- abort transaction        │
│  SET execution.on_limit = "commit"     -- commit partial           │
│  SET execution.on_limit = "defer"      -- commit, continue later   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part V: Timing Configurations

## 5.1 Preset Configurations

Common configurations as presets:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     CONFIGURATION PRESETS                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  PRESET: database                                                   │
│  ────────────────────                                               │
│                                                                      │
│  Traditional database behavior.                                     │
│                                                                      │
│    time.now_source = "wall"                                        │
│    time.tick_on = "commit"                                         │
│    execution.input_mode = "immediate"                              │
│    execution.rule_mode = "eager"                                   │
│    execution.quiescence = "strict"                                 │
│                                                                      │
│                                                                      │
│  PRESET: game                                                       │
│  ────────────────                                                   │
│                                                                      │
│  Fixed timestep game loop.                                         │
│                                                                      │
│    time.now_source = "logical"                                     │
│    time.tick_interval = 16ms                                       │
│    execution.input_mode = "batched"                                │
│    execution.rule_mode = "eager"                                   │
│    execution.quiescence = "bounded"                                │
│                                                                      │
│                                                                      │
│  PRESET: simulation                                                 │
│  ──────────────────────                                             │
│                                                                      │
│  Deterministic simulation, manual time control.                    │
│                                                                      │
│    time.now_source = "logical"                                     │
│    time.tick_on = "manual"                                         │
│    execution.input_mode = "queued"                                 │
│    execution.rule_mode = "eager"                                   │
│    execution.quiescence = "strict"                                 │
│                                                                      │
│                                                                      │
│  PRESET: streaming                                                  │
│  ────────────────────                                               │
│                                                                      │
│  High-throughput event processing.                                 │
│                                                                      │
│    time.now_source = "wall"                                        │
│    time.tick_interval = 100ms                                      │
│    execution.input_mode = "queued"                                 │
│    execution.rule_mode = "deferred"                                │
│    execution.quiescence = "best_effort"                            │
│    execution.time_budget = 90ms                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Configuration Syntax

```
-- Load preset
CONFIGURE PRESET game

-- Override specific settings
SET time.tick_interval = 33ms    -- 30 Hz instead of 60 Hz
SET execution.max_rule_actions = 50000

-- Query current configuration
SHOW CONFIGURATION
SHOW CONFIGURATION time
SHOW CONFIGURATION execution
```

> **Note:** Per-scope timing configuration is done via world interiors. See [INTERIOR.md](./INTERIOR.md) for `[time: shared]`, `[time: independent]`, and `[time: ratio(N)]` options.

---

# Part VI: Determinism & Replay

## 6.1 Determinism Requirements

For deterministic execution (same inputs → same outputs):

```
┌─────────────────────────────────────────────────────────────────────┐
│                   DETERMINISM REQUIREMENTS                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  REQUIRED                                                           │
│  ────────                                                           │
│  • Use logical time, not wall time                                 │
│  • Fixed rule priority ordering                                    │
│  • Deterministic ID allocation                                     │
│  • Same mutation order                                             │
│  • No external randomness                                          │
│                                                                      │
│  PROHIBITED                                                         │
│  ──────────                                                         │
│  • wall_time() in rules/constraints                                │
│  • Non-deterministic functions (random(), uuid())                  │
│  • Concurrent mutation without defined merge                       │
│  • Time-based limits (execution.max_execution_time)                │
│                                                                      │
│  Configuration for determinism:                                     │
│                                                                      │
│  SET execution.deterministic = true                                │
│  -- Enforces: logical time, count-based limits, ordered execution  │
│  -- Errors on: wall_time() usage, random functions                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 6.2 Replay

With deterministic configuration, execution can be replayed:

```
REPLAY SEMANTICS
────────────────

Record:
  • Initial graph state (or snapshot)
  • Sequence of (tick, mutations) pairs
  • Configuration

Replay:
  • Load initial state
  • Apply configuration
  • For each (tick, mutations):
      Apply mutations
      TICK
  • Final state matches original

Use cases:
  • Debugging ("what happened at tick 1000?")
  • Testing (deterministic test cases)
  • Replication (followers replay leader's log)
  • Time travel (restore to any tick)
```

---

# Part VII: Interaction with Other Systems

## 7.1 Authorization Timing

> **Cross-reference:** See [POLICY.md](../POLICY.md) for full policy evaluation semantics.

When is authorization checked?

```
IMMEDIATE INPUT MODE
────────────────────

  Mutation arrives → Authorization check → Process

  Authorization checked synchronously.
  Reject immediately if denied.


QUEUED INPUT MODE
─────────────────

  Option A: Check at queue time
    Mutation → Auth check → Queue → Process
    Pro: Fast rejection
    Con: Authorization state may change before processing

  Option B: Check at process time
    Mutation → Queue → Auth check → Process
    Pro: Authorization reflects current state
    Con: Wasted queue space for denied mutations

  Configuration:
    SET authorization.check_at = "queue" | "process"
    Default: "process"
```

## 7.2 Watch Timing

> **Cross-reference:** See [WATCH.md](../WATCH.md) for full watch semantics including visibility options.

When do subscribers see changes?

```
┌─────────────────────────────────────────────────────────────────────┐
│                   SUBSCRIPTION VISIBILITY                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  COMMITTED (default)                                                │
│  ───────────────────                                                │
│                                                                      │
│  Subscribers see changes after commit.                             │
│  Consistent view.                                                   │
│                                                                      │
│     Mutation → Rules → Commit → Notify subscribers                 │
│                                                                      │
│                                                                      │
│  IMMEDIATE                                                          │
│  ─────────                                                          │
│                                                                      │
│  Subscribers see changes during transaction.                       │
│  May see uncommitted state (rolled back later).                    │
│                                                                      │
│     Mutation → Notify → Rules → Notify → Commit                    │
│                                                                      │
│  Configuration:                                                     │
│                                                                      │
│     WATCH pattern [visibility: committed | immediate]              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.3 Branching Timing

In branching execution (v2+):

```
Each branch has independent logical time.

Branch point:
  Parent at tick N
  Child A at tick N
  Child B at tick N

After branching:
  TICK in A: A at tick N+1, B still at N
  TICK in B: B at tick N+1, A unchanged

Global tick (TICK ALL):
  Advances all branches

Query:
  logical_time()           -- current branch
  logical_time(#branch_a)  -- specific branch
```

## 7.4 Rule Execution Timing

> **Cross-reference:** See [RULES.md](../RULES.md) for full rule semantics including firing order.

How timing affects rule execution:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    RULE EXECUTION TIMING                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  EAGER MODE                                                         │
│  ──────────                                                         │
│                                                                      │
│  Rules fire immediately after each mutation.                        │
│  All rules see same logical_time() within a tick.                  │
│  Cascading rules execute within same transaction.                  │
│                                                                      │
│     Mutation → Rule₁ → Rule₂ → ... → Quiescence → Commit           │
│              └────── all see logical_time() = N ──────┘            │
│                                                                      │
│  DEFERRED MODE                                                      │
│  ─────────────                                                      │
│                                                                      │
│  Rules fire on next tick.                                          │
│  Mutations see logical_time() = N.                                 │
│  Rules see logical_time() = N+1.                                   │
│                                                                      │
│     Mutation (tick N) → Commit                                     │
│     TICK                                                            │
│     Rules (tick N+1) → Quiescence                                  │
│                                                                      │
│  SCOPED RULES (v2+)                                                │
│  ──────────────────                                                │
│                                                                      │
│  Rules declared in world interiors fire on their scope's tick.     │
│  See INTERIOR.md for scoped timing configuration.                  │
│                                                                      │
│     node Simulation [has_interior] {                               │
│       interior: ontology [time: ratio(10)] {                       │
│         rule physics_step: p: Particle => SET p.pos = ...          │
│       }                                                            │
│     }                                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VIII: Grammar Extensions

```
TimeExpr =
    "wall_time" "(" ")"
  | "logical_time" "(" ScopeRef? ")"         -- optional scope for interior time
  | "now" "(" ")"

ScopeRef =
    "SELF" | "PARENT" | "ROOT" | NodeRef

TickStmt =
    "TICK"
  | "TICK" IntLiteral                        -- advance by N

ConfigStmt =
    "CONFIGURE" "PRESET" Identifier
  | "SET" ConfigPath "=" ConfigValue
  | "SHOW" "CONFIGURATION" ConfigPath?

ConfigPath = Identifier ("." Identifier)*
```

> **Note:** `logical_time(ScopeRef)` queries a specific interior's tick count. Scoped tick statements (`TICK IN #scope`) are defined in [INTERIOR.md](./INTERIOR.md).

---

# Part IX: Error Model

## 9.1 Error Categories

Timing and execution errors are grouped into categories:

| Category | Code Range | Description |
|----------|------------|-------------|
| Tick | E9001-E9010 | Tick advancement errors |
| Configuration | E9011-E9020 | Invalid configuration settings |
| Execution | E9021-E9030 | Execution limit and runtime errors |
| Determinism | E9031-E9040 | Determinism mode violations |

## 9.2 Tick Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9001 | INVALID_TICK_AMOUNT | Negative or zero tick amount | `INVALID_TICK_AMOUNT: Tick amount must be positive, got {amount}` |

> **Note:** Scoped tick errors (invalid scope reference, circular time relationships) are defined in [INTERIOR.md](./INTERIOR.md).

## 9.3 Configuration Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9011 | INVALID_CONFIG_VALUE | Configuration value out of range or wrong type | `INVALID_CONFIG_VALUE: '{path}' expects {expected}, got {actual}` |
| E9012 | UNKNOWN_CONFIG_PATH | Configuration path doesn't exist | `UNKNOWN_CONFIG_PATH: Configuration path '{path}' is not recognized` |
| E9013 | INCOMPATIBLE_SETTINGS | Configuration settings conflict | `INCOMPATIBLE_SETTINGS: '{setting1}' is incompatible with '{setting2}'` |
| E9014 | UNKNOWN_PRESET | Preset name not recognized | `UNKNOWN_PRESET: Preset '{name}' is not defined` |
| E9015 | INVALID_DURATION | Duration format invalid | `INVALID_DURATION: '{value}' is not a valid duration` |

## 9.4 Execution Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9021 | RULE_DEPTH_EXCEEDED | Nested rule triggers exceed limit | `RULE_DEPTH_EXCEEDED: Maximum rule depth {limit} exceeded` |
| E9022 | RULE_ACTIONS_EXCEEDED | Total rule actions exceed limit | `RULE_ACTIONS_EXCEEDED: Maximum rule actions {limit} exceeded` |
| E9023 | EXECUTION_TIMEOUT | Wall-clock budget exhausted | `EXECUTION_TIMEOUT: Execution time {elapsed}ms exceeded limit {limit}ms` |
| E9024 | QUEUE_OVERFLOW | Mutation queue capacity exceeded | `QUEUE_OVERFLOW: Mutation queue capacity {limit} exceeded` |
| E9025 | BATCH_SIZE_EXCEEDED | Batch mutation limit exceeded | `BATCH_SIZE_EXCEEDED: Batch size {count} exceeds limit {limit}` |
| E9026 | QUIESCENCE_NOT_REACHED | Strict quiescence mode, limit hit | `QUIESCENCE_NOT_REACHED: Rules did not reach quiescence within limits` |

## 9.5 Determinism Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9031 | WALL_TIME_IN_DETERMINISTIC | wall_time() used in deterministic mode | `WALL_TIME_IN_DETERMINISTIC: wall_time() is not allowed in deterministic mode` |
| E9032 | RANDOM_IN_DETERMINISTIC | Random function in deterministic mode | `RANDOM_IN_DETERMINISTIC: random() is not allowed in deterministic mode` |
| E9033 | TIME_LIMIT_IN_DETERMINISTIC | Time-based limit in deterministic mode | `TIME_LIMIT_IN_DETERMINISTIC: max_execution_time is not allowed in deterministic mode; use count-based limits` |
| E9034 | NONDETERMINISTIC_FUNCTION | Non-deterministic function in constraint | `NONDETERMINISTIC_FUNCTION: Function '{name}' is non-deterministic and cannot be used in constraints` |

---

# Part X: Versioning Considerations

## 10.1 v1 Anticipation

v1 implementations can prepare for timing features by:

**Schema design:**
- Include `created_at: Timestamp` attributes for audit
- Use integer fields like `turn` or `step` for logical time references
- Avoid hardcoded timing assumptions

**Query patterns:**
- Use relative time comparisons where possible
- Parameterize time values rather than hardcoding

## 10.2 v2 Implementation

v2 introduces the full timing model:

**Core features:**
- `wall_time()`, `logical_time()`, `now()` functions
- Configurable `now()` binding
- TICK statement for explicit time advancement
- Execution presets (database, game, simulation, streaming)
- Execution limits and quiescence modes

**Configuration system:**
- `SET` statements for runtime configuration
- `CONFIGURE PRESET` for preset loading
- Per-ontology timing configuration

## 10.3 v2+ Extensions

Future versions may extend with:

**Scoped timing (via INTERIOR.md):**
- World interiors with independent tick rates
- Hierarchical time relationships (shared, independent, ratio)
- See [INTERIOR.md](./INTERIOR.md) Part VIII for implementation

**Advanced replay:**
- Full deterministic replay infrastructure
- Time-travel debugging
- Checkpoint/restore semantics

**Branching time (v3+):**
- Per-branch logical time
- Branch synchronization primitives
- Temporal queries across branches

---

# Part XI: Summary

## 11.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **Wall time** | Real-world clock, continuous, non-deterministic |
| **Logical time** | Abstract counter, explicit advancement, deterministic |
| **Tick** | Unit of logical time advancement |
| **Interior** | World scope with independent tick rate (see [INTERIOR.md](./INTERIOR.md)) |
| **Quiescence** | State where no rules can fire |
| **Input mode** | How mutations enter (immediate, queued, batched) |
| **Rule mode** | When rules fire (eager, deferred, explicit) |

## 11.2 Configuration Summary

| Setting | Options | Default |
|---------|---------|---------|
| `time.now_source` | wall, logical, hybrid | wall |
| `time.tick_on` | manual, commit, interval | commit |
| `time.tick_interval` | Duration | — |
| `execution.input_mode` | immediate, queued, batched | immediate |
| `execution.rule_mode` | eager, deferred, explicit | eager |
| `execution.quiescence` | strict, bounded, best_effort | strict |
| `execution.on_limit` | rollback, commit, defer | rollback |
| `execution.deterministic` | true, false | false |

## 11.3 Preset Summary

| Preset | Use Case | Key Characteristics |
|--------|----------|---------------------|
| database | Traditional DB | Wall time, immediate, eager, strict |
| game | Fixed timestep | Logical time, batched, bounded |
| simulation | Deterministic sim | Logical time, manual tick, strict |
| streaming | High throughput | Wall time, queued, best-effort |

---

# Appendix A: Complete Grammar

This appendix consolidates all grammar extensions introduced by the timing and execution model.

```ebnf
(* Time Expressions *)
TimeExpr         = "wall_time" "(" ")"
                 | "logical_time" "(" ScopeRef? ")"
                 | "now" "(" ")" ;

ScopeRef         = "SELF" | "PARENT" | "ROOT" | NodeRef ;

(* Tick Statements *)
TickStmt         = "TICK"
                 | "TICK" IntLiteral ;

(* Configuration Statements *)
ConfigStmt       = "CONFIGURE" "PRESET" Identifier
                 | "SET" ConfigPath "=" ConfigValue
                 | "SHOW" "CONFIGURATION" ConfigPath? ;

ConfigPath       = Identifier ("." Identifier)* ;

ConfigValue      = StringLiteral
                 | IntLiteral
                 | Duration
                 | BoolLiteral ;

Duration         = IntLiteral ("ms" | "s" | "m" | "h") ;

BoolLiteral      = "true" | "false" ;

(* Process Rules Statement *)
ProcessRulesStmt = "PROCESS" "RULES" ;
```

> **Note:** `logical_time(ScopeRef)` queries a specific interior's tick count. Scoped tick syntax (`TICK IN #scope`, `TICK ALL`, `TICK CHILDREN`) is defined in [INTERIOR.md](./INTERIOR.md).

---

# Appendix B: Layer 0 Extensions

The timing model extends Layer 0 with configuration metadata.

## B.1 Nodes

```mew
node _TimingConfig [sealed, singleton] {
  now_source: String [in: ["wall", "logical", "hybrid"]] = "wall",
  tick_on: String [in: ["manual", "commit", "interval"]] = "commit",
  tick_interval_ms: Int?,
  doc: String?
}

node _ExecutionConfig [sealed, singleton] {
  input_mode: String [in: ["immediate", "queued", "batched"]] = "immediate",
  rule_mode: String [in: ["eager", "deferred", "explicit"]] = "eager",
  quiescence: String [in: ["strict", "bounded", "best_effort"]] = "strict",
  on_limit: String [in: ["rollback", "commit", "defer"]] = "rollback",
  deterministic: Bool = false,
  max_rule_depth: Int = 100,
  max_rule_actions: Int = 10000,
  max_execution_time_ms: Int = 1000,
  max_queue_size: Int = 10000,
  max_batch_size: Int = 1000,
  time_budget_ms: Int?,
  queue_capacity: Int?,
  batch_window_ms: Int?,
  doc: String?
}

node _TickState [sealed, singleton] {
  global_tick: Int = 0,
  last_tick_at: Timestamp?
}
```

> **Note:** Scoped timing metadata (`_Interior` with time configuration) is defined in [INTERIOR.md](./INTERIOR.md).

## B.2 Edges

```mew
edge _ontology_has_timing_config(ontology: _Ontology, config: _TimingConfig) {}

edge _ontology_has_execution_config(ontology: _Ontology, config: _ExecutionConfig) {}
```

## B.3 Constraints

```mew
constraint _singleton_timing_config:
  ontology: _Ontology =>
    COUNT(c: _TimingConfig, _ontology_has_timing_config(ontology, c)) <= 1

constraint _singleton_execution_config:
  ontology: _Ontology =>
    COUNT(c: _ExecutionConfig, _ontology_has_execution_config(ontology, c)) <= 1
```

---

# Appendix C: Glossary

| Term | Definition |
|------|------------|
| **Batched input** | Mutations accumulate until tick, then processed together |
| **Best-effort quiescence** | Process rules until time budget exhausted, commit current state |
| **Bounded quiescence** | Process rules up to limit, commit whatever state exists |
| **Deferred rules** | Rules fire on next tick rather than immediately |
| **Determinism** | Same inputs always produce same outputs |
| **Eager rules** | Rules fire immediately after each mutation |
| **Explicit rules** | Rules only fire when explicitly invoked via PROCESS RULES |
| **Global tick** | Single tick counter for entire graph |
| **Hybrid time** | `now()` returns both wall and logical time as tuple |
| **Immediate input** | Mutations processed synchronously, client blocks |
| **Interior** | World scope with independent timing; see [INTERIOR.md](./INTERIOR.md) |
| **Logical time** | Abstract counter that advances explicitly via tick |
| **Preset** | Named bundle of timing and execution configuration |
| **Queued input** | Mutations added to queue, acknowledged, processed later |
| **Quiescence** | State where no rules can fire |
| **Replay** | Re-executing recorded mutations to reproduce state |
| **Scoped timing** | Different tick rates per interior; see [INTERIOR.md](./INTERIOR.md) |
| **Strict quiescence** | Transaction waits for full quiescence; rollback if limits hit |
| **Tick** | Fundamental unit of logical time advancement |
| **Tick trigger** | What causes a tick: manual, per-commit, periodic, or hybrid |
| **Wall time** | Real-world clock time, continuous, non-deterministic |

---

*End of MEW Timing & Execution Model Capability*
