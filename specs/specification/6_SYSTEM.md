# MEW Language Specification

## Part VI: System Operations & Reference

**Version:** 1.0
**Status:** Draft
**Scope:** Administration, versioning, and complete reference

---

# 1. Administration

## 1.1 Overview

Administration statements manage schema, indexes, and engine state.

```
AdminStmt =
    LoadStmt
  | ExtendStmt
  | ShowStmt
  | IndexStmt
  | DropIndexStmt
```

## 1.2 LOAD ONTOLOGY

Load an ontology from file or text.

```
LoadStmt = 
  "load" "ontology" ("from" StringLiteral | "{" OntologySource "}")

-- From file
LOAD ONTOLOGY FROM "path/to/ontology.hog"

-- Inline
LOAD ONTOLOGY {
  node Event {
    timestamp: Timestamp
  }
  edge causes(from: Event, to: Event)
}
```

### 29.2.1 Load Behavior

| Scenario | Behavior |
|----------|----------|
| First load | Install ontology |
| Same ontology | No-op (idempotent) |
| Different with same name | Error (use EXTEND) |
| Inheritance from unloaded | Error |

### 29.2.2 Result

```typescript
interface LoadResult {
  success: boolean
  ontologyName: string
  typesLoaded: number
  constraintsLoaded: number
  rulesLoaded: number
  errors?: string[]
}
```

## 1.3 EXTEND ONTOLOGY

Add to existing ontology without replacing.

```
ExtendStmt =
  "extend" "ontology" Identifier? "{" Declaration* "}"

-- Extend current/default ontology
EXTEND ONTOLOGY {
  node NewType {
    field: String
  }
}

-- Extend named ontology
EXTEND ONTOLOGY TaskManagement {
  node Priority {
    level: Int,
    name: String
  }
}
```

### 29.3.1 Extension Rules

- Can add new types, edges, constraints, rules
- Cannot modify existing definitions
- Cannot remove existing definitions
- New types can inherit from existing types

```
EXTEND ONTOLOGY {
  -- Add new type inheriting from existing
  node SpecialTask : Task {
    special_field: String
  }
  
  -- Add new edge using existing types
  edge reviewed_by(task: Task, person: Person)
}
```

## 1.4 SHOW Statements

Inspect schema and engine state.

```
ShowStmt =
    "show" "types"
  | "show" "edges"
  | "show" "constraints"
  | "show" "rules"
  | "show" "indexes"
  | "show" "type" Identifier
  | "show" "edge" Identifier
  | "show" "constraint" Identifier
  | "show" "rule" Identifier
  | "show" "statistics"
  | "show" "status"
```

### 29.4.1 SHOW TYPES

```
SHOW TYPES

-- Result:
| Name    | Attributes | Parent | Modifiers |
|---------|------------|--------|-----------|
| Task    | 8          | null   |           |
| Person  | 5          | null   |           |
| Project | 4          | null   |           |
```

### 29.4.2 SHOW TYPE \<name\>

```
SHOW TYPE Task

-- Result:
Type: Task
Parents: []
Attributes:
  title: String [required, length: 1..500]
  description: String?
  status: String [in: ["todo", "in_progress", "done"]] = "todo"
  priority: Int [0..10] = 5
  created_at: Timestamp [required, indexed: desc]
  completed_at: Timestamp?
```

### 29.4.3 SHOW EDGES

```
SHOW EDGES

-- Result:
| Name        | Signature                  | Modifiers          |
|-------------|----------------------------|--------------------|
| causes      | (Event, Event)             |                    |
| assigned_to | (Task, Person)             | task -> 0..1       |
| belongs_to  | (Task, Project)            | task -> 1, cascade |
```

### 29.4.4 SHOW CONSTRAINTS

```
SHOW CONSTRAINTS

-- Result:
| Name                    | Type | Pattern              | Message                    |
|-------------------------|------|----------------------|----------------------------|
| task_title_required     | hard | t: Task              | Title required             |
| temporal_order          | hard | causes(e1, e2)       | Cause must precede effect  |
| prefer_description      | soft | t: Task              | Tasks should have desc     |
```

### 29.4.5 SHOW RULES

```
SHOW RULES

-- Result:
| Name                   | Priority | Auto | Pattern        |
|------------------------|----------|------|----------------|
| auto_complete_timestamp| 10       | yes  | t: Task        |
| propagate_completion   | 5        | yes  | subtask_of(...)| 
```

### 29.4.6 SHOW INDEXES

```
SHOW INDEXES

-- Result:
| Name               | Type      | Target           | Order |
|--------------------|-----------|------------------|-------|
| task_created_at    | attribute | Task.created_at  | desc  |
| person_email       | attribute | Person.email     | asc   |
| assigned_to_idx    | edge      | assigned_to      | -     |
```

### 29.4.7 SHOW STATISTICS

```
SHOW STATISTICS

-- Result:
Nodes:
  Total: 15,432
  By type:
    Task: 8,234
    Person: 1,543
    Project: 234
    ...

Edges:
  Total: 45,678
  By type:
    assigned_to: 7,234
    belongs_to: 8,234
    causes: 12,456
    ...

Storage:
  Size: 234 MB
  Index size: 45 MB
```

### 29.4.8 SHOW STATUS

```
SHOW STATUS

-- Result:
Engine: running
Uptime: 4 days, 3 hours
Ontologies loaded: 3
Active transactions: 2
Pending rules: 0
Last snapshot: 2024-01-15T10:30:00Z
```

## 1.5 INDEX Management

### 29.5.1 CREATE INDEX

```
IndexStmt =
  "create" "index" Identifier? "on" IndexTarget

IndexTarget =
    TypeExpr "(" Identifier ")"           -- attribute index
  | EdgeType                               -- edge index

-- Attribute index
CREATE INDEX task_priority ON Task(priority)

-- With order
CREATE INDEX task_created ON Task(created_at DESC)

-- Edge index
CREATE INDEX assigned_idx ON assigned_to

-- Auto-named
CREATE INDEX ON Person(email)
-- Creates index named "person_email_idx"
```

### 29.5.2 DROP INDEX

```
DropIndexStmt = "drop" "index" Identifier

DROP INDEX task_priority
```

### 29.5.3 Index vs [indexed] Modifier

```
-- In ontology (declarative):
node Task {
  created_at: Timestamp [indexed: desc]
}

-- At runtime (imperative):
CREATE INDEX task_created ON Task(created_at DESC)
```

Both achieve the same result. Ontology `[indexed]` is preferred for permanent indexes.

## 1.6 Result Format

```typescript
interface AdminResult {
  success: boolean
  action: string
  data?: any
  errors?: string[]
}
```

## 1.7 AST

```typescript
interface LoadStmt {
  kind: "Load"
  source: { kind: "file", path: string } | { kind: "inline", text: string }
}

interface ExtendStmt {
  kind: "Extend"
  ontology: string | null
  declarations: Declaration[]
}

interface ShowStmt {
  kind: "Show"
  target: "types" | "edges" | "constraints" | "rules" | "indexes" | "statistics" | "status"
  name?: string
}

interface CreateIndexStmt {
  kind: "CreateIndex"
  name: string | null
  target: { kind: "attribute", type: string, attribute: string, order: "asc" | "desc" }
        | { kind: "edge", edgeType: string }
}

interface DropIndexStmt {
  kind: "DropIndex"
  name: string
}
```

---

# 2. Versioning

## 2.1 Overview

Versioning provides time-travel and branching capabilities.

```
VersionStmt =
    SnapshotStmt
  | CheckoutStmt
  | ShowVersionsStmt
  | DiffStmt
  | BranchStmt
  | MergeStmt
```

## 2.2 SNAPSHOT

Create a named checkpoint of current state.

```
SnapshotStmt = "snapshot" StringLiteral?

-- Named snapshot
SNAPSHOT "before-migration"

-- Auto-named (timestamp-based)
SNAPSHOT
-- Creates: "snapshot_2024-01-15T10:30:00Z"
```

### 30.2.1 Snapshot Contents

A snapshot captures:
- All nodes and their attributes
- All edges and their attributes
- Current ontology state
- Index definitions

### 30.2.2 Result

```typescript
interface SnapshotResult {
  success: boolean
  snapshotId: string
  label: string
  timestamp: Timestamp
  nodeCount: number
  edgeCount: number
}
```

## 2.3 SHOW VERSIONS

List available snapshots and branches.

```
ShowVersionsStmt = "show" "versions"

SHOW VERSIONS

-- Result:
| ID          | Label             | Timestamp           | Branch | Nodes  | Edges  |
|-------------|-------------------|---------------------|--------|--------|--------|
| v_abc123    | before-migration  | 2024-01-15T10:30:00 | main   | 15,432 | 45,678 |
| v_def456    | after-migration   | 2024-01-15T11:00:00 | main   | 16,234 | 48,901 |
| v_ghi789    | experiment-1      | 2024-01-16T09:00:00 | exp    | 15,432 | 45,678 |
```

## 2.4 CHECKOUT

Switch to a different version (read-only view or restore).

```
CheckoutStmt = 
  "checkout" VersionRef CheckoutMode?

VersionRef = StringLiteral | "~" Identifier

CheckoutMode = "readonly" | "restore"

-- View historical state (read-only)
CHECKOUT "before-migration" READONLY

-- Restore to historical state (destructive)
CHECKOUT "before-migration" RESTORE
```

### 30.4.1 READONLY Mode

```
CHECKOUT "v_abc123" READONLY

-- Now queries return data from that snapshot
MATCH t: Task RETURN t  -- shows tasks from v_abc123

-- Transformations are blocked
SPAWN t: Task { title = "Test" }
-- ERROR: Cannot transform in readonly checkout mode

-- Return to current
CHECKOUT ~HEAD
```

### 30.4.2 RESTORE Mode

```
CHECKOUT "before-migration" RESTORE
-- WARNING: This will overwrite current state

-- Current state is now identical to snapshot
-- Changes since snapshot are lost
```

### 30.4.3 Result

```typescript
interface CheckoutResult {
  success: boolean
  version: string
  mode: "readonly" | "restore"
  timestamp: Timestamp
  nodeCount: number
  edgeCount: number
}
```

## 2.5 DIFF

Compare two versions.

```
DiffStmt = "diff" VersionRef VersionRef

DIFF "before-migration" "after-migration"

-- Result:
Comparing before-migration → after-migration

Nodes added: 802
  Task: 523
  Person: 45
  ...

Nodes removed: 0

Nodes modified: 234
  Task: 200 (status, priority)
  Person: 34 (email)

Edges added: 3,223
  assigned_to: 523
  ...

Edges removed: 0
```

### 30.5.1 Detailed Diff

```
DIFF "v1" "v2" DETAILED

-- Shows individual changes:
+ Node Task #task_123 { title: "New task", ... }
~ Node Task #task_456 { status: "todo" → "done" }
+ Edge assigned_to(#task_123, #person_789)
...
```

### 30.5.2 Result

```typescript
interface DiffResult {
  from: string
  to: string
  summary: {
    nodesAdded: number
    nodesRemoved: number
    nodesModified: number
    edgesAdded: number
    edgesRemoved: number
  }
  details?: Change[]
}

interface Change {
  type: "add" | "remove" | "modify"
  target: "node" | "edge"
  id: string
  before?: Record<string, Value>
  after?: Record<string, Value>
}
```

## 2.6 Branching

### 30.6.1 CREATE BRANCH

```
BranchStmt =
    "create" "branch" StringLiteral ("from" VersionRef)?
  | "switch" "branch" StringLiteral
  | "show" "branches"
  | "delete" "branch" StringLiteral

-- Create branch from current
CREATE BRANCH "experiment"

-- Create branch from specific version
CREATE BRANCH "experiment" FROM "before-migration"
```

### 30.6.2 SWITCH BRANCH

```
SWITCH BRANCH "experiment"

-- Now working on experiment branch
SPAWN t: Task { title = "Experimental task" }
-- Only affects experiment branch
```

### 30.6.3 SHOW BRANCHES

```
SHOW BRANCHES

-- Result:
| Name       | Created             | Base           | Current |
|------------|---------------------|----------------|---------|
| main       | 2024-01-01T00:00:00 | -              | *       |
| experiment | 2024-01-16T09:00:00 | before-migration|        |
| feature-x  | 2024-01-17T14:00:00 | main           |         |
```

### 30.6.4 DELETE BRANCH

```
DELETE BRANCH "experiment"
-- Deletes branch and all its snapshots
```

## 2.7 MERGE

Merge one branch into another.

```
MergeStmt = "merge" "branch" StringLiteral ("into" StringLiteral)?

-- Merge experiment into current branch
MERGE BRANCH "experiment"

-- Merge experiment into main
MERGE BRANCH "experiment" INTO "main"
```

### 30.7.1 Merge Semantics

| Scenario | Behavior |
|----------|----------|
| No conflicts | Auto-merge |
| Same node modified differently | Conflict (requires resolution) |
| Node deleted in one, modified in other | Conflict |
| Independent changes | Auto-merge |

### 30.7.2 Conflict Resolution

```
MERGE BRANCH "experiment"

-- Result (if conflicts):
Merge conflicts detected:
  Node #task_123:
    main: { status: "done", priority: 5 }
    experiment: { status: "in_progress", priority: 8 }

Use RESOLVE to fix conflicts, then COMMIT MERGE.
```

```
-- Resolve by choosing version
RESOLVE #task_123 USING "experiment"

-- Or manual resolution
RESOLVE #task_123 {
  status = "done",
  priority = 8
}

-- Complete merge
COMMIT MERGE
```

### 30.7.3 Result

```typescript
interface MergeResult {
  success: boolean
  conflicts: Conflict[]
  merged: {
    nodesAdded: number
    nodesModified: number
    edgesAdded: number
  }
}

interface Conflict {
  id: string
  type: "node" | "edge"
  sourceValue: Record<string, Value>
  targetValue: Record<string, Value>
}
```

## 2.8 AST

```typescript
interface SnapshotStmt {
  kind: "Snapshot"
  label: string | null
}

interface CheckoutStmt {
  kind: "Checkout"
  version: string
  mode: "readonly" | "restore"
}

interface DiffStmt {
  kind: "Diff"
  from: string
  to: string
  detailed: boolean
}

interface CreateBranchStmt {
  kind: "CreateBranch"
  name: string
  from: string | null
}

interface SwitchBranchStmt {
  kind: "SwitchBranch"
  name: string
}

interface MergeBranchStmt {
  kind: "MergeBranch"
  source: string
  target: string | null
}
```

---

# 3. Query Control and Safety

## 3.1 TIMEOUT Clause

Limit query execution time to prevent runaway queries.

### 31.1.1 Syntax

```
TimeoutClause = "timeout" Duration

Duration = IntLiteral TimeUnit
TimeUnit = "ms" | "s" | "m" | "h"
```

### 31.1.2 Examples

```
-- Timeout after 5 seconds
MATCH t: Task, depends_on+(t, _)
RETURN t
TIMEOUT 5s

-- Timeout after 100 milliseconds
MATCH p: Person, follows+(p, _) [depth: 50]
RETURN p
TIMEOUT 100ms

-- Timeout after 1 minute (for complex reports)
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t)
TIMEOUT 1m
```

### 31.1.3 Behavior

When timeout is reached:
- Query execution stops immediately
- Partial results are **not** returned
- Error is raised with code E5002

```
ERROR [E5002] TIMEOUT

  Query execution timed out
  
  At: MATCH t: Task, depends_on+(t, _) RETURN t
  
  Timeout: 5s
  Elapsed: 5.001s
  
  Hint: Add LIMIT, refine WHERE clause, or increase timeout.
```

### 31.1.4 Engine Default

```
-- Set engine-level default timeout
SET engine.default_timeout = "30s"

-- Queries without explicit TIMEOUT use this default
-- Set to "none" to disable default timeout (not recommended)
```

## 3.2 Safety Limits and Warnings

### 31.2.1 Unbounded Result Warning

Queries without LIMIT that return large result sets trigger a warning:

```
MATCH t: Task
RETURN t

-- WARNING [E5001]: Query returned 10,000 rows (engine limit reached).
--                  Total matching: 1,234,567 rows.
--                  Add LIMIT or refine WHERE clause.
```

**Engine configuration:**
```
SET engine.max_unbounded_results = 10000  -- default
SET engine.warn_unbounded_results = true  -- default
```

### 31.2.2 Cartesian Product Warning

Patterns with unconnected components produce a warning:

```
MATCH t: Task, p: Person
RETURN t, p

-- WARNING: Pattern has unconnected components (t, p).
--          This produces a Cartesian product: |Task| × |Person| rows.
--          Did you mean to add an edge pattern?
```

### 31.2.3 COLLECT Size Limit

COLLECT aggregation has a configurable limit:

```
MATCH t: Task
RETURN COLLECT(t) AS all_tasks

-- If more than limit, error:
-- ERROR [E5003]: COLLECT exceeded size limit (10,000).
--                Total items: 50,000.
--                Use LIMIT in subquery or increase engine.max_collect_size.
```

**Engine configuration:**
```
SET engine.max_collect_size = 10000  -- default
```

### 31.2.4 Cascade Count Limit

Cascade operations (from KILL with cascade) have a limit:

```
KILL #root_org CASCADE

-- If cascade would affect too many nodes:
-- ERROR [E5004]: Cascade would affect 100,000+ nodes.
--                Limit: 10,000
--                Use KILL ... FORCE CASCADE or delete in batches.
```

**Override with FORCE:**
```
KILL #root_org FORCE CASCADE  -- Bypasses limit (use with caution)
```

**Engine configuration:**
```
SET engine.max_cascade_count = 10000  -- default
```

### 31.2.5 Bulk Operation Safety

Bulk KILL/UNLINK without WHERE requires explicit LIMIT or FORCE:

```
-- ERROR: Bulk KILL without WHERE requires LIMIT or FORCE
KILL { MATCH t: Task RETURN t }

-- Valid options:
KILL { MATCH t: Task RETURN t LIMIT 1000 }           -- With limit
KILL { MATCH t: Task WHERE t.archived RETURN t }     -- With WHERE
KILL { MATCH t: Task RETURN t } FORCE                -- Explicit override
```

### 31.2.6 CHECKOUT RESTORE Confirmation

Destructive RESTORE requires confirmation:

```
CHECKOUT "old-version" RESTORE

-- ERROR: RESTORE is destructive and will overwrite current state.
--        Use CHECKOUT ... RESTORE CONFIRM to proceed.
--        Consider creating a snapshot first: SNAPSHOT "before-restore"

CHECKOUT "old-version" RESTORE CONFIRM  -- Proceeds
```

## 3.3 DRY RUN Mode

Preview the effects of a statement without executing it.

### 31.3.1 Syntax

```
DryRunStmt = "dry" "run" TransformationStmt
```

### 31.3.2 Examples

```
DRY RUN KILL { MATCH t: Task WHERE t.archived = true RETURN t }

-- Result:
Dry run complete (no changes made):
  Would kill: 1,234 nodes
  Would cascade: 0 nodes
  Would unlink: 3,456 edges
  
  Sample affected:
    Task #task_001 { title: "Old task 1" }
    Task #task_002 { title: "Old task 2" }
    ... (1,232 more)
```

```
DRY RUN SET { MATCH t: Task WHERE t.status = "pending" RETURN t }.status = "archived"

-- Result:
Dry run complete (no changes made):
  Would modify: 567 nodes
  
  Sample changes:
    Task #task_100: status "pending" → "archived"
    Task #task_101: status "pending" → "archived"
    ... (565 more)
```

### 31.3.3 Result Format

```typescript
interface DryRunResult {
  executed: false
  wouldAffect: {
    nodesKilled?: number
    nodesCascaded?: number
    nodesModified?: number
    edgesLinked?: number
    edgesUnlinked?: number
  }
  samples?: {
    type: "node" | "edge"
    id: string
    change: "kill" | "modify" | "link" | "unlink"
    before?: Record<string, Value>
    after?: Record<string, Value>
  }[]
  warnings?: string[]
}
```

---

## 3.1 Debug Operations

## 3.1 Overview

Debug statements help understand query execution and performance.

```
DebugStmt =
    ExplainStmt
  | ProfileStmt
```

## 3.2 EXPLAIN

Show the execution plan for a statement without executing it.

```
ExplainStmt = "explain" Statement

EXPLAIN MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name
```

### 31.2.1 Output

```
EXPLAIN MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name

-- Result:
Query Plan:
├─ Scan: Task (estimated: 8,234 rows)
│  └─ Filter: status = "done" (estimated: 2,000 rows)
├─ Edge Lookup: belongs_to (from task)
│  └─ Index: belongs_to_task_idx
├─ Node Fetch: Project
└─ Project: t.title, p.name

Estimated cost: 2,450
Indexes used: belongs_to_task_idx
```

### 31.2.2 Plan Components

| Component | Description |
|-----------|-------------|
| Scan | Full table scan of type |
| Index Scan | Scan using index |
| Filter | Apply WHERE condition |
| Edge Lookup | Find edges from/to node |
| Node Fetch | Retrieve node by ID |
| Join | Combine multiple sources |
| Project | Select output columns |
| Sort | ORDER BY |
| Limit | LIMIT/OFFSET |
| Aggregate | COUNT, SUM, etc. |

## 3.3 PROFILE

Execute statement and show actual performance metrics.

```
ProfileStmt = "profile" Statement

PROFILE MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name
```

### 31.3.1 Output

```
PROFILE MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name

-- Result:
Execution Profile:
├─ Scan: Task
│  ├─ Rows scanned: 8,234
│  ├─ Rows passed: 2,156
│  ├─ Time: 12ms
│  └─ Filter: status = "done"
├─ Edge Lookup: belongs_to
│  ├─ Lookups: 2,156
│  ├─ Edges found: 2,156
│  ├─ Time: 3ms
│  └─ Index: belongs_to_task_idx (hit rate: 100%)
├─ Node Fetch: Project
│  ├─ Fetches: 234 (deduplicated)
│  └─ Time: 1ms
└─ Project
   └─ Time: <1ms

Total time: 16ms
Rows returned: 2,156
Memory used: 4.2 MB
```

### 31.3.2 Profile Metrics

| Metric | Description |
|--------|-------------|
| Rows scanned | Total rows examined |
| Rows passed | Rows passing filter |
| Time | Execution time for step |
| Index hit rate | Cache/index effectiveness |
| Memory used | Peak memory consumption |
| Disk reads | I/O operations (if applicable) |

## 3.4 Optimization Hints

Based on EXPLAIN/PROFILE, optimize by:

```
-- Add missing index
CREATE INDEX task_status ON Task(status)

-- Rewrite query to use index
-- Before (full scan):
MATCH t: Task WHERE contains(t.title, "urgent")

-- After (if indexed):
MATCH t: Task WHERE t.priority >= 8
```

## 3.5 AST

```typescript
interface ExplainStmt {
  kind: "Explain"
  statement: Statement
}

interface ProfileStmt {
  kind: "Profile"
  statement: Statement
}
```

---

# 4. Complete Grammar (GQL)

For shared constructs (Patterns, Types, Expressions, Literals), see **Part I: Foundations §6.3**.

```ebnf
(* Statements *)
Statement        = ObservationStmt | TransformationStmt | TransactionStmt 
                 | AdminStmt | VersionStmt | DebugStmt

(* Observation *)
ObservationStmt  = MatchStmt | WalkStmt | InspectStmt

MatchStmt        = "match" Pattern OptionalMatchClause* WhereClause? ReturnClause OrderClause? LimitClause? TimeoutClause?
OptionalMatchClause = "optional" "match" Pattern
WhereClause      = "where" Expr
ReturnClause     = "return" "distinct"? Projection ("," Projection)*
Projection       = Expr ("as" Identifier)? | "*"
OrderClause      = "order" "by" OrderTerm ("," OrderTerm)*
OrderTerm        = Expr ("asc" | "desc")?
LimitClause      = "limit" IntLiteral ("offset" IntLiteral)?

WalkStmt         = "walk" "from" StartExpr FollowClause+ UntilClause? WalkReturn
StartExpr        = Expr | "{" MatchStmt "}"
FollowClause     = "follow" EdgeSpec Direction? DepthSpec?
EdgeSpec         = Identifier ("|" Identifier)* | "*"
Direction        = "outbound" | "inbound" | "any"
DepthSpec        = "[" "depth:" IntLiteral (".." IntLiteral)? "]"
UntilClause      = "until" Expr
WalkReturn       = "return" ("path" | "nodes" | "edges" | "terminal" | Projection ("," Projection)*)

InspectStmt      = "inspect" IdRef ReturnClause?
IdRef            = "#" (Identifier | StringLiteral)

(* Transformation *)
TransformationStmt = SpawnStmt | KillStmt | LinkStmt | UnlinkStmt | SetStmt

SpawnStmt        = "spawn" Identifier ":" TypeExpr AttrBlock? ReturningClause?
AttrBlock        = "{" (AttrAssign ("," AttrAssign)*)? "}"
AttrAssign       = Identifier "=" Expr
ReturningClause  = "returning" ("id" | "*" | Identifier ("," Identifier)*)

KillStmt         = "kill" KillTarget CascadeClause? ReturningClause?
KillTarget       = IdRef | "{" MatchStmt "}"
CascadeClause    = "cascade" | "no" "cascade"

LinkStmt         = "link" ("if" "not" "exists")? Identifier "(" TargetList ")" ("as" Identifier)? AttrBlock? ReturningClause?
TargetList       = TargetRef ("," TargetRef)*
TargetRef        = IdRef | "{" MatchStmt "}" | SpawnExpr
SpawnExpr        = "spawn" TypeExpr AttrBlock? ("as" Identifier)?

UnlinkStmt       = "unlink" UnlinkTarget ReturningClause?
UnlinkTarget     = IdRef | EdgePattern | "{" MatchStmt "}"

SetStmt          = "set" SetTarget ("." Identifier "=" Expr | AttrBlock) ReturningClause?
SetTarget        = IdRef | "{" MatchStmt "}"

(* Transactions *)
TransactionStmt  = BeginStmt | CommitStmt | RollbackStmt | SavepointStmt | RollbackToStmt
BeginStmt        = "begin" ("read" "committed" | "serializable")?
CommitStmt       = "commit"
RollbackStmt     = "rollback"
SavepointStmt    = "savepoint" Identifier
RollbackToStmt   = "rollback" "to" Identifier

(* Administration *)
AdminStmt        = LoadStmt | ExtendStmt | ShowStmt | CreateIndexStmt | DropIndexStmt
LoadStmt         = "load" "ontology" ("from" StringLiteral | "{" OntologySource "}")
ExtendStmt       = "extend" "ontology" Identifier? "{" Declaration* "}"
ShowStmt         = "show" ShowTarget
ShowTarget       = "types" | "edges" | "constraints" | "rules" | "indexes" 
                 | "statistics" | "status" | "branches" | "versions"
                 | "type" Identifier | "edge" Identifier 
                 | "constraint" Identifier | "rule" Identifier
CreateIndexStmt  = "create" "index" Identifier? "on" IndexTarget
IndexTarget      = TypeExpr "(" Identifier ("asc" | "desc")? ")" | Identifier
DropIndexStmt    = "drop" "index" Identifier

(* Versioning *)
VersionStmt      = SnapshotStmt | CheckoutStmt | DiffStmt | BranchStmt
SnapshotStmt     = "snapshot" StringLiteral?
CheckoutStmt     = "checkout" VersionRef ("readonly" | "restore")?
VersionRef       = StringLiteral | "@" Identifier
DiffStmt         = "diff" VersionRef VersionRef "detailed"?
BranchStmt       = "create" "branch" StringLiteral ("from" VersionRef)?
                 | "switch" "branch" StringLiteral
                 | "delete" "branch" StringLiteral
                 | "merge" "branch" StringLiteral ("into" StringLiteral)?

(* Debug *)
DebugStmt        = ExplainStmt | ProfileStmt | DryRunStmt
ExplainStmt      = "explain" Statement
ProfileStmt      = "profile" Statement
DryRunStmt       = "dry" "run" TransformationStmt

(* Query Control *)
TimeoutClause    = "timeout" Duration
Duration         = IntLiteral TimeUnit
TimeUnit         = "ms" | "s" | "min" | "h"
```

---

# 5. Quick Reference

See individual sections for detailed syntax and examples:
- **Observation**: §20 MATCH, §21 WALK, §22 INSPECT
- **Transformation**: §23 SPAWN, §24 KILL, §25 LINK, §26 UNLINK, §27 SET
- **Transaction**: §28 BEGIN/COMMIT/ROLLBACK
- **Administration**: §29 LOAD, EXTEND, SHOW, INDEX
- **Versioning**: §30 SNAPSHOT, CHECKOUT, DIFF, BRANCH, MERGE
- **Debug**: §31 Query Control, §32 EXPLAIN/PROFILE

---

# 6. Summary

## 6.1 Part III Contents

| Section | Contents |
|---------|----------|
| 19. Overview | Purpose, execution model, statement types |
| 20. MATCH | Pattern matching, WHERE, RETURN, ORDER, LIMIT |
| 21. WALK | Path traversal, FOLLOW, UNTIL, directions |
| 22. INSPECT | Direct node/edge access by ID |
| 23. SPAWN | Create nodes |
| 24. KILL | Remove nodes, cascade behavior |
| 25. LINK | Create edges, inline SPAWN |
| 26. UNLINK | Remove edges |
| 27. SET | Modify attributes, block syntax |
| 28. Transactions | BEGIN, COMMIT, ROLLBACK, savepoints |
| 29. Administration | LOAD, EXTEND, SHOW, INDEX |
| 30. Versioning | SNAPSHOT, CHECKOUT, DIFF, BRANCH, MERGE |
| 31. Query Control | TIMEOUT, safety limits, DRY RUN |
| 32. Debug | EXPLAIN, PROFILE |
| 33. Grammar | Complete EBNF |

## 6.2 Design Principles

| Principle | How Applied |
|-----------|-------------|
| Graph-native | Vocabulary: SPAWN/KILL, LINK/UNLINK, WALK |
| Observation vs Transformation | Clear separation of read/write operations |
| Explicit | All behavior declared, no hidden defaults |
| Composable | Match patterns reused across operations |
| Inspectable | SHOW, EXPLAIN, PROFILE for transparency |

---

*End of Part III: GQL (Runtime)*

---

# Appendix A: Reserved Keywords

See **Part I: Foundations §2.5** for the complete list of reserved keywords shared across both languages.

---

# Appendix B: Operator Precedence

See **Part I: Foundations §2.8.1** for operator precedence table.

---

# Appendix C: Built-in Functions

For string, numeric, timestamp, and general functions, see **Part I: Foundations §5.7.1**.

## Aggregate Functions (GQL-specific)

| Function | Signature | Description |
|----------|-----------|-------------|
| `COUNT(x)` | Any → Int | Count matches |
| `COUNT(DISTINCT x)` | Any → Int | Count unique |
| `SUM(x)` | Number → Number | Sum |
| `AVG(x)` | Number → Float | Average |
| `MIN(x)` | Comparable → Same | Minimum |
| `MAX(x)` | Comparable → Same | Maximum |
| `COLLECT(x)` | Any → List | Collect to list |

---

# Appendix D: Error Message Format

## D.1 Standard Error Structure

All errors follow a consistent structure:

```typescript
interface Error {
  code: string              // Machine-readable code, e.g., "E2003"
  category: ErrorCategory   // Error classification
  message: string           // Human-readable message
  location?: {
    line: number
    column: number
    snippet: string         // Relevant source code
  }
  context?: {
    nodeId?: string
    edgeId?: string
    constraintName?: string
    ruleName?: string
    attributeName?: string
    typeName?: string
  }
  hints?: string[]          // Suggested fixes
}

type ErrorCategory =
  | "SYNTAX_ERROR"
  | "TYPE_ERROR"
  | "CONSTRAINT_VIOLATION"
  | "NOT_FOUND"
  | "PERMISSION_DENIED"
  | "LIMIT_EXCEEDED"
  | "INTERNAL_ERROR"
```

## D.2 Error Code Ranges

| Range | Category |
|-------|----------|
| E1xxx | Syntax errors |
| E2xxx | Constraint violations |
| E3xxx | Type errors |
| E4xxx | Not found errors |
| E5xxx | Limit/performance errors |
| E9xxx | Internal errors |

## D.3 Example Error Messages

**Constraint Violation:**
```
ERROR [E2003] CONSTRAINT_VIOLATION

  Constraint 'temporal_order' violated
  
  At: LINK causes(#event_a, #event_b)
  
  Context:
    event_a.timestamp = 1705320000000 (2024-01-15T10:00:00Z)
    event_b.timestamp = 1705316400000 (2024-01-15T09:00:00Z)
  
  Message: Cause must precede effect
  
  Hint: Ensure event_a.timestamp < event_b.timestamp
```

**Type Error:**
```
ERROR [E3001] TYPE_ERROR

  Type mismatch in attribute assignment
  
  At line 5, column 14:
    SPAWN t: Task { priority = "high" }
                             ^^^^^^^
  
  Expected: Int
  Got: String
  
  Hint: Use numeric priority (e.g., priority = 5)
```

**Not Found:**
```
ERROR [E4001] NOT_FOUND

  Node not found
  
  At: SET #nonexistent_node.status = "done"
  
  ID: nonexistent_node
  
  Hint: Verify the node ID exists. Use INSPECT #id to check.
```

**Limit Exceeded:**
```
WARNING [E5001] LIMIT_EXCEEDED

  Observation result limit reached
  
  At: MATCH t: Task RETURN t
  
  Returned: 10,000 rows (limit)
  Total matching: 1,234,567 rows
  
  Hint: Add WHERE clause or LIMIT to reduce results.
```

---

