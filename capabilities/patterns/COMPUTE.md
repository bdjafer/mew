# MEW Compute Pattern

**Version:** 1.0
**Status:** Draft
**Scope:** Code execution, invocation patterns, and the control/compute plane separation

---

# Part I: Motivation

## 1.1 The Core Problem

MEW rules express declarative transformations: pattern → production. They are analyzable, parallelizable, and guaranteed to terminate. But some computation cannot be expressed declaratively:

| Computation Type | Why Rules Can't Express It |
|------------------|---------------------------|
| External API calls | Rules have no I/O |
| ML inference | Requires models, tensor operations |
| Document parsing | Complex algorithms |
| Cryptographic operations | Specialized libraries |
| Unbounded iteration | Rules match finite patterns |

Adding Turing-complete execution to the kernel would destroy the properties that make rules tractable.

## 1.2 The Core Insight

**Separate control plane from compute plane.**

```
┌─────────────────────────────────────────────────────────────────────┐
│                 CONTROL PLANE vs COMPUTE PLANE                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CONTROL PLANE (Graph)              COMPUTE PLANE (Execution)      │
│   ─────────────────────              ─────────────────────────      │
│                                                                      │
│   Definitions                        Actualization                  │
│   What code exists                   Running code                   │
│   When to invoke                     Doing the work                 │
│   Inputs and outputs                 Processing                     │
│   Status and results                 Side effects                   │
│                                                                      │
│   Small, structured                  Arbitrary computation          │
│   Queryable, indexable               Opaque to kernel               │
│   Kernel's concern                   Runtime's concern              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

The graph stores *definitions* of code and *requests* to execute it. External runtimes *actualize* those requests. The kernel never executes code.

## 1.3 The Analogy

A job board contains:
- Job postings (what work needs doing)
- Requirements (inputs needed)
- Status (open, in progress, completed)

Workers check the board, claim jobs, do the work, report completion. The board doesn't do the work. It coordinates.

MEW works the same way. The kernel coordinates; runtimes compute.

## 1.4 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Kernel never executes** | Graph stores definitions and status, never runs code |
| **Code is bytes** | Code stored externally, referenced by hash |
| **Invocation is the primitive** | All execution flows through invocation nodes |
| **Runtimes are external** | Processes that watch the graph and act |

---

# Part II: Core Concepts

## 2.1 Code Module

A code module is a node representing executable code:

```
node CodeModule {
  name: String [required]
  source_hash: Hash [required]        -- code bytes in external storage
  language: String [required]         -- "wasm", "python", "javascript", etc.
  entry_point: String?
}
```

Code is stored externally, referenced by hash — the same pattern as content storage. The kernel stores fixed-size references only.

Benefits:
- Code is content-addressed (same code = same hash)
- Code is immutable (changing code = new hash = new module)
- Code is deduplicated

The `language` field tells runtimes how to execute. Runtimes advertise which languages they support.

## 2.2 Invocation

An invocation is a node representing a request to execute:

```
node Invocation {
  status: String [required]           -- "pending", "running", "completed", "failed"
  created_at: Timestamp
  started_at: Timestamp?
  completed_at: Timestamp?
  error: String?
}

edge invokes(Invocation, CodeModule)
edge input(Invocation, any)
edge output(Invocation, any)
```

Inputs are linked before execution. Outputs are linked after completion.

## 2.3 Runtime

A runtime is an external process that:

1. **Watches** for pending invocations matching its capabilities
2. **Claims** an invocation by setting status = "running"
3. **Fetches** code bytes from storage
4. **Gathers** inputs by querying input edges
5. **Executes** code
6. **Applies** outputs and mutations to the graph
7. **Reports** completion or failure by updating status

The claim must be atomic to prevent multiple runtimes executing the same invocation.

## 2.4 The Invocation Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                   INVOCATION LIFECYCLE                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   PENDING                                                           │
│   ───────                                                           │
│   Invocation created, waiting for a runtime to claim it.           │
│                                                                      │
│        │                                                             │
│        ▼                                                             │
│                                                                      │
│   RUNNING                                                           │
│   ───────                                                           │
│   Runtime claimed it, execution in progress.                       │
│                                                                      │
│        │                                                             │
│        ├─────────────────────┐                                      │
│        ▼                     ▼                                      │
│                                                                      │
│   COMPLETED                FAILED                                   │
│   ─────────                ──────                                   │
│   Outputs linked.          Error recorded.                         │
│   Mutations applied.       No mutations applied.                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part III: The Kernel Boundary

## 3.1 What the Kernel Does

- Stores code module nodes (with hash references)
- Stores invocation nodes (with status)
- Stores input/output edges
- Indexes invocations (by status, by code module, etc.)
- Enforces policy on mutations

## 3.2 What the Kernel Does Not Do

- Fetch code bytes
- Execute code
- Validate code correctness
- Manage runtime processes
- Sandbox execution
- Handle execution errors

The kernel is a coordinator, not an executor.

---

# Part IV: Execution Models

## 4.1 Pure Model

Code receives serialized inputs, returns structured outputs. Runtime interprets outputs as mutations.

```
-- Code returns:
{
  "outputs": { "result": { "title": "..." } },
  "mutations": [
    { "op": "spawn", "type": "Tag", "attrs": { "value": "important" } },
    { "op": "link", "edge": "has_tag", "from": "$input.doc", "to": "$spawned.0" }
  ]
}
```

Auditable. Sandboxed. Less flexible.

## 4.2 API Model

Code receives a graph client, makes calls directly.

```python
def process(ctx):
    doc = ctx.inputs["document"]
    parsed = parse(doc)
    
    result = ctx.spawn("ParsedDocument", title=parsed.title)
    for tag in parsed.tags:
        tag_node = ctx.spawn("Tag", value=tag)
        ctx.link("has_tag", result, tag_node)
    
    return result
```

Powerful. Harder to audit.

Both models are valid. Runtimes choose based on security and capability needs.

## 4.3 Identity

When code mutates the graph, policy checks apply. The identity used for those checks is a design decision:

- **Invocation creator's identity**: Code can do what the caller could do
- **Explicit grants on code module**: Code has specific declared capabilities
- **Some combination**: Intersection or union of the above

The natural model: calling a function inherits caller's permissions, with ability to restrict or extend via explicit grants.

---

# Part V: Trigger Patterns

Invocations can be created explicitly or automatically.

## 5.1 Procedure (Explicit)

Client creates invocation directly:

```
SPAWN inv: Invocation { status = "pending" }
LINK invokes(inv, #my_code_module)
LINK input(inv, #data_to_process)
```

This is the primitive. All other patterns build on this.

## 5.2 Reactor (Pattern-Triggered)

Automatically invoke when a graph pattern matches:

```
node Reactor {
  trigger_pattern: String [required]  -- serialized MEW pattern
  enabled: Bool = true
}

edge executes(Reactor, CodeModule)
```

A runtime watches for Reactors, creates a WATCH on each trigger_pattern, and spawns Invocations when matches occur.

```
-- Example: auto-parse new documents
Reactor {
  trigger_pattern = "d: Document WHERE d.parsed = false"
}
```

When an unparsed Document appears, the runtime creates an Invocation with that Document as input.

## 5.3 Scheduler (Time-Triggered)

Automatically invoke on a time schedule:

```
node Scheduler {
  cron: String [required]             -- "0 9 * * *" = 9am daily
  enabled: Bool = true
}

edge executes(Scheduler, CodeModule)
```

A runtime watches for Schedulers, maintains timers, and spawns Invocations when schedules fire.

## 5.4 Tracking Triggers

An edge can track which trigger created an invocation:

```
edge triggered_by(Invocation, Reactor | Scheduler)
```

Enables querying invocations by trigger, debugging, metrics.

---

# Part VI: Relationship to Rules

## 6.1 Comparison

| Aspect | MEW Rules | Code Invocation |
|--------|-----------|-----------------|
| **Expression** | Declarative | Imperative |
| **Execution** | Kernel | Runtime |
| **Activation** | Always active | On-demand |
| **Termination** | Guaranteed | Not guaranteed |
| **Effects** | Graph only | Arbitrary |

## 6.2 Rules Can Create Invocations

Rules can trigger computation by spawning invocations:

```
rule invoke_on_approval:
  doc: Document
  WHERE doc.status = "approved"
    AND NOT EXISTS(inv: Invocation, input(inv, doc), invokes(inv, #publish_module))
  =>
  SPAWN inv: Invocation { status = "pending" },
  LINK invokes(inv, #publish_module),
  LINK input(inv, doc)
```

This bridges declarative and imperative: rules detect conditions, code handles complex actions.

## 6.3 Choosing Between Rules and Code

| Use Rules When | Use Code When |
|----------------|---------------|
| Logic is pattern-based | Logic requires iteration |
| Effects are graph-only | Effects include I/O |
| Must always be active | Should run on-demand |
| Termination must be guaranteed | Complex algorithms needed |

---

# Part VII: Examples

## 7.1 Document Processing

```
-- Code module for parsing
CodeModule {
  name = "document_parser"
  source_hash = 0x7f3a...
  language = "python"
}

-- Reactor to auto-trigger
Reactor {
  trigger_pattern = "d: Document WHERE d.status = 'uploaded'"
}
executes(reactor, parser)

-- Flow:
-- 1. Document uploaded
-- 2. Reactor triggers, creates Invocation
-- 3. Runtime executes parser
-- 4. Parser creates structured output, updates Document status
```

## 7.2 Scheduled Sync

```
-- Code module for API sync
CodeModule {
  name = "external_sync"
  source_hash = 0x8b2c...
  language = "javascript"
}

-- Scheduler for hourly runs
Scheduler {
  cron = "0 * * * *"
}
executes(scheduler, syncer)

-- Flow:
-- 1. Every hour, scheduler fires
-- 2. Runtime creates Invocation
-- 3. Syncer calls external APIs, updates graph
```

## 7.3 Explicit Procedure

```
-- Client requests report generation
SPAWN inv: Invocation { status = "pending" }
LINK invokes(inv, #report_generator)
LINK input(inv, #report_request)

-- Flow:
-- 1. Client creates Invocation
-- 2. Runtime executes generator
-- 3. Generator produces output, links to Invocation
```

---

# Part VIII: Summary

## 8.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **Control Plane** | Graph: definitions, requests, status |
| **Compute Plane** | External: execution, processing |
| **Code Module** | Reference to executable code (hash + metadata) |
| **Invocation** | Request to execute with inputs → outputs |
| **Runtime** | External process that executes invocations |
| **Trigger** | Pattern that automatically creates invocations |

## 8.2 Core Invariants

| Invariant | Meaning |
|-----------|---------|
| **Kernel never executes** | Stores definitions and status only |
| **Code is bytes** | Referenced by hash, stored externally |
| **Invocation is the primitive** | All execution flows through invocations |
| **Status is authoritative** | pending → running → completed/failed |

## 8.3 The Pattern

```
┌─────────────────────────────────────────────────────────────────────┐
│                       THE PATTERN                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   MEW rules handle declarative, analyzable, graph-only logic.      │
│   Invocations handle imperative, arbitrary, external computation.  │
│                                                                      │
│   The graph defines:                                                │
│   • What code exists (CodeModule)                                  │
│   • What needs to run (Invocation)                                 │
│   • When to trigger (Reactor, Scheduler)                           │
│   • What happened (status, outputs)                                │
│                                                                      │
│   Runtimes actualize:                                               │
│   • Watch for work                                                 │
│   • Execute code                                                   │
│   • Report results                                                 │
│                                                                      │
│   The kernel coordinates. Runtimes compute.                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of MEW Compute Specification*