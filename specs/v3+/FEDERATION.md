# MEW Federation

**Version:** 1.0 (Minimum Viable)
**Status:** Specification
**Scope:** Data synchronization between MEW kernel instances

---

# Part I: Context & Motivation

## 1.1 The Problem

A single MEW kernel has limits:

| Limit | Consequence |
|-------|-------------|
| Single machine | Bounded by one machine's resources |
| Single location | Latency for distant users |
| Single point of failure | Downtime affects everyone |
| Single governance | One authority controls everything |

Real-world systems need multiple kernels working together.

## 1.2 What Federation Provides

```
┌─────────────────────────────────────────────────────────────────────┐
│                    FEDERATION ENABLES                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  GEO-DISTRIBUTION                                                   │
│  Kernels in different regions, data close to users.                │
│                                                                      │
│  RESILIENCE                                                         │
│  Kernel failure doesn't lose all data.                             │
│                                                                      │
│  SCALE                                                              │
│  Distribute load across multiple kernels.                          │
│                                                                      │
│  AUTONOMY                                                           │
│  Different organizations run their own kernels,                    │
│  selectively share data.                                           │
│                                                                      │
│  INTEROPERABILITY                                                   │
│  Different systems exchange structured data                        │
│  with shared semantics.                                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 1.3 Scope of This Specification

This is **Minimum Viable Federation (MVF)**. It covers:

| In Scope | Out of Scope (Future) |
|----------|----------------------|
| Two kernels | Multi-kernel topologies |
| Same ontology | Type mapping between different ontologies |
| One-way sync | Bidirectional sync |
| Explicit sync declarations | Automatic discovery |
| Last-write-wins conflicts | Advanced conflict resolution |
| Point-to-point | Gossip protocols |
| Remote queries (read-only) | Cross-kernel transactions |

MVF is the foundation. Future versions build on it.

## 1.4 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Explicit over automatic** | Sync must be declared, not implicit |
| **Local-first** | Local operations never blocked by remote |
| **Eventually consistent** | Accept temporary divergence |
| **Simple conflict resolution** | LWW by default, predictable behavior |
| **Minimal protocol** | Small set of operations, easy to implement |

---

# Part II: Core Model

## 2.1 Concepts

```
┌─────────────────────────────────────────────────────────────────────┐
│                    FEDERATION CONCEPTS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  KERNEL                                                             │
│  A MEW instance with its own state and execution.                  │
│  Identified by a globally unique URI.                              │
│                                                                      │
│  REMOTE                                                             │
│  A reference to another kernel for federation.                     │
│  Configured locally with connection details.                       │
│                                                                      │
│  SYNC                                                               │
│  Data flow from one kernel to another.                             │
│  Defined by pattern (what), direction (which way), policy (how).   │
│                                                                      │
│  FEDERATION                                                         │
│  An established relationship between two kernels                   │
│  enabling sync and remote queries.                                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Kernel Identity

Every kernel has a globally unique identity:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    KERNEL IDENTITY                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  URI FORMAT                                                         │
│  ──────────                                                         │
│  mew://<host>:<port>/<kernel_id>                                   │
│                                                                      │
│  Examples:                                                          │
│    mew://us-east.example.com:9000/production                       │
│    mew://localhost:9000/dev                                        │
│    mew://10.0.1.5:9000/warehouse_a                                 │
│                                                                      │
│  KERNEL ID                                                          │
│  ─────────                                                          │
│  • Assigned at kernel creation                                     │
│  • Immutable for kernel lifetime                                   │
│  • Used in entity references across kernels                        │
│                                                                      │
│  LOCAL KERNEL                                                       │
│  ────────────                                                       │
│  The current kernel is referenced as:                              │
│    LOCAL                                                           │
│    or implicitly (no kernel specified)                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.3 Entity Identity Across Kernels

```
┌─────────────────────────────────────────────────────────────────────┐
│                 CROSS-KERNEL IDENTITY                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  LOCAL REFERENCE                                                    │
│  ───────────────                                                    │
│  #task_123                                                         │
│  Meaningful only within one kernel.                                │
│                                                                      │
│  QUALIFIED REFERENCE                                                │
│  ───────────────────                                                │
│  mew://host:port/kernel/task_123                                   │
│  Globally unique, specifies which kernel.                          │
│                                                                      │
│  SYNC ID                                                            │
│  ───────                                                            │
│  Optional attribute for correlation across kernels.                │
│                                                                      │
│  node Task {                                                       │
│    sync_id: String? [unique],  -- set on first sync               │
│    ...                                                             │
│  }                                                                 │
│                                                                      │
│  When entity syncs:                                                │
│    • Source assigns sync_id (if not present)                       │
│    • Destination uses sync_id to match existing entity             │
│    • Same sync_id = same logical entity across kernels             │
│                                                                      │
│  SYNC_ID GENERATION                                                │
│  ─────────────────                                                  │
│  Default: kernel_id + "/" + local_id                               │
│  Example: "us-east-prod/task_123"                                  │
│                                                                      │
│  Custom: application can set sync_id explicitly                    │
│  Example: external system ID, UUID, etc.                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.4 Ontology Requirement

MVF requires **same ontology** on both kernels:

```
┌─────────────────────────────────────────────────────────────────────┐
│                 ONTOLOGY COMPATIBILITY                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  REQUIREMENT: Both kernels must have compatible ontologies.        │
│                                                                      │
│  COMPATIBLE MEANS                                                   │
│  ────────────────                                                   │
│  • Same types being synced exist in both                          │
│  • Same attributes with compatible types                          │
│  • Same constraints (or destination is subset)                    │
│                                                                      │
│  HOW TO ACHIEVE                                                     │
│  ───────────────                                                    │
│  Option A: Shared governance (same blockchain)                     │
│    Both kernels follow same canonical ontology.                    │
│    Blockchain ensures compatibility.                               │
│                                                                      │
│  Option B: Manual coordination                                     │
│    Operators ensure ontologies match.                              │
│    Federation fails if incompatible.                               │
│                                                                      │
│  COMPATIBILITY CHECK                                                │
│  ───────────────────                                                │
│  On federation establishment:                                      │
│    • Exchange schema fingerprints for synced types                │
│    • If mismatch: warning or error (configurable)                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part III: Federation Lifecycle

## 3.1 Establishing Federation

```
┌─────────────────────────────────────────────────────────────────────┐
│                 FEDERATION ESTABLISHMENT                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. DECLARE REMOTE                                                  │
│     Register the remote kernel locally.                            │
│                                                                      │
│     REMOTE partner                                                 │
│       [uri: "mew://partner.example.com:9000/main"]                │
│       [auth: ...]                                                  │
│                                                                      │
│  2. CONNECT                                                         │
│     Establish connection, verify identity.                         │
│                                                                      │
│     CONNECT TO REMOTE partner                                      │
│                                                                      │
│  3. VERIFY COMPATIBILITY                                            │
│     Exchange schema info, check compatibility.                     │
│                                                                      │
│     (automatic on connect)                                         │
│                                                                      │
│  4. DECLARE SYNC                                                    │
│     Define what data flows and how.                                │
│                                                                      │
│     SYNC TO REMOTE partner                                         │
│       MATCH t: Task WHERE t.shared = true                         │
│       [mode: push]                                                 │
│                                                                      │
│  5. ACTIVE                                                          │
│     Federation operational, sync running.                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.2 Federation States

```
┌─────────────────────────────────────────────────────────────────────┐
│                  FEDERATION STATES                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐                                                   │
│  │   DECLARED   │  Remote registered, not connected                │
│  └──────┬───────┘                                                   │
│         │ CONNECT                                                   │
│         ▼                                                            │
│  ┌──────────────┐                                                   │
│  │  CONNECTING  │  Handshake in progress                           │
│  └──────┬───────┘                                                   │
│         │ success                                                   │
│         ▼                                                            │
│  ┌──────────────┐         connection lost                          │
│  │   CONNECTED  │◄────────────────────────────┐                    │
│  │              │                              │                    │
│  │  • Queries   │  reconnect                   │                    │
│  │  • Sync      │                              │                    │
│  └──────┬───────┘                              │                    │
│         │ connection lost              ┌───────┴──────┐             │
│         └─────────────────────────────▶│ DISCONNECTED │             │
│                                        │              │             │
│         ┌──────────────────────────────│  • Buffering │             │
│         │ DISCONNECT (explicit)        │  • Retrying  │             │
│         ▼                              └──────────────┘             │
│  ┌──────────────┐                                                   │
│  │   DISABLED   │  Explicitly disabled, no retry                   │
│  └──────────────┘                                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.3 Terminating Federation

```
-- Disable sync but keep remote definition
DISABLE REMOTE partner

-- Re-enable
ENABLE REMOTE partner

-- Remove federation entirely
DROP REMOTE partner
  [keep_data: true]     -- keep synced data locally
  [cascade: true]       -- also drop sync declarations
```

---

# Part IV: Remote Declaration

## 4.1 Syntax

```
RemoteDecl =
    "REMOTE" Identifier RemoteOptions

RemoteOptions = "[" RemoteOption ("," RemoteOption)* "]"

RemoteOption =
    "uri" ":" StringLiteral
  | "auth" ":" AuthSpec
  | "retry" ":" RetrySpec
  | "timeout" ":" Duration
  | "compatibility" ":" ("strict" | "warn" | "ignore")
```

## 4.2 Authentication Options

```
┌─────────────────────────────────────────────────────────────────────┐
│                  AUTHENTICATION                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  NONE (testing only)                                               │
│  ──────────────────                                                 │
│  [auth: none]                                                      │
│  No authentication. Only for development.                          │
│                                                                      │
│  SHARED SECRET                                                      │
│  ─────────────                                                      │
│  [auth: secret("my-shared-secret")]                                │
│  Both kernels know same secret.                                    │
│  Simple but secret management needed.                              │
│                                                                      │
│  MUTUAL TLS                                                         │
│  ──────────                                                         │
│  [auth: mtls(                                                      │
│    cert: "/path/to/cert.pem",                                     │
│    key: "/path/to/key.pem",                                       │
│    ca: "/path/to/ca.pem"                                          │
│  )]                                                                │
│  Certificate-based mutual authentication.                          │
│  Recommended for production.                                       │
│                                                                      │
│  TOKEN                                                              │
│  ─────                                                              │
│  [auth: token("bearer-token-here")]                                │
│  Bearer token authentication.                                      │
│  Good for cloud environments.                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.3 Connection Options

```
REMOTE partner
  [uri: "mew://partner.example.com:9000/main"]
  [auth: mtls(...)]
  
  -- Connection behavior
  [timeout: 30s]                    -- connection timeout
  [retry: exponential(             
    initial: 1s,                    -- first retry after 1s
    max: 60s,                       -- max retry interval
    multiplier: 2                   -- double each time
  )]
  
  -- Compatibility checking
  [compatibility: strict]           -- fail if schema mismatch
  -- [compatibility: warn]          -- warn but continue
  -- [compatibility: ignore]        -- no check
```

## 4.4 Remote as Entity

Remotes are stored as Layer 0 nodes:

```
node _Remote {
  name: String [required, unique],
  uri: String [required],
  status: String [in: ["declared", "connecting", "connected", 
                       "disconnected", "disabled"]],
  auth_type: String,
  created_at: Timestamp,
  last_connected: Timestamp?,
  last_error: String?,
  
  -- Stats
  sync_lag: Duration?,
  messages_sent: Int = 0,
  messages_received: Int = 0,
  errors: Int = 0
}
```

Queryable:

```
MATCH r: _Remote WHERE r.status = "disconnected"
RETURN r.name, r.last_error, r.last_connected
```

---

# Part V: Sync Declarations

## 5.1 Sync Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SYNC MODEL                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  A SYNC declaration defines:                                        │
│                                                                      │
│  WHAT        Pattern defining entities to sync                     │
│  DIRECTION   Push (local→remote) or Pull (remote→local)            │
│  WHEN        Trigger (immediate, periodic, manual)                 │
│  HOW         Conflict resolution, filtering                        │
│                                                                      │
│  PUSH (outbound)                                                    │
│  ──────────────                                                     │
│  Local changes matching pattern → sent to remote.                  │
│                                                                      │
│    SYNC TO REMOTE partner                                          │
│      MATCH t: Task WHERE t.shared = true                          │
│                                                                      │
│  PULL (inbound)                                                     │
│  ─────────────                                                      │
│  Remote changes matching pattern → received locally.               │
│                                                                      │
│    SYNC FROM REMOTE partner                                        │
│      MATCH t: Task WHERE t.shared = true                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Sync Syntax

```
SyncDecl =
    "SYNC" Direction "REMOTE" Identifier
    MatchPattern
    SyncOptions?

Direction = "TO" | "FROM"

SyncOptions = "[" SyncOption ("," SyncOption)* "]"

SyncOption =
    "mode" ":" SyncMode
  | "trigger" ":" TriggerSpec
  | "conflict" ":" ConflictSpec
  | "include_edges" ":" Bool
  | "buffer" ":" IntLiteral
  | "batch_size" ":" IntLiteral
  | "on_error" ":" ErrorAction

SyncMode = "push" | "pull"

TriggerSpec =
    "immediate"                     -- sync on every change
  | "periodic" "(" Duration ")"     -- sync every N
  | "manual"                        -- only explicit SYNC NOW

ConflictSpec =
    "last_write_wins"
  | "source_wins"
  | "destination_wins"
  | "reject"

ErrorAction = "skip" | "retry" | "stop"
```

## 5.3 Push Sync Examples

```
-- Sync all shared tasks to partner
SYNC TO REMOTE partner
  MATCH t: Task WHERE t.shared = true
  [trigger: immediate]
  [include_edges: true]          -- also sync edges involving matched nodes

-- Sync completed projects once per hour
SYNC TO REMOTE archive
  MATCH p: Project WHERE p.status = "completed"
  [trigger: periodic(1h)]
  [conflict: source_wins]        -- local always wins

-- Manual sync for sensitive data
SYNC TO REMOTE backup
  MATCH s: Secret
  [trigger: manual]
```

## 5.4 Pull Sync Examples

```
-- Pull product catalog from central
SYNC FROM REMOTE central
  MATCH p: Product
  [trigger: periodic(5m)]
  [conflict: source_wins]        -- central is authoritative

-- Pull alerts immediately
SYNC FROM REMOTE monitoring
  MATCH a: Alert WHERE a.severity = "critical"
  [trigger: immediate]
```

## 5.5 Sync with Edges

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EDGE HANDLING                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  [include_edges: false] (default)                                  │
│  ─────────────────────────────────                                  │
│  Only sync nodes matching pattern.                                 │
│  Edges not synced.                                                 │
│  References may dangle.                                            │
│                                                                      │
│  [include_edges: true]                                             │
│  ─────────────────────                                              │
│  Also sync edges where matched node is source or target.           │
│                                                                      │
│  Example:                                                          │
│    SYNC TO REMOTE partner                                          │
│      MATCH t: Task WHERE t.shared = true                          │
│      [include_edges: true]                                        │
│                                                                      │
│  Syncs:                                                            │
│    • Task nodes matching pattern                                   │
│    • assigned_to edges from those tasks                           │
│    • belongs_to edges from those tasks                            │
│    • etc.                                                          │
│                                                                      │
│  Does NOT sync:                                                    │
│    • Target nodes of those edges (unless also matched)            │
│                                                                      │
│  [include_edges: transitive(depth)]                                │
│  ───────────────────────────────────                                │
│  Follow edges to specified depth.                                  │
│                                                                      │
│    SYNC TO REMOTE partner                                          │
│      MATCH t: Task                                                 │
│      [include_edges: transitive(2)]                               │
│                                                                      │
│  Syncs tasks + connected entities up to 2 hops.                   │
│  Use with caution (can sync large subgraphs).                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.6 Reference Handling

What happens when a synced entity references an entity that wasn't synced?

```
┌─────────────────────────────────────────────────────────────────────┐
│                  REFERENCE HANDLING                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SCENARIO                                                           │
│  ────────                                                           │
│  Task { assignee = #alice }                                        │
│  Task is synced, Person #alice is not.                            │
│                                                                      │
│  OPTIONS                                                            │
│  ───────                                                            │
│                                                                      │
│  [refs: qualified]  (default)                                      │
│    assignee = mew://source-kernel/alice                           │
│    Stored as qualified reference.                                  │
│    Destination knows it's external.                                │
│    Can query remote to resolve.                                    │
│                                                                      │
│  [refs: null]                                                      │
│    assignee = null                                                 │
│    Reference cleared.                                              │
│    Data loss but simple.                                           │
│                                                                      │
│  [refs: sync]                                                      │
│    Also sync #alice.                                               │
│    Transitive closure (careful!).                                  │
│                                                                      │
│  [refs: require]                                                   │
│    Fail sync if reference can't be resolved.                       │
│    Strict but may block sync.                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VI: Conflict Resolution

## 6.1 When Conflicts Occur

Conflicts only occur on **pull** (inbound sync):

```
Local state:   Task { id = "X", title = "Local title", updated_at = T1 }
Remote state:  Task { id = "X", title = "Remote title", updated_at = T2 }

Remote entity arrives via sync.
Local entity with same sync_id exists.
Different attribute values.
→ Conflict.
```

## 6.2 Resolution Strategies

```
┌─────────────────────────────────────────────────────────────────────┐
│                CONFLICT STRATEGIES                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  LAST_WRITE_WINS (default)                                         │
│  ─────────────────────────                                          │
│  Entity with later updated_at wins.                                │
│  Requires synchronized clocks (approximately).                     │
│  Simple, predictable, may lose data.                               │
│                                                                      │
│  [conflict: last_write_wins]                                       │
│                                                                      │
│                                                                      │
│  SOURCE_WINS                                                        │
│  ───────────                                                        │
│  Incoming (remote) data always wins.                               │
│  Local changes overwritten.                                        │
│  Good when remote is authoritative.                                │
│                                                                      │
│  [conflict: source_wins]                                           │
│                                                                      │
│                                                                      │
│  DESTINATION_WINS                                                   │
│  ────────────────                                                   │
│  Local data always wins.                                           │
│  Incoming changes ignored if conflict.                             │
│  Good when local is authoritative.                                 │
│                                                                      │
│  [conflict: destination_wins]                                      │
│                                                                      │
│                                                                      │
│  REJECT                                                             │
│  ──────                                                             │
│  Conflict is an error.                                             │
│  Incoming entity rejected.                                         │
│  Logged for manual resolution.                                     │
│                                                                      │
│  [conflict: reject]                                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 6.3 Conflict Logging

All conflicts are logged regardless of resolution:

```
node _SyncConflict {
  sync_id: String,
  remote: String,
  entity_type: String,
  local_version: String,      -- serialized local state
  remote_version: String,     -- serialized remote state
  resolution: String,         -- "local_kept", "remote_applied", "rejected"
  resolved_at: Timestamp,
  resolved_by: String         -- "auto:lww", "auto:source_wins", "manual"
}

-- Query recent conflicts
MATCH c: _SyncConflict 
WHERE c.resolved_at > now() - 24h
RETURN c
```

## 6.4 Conflict Hooks (Future)

```
-- Not in MVF, but future extension:

SYNC FROM REMOTE partner
  MATCH t: Task
  [conflict: custom(resolve_task_conflict)]

rule resolve_task_conflict [conflict_handler]:
  local: Task, remote: Task
  WHERE local.sync_id = remote.sync_id
  =>
  -- Custom merge logic
  SET local.title = COALESCE(remote.title, local.title)
  SET local.priority = MAX(remote.priority, local.priority)
  SET local.updated_at = MAX(remote.updated_at, local.updated_at)
```

---

# Part VII: Remote Queries

## 7.1 Query Syntax

Read data from remote kernel without syncing:

```
-- Query remote
MATCH t: Task FROM REMOTE partner
WHERE t.status = "open"
RETURN t.title, t.priority

-- Query with local join (fetch-then-join)
MATCH t: Task FROM REMOTE partner,
      p: Project FROM LOCAL
WHERE t.project_id = p.id
RETURN t.title, p.name
```

## 7.2 Query Semantics

```
┌─────────────────────────────────────────────────────────────────────┐
│                  REMOTE QUERY SEMANTICS                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  EXECUTION                                                          │
│  ─────────                                                          │
│  1. Pattern analyzed                                               │
│  2. Remote portion sent to remote kernel                           │
│  3. Remote executes query, returns results                         │
│  4. Local portion executed locally                                 │
│  5. Results joined if needed                                       │
│  6. Final results returned                                         │
│                                                                      │
│  CONSISTENCY                                                        │
│  ───────────                                                        │
│  Remote query sees remote's committed state.                       │
│  No transaction spanning local and remote.                         │
│  Results may be stale by time they arrive.                         │
│                                                                      │
│  CACHING                                                            │
│  ───────                                                            │
│  Results not cached by default.                                    │
│  Each query hits remote.                                           │
│                                                                      │
│  [cache: ttl(5m)]  -- optional caching                            │
│                                                                      │
│  AUTHORIZATION                                                      │
│  ─────────────                                                      │
│  Remote enforces its authorization.                                │
│  Query may return partial results if not fully authorized.        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.3 Remote Query Limitations

```
┌─────────────────────────────────────────────────────────────────────┐
│                 REMOTE QUERY LIMITATIONS                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ALLOWED                                                            │
│  ───────                                                            │
│  • MATCH (read)                                                    │
│  • WHERE with serializable predicates                              │
│  • RETURN with projections                                         │
│  • ORDER BY, LIMIT                                                 │
│                                                                      │
│  NOT ALLOWED (in MVF)                                               │
│  ─────────────────────                                              │
│  • SPAWN, SET, KILL (mutations)                                    │
│  • Aggregations (in v1)                                            │
│  • Pattern variables bound across local and remote                 │
│  • Transactions spanning remotes                                   │
│                                                                      │
│  ERROR HANDLING                                                     │
│  ──────────────                                                     │
│  • Remote unreachable → error (no partial results)                │
│  • Remote timeout → error                                          │
│  • Authorization failure → error or partial (configurable)        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VIII: Sync Operations

## 8.1 Manual Sync

```
-- Trigger sync for a specific declaration
SYNC NOW TO REMOTE partner
  MATCH t: Task

-- Trigger all syncs to a remote
SYNC NOW ALL TO REMOTE partner

-- Trigger all syncs (all remotes)
SYNC NOW ALL
```

## 8.2 Initial Sync

When sync declaration is created, perform initial sync:

```
SYNC TO REMOTE partner
  MATCH t: Task WHERE t.shared = true
  [initial: full]       -- sync all existing matches (default)
  [initial: none]       -- only sync future changes
  [initial: since(T)]   -- sync changes since timestamp T
```

## 8.3 Sync Status

```
-- Check sync status
SHOW SYNC STATUS

┌────────────────────────────────────────────────────────────────┐
│ Remote   │ Direction │ Pattern      │ Status  │ Lag    │ Queue │
├──────────┼───────────┼──────────────┼─────────┼────────┼───────┤
│ partner  │ push      │ Task(shared) │ active  │ 0s     │ 0     │
│ archive  │ push      │ Project(*)   │ active  │ 45s    │ 12    │
│ central  │ pull      │ Product(*)   │ syncing │ 5m     │ 0     │
└────────────────────────────────────────────────────────────────┘

-- Detailed sync info
DESCRIBE SYNC TO REMOTE partner
  MATCH t: Task

{
  remote: "partner",
  direction: "push",
  pattern: "Task WHERE shared = true",
  status: "active",
  last_sync: "2024-01-15T10:30:00Z",
  entities_synced: 1523,
  entities_pending: 0,
  errors: 0,
  last_error: null,
  config: { trigger: "immediate", conflict: "last_write_wins" }
}
```

## 8.4 Pause and Resume

```
-- Pause a sync (buffering continues)
PAUSE SYNC TO REMOTE partner
  MATCH t: Task

-- Resume
RESUME SYNC TO REMOTE partner
  MATCH t: Task

-- Pause all syncs to a remote
PAUSE REMOTE partner

-- Resume all
RESUME REMOTE partner
```

---

# Part IX: Wire Protocol

## 9.1 Message Types

```
┌─────────────────────────────────────────────────────────────────────┐
│                     WIRE PROTOCOL                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  CONNECTION                                                         │
│  ──────────                                                         │
│  HANDSHAKE        Establish connection, exchange capabilities      │
│  HANDSHAKE_ACK    Accept connection                                │
│  PING             Keep-alive                                       │
│  PONG             Keep-alive response                              │
│  DISCONNECT       Graceful close                                   │
│                                                                      │
│  SCHEMA                                                             │
│  ──────                                                             │
│  SCHEMA_REQUEST   Request schema for types                         │
│  SCHEMA_RESPONSE  Return schema information                        │
│                                                                      │
│  QUERY                                                              │
│  ─────                                                              │
│  QUERY_REQUEST    Execute query                                    │
│  QUERY_RESPONSE   Return results                                   │
│  QUERY_ERROR      Query failed                                     │
│                                                                      │
│  SYNC                                                               │
│  ────                                                               │
│  SYNC_PUSH        Send entities to remote                          │
│  SYNC_ACK         Acknowledge receipt                              │
│  SYNC_NACK        Reject (error)                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 9.2 Message Format

```
┌─────────────────────────────────────────────────────────────────────┐
│                    MESSAGE STRUCTURE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  HEADER                                                             │
│  ──────                                                             │
│  {                                                                  │
│    "version": "1.0",                                               │
│    "type": "SYNC_PUSH",                                            │
│    "id": "msg_abc123",           // for correlation                │
│    "timestamp": "2024-01-15T10:30:00.000Z",                       │
│    "source": "mew://kernel-a/main",                                │
│    "destination": "mew://kernel-b/main"                            │
│  }                                                                  │
│                                                                      │
│  BODY (varies by type)                                             │
│  ────                                                               │
│                                                                      │
│  SYNC_PUSH:                                                        │
│  {                                                                  │
│    "entities": [                                                   │
│      {                                                             │
│        "type": "Task",                                             │
│        "sync_id": "kernel-a/task_123",                            │
│        "data": { "title": "...", "status": "..." },               │
│        "updated_at": "2024-01-15T10:29:00.000Z",                  │
│        "deleted": false                                            │
│      },                                                            │
│      ...                                                           │
│    ],                                                              │
│    "edges": [                                                      │
│      {                                                             │
│        "type": "assigned_to",                                      │
│        "sync_id": "kernel-a/edge_456",                            │
│        "source": "kernel-a/task_123",                             │
│        "target": "kernel-a/person_789",                           │
│        "data": { ... }                                            │
│      },                                                            │
│      ...                                                           │
│    ],                                                              │
│    "sequence": 1234                    // for ordering            │
│  }                                                                  │
│                                                                      │
│  QUERY_REQUEST:                                                    │
│  {                                                                  │
│    "query": "MATCH t: Task WHERE t.status = 'open' RETURN t",    │
│    "params": { ... }                                               │
│  }                                                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 9.3 Transport

```
┌─────────────────────────────────────────────────────────────────────┐
│                      TRANSPORT                                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  PRIMARY: TCP + TLS                                                │
│  ──────────────────                                                 │
│  • Long-lived connections                                          │
│  • Multiplexed streams                                             │
│  • TLS 1.3 required                                                │
│                                                                      │
│  ENCODING: JSON (MVF) / MessagePack (future)                       │
│  ───────────────────────────────────────────                        │
│  • Human-readable for debugging                                    │
│  • MessagePack for efficiency later                                │
│                                                                      │
│  COMPRESSION: Optional gzip                                        │
│  ──────────────────────────                                         │
│  • For large sync batches                                          │
│  • Negotiated in handshake                                         │
│                                                                      │
│  PORT: 9001 (default for federation)                               │
│  ───────────────────────────────────                                │
│  • Separate from client API port                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part X: Failure Handling

## 10.1 Connection Failures

```
┌─────────────────────────────────────────────────────────────────────┐
│                  CONNECTION FAILURES                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  INITIAL CONNECTION FAILURE                                        │
│  ──────────────────────────                                         │
│  Remote unreachable on first connect.                              │
│                                                                      │
│  Behavior:                                                          │
│    • Remote stays in DECLARED state                                │
│    • Retry according to retry policy                               │
│    • Sync declarations pending until connected                     │
│                                                                      │
│  DISCONNECTION                                                      │
│  ────────────                                                       │
│  Connection lost during operation.                                 │
│                                                                      │
│  Behavior:                                                          │
│    • Remote transitions to DISCONNECTED                            │
│    • Outgoing sync buffers locally (up to limit)                  │
│    • Retry connection per policy                                   │
│    • On reconnect: drain buffer, resume normal sync               │
│                                                                      │
│  EXTENDED OUTAGE                                                    │
│  ────────────────                                                   │
│  Remote unreachable for extended period.                          │
│                                                                      │
│  Behavior:                                                          │
│    • Buffer fills to limit                                         │
│    • Oldest entries dropped OR sync paused (configurable)         │
│    • Alert generated                                               │
│    • On reconnect: reconciliation may be needed                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.2 Sync Failures

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SYNC FAILURES                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ENTITY REJECTED                                                    │
│  ───────────────                                                    │
│  Remote rejects entity (schema violation, authorization).          │
│                                                                      │
│  [on_error: skip]     -- log and continue                          │
│  [on_error: retry]    -- retry N times then skip                   │
│  [on_error: stop]     -- pause sync, alert                         │
│                                                                      │
│  BATCH FAILURE                                                      │
│  ─────────────                                                      │
│  Entire batch rejected.                                            │
│                                                                      │
│  Behavior:                                                          │
│    • Retry batch                                                   │
│    • If persistent: split batch, retry individually               │
│    • Log failures                                                  │
│                                                                      │
│  CONSTRAINT VIOLATION                                               │
│  ────────────────────                                               │
│  Incoming entity violates local constraints.                       │
│                                                                      │
│  [constraint_error: reject]    -- reject entity                    │
│  [constraint_error: quarantine] -- store in _Quarantine           │
│  [constraint_error: force]     -- apply anyway (dangerous)        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.3 Recovery

```
-- Force re-sync from scratch
SYNC RESET TO REMOTE partner
  MATCH t: Task
  [mode: full]          -- re-sync all, not just deltas

-- Reconcile (compare and fix differences)
SYNC RECONCILE WITH REMOTE partner
  MATCH t: Task
  
-- Returns:
{
  "matching": 1523,
  "local_only": 12,
  "remote_only": 5,
  "conflicting": 3
}

-- Apply reconciliation
SYNC RECONCILE WITH REMOTE partner
  MATCH t: Task
  [apply: true]
  [conflict: source_wins]
```

---

# Part XI: Monitoring & Observability

## 11.1 Metrics

```
┌─────────────────────────────────────────────────────────────────────┐
│                     FEDERATION METRICS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  CONNECTION                                                         │
│  ──────────                                                         │
│  federation_connections_active                                     │
│  federation_connection_errors_total                                │
│  federation_reconnections_total                                    │
│                                                                      │
│  SYNC                                                               │
│  ────                                                               │
│  federation_sync_entities_total{remote,direction}                  │
│  federation_sync_bytes_total{remote,direction}                     │
│  federation_sync_lag_seconds{remote,direction}                     │
│  federation_sync_queue_size{remote,direction}                      │
│  federation_sync_errors_total{remote,direction,type}               │
│                                                                      │
│  CONFLICTS                                                          │
│  ─────────                                                          │
│  federation_conflicts_total{remote,resolution}                     │
│  federation_conflicts_pending                                      │
│                                                                      │
│  QUERIES                                                            │
│  ───────                                                            │
│  federation_query_duration_seconds{remote}                         │
│  federation_query_total{remote,status}                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 11.2 Events

```
-- Queryable event log
node _FederationEvent {
  type: String,          -- "connected", "disconnected", "sync_complete", 
                         -- "sync_error", "conflict"
  remote: String,
  timestamp: Timestamp,
  details: String,       -- JSON
  severity: String       -- "info", "warning", "error"
}

-- Recent federation events
MATCH e: _FederationEvent
WHERE e.timestamp > now() - 1h
ORDER BY e.timestamp DESC
RETURN e
```

## 11.3 Health Checks

```
-- Check federation health
SHOW FEDERATION HEALTH

┌────────────────────────────────────────────────────────────────┐
│ Remote   │ Status    │ Latency │ Sync Lag │ Queue │ Health    │
├──────────┼───────────┼─────────┼──────────┼───────┼───────────┤
│ partner  │ connected │ 15ms    │ 0s       │ 0     │ healthy   │
│ archive  │ connected │ 45ms    │ 2m       │ 150   │ degraded  │
│ central  │ disconn.  │ -       │ 15m      │ 500   │ unhealthy │
└────────────────────────────────────────────────────────────────┘
```

---

# Part XII: Authorization

## 12.1 Federation Authorization

```
┌─────────────────────────────────────────────────────────────────────┐
│               FEDERATION AUTHORIZATION                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WHO CAN FEDERATE                                                   │
│  ────────────────                                                   │
│  Creating and managing federation requires authorization.          │
│                                                                      │
│  authorization manage_federation:                                  │
│    ON REMOTE(_) | SYNC(_)                                          │
│    ALLOW IF has_capability(current_actor(), "federation_admin")    │
│                                                                      │
│  PER-REMOTE AUTHORIZATION                                          │
│  ────────────────────────                                           │
│  What can each remote do?                                          │
│                                                                      │
│  REMOTE partner                                                    │
│    [uri: "..."]                                                    │
│    [permissions: {                                                 │
│      can_query: [Task, Project],       -- types they can read     │
│      can_receive: [Task],              -- types we push to them   │
│      can_send: [Product]               -- types they push to us   │
│    }]                                                              │
│                                                                      │
│  Incoming sync is filtered:                                        │
│    • Entity type must be in can_send                              │
│    • Standard authorization also applies                           │
│                                                                      │
│  Outgoing sync is filtered:                                        │
│    • Only entities in can_receive are sent                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 12.2 Data Filtering

```
-- Only sync entities the actor can see
SYNC TO REMOTE partner
  MATCH t: Task WHERE t.shared = true
  [filter_by: authorization]    -- apply authorization rules

-- This means:
-- For each entity matching pattern:
--   If current sync actor can read it: include in sync
--   Otherwise: exclude
```

---

# Part XIII: Configuration

## 13.1 Global Federation Settings

```
-- Federation enabled/disabled
SET federation.enabled = true

-- Default retry policy
SET federation.retry.initial = 1s
SET federation.retry.max = 60s
SET federation.retry.multiplier = 2

-- Default buffer limits
SET federation.buffer.max_size = 10000
SET federation.buffer.max_bytes = 100MB
SET federation.buffer.on_full = "drop_oldest"  -- or "pause"

-- Sync batching
SET federation.sync.batch_size = 100
SET federation.sync.batch_timeout = 1s
```

## 13.2 Per-Remote Overrides

```
REMOTE partner
  [uri: "..."]
  [buffer: 50000]              -- override default
  [batch_size: 500]
```

---

# Part XIV: Grammar Summary

```
-- Remote declaration
RemoteDecl =
    "REMOTE" Identifier RemoteOptions

-- Sync declaration
SyncDecl =
    "SYNC" ("TO" | "FROM") "REMOTE" Identifier
    MatchPattern
    SyncOptions?

-- Remote query
RemoteQuery =
    "MATCH" Pattern "FROM" "REMOTE" Identifier
    WhereClause?
    ReturnClause

-- Operations
ConnectStmt = "CONNECT" "TO" "REMOTE" Identifier
DisconnectStmt = "DISCONNECT" "FROM" "REMOTE" Identifier
EnableStmt = "ENABLE" "REMOTE" Identifier
DisableStmt = "DISABLE" "REMOTE" Identifier
DropRemoteStmt = "DROP" "REMOTE" Identifier DropOptions?

SyncNowStmt = "SYNC" "NOW" SyncTarget?
SyncTarget = ("TO" | "FROM") "REMOTE" Identifier MatchPattern?
           | "ALL" ("TO" "REMOTE" Identifier)?

PauseSyncStmt = "PAUSE" ("SYNC" SyncTarget | "REMOTE" Identifier)
ResumeSyncStmt = "RESUME" ("SYNC" SyncTarget | "REMOTE" Identifier)

SyncResetStmt = "SYNC" "RESET" SyncTarget ResetOptions?
SyncReconcileStmt = "SYNC" "RECONCILE" "WITH" "REMOTE" Identifier
                    MatchPattern ReconcileOptions?

-- Information
ShowSyncStmt = "SHOW" "SYNC" "STATUS"
DescribeSyncStmt = "DESCRIBE" "SYNC" SyncTarget
ShowHealthStmt = "SHOW" "FEDERATION" "HEALTH"
```

---

# Part XV: Examples

## 15.1 Basic Push Sync

```
-- Setup: US kernel pushes shared tasks to EU kernel

-- Step 1: Declare remote
REMOTE eu_region
  [uri: "mew://eu.example.com:9001/main"]
  [auth: mtls(cert: "/etc/mew/cert.pem", key: "/etc/mew/key.pem")]

-- Step 2: Connect
CONNECT TO REMOTE eu_region

-- Step 3: Declare sync
SYNC TO REMOTE eu_region
  MATCH t: Task WHERE t.region = "global" OR t.region = "eu"
  [trigger: immediate]
  [include_edges: true]

-- Now: any matching Task created/updated in US syncs to EU
```

## 15.2 Basic Pull Sync

```
-- Setup: Branch office pulls product catalog from HQ

REMOTE headquarters
  [uri: "mew://hq.example.com:9001/main"]
  [auth: token("branch-office-token")]

CONNECT TO REMOTE headquarters

SYNC FROM REMOTE headquarters
  MATCH p: Product WHERE p.active = true
  [trigger: periodic(15m)]
  [conflict: source_wins]

-- Now: product catalog syncs every 15 minutes
```

## 15.3 Remote Query

```
-- Query partner's inventory without syncing

REMOTE supplier
  [uri: "mew://supplier.example.com:9001/catalog"]
  [auth: mtls(...)]

CONNECT TO REMOTE supplier

-- Ad-hoc query
MATCH i: Inventory FROM REMOTE supplier
WHERE i.product_id = "SKU-123"
RETURN i.quantity, i.location, i.updated_at

-- Join with local data
MATCH o: Order FROM LOCAL,
      i: Inventory FROM REMOTE supplier
WHERE o.product_id = i.product_id
  AND o.quantity > i.quantity
RETURN o.id, o.quantity AS ordered, i.quantity AS available
```

## 15.4 Disaster Recovery Replica

```
-- Primary pushes everything to backup

REMOTE backup
  [uri: "mew://dr.example.com:9001/main"]
  [auth: mtls(...)]
  [buffer: 100000]              -- large buffer for outages

CONNECT TO REMOTE backup

-- Sync all entities
SYNC TO REMOTE backup
  MATCH _: any
  [trigger: immediate]
  [include_edges: true]

-- Monitor sync lag
SUBSCRIBE MATCH r: _Remote WHERE r.name = "backup"
  [mode: watch]
  ON MATCH DO
    IF r.sync_lag > 5m THEN
      SPAWN Alert { message = "DR sync lag: " + r.sync_lag }
```

---

# Part XVI: Future Extensions

## 16.1 Not in MVF

| Feature | Why Deferred |
|---------|--------------|
| Bidirectional sync | Complex conflict handling |
| Type mapping | Requires schema negotiation |
| Multi-hop federation | Routing, loop detection |
| Cross-kernel transactions | Distributed transactions are hard |
| Gossip protocols | Complexity |
| CRDTs | Limited applicability |
| Semantic conflict resolution | Application-specific |

## 16.2 Extension Path

```
v1.0 (MVF)
├── Two kernels
├── Same ontology
├── One-way sync
├── LWW conflicts
└── Remote queries

v1.1
├── Bidirectional sync
├── Advanced conflict strategies
└── Sync filters (server-side)

v2.0
├── Type mapping
├── Schema negotiation
├── Multi-kernel topologies
└── Hierarchical sync

v3.0
├── Cross-kernel transactions (sagas)
├── Global queries
└── Federation governance
```

---

# Part XVII: Summary

## 17.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **Remote** | Reference to another MEW kernel |
| **Sync** | Declaration of data flow between kernels |
| **Push** | Send local changes to remote |
| **Pull** | Receive remote changes locally |
| **sync_id** | Correlation identifier across kernels |
| **Conflict** | Same entity modified in both kernels |

## 17.2 Operations

| Operation | Purpose |
|-----------|---------|
| `REMOTE` | Declare a remote kernel |
| `CONNECT` | Establish connection |
| `SYNC TO/FROM` | Declare sync relationship |
| `SYNC NOW` | Trigger manual sync |
| `MATCH FROM REMOTE` | Query remote data |

## 17.3 Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Consistency | Eventual | CAP: availability over consistency |
| Conflicts | LWW default | Simple, predictable |
| Direction | One-way (MVF) | Avoid complex conflicts |
| Identity | sync_id correlation | Flexible, explicit |
| Ontology | Same required | Avoid mapping complexity |

## 17.4 What Federation Provides

```
┌─────────────────────────────────────────────────────────────────────┐
│                 FEDERATION SUMMARY                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  MVF enables:                                                       │
│                                                                      │
│  • Data replication between MEW kernels                            │
│  • Geo-distribution (kernels in different regions)                 │
│  • Read replicas (pull sync from primary)                          │
│  • Data export (push to archive/warehouse)                         │
│  • Cross-system queries (without full sync)                        │
│                                                                      │
│  MVF is built on:                                                  │
│                                                                      │
│  • Explicit declarations (REMOTE, SYNC)                            │
│  • Pattern-based selection (what to sync)                          │
│  • Eventually consistent model                                     │
│  • Simple conflict resolution (LWW)                                │
│  • Standard wire protocol                                          │
│                                                                      │
│  MVF defers:                                                       │
│                                                                      │
│  • Bidirectional sync                                              │
│  • Schema evolution across kernels                                 │
│  • Multi-kernel topologies                                         │
│  • Distributed transactions                                        │
│                                                                      │
│  These are genuine complexity that should be added incrementally.  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of MEW Federation Specification (MVF)*