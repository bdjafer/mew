# HOHG Language Specification

## Part III: HOHG Language (Runtime)

**Version:** 1.0
**Status:** Draft
**Scope:** Runtime language for graph operations

---

# 19. HOHG Language Overview

## 19.1 Purpose

The HOHG Language is the runtime interface to the graph. It provides:

| Category | Operations | Purpose |
|----------|------------|---------|
| **Observation** | MATCH, WALK, INSPECT | Read without changing |
| **Transformation** | SPAWN, KILL, LINK, UNLINK, SET | Change the graph |
| **Transaction** | BEGIN, COMMIT, ROLLBACK | Group operations atomically |
| **Administration** | LOAD, EXTEND, SHOW, INDEX | Manage schema and engine |
| **Versioning** | SNAPSHOT, CHECKOUT, DIFF | Time travel and branching |
| **Debug** | EXPLAIN, PROFILE | Understand performance |

## 19.2 Execution Model

HOHG Language statements are **interpreted** against a running engine:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Statement   │────▶│   Engine     │────▶│   Result     │
│  (text)      │     │   (runtime)  │     │   (data)     │
└──────────────┘     └──────────────┘     └──────────────┘
```

Unlike the Ontology DSL (compiled once), HOHG statements execute immediately.

## 19.3 Statement Structure

```
Statement =
    ObservationStmt
  | TransformationStmt
  | TransactionStmt
  | AdminStmt
  | VersionStmt
  | DebugStmt
```

Statements can be:
- **Single-line:** Execute immediately
- **Multi-line:** Continue until complete
- **In transaction:** Grouped with BEGIN/COMMIT

## 19.4 Result Types

| Statement Type | Returns |
|----------------|---------|
| Observation | Stream of matches/nodes/edges |
| Transformation | Affected IDs + count |
| Transaction | Success/failure |
| Admin | Status/metadata |
| Version | Version info |
| Debug | Execution plan/statistics |

---

# 20. Observation: MATCH

## 20.1 Purpose

MATCH finds all subgraphs matching a pattern. It is the primary read operation.

## 20.2 Syntax

```
MatchStmt = 
  "match" Pattern
  WhereClause?
  ReturnClause               -- REQUIRED
  OrderClause?
  LimitClause?

WhereClause = "where" Expr

ReturnClause = "return" Projection ("," Projection)*

Projection = 
    Expr
  | Expr "as" Identifier
  | "*"

OrderClause = "order" "by" OrderTerm ("," OrderTerm)*

OrderTerm = Expr ("asc" | "desc")?

LimitClause = "limit" IntLiteral ("offset" IntLiteral)?
```

**Note:** The RETURN clause is **required**. MATCH without RETURN is a compile error:

```
-- INVALID: Missing RETURN
MATCH t: Task WHERE t.priority > 5
-- ERROR: MATCH statement requires RETURN clause.
--        Did you mean: MATCH t: Task WHERE t.priority > 5 RETURN t

-- VALID:
MATCH t: Task WHERE t.priority > 5
RETURN t
```

### 20.2.1 MATCH in Different Contexts

MATCH behavior varies by context:

| Context | RETURN Required? | Purpose |
|---------|------------------|---------|
| **Statement** | ✅ Yes | Specifies what to return to caller |
| **Subquery** (in KILL, SET, etc.) | ✅ Yes | Specifies what to operate on |
| **EXISTS pattern** | ❌ No | EXISTS uses Pattern, not MATCH |

**Examples:**
```
-- Statement MATCH: RETURN required
MATCH t: Task WHERE t.status = "done"
RETURN t

-- Subquery MATCH: RETURN specifies targets
KILL { MATCH t: Task WHERE t.archived RETURN t }
SET { MATCH t: Task WHERE t.old RETURN t }.status = "archived"

-- EXISTS: Uses Pattern syntax, not MATCH
WHERE EXISTS(p: Person, assigned_to(t, p))
-- Note: This is a Pattern, not a MATCH statement
```

## 20.3 Basic Examples

```
-- Find all events
MATCH e: Event
RETURN e

-- Find events with condition
MATCH e: Event
WHERE e.timestamp > 1000
RETURN e

-- Find causal pairs
MATCH e1: Event, e2: Event, causes(e1, e2)
RETURN e1, e2

-- With edge binding
MATCH e1: Event, e2: Event, causes(e1, e2) AS c
WHERE c.strength > 0.5
RETURN e1.name, e2.name, c.strength

-- Projection with alias
MATCH t: Task
RETURN t.title AS name, t.priority AS prio

-- Return everything
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN *
```

## 20.4 Pattern Matching

### 20.4.1 Node Patterns

```
-- Single type
MATCH p: Person
RETURN p

-- Union type
MATCH e: Task | Project
RETURN e

-- Any type
MATCH n: any
RETURN n
```

### 20.4.2 Edge Patterns

```
-- Simple edge
MATCH p: Person, t: Team, member_of(p, t)
RETURN p, t

-- With edge binding
MATCH p: Person, t: Team, member_of(p, t) AS m
RETURN p.name, t.name, m.role

-- Anonymous target
MATCH t: Task, assigned_to(t, _)
RETURN t  -- tasks that are assigned to someone

-- Multiple edges
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person)
RETURN t, p, person
```

### 20.4.3 Higher-Order Patterns

```
-- Match edge about edge
MATCH e1: Event, e2: Event,
      causes(e1, e2) AS c,
      confidence(c, level)
WHERE level > 0.7
RETURN e1, e2, level

-- Any higher-order edge
MATCH e1: Event, e2: Event,
      causes(e1, e2) AS c,
      meta: edge<any>(c, _)
RETURN c, meta
```

### 20.4.4 Transitive Patterns

```
-- One or more hops
MATCH a: Person, b: Person, follows+(a, b)
RETURN a, b

-- Zero or more hops
MATCH a: Task, b: Task, depends_on*(a, b)
RETURN a, b

-- With depth limit
MATCH a: Person, b: Person, follows+(a, b) [depth: 5]
RETURN a, b
```

### 20.4.5 Negative Patterns

```
-- Tasks without assignment
MATCH t: Task
WHERE NOT EXISTS(assigned_to(t, _))
RETURN t

-- Persons not in any team
MATCH p: Person
WHERE NOT EXISTS(t: Team, member_of(p, t))
RETURN p

-- Complex negation
MATCH t: Task
WHERE NOT EXISTS(
  p: Person, assigned_to(t, p)
  WHERE p.active = true
)
RETURN t  -- tasks not assigned to active persons
```

## 20.4.6 OPTIONAL MATCH

OPTIONAL MATCH attempts to match a pattern, but returns NULL for unmatched variables instead of filtering out the row.

**Syntax:**
```
MatchStmt = 
  "match" Pattern
  OptionalMatchClause*
  WhereClause?
  ReturnClause
  ...

OptionalMatchClause = "optional" "match" Pattern
```

**Basic Example:**
```
-- Get all tasks, with assignee if exists (NULL if unassigned)
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p)
RETURN t.title, p.name
```

| t.title | p.name |
|---------|--------|
| "Task A" | "Alice" |
| "Task B" | NULL |
| "Task C" | "Bob" |

**Without OPTIONAL MATCH:**
```
-- Only returns tasks WITH assignees
MATCH t: Task, assigned_to(t, p)
RETURN t.title, p.name
```

| t.title | p.name |
|---------|--------|
| "Task A" | "Alice" |
| "Task C" | "Bob" |

**Multiple OPTIONAL MATCH:**
```
MATCH t: Task
OPTIONAL MATCH assigned_to(t, assignee)
OPTIONAL MATCH belongs_to(t, project)
RETURN t.title, assignee.name, project.name
```

**OPTIONAL MATCH with WHERE:**
```
-- Get tasks with their high-priority assignees (if any)
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p) WHERE p.priority_level > 5
RETURN t.title, p.name
-- p is NULL if no high-priority assignee exists
```

**Semantics:**
- Variables from OPTIONAL MATCH are bound to NULL if pattern doesn't match
- WHERE on OPTIONAL MATCH filters the optional pattern, not the entire result
- Multiple OPTIONAL MATCH clauses are independent

**Common Use Cases:**
```
-- Left outer join equivalent
MATCH t: Task
OPTIONAL MATCH belongs_to(t, p)
RETURN t, p

-- Get entity with optional metadata
MATCH u: User
OPTIONAL MATCH has_profile(u, profile)
OPTIONAL MATCH has_avatar(u, avatar)
RETURN u.name, profile.bio, avatar.url

-- Aggregate with optional relationships
MATCH p: Project
OPTIONAL MATCH t: Task, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count
-- Projects with 0 tasks return task_count = 0
```

## 20.5 WHERE Clause

### 20.5.1 Basic Conditions

```
MATCH t: Task
WHERE t.priority > 5
RETURN t

MATCH t: Task
WHERE t.status = "done" AND t.priority >= 8
RETURN t

MATCH t: Task
WHERE t.title != null AND length(t.title) > 10
RETURN t
```

### 20.5.2 Attribute Comparisons

```
-- Compare attributes
MATCH e1: Event, e2: Event, causes(e1, e2)
WHERE e1.timestamp < e2.timestamp
RETURN e1, e2

-- String operations
MATCH p: Person
WHERE starts_with(p.email, "admin")
RETURN p

-- Null checks
MATCH t: Task
WHERE t.description != null
RETURN t
```

### 20.5.3 EXISTS in WHERE

```
-- Positive existence
MATCH t: Task
WHERE EXISTS(p: Person, assigned_to(t, p))
RETURN t

-- Negative existence
MATCH t: Task
WHERE NOT EXISTS(depends_on(t, _))
RETURN t

-- Nested conditions in EXISTS
MATCH t: Task
WHERE EXISTS(
  p: Person, assigned_to(t, p)
  WHERE p.role = "admin"
)
RETURN t
```

### 20.5.4 Aggregates in WHERE

**Unlike SQL, HOHG allows aggregate functions directly in WHERE clauses:**

```
-- Find tasks with more than 2 assignees
MATCH t: Task
WHERE COUNT(p: Person, assigned_to(t, p)) > 2
RETURN t

-- Find projects with no tasks
MATCH p: Project
WHERE COUNT(t: Task, belongs_to(t, p)) = 0
RETURN p

-- Find people following more than they're followed by
MATCH p: Person
WHERE COUNT(f: Person, follows(p, f)) > COUNT(f: Person, follows(f, p))
RETURN p
```

**Syntax:**
```
AggregateInWhere = AggregateFunc "(" Pattern ")"
AggregateFunc = "COUNT" | "SUM" | "AVG" | "MIN" | "MAX"
```

**Semantics:**
- The aggregate is computed for each candidate row
- Pattern variables are scoped to the aggregate expression
- Outer variables can be referenced (correlated subquery)

**Comparison with SQL:**
```sql
-- SQL requires subquery:
SELECT * FROM tasks t
WHERE (SELECT COUNT(*) FROM assignments a WHERE a.task_id = t.id) > 2

-- HOHG allows inline:
MATCH t: Task
WHERE COUNT(p: Person, assigned_to(t, p)) > 2
RETURN t
```

**Performance note:** Aggregates in WHERE may require per-row computation. For large result sets, consider restructuring with explicit aggregation and filtering.

## 20.6 RETURN Clause

### 20.6.1 Projections

```
-- Return whole nodes
MATCH t: Task
RETURN t

-- Return specific attributes
MATCH t: Task
RETURN t.title, t.priority

-- Return with aliases
MATCH t: Task
RETURN t.title AS name, t.priority AS prio

-- Return expressions
MATCH t: Task
RETURN t.title, t.priority * 10 AS weighted

-- Return all bound variables
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN *
```

### 20.6.2 DISTINCT

```
-- Remove duplicates
MATCH t: Task, p: Person, assigned_to(t, p)
RETURN DISTINCT p

-- Distinct on projection
MATCH t: Task
RETURN DISTINCT t.status
```

### 20.6.3 Aggregations

```
-- Count
MATCH t: Task
RETURN COUNT(t)

-- Count with condition
MATCH t: Task
WHERE t.status = "done"
RETURN COUNT(t) AS completed

-- Multiple aggregations
MATCH t: Task
RETURN COUNT(t) AS total,
       MIN(t.priority) AS min_prio,
       MAX(t.priority) AS max_prio,
       AVG(t.priority) AS avg_prio

-- Group by (implicit)
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count
```

### 20.6.4 Aggregation Functions

| Function | Description | Input | Output |
|----------|-------------|-------|--------|
| `COUNT(x)` | Count matches | Any | Int |
| `COUNT(DISTINCT x)` | Count unique | Any | Int |
| `SUM(x)` | Sum values | Int/Float | Same |
| `AVG(x)` | Average | Int/Float | Float |
| `MIN(x)` | Minimum | Comparable | Same |
| `MAX(x)` | Maximum | Comparable | Same |
| `COLLECT(x)` | Collect into list | Any | List |

### 20.6.4.1 COLLECT Limits

COLLECT aggregation has a configurable size limit to prevent memory issues:

```
-- Default: engine limit (default 10,000)
COLLECT(t) AS all_tasks

-- Explicit limit (truncates silently)
COLLECT(t) [limit: 100] AS top_tasks

-- Explicit unlimited (use with caution)
COLLECT(t) [limit: none] AS all_tasks
```

**Engine configuration:**
```
SET engine.default_collect_limit = 10000
```

**Behavior when limit exceeded:**
```
-- With default limit:
MATCH t: Task
RETURN COLLECT(t)  -- If > 10,000 tasks exist

ERROR [E5003]: COLLECT exceeded size limit
  Limit: 10,000 (engine default)
  Items: 50,000+
  
  Hint: Use COLLECT(t) [limit: N] to truncate,
        or COLLECT(t) [limit: none] to allow unlimited.
```

**Examples:**
```
-- Get first 10 tasks per project
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COLLECT(t) [limit: 10] AS sample_tasks

-- Get all (when you know it's bounded)
MATCH t: Task WHERE t.status = "critical"
RETURN COLLECT(t) [limit: none] AS critical_tasks
```

### 20.6.5 Grouping

When mixing aggregations with non-aggregated values, non-aggregated values become grouping keys:

```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t), AVG(t.priority)
--     ^^^^^^
--     grouping key
```

Returns one row per unique `p.name`.

## 20.7 ORDER BY Clause

```
-- Single order
MATCH t: Task
RETURN t
ORDER BY t.priority DESC

-- Multiple order
MATCH t: Task
RETURN t
ORDER BY t.priority DESC, t.created_at ASC

-- Order by expression
MATCH t: Task
RETURN t
ORDER BY length(t.title)

-- Order by alias
MATCH t: Task
RETURN t.title, t.priority * 10 AS score
ORDER BY score DESC
```

Default order is ASC (ascending).

## 20.8 LIMIT and OFFSET

```
-- Limit results
MATCH t: Task
RETURN t
LIMIT 10

-- Pagination
MATCH t: Task
RETURN t
ORDER BY t.created_at DESC
LIMIT 20 OFFSET 40

-- First N
MATCH t: Task
ORDER BY t.priority DESC
LIMIT 1
RETURN t  -- highest priority task
```

## 20.9 Complete Examples

```
-- Complex query with all clauses
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person) AS a
WHERE t.status != "done"
  AND p.status = "active"
  AND person.active = true
RETURN t.title AS task,
       p.name AS project,
       person.name AS assignee,
       a.assigned_at AS since
ORDER BY t.priority DESC, a.assigned_at ASC
LIMIT 50

-- Aggregation with grouping
MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.created_at > now() - 604800000  -- last 7 days
RETURN p.name AS project,
       COUNT(t) AS total,
       COUNT(t WHERE t.status = "done") AS completed
ORDER BY total DESC
LIMIT 10

-- Path query
MATCH start: Person, end: Person, follows+(start, end) [depth: 3]
WHERE start.name = "Alice"
RETURN end.name, end.email
```

## 20.10 Result Format

MATCH returns a **stream of result rows**:

```typescript
interface MatchResult {
  columns: string[]           // column names
  rows: ResultRow[]           // result data
  stats: {
    matchCount: number        // patterns matched
    returnCount: number       // rows returned
    executionTime: number     // milliseconds
  }
}

interface ResultRow {
  [column: string]: Value     // column name → value
}

type Value = 
  | string | number | boolean | null
  | Timestamp
  | NodeRef | EdgeRef
  | Value[]                   // for COLLECT
```

## 20.11 AST

```typescript
interface MatchStmt {
  kind: "Match"
  pattern: Pattern
  where: Expr | null
  return: ReturnClause | null
  orderBy: OrderTerm[] | null
  limit: number | null
  offset: number | null
}

interface ReturnClause {
  distinct: boolean
  projections: Projection[]
}

interface Projection {
  expr: Expr
  alias: string | null
}

interface OrderTerm {
  expr: Expr
  direction: "asc" | "desc"
}
```

---

# 21. Observation: WALK

## 21.1 Purpose

WALK performs path traversal from a starting point, following edges according to specified rules. Unlike MATCH (which finds patterns), WALK navigates structure.

## 21.2 Syntax

```
WalkStmt =
  "walk" "from" StartExpr
  FollowClause+
  UntilClause?
  ReturnClause

StartExpr = Expr | "{" MatchStmt "}"

FollowClause = "follow" EdgeSpec Direction? DepthSpec?

EdgeSpec = 
    Identifier                    -- specific edge type
  | Identifier "|" Identifier     -- multiple edge types
  | "*"                           -- any edge

Direction = "outbound" | "inbound" | "any"

DepthSpec = "[" "depth:" IntLiteral (".." IntLiteral)? "]"

UntilClause = "until" Expr

ReturnClause = 
    "return" "path"
  | "return" "nodes"
  | "return" "edges"
  | "return" "terminal"
  | "return" Projection ("," Projection)*
```

## 21.3 Basic Examples

```
-- Walk from a specific node
WALK FROM #node_123
FOLLOW causes
RETURN nodes

-- Walk from matched nodes
WALK FROM { MATCH p: Person WHERE p.name = "Alice" RETURN p }
FOLLOW follows [depth: 3]
RETURN terminal

-- Follow multiple edge types
WALK FROM #task_456
FOLLOW depends_on | subtask_of
RETURN nodes

-- Follow any edge
WALK FROM #event_789
FOLLOW * [depth: 2]
RETURN nodes
```

## 21.4 Starting Points

### 21.4.1 By ID

```
WALK FROM #node_abc123
FOLLOW causes
RETURN nodes
```

The `@` prefix denotes a node/edge ID.

### 21.4.2 From MATCH Result

```
WALK FROM { MATCH t: Task WHERE t.priority = 10 RETURN t }
FOLLOW depends_on
RETURN nodes
```

**Multiple starting points behavior:**

When the inner MATCH returns multiple nodes, WALK traverses from **all** of them:

```
-- If 5 high-priority tasks exist:
WALK FROM { MATCH t: Task WHERE t.priority = 10 RETURN t }
FOLLOW depends_on
RETURN NODES

-- Behavior:
-- 1. Walk from task_1, collect reachable nodes
-- 2. Walk from task_2, collect reachable nodes
-- 3. ... (for all 5 tasks)
-- 4. UNION all results (deduplicated)
-- 5. Return unique nodes
```

| Return Type | Multiple Starts Behavior |
|-------------|--------------------------|
| `RETURN NODES` | Union of all visited nodes (deduplicated) |
| `RETURN EDGES` | Union of all traversed edges (deduplicated) |
| `RETURN PATH` | All paths from all starting points |
| `RETURN TERMINAL` | Union of all terminal nodes (deduplicated) |

**Order:** Results are not guaranteed to be in any particular order unless ORDER BY is used in the inner MATCH.

### 21.4.3 From Variable

```
-- In programmatic context
WALK FROM $startNode
FOLLOW causes
RETURN path
```

## 21.5 FOLLOW Clause

### 21.5.1 Edge Type

```
-- Single edge type
FOLLOW causes

-- Multiple edge types (OR)
FOLLOW causes | leads_to | triggers

-- Any edge
FOLLOW *
```

### 21.5.2 Direction

```
-- Outbound: follow edge from source to target (default)
FOLLOW causes OUTBOUND

-- Inbound: follow edge from target to source
FOLLOW causes INBOUND

-- Any direction
FOLLOW causes ANY
```

Default is `OUTBOUND`.

### 21.5.3 Depth Control

```
-- Exactly N hops
FOLLOW causes [depth: 3]

-- Range of hops
FOLLOW causes [depth: 1..5]

-- Unlimited (use with caution)
FOLLOW causes [depth: 1..100]
```

Default depth is `1..100`.

### 21.5.4 Multiple FOLLOW Clauses

Multiple FOLLOW clauses are executed in sequence:

```
WALK FROM #person_123
FOLLOW member_of          -- person → team
FOLLOW owns               -- team → project
FOLLOW contains           -- project → task
RETURN terminal
```

This walks: Person → Team → Project → Task

## 21.6 UNTIL Clause

Stop traversal when condition is met:

```
-- Stop at completed tasks
WALK FROM #task_start
FOLLOW depends_on
UNTIL node.status = "done"
RETURN nodes

-- Stop at depth or condition
WALK FROM #person_123
FOLLOW follows [depth: 10]
UNTIL node.role = "admin"
RETURN path
```

### 21.6.1 UNTIL Semantics

- Traversal stops at nodes matching UNTIL condition
- Those nodes ARE included in results
- Traversal continues on other branches

```
-- Graph: A → B → C → D
--            ↓
--            E → F

WALK FROM #A
FOLLOW edge
UNTIL node.name = "C"
RETURN nodes

-- Returns: A, B, C, E, F
-- (stopped at C, but continued through E)
```

## 21.7 RETURN Options

### 21.7.1 Return NODES

```
WALK FROM #start
FOLLOW causes
RETURN NODES
```

Returns all visited nodes (deduplicated).

### 21.7.2 Return EDGES

```
WALK FROM #start
FOLLOW causes
RETURN EDGES
```

Returns all traversed edges.

### 21.7.3 Return PATH

```
WALK FROM #start
FOLLOW causes
RETURN PATH
```

Returns full paths from start to terminals:

```typescript
interface PathResult {
  paths: Path[]
}

interface Path {
  nodes: NodeRef[]    // ordered start to end
  edges: EdgeRef[]    // edges between nodes
}
```

### 21.7.4 Return TERMINAL

```
WALK FROM #start
FOLLOW causes
RETURN TERMINAL
```

Returns only endpoints (nodes with no outgoing edges of the followed type).

### 21.7.5 Return Projections

```
WALK FROM #start
FOLLOW causes
RETURN node.name, node.timestamp, depth
```

Special variables available:
- `node` — current node in traversal
- `edge` — current edge
- `depth` — distance from start
- `path` — path to current node

## 21.8 Traversal Semantics

### 21.8.1 Cycle Handling

Cycles are detected and handled:

```
-- Graph: A → B → C → A (cycle)

WALK FROM #A
FOLLOW edge
RETURN NODES

-- Returns: A, B, C
-- Each node visited once
```

### 21.8.2 Depth-First vs Breadth-First

Default traversal is **breadth-first** (BFS). Use modifier for depth-first:

```
WALK FROM #start [strategy: dfs]
FOLLOW causes
RETURN PATH
```

### 21.8.3 Multiple Paths

When multiple paths exist to a node:

- `RETURN NODES` — node appears once
- `RETURN PATH` — all paths returned
- `RETURN EDGES` — each edge appears once

## 21.9 Complete Examples

```
-- Find all downstream tasks
WALK FROM #task_root
FOLLOW subtask_of INBOUND [depth: 10]
RETURN NODES

-- Find dependency chain to completion
WALK FROM #blocked_task
FOLLOW depends_on
UNTIL node.status = "done"
RETURN PATH

-- Find social network within 3 degrees
WALK FROM { MATCH p: Person WHERE p.email = "alice#example.com" RETURN p }
FOLLOW follows ANY [depth: 3]
RETURN node.name, node.email, depth

-- Multi-hop traversal
WALK FROM #person_123
FOLLOW member_of               -- person → team
FOLLOW owns                    -- team → project  
FOLLOW contains                -- project → task
RETURN terminal.title, terminal.status
```

## 21.10 Result Format

```typescript
interface WalkResult {
  // For RETURN NODES
  nodes?: NodeRef[]
  
  // For RETURN EDGES
  edges?: EdgeRef[]
  
  // For RETURN PATH
  paths?: Path[]
  
  // For RETURN TERMINAL
  terminal?: NodeRef[]
  
  // For projections
  rows?: ResultRow[]
  
  stats: {
    nodesVisited: number
    edgesTraversed: number
    maxDepthReached: number
    executionTime: number
  }
}
```

## 21.11 AST

```typescript
interface WalkStmt {
  kind: "Walk"
  from: StartExpr
  follow: FollowClause[]
  until: Expr | null
  return: WalkReturn
  strategy: "bfs" | "dfs"
}

interface FollowClause {
  edgeTypes: string[] | "*"
  direction: "outbound" | "inbound" | "any"
  minDepth: number
  maxDepth: number
}

type WalkReturn =
  | { kind: "nodes" }
  | { kind: "edges" }
  | { kind: "path" }
  | { kind: "terminal" }
  | { kind: "projections", projections: Projection[] }
```

---

# 22. Observation: INSPECT

## 22.1 Purpose

INSPECT retrieves a specific node or edge by ID. It's direct access without pattern matching.

## 22.2 Syntax

```
InspectStmt = 
  "inspect" IdRef
  ReturnClause?

IdRef = "#" (Identifier | StringLiteral)
```

## 22.2.1 ID Reference Syntax

IDs are referenced using the `#` prefix:

```
#simple_id_123          -- Simple: alphanumeric + underscore
#"550e8400-e29b-41d4"   -- Quoted: any characters (UUIDs, hyphens, etc.)
```

**When to use quoted IDs:**
- UUIDs: `#"550e8400-e29b-41d4-a716-446655440000"`
- IDs with hyphens: `#"task-123-abc"`
- IDs with special characters: `#"node:type:123"`

**Note:** The `@` prefix is reserved for timestamp literals (e.g., `@2024-01-15`).

## 22.3 Examples

```
-- Inspect node
INSPECT #node_abc123

-- Inspect edge
INSPECT #edge_xyz789

-- With projection
INSPECT #task_123
RETURN title, status, priority

-- Full node with type info
INSPECT #person_456
RETURN *, _type, _id
```

## 22.4 Result

Without RETURN clause, returns full node/edge:

```typescript
interface InspectResult {
  found: boolean
  type: "node" | "edge"
  data: {
    _id: string
    _type: string
    [attr: string]: Value
    // For edges:
    _targets?: string[]
  } | null
}
```

## 22.5 Not Found

If ID doesn't exist, returns `found: false`:

```
INSPECT #nonexistent_id
-- { found: false, type: null, data: null }
```

No error is thrown; caller checks `found`.

## 22.6 AST

```typescript
interface InspectStmt {
  kind: "Inspect"
  id: string
  return: Projection[] | null
}
```

---

# 23. Transformation: SPAWN

## 23.1 Purpose

SPAWN creates a new node in the graph.

## 23.2 Syntax

```
SpawnStmt =
  "spawn" Identifier ":" TypeExpr AttrBlock?
  ReturningClause?

AttrBlock = "{" (AttrAssignment ("," AttrAssignment)*)? "}"

AttrAssignment = Identifier "=" Expr

ReturningClause = "returning" ("id" | "*" | Identifier ("," Identifier)*)
```

## 23.3 Examples

```
-- Basic spawn
SPAWN t: Task {
  title = "New task",
  priority = 5
}

-- With all attributes
SPAWN p: Person {
  name = "Alice Smith",
  email = "alice#example.com",
  role = "admin",
  active = true
}

-- Minimal (relies on defaults)
SPAWN t: Task {
  title = "Minimal task"
}

-- Get created ID back
SPAWN t: Task { title = "Test" }
RETURNING id

-- Get full created node
SPAWN p: Person { name = "Bob", email = "bob#example.com" }
RETURNING *
```

## 23.4 Attribute Values

### 23.4.1 Literals

```
SPAWN t: Task {
  title = "Hello",
  priority = 5,
  score = 3.14,
  active = true,
  description = null
}
```

### 23.4.2 Expressions

```
SPAWN t: Task {
  title = "Task " ++ $suffix,
  created_at = now(),
  priority = $basePriority + 1
}
```

### 23.4.3 Defaults

Attributes with defaults in ontology can be omitted:

```
-- Ontology: status: String = "todo"
SPAWN t: Task { title = "Test" }
-- t.status = "todo" (default applied)
```

## 23.5 Validation

Before committing, the engine validates:

1. **Type exists:** The specified type is declared in ontology
2. **Required attributes:** All `[required]` attributes are present
3. **Type checking:** Attribute values match declared types
4. **Constraints:** All attribute modifiers satisfied

```
-- Error: missing required attribute
SPAWN p: Person { name = "Alice" }
-- ERROR: Required attribute 'email' not provided

-- Error: type mismatch
SPAWN t: Task { title = "Test", priority = "high" }
-- ERROR: Type mismatch: priority expects Int, got String

-- Error: constraint violation
SPAWN p: Person { name = "A", email = "invalid" }
-- ERROR: Constraint 'person_email_match' violated
```

## 23.6 RETURNING Clause

Control what's returned after creation:

```
-- Return only ID (default)
SPAWN t: Task { title = "Test" }
RETURNING id
-- { id: "task_abc123" }

-- Return full node
SPAWN t: Task { title = "Test" }
RETURNING *
-- { id: "task_abc123", _type: "Task", title: "Test", status: "todo", ... }

-- Return specific attributes
SPAWN p: Person { name = "Alice", email = "alice#example.com" }
RETURNING id, name, email
-- { id: "person_xyz", name: "Alice", email: "alice#example.com" }
```

## 23.7 Result Format

```typescript
interface SpawnResult {
  success: boolean
  id: string                     // created node ID
  data: Record<string, Value>    // based on RETURNING
  errors?: string[]              // if validation failed
}
```

## 23.8 AST

```typescript
interface SpawnStmt {
  kind: "Spawn"
  variable: string
  type: TypeExpr
  attributes: AttrAssignment[]
  returning: ReturningClause
}

type ReturningClause =
  | { kind: "id" }
  | { kind: "all" }
  | { kind: "fields", fields: string[] }
```

---

# 24. Transformation: KILL

## 24.1 Purpose

KILL removes a node from the graph.

## 24.2 Syntax

```
KillStmt =
  "kill" Target
  CascadeClause?
  ReturningClause?

Target = 
    IdRef                           -- by ID
  | "{" MatchStmt "}"               -- by pattern

CascadeClause = "cascade" | "no" "cascade"
```

## 24.3 Examples

```
-- Kill by ID
KILL #task_123

-- Kill by pattern
KILL { MATCH t: Task WHERE t.status = "archived" RETURN t }

-- With cascade behavior
KILL #project_456 CASCADE

-- Prevent cascade (explicit)
KILL #person_789 NO CASCADE

-- Return killed ID
KILL #task_123
RETURNING id
```

## 24.4 Edge Handling

When a node is killed, connected edges are handled according to their `on_kill_*` modifiers:

| Modifier | Behavior |
|----------|----------|
| `unlink` (default) | Edge is removed |
| `cascade` | Connected node is killed |
| `prevent` | Kill operation fails |

### 24.4.1 Default: Unlink

```
-- Edge: assigned_to(task, person) [on_kill_target: unlink]

KILL #person_123
-- All assigned_to edges pointing to person are removed
-- Tasks remain (just unassigned)
```

### 24.4.2 Cascade

```
-- Edge: belongs_to(task, project) [on_kill_target: cascade]

KILL #project_456
-- All tasks belonging to project are also killed
-- (and their edges, recursively)
```

### 24.4.3 Prevent

```
-- Edge: member_of(person, team) [on_kill_target: prevent]

KILL #team_789
-- ERROR: Cannot kill Team: has 5 member_of edges
-- Operation fails, nothing changes
```

## 24.5 CASCADE Override

The CASCADE clause can override ontology-defined behavior:

```
-- Force cascade even if ontology says unlink
KILL #project_123 CASCADE

-- Prevent cascade even if ontology says cascade
KILL #project_123 NO CASCADE
-- Note: unlinks edges but doesn't cascade
```

**Note:** Cannot override `prevent` — if ontology says prevent, the only option is to first remove blocking edges.

## 24.6 Bulk Kill

Pattern-based kill operates on all matches:

```
-- Kill all archived tasks
KILL { MATCH t: Task WHERE t.status = "archived" RETURN t }
-- Returns count of killed nodes

-- Kill with limit (safety)
KILL { 
  MATCH t: Task WHERE t.created_at < $cutoff RETURN t LIMIT 1000 
}
```

## 24.7 Result Format

```typescript
interface KillResult {
  success: boolean
  killedCount: number
  killedIds: string[]
  cascadeCount: number           // nodes killed via cascade
  unlinkedEdges: number          // edges removed
  errors?: string[]
}
```

## 24.8 AST

```typescript
interface KillStmt {
  kind: "Kill"
  target: IdRef | MatchStmt
  cascade: "default" | "cascade" | "no-cascade"
  returning: ReturningClause | null
}
```

---

# 25. Transformation: LINK

## 25.1 Purpose

LINK creates a new edge connecting nodes.

## 25.2 Syntax

```
LinkStmt =
  "link" EdgeType "(" TargetList ")" AliasClause? AttrBlock?
  ReturningClause?

TargetList = TargetRef ("," TargetRef)*

TargetRef = 
    IdRef                           -- by ID
  | "{" MatchStmt "}"               -- by pattern (must return single)
  
AliasClause = "as" Identifier
```

## 25.3 Examples

```
-- Basic link by ID
LINK causes(#event_123, #event_456)

-- With attributes
LINK assigned_to(#task_123, #person_456) {
  assigned_at = now(),
  role = "owner"
}

-- With alias (for returning)
LINK causes(#e1, #e2) AS c
RETURNING *

-- Using pattern to find target
LINK belongs_to(
  #task_new,
  { MATCH p: Project WHERE p.name = "Main" RETURN p }
)

-- Higher-order edge
LINK confidence(#causes_edge_123, 0.9) {
  assessed_by = "expert"
}
```

## 25.4 Target Resolution

### 25.4.1 By ID

```
LINK causes(#node_a, #node_b)
```

Direct reference to existing nodes.

### 25.4.2 By Pattern

```
LINK assigned_to(
  #task_123,
  { MATCH p: Person WHERE p.email = "alice#example.com" RETURN p }
)
```

Pattern must return exactly one node. Error if zero or multiple.

### 25.4.3 Mixed

```
LINK belongs_to(
  #new_task,
  { MATCH p: Project WHERE p.id = $projectId RETURN p }
)
```

### 25.4.4 Inline SPAWN

Create a node and link it in a single statement:

```
LINK belongs_to(
  SPAWN Task { title = "New task", priority = 5 },
  #project_123
)
RETURNING id
```

**Syntax:**
```
TargetRef = 
    IdRef                           -- by ID
  | "{" MatchStmt "}"               -- by pattern (must return single)
  | SpawnExpr                       -- inline spawn

SpawnExpr = "spawn" TypeExpr AttrBlock? ("as" Identifier)?
```

**Examples:**
```
-- Create task and assign in one statement
LINK assigned_to(
  SPAWN Task { title = "Review PR #123" },
  #person_alice
) AS assignment
RETURNING assignment.id

-- Create both endpoints with AS binding
LINK causes(
  SPAWN Event { name = "User clicked", timestamp = now() } AS cause,
  SPAWN Event { name = "Page loaded", timestamp = now() + 100 } AS effect
) AS causation
RETURNING cause.id AS cause_id, effect.id AS effect_id, causation.id AS edge_id

-- Create node, link to existing project
LINK belongs_to(
  SPAWN Task { title = "Implement feature X" },
  { MATCH p: Project WHERE p.name = "Main" RETURN p }
)
```

### 25.4.5 AS Binding in Inline SPAWN

Use `AS` to bind spawned nodes for use in RETURNING:

```
LINK belongs_to(
  SPAWN Task { title = "New task" } AS t,
  #project_123
) AS e
RETURNING t.id AS task_id, e.id AS edge_id
```

**Scope:** Bindings are available only in the RETURNING clause of the same statement.

**Multiple spawns:**
```
LINK collaboration(
  SPAWN Person { name = "Alice" } AS alice,
  SPAWN Person { name = "Bob" } AS bob
) AS collab
RETURNING alice.id, bob.id, collab.id
```

**Without AS:** If no binding needed, RETURNING can still access the edge:
```
LINK assigned_to(SPAWN Task { title = "Test" }, #person)
RETURNING id  -- edge ID
```

**Semantics:**
- SPAWN expressions are evaluated first (left to right)
- Created nodes are available for the LINK
- If LINK fails (constraint violation), all SPAWNs are rolled back
- RETURNING can access both the edge and spawned node IDs via AS bindings

**Benefit:** Eliminates the need for:
```
-- Old way (two statements)
SPAWN t: Task { title = "New task" } RETURNING id
LINK belongs_to(#t, #project_123)

-- New way (one statement)
LINK belongs_to(SPAWN Task { title = "New task" }, #project_123)
```

## 25.5 Edge Attributes

```
LINK assigned_to(#task, #person) {
  assigned_at = now(),
  role = "reviewer",
  notes = "Urgent review needed"
}
```

Same rules as node attributes:
- Required attributes must be provided
- Defaults apply for omitted attributes
- Type checking enforced

## 25.6 Validation

Before committing:

1. **Edge type exists:** Declared in ontology
2. **Target count:** Matches edge arity
3. **Target types:** Each target matches signature type
4. **Target exists:** All referenced nodes exist
5. **Constraints:** Edge modifiers satisfied (no_self, acyclic, cardinality, etc.)

```
-- Error: self-loop on no_self edge
LINK depends_on(#task_123, #task_123)
-- ERROR: Edge 'depends_on' does not allow self-loops

-- Error: cardinality violation
LINK assigned_to(#task_123, #person_789)
-- If task already has assignee and [task -> 0..1]:
-- ERROR: Cardinality constraint violated: task -> 0..1

-- Error: acyclic violation
LINK parent_of(#person_a, #person_b)
-- If this would create cycle:
-- ERROR: Edge 'parent_of' would create cycle
```

## 25.7 Uniqueness and Duplicate Edges

### 25.7.1 Default: Duplicates Allowed

By default, edges are **distinct entities**. Multiple edges of the same type can connect the same nodes:

```
edge tagged(task: Task, tag: Tag) {}  -- no [unique]

LINK tagged(#task_1, #tag_urgent)     -- creates edge_001
LINK tagged(#task_1, #tag_urgent)     -- creates edge_002 (different ID!)

-- Both edges exist. This may be intentional (e.g., multiple tagging events)
-- or accidental. Use [unique] to prevent if undesired.
```

### 25.7.2 With [unique]: Duplicates Prevented

If edge type has `[unique]` modifier:

```
edge member_of(person: Person, team: Team) [unique] {}

-- First link succeeds
LINK member_of(#person_123, #team_456)

-- Duplicate link fails
LINK member_of(#person_123, #team_456)
-- ERROR: Duplicate edge: member_of(person_123, team_456) already exists
```

### 25.7.3 When to Use [unique]

| Scenario | Use [unique]? | Reason |
|----------|---------------|--------|
| Membership (person → team) | ✅ Yes | Person is either a member or not |
| Assignment (task → person) | ✅ Yes | Task has at most one assignee |
| Tagging (item → tag) | ✅ Usually | Unless tracking multiple tag events |
| Event causation | ❌ No | Multiple causal links may exist |
| Audit trail edges | ❌ No | Each event is distinct |

## 25.8 LINK IF NOT EXISTS

Create an edge only if it doesn't already exist. Idempotent operation.

### 25.8.1 Syntax

```
LinkIfNotExistsStmt = 
  "link" "if" "not" "exists" EdgeType "(" Targets ")" 
  AttrBlock? 
  AsClause?
  ReturningClause?
```

### 25.8.2 Examples

```
-- Create assignment if not already assigned
LINK IF NOT EXISTS assigned_to(#task, #person) { assigned_at = now() }
RETURNING CREATED

-- Result: { created: true, id: "edge_123" } or { created: false, id: "edge_456" }
```

```
-- Idempotent membership
LINK IF NOT EXISTS member_of(#person, #team)

-- Can be called multiple times safely
-- First call: creates edge
-- Subsequent calls: no-op, returns existing edge
```

### 25.8.3 Semantics

| Scenario | Behavior | RETURNING CREATED |
|----------|----------|-------------------|
| Edge doesn't exist | Creates new edge | `true` |
| Edge already exists | No-op, returns existing | `false` |
| Edge type has `[unique]` | Same as without (redundant but allowed) | — |

**Matching criteria:** An edge "exists" if there's an edge of the same type connecting the same targets in the same order (or any order for symmetric edges).

**Attributes on existing edge:** If the edge exists, provided attributes are **ignored** (not updated). Use SET to update.

```
-- Edge exists with assigned_at = yesterday
LINK IF NOT EXISTS assigned_to(#task, #person) { assigned_at = now() }
-- Returns existing edge; assigned_at still = yesterday

-- To update if exists, use separate SET:
LINK IF NOT EXISTS assigned_to(#task, #person) AS a { assigned_at = now() }
SET a.last_checked = now()  -- Always updates
```

### 25.8.4 Use Cases

```
-- Idempotent API endpoint
LINK IF NOT EXISTS follows(#user, #target)
RETURNING id, CREATED AS is_new

-- Ensure relationship (defensive programming)
LINK IF NOT EXISTS belongs_to(#task, #project)

-- Batch import (skip duplicates)
FOR item IN $items:
  LINK IF NOT EXISTS tagged(#item.entity, #item.tag)
```

## 25.9 Result Format

```typescript
interface LinkResult {
  success: boolean
  id: string                     // created edge ID
  data: Record<string, Value>    // based on RETURNING
  errors?: string[]
}
```

## 25.10 AST

```typescript
interface LinkStmt {
  kind: "Link"
  edgeType: string
  targets: TargetRef[]
  alias: string | null
  attributes: AttrAssignment[]
  returning: ReturningClause | null
}
```

---

# 26. Transformation: UNLINK

## 26.1 Purpose

UNLINK removes an edge from the graph.

## 26.2 Syntax

```
UnlinkStmt =
  "unlink" Target
  ReturningClause?

Target =
    IdRef                           -- by edge ID
  | EdgePattern                     -- by pattern
  | "{" MatchStmt "}"               -- by match
```

## 26.3 Examples

```
-- Unlink by edge ID
UNLINK #edge_abc123

-- Unlink by pattern (all matching)
UNLINK assigned_to(#task_123, _)
-- Removes all assignments for task_123

-- Unlink specific edge
UNLINK causes(#event_a, #event_b)

-- Unlink via match
UNLINK {
  MATCH t: Task, p: Person, assigned_to(t, p) AS e
  WHERE p.active = false
  RETURN e
}
```

## 26.4 Edge Identification

### 26.4.1 By ID

```
UNLINK #edge_xyz789
```

Direct reference to specific edge.

### 26.4.2 By Endpoints

```
UNLINK causes(#node_a, #node_b)
```

Removes the edge connecting these specific nodes.

### 26.4.3 By Partial Pattern

```
-- Remove all causes edges from event_a
UNLINK causes(#event_a, _)

-- Remove all edges to this node
UNLINK _(#_, #person_123)
-- ERROR: Must specify edge type

-- Use match for complex patterns
UNLINK {
  MATCH causes(e1, #person_123) AS edge
  RETURN edge
}
```

### 26.4.4 By Match

```
UNLINK {
  MATCH t: Task, assigned_to(t, _) AS e
  WHERE t.status = "done"
  RETURN e
}
-- Removes all assignments from completed tasks
```

## 26.5 Higher-Order Edge Cascade

When unlinking an edge, all higher-order edges referencing it are automatically unlinked:

```
-- Base edge
LINK causes(#e1, #e2) AS c

-- Higher-order edge
LINK confidence(c, 0.9)

-- Unlink base edge
UNLINK #c
-- The confidence edge is automatically unlinked
```

## 26.6 Bulk Unlink

Pattern-based unlink operates on all matches:

```
-- Remove all assignments older than 30 days
UNLINK {
  MATCH assigned_to(_, _) AS e
  WHERE e.assigned_at < now() - 2592000000
  RETURN e
}
RETURNING COUNT(*)
```

## 26.7 Result Format

```typescript
interface UnlinkResult {
  success: boolean
  unlinkedCount: number
  unlinkedIds: string[]
  cascadeCount: number           // higher-order edges removed
  errors?: string[]
}
```

## 26.8 AST

```typescript
interface UnlinkStmt {
  kind: "Unlink"
  target: IdRef | EdgePattern | MatchStmt
  returning: ReturningClause | null
}
```

---

# 27. Transformation: SET

## 27.1 Purpose

SET modifies attribute values on existing nodes or edges.

## 27.2 Syntax

```
SetStmt =
  "set" Target "." Identifier "=" Expr
  ReturningClause?

Target =
    IdRef                           -- single by ID
  | "{" MatchStmt "}"               -- bulk by pattern
```

## 27.3 Examples

```
-- Set single attribute
SET #task_123.status = "done"

-- Set multiple (separate statements)
SET #task_123.status = "done"
SET #task_123.completed_at = now()

-- Set with expression
SET #task_123.priority = #task_123.priority + 1

-- Bulk set
SET { MATCH t: Task WHERE t.status = "in_progress" RETURN t }.status = "blocked"

-- Set on edge
SET #assignment_edge.role = "reviewer"

-- Set to null (clear optional)
SET #task_123.description = null

-- Return updated value
SET #task_123.status = "done"
RETURNING status
```

## 27.4 Multiple Attributes

### 27.4.1 Block Syntax (Recommended)

Set multiple attributes in a single statement using block syntax:

```
SET #task_123 {
  status = "done",
  completed_at = now(),
  completed_by = "user_456"
}
```

**Grammar:**
```
SetStmt =
    "set" Target "." Identifier "=" Expr ReturningClause?     -- single
  | "set" Target AttrBlock ReturningClause?                   -- multiple

AttrBlock = "{" (AttrAssign ("," AttrAssign)*)? "}"
```

**Examples:**
```
-- Update task completion
SET #task_123 {
  status = "done",
  completed_at = now()
}

-- With RETURNING
SET #person_456 {
  name = "Alice Smith",
  email = "alice.smith#example.com"
}
RETURNING name, email

-- Bulk update with block
SET { MATCH t: Task WHERE t.status = "pending" RETURN t } {
  status = "archived",
  archived_at = now()
}
```

### 27.4.2 Separate Statements

Alternatively, use multiple SET statements in a transaction:

```
BEGIN
  SET #task_123.status = "done"
  SET #task_123.completed_at = now()
  SET #task_123.completed_by = "user_456"
COMMIT
```

**Note:** Block syntax is preferred as it's more concise and clearly shows all changes together.

## 27.5 Validation

Before committing:

1. **Node/edge exists:** Target must exist
2. **Attribute exists:** Declared on the type
3. **Type checking:** Value matches attribute type
4. **Constraints:** All attribute modifiers satisfied

```
-- Error: attribute doesn't exist
SET #task_123.nonexistent = "value"
-- ERROR: Attribute 'nonexistent' not found on type 'Task'

-- Error: type mismatch
SET #task_123.priority = "high"
-- ERROR: Type mismatch: priority expects Int, got String

-- Error: constraint violation
SET #person_123.age = -5
-- ERROR: Constraint 'person_age_min' violated: age must be >= 0

-- Error: required to null
SET #person_123.email = null
-- ERROR: Cannot set required attribute 'email' to null
```

## 27.6 Bulk SET

```
-- Set all matching
SET { MATCH t: Task WHERE t.project_id = #old_project RETURN t }.project_id = #new_project

-- With limit (safety)
SET { 
  MATCH t: Task WHERE t.status = "pending" RETURN t LIMIT 100 
}.status = "archived"
```

## 27.7 Computed Values

SET can use expressions referencing current values:

```
-- Increment
SET #counter_node.value = #counter_node.value + 1

-- Conditional (requires full expression support)
SET #task_123.priority = MIN(#task_123.priority + 1, 10)

-- String manipulation
SET #person_123.name = UPPER(#person_123.name)
```

## 27.8 Result Format

```typescript
interface SetResult {
  success: boolean
  modifiedCount: number
  modifiedIds: string[]
  errors?: string[]
}
```

## 27.9 AST

```typescript
interface SetStmt {
  kind: "Set"
  target: IdRef | MatchStmt
  attribute: string
  value: Expr
  returning: ReturningClause | null
}
```

---

# 27.10 Parameterized Queries

## 27.10.1 Overview

Parameterized queries allow safe, reusable queries with external values. Parameters prevent injection attacks and enable query plan caching.

## 27.10.2 Syntax

```
Parameter = "$" Identifier

-- In queries:
MATCH t: Task WHERE t.id = $taskId RETURN t
SPAWN t: Task { title = $title, priority = $priority }
LINK assigned_to(#task_123, $personId)
```

## 27.10.3 Parameter Binding

Parameters are bound when executing the query:

```typescript
// API usage (pseudocode)
engine.execute(
  "MATCH t: Task WHERE t.status = $status RETURN t",
  { status: "done" }
)

engine.execute(
  "SPAWN t: Task { title = $title, priority = $priority } RETURNING t.id",
  { title: "New task", priority: 5 }
)
```

## 27.10.4 Type Checking

Parameters are type-checked against their usage context:

```
-- $priority inferred as Int from Task.priority
MATCH t: Task WHERE t.priority > $priority RETURN t

-- $name inferred as String from Person.name
SPAWN p: Person { name = $name }

-- Type error if wrong type provided at execution:
-- ERROR: Parameter $priority expected Int, got String
```

## 27.10.5 Parameter Scope

Parameters are:
- **Immutable:** Cannot be modified within a query
- **Query-scoped:** Available throughout the entire query
- **Not nullable by default:** Use `$param?` for optional parameters

```
-- Optional parameter with default
MATCH t: Task 
WHERE t.status = COALESCE($status, "pending")
RETURN t

-- Or explicit optional syntax
MATCH t: Task
WHERE $filter? = null OR t.category = $filter
RETURN t
```

## 27.10.6 Prepared Statements

For frequently executed queries, use prepared statements:

```
-- Prepare (parsed and planned once)
PREPARE find_tasks AS
  MATCH t: Task WHERE t.status = $status RETURN t

-- Execute (reuses plan)
EXECUTE find_tasks WITH status = "done"
EXECUTE find_tasks WITH status = "pending"

-- Drop when no longer needed
DROP PREPARED find_tasks
```

**Benefits:**
- Query parsed once, executed many times
- Plan caching for performance
- Type checking at prepare time

## 27.10.7 Parameter in Bulk Operations

```
KILL { MATCH t: Task WHERE t.project_id = $projectId RETURN t }
WITH projectId = "proj_123"

SET { MATCH t: Task WHERE t.assignee = $oldAssignee RETURN t }.assignee = $newAssignee
WITH oldAssignee = "user_old", newAssignee = "user_new"
```

## 27.10.8 Security

Parameters are **always** treated as values, never as identifiers or operators:

```
-- SAFE: $value is always a value
MATCH t: Task WHERE t.title = $value RETURN t
-- Even if $value contains "'; DROP TABLE tasks; --"

-- NOT POSSIBLE: Cannot parameterize type names or attributes
MATCH t: $type RETURN t        -- ERROR: Type cannot be parameterized
MATCH t: Task RETURN t.$attr   -- ERROR: Attribute cannot be parameterized
```

---

# 28. Transaction Control

## 28.1 Purpose

Transactions group multiple operations into an atomic unit. Either all succeed or all fail.

## 28.2 Syntax

```
TransactionStmt =
    BeginStmt
  | CommitStmt
  | RollbackStmt

BeginStmt = "begin" IsolationLevel?

IsolationLevel = "read" "committed" | "serializable"

CommitStmt = "commit"

RollbackStmt = "rollback"
```

## 28.3 Basic Usage

```
BEGIN
  SPAWN p: Project { name = "New Project" }
  SPAWN t: Task { title = "First task" }
  LINK belongs_to(#t, #p)
COMMIT
```

If any statement fails, everything rolls back.

## 28.4 Auto-Commit Mode

Without explicit BEGIN, each statement is its own transaction:

```
-- Auto-commit: each is separate transaction
SPAWN t: Task { title = "Task 1" }  -- commits immediately
SPAWN t: Task { title = "Task 2" }  -- commits immediately
```

## 28.5 Explicit Transactions

```
BEGIN
  -- Multiple operations
  SPAWN t: Task { title = "Test" }
  SET #existing_task.status = "done"
  LINK depends_on(#t, #other_task)
  -- Nothing committed yet
COMMIT  -- All committed atomically
```

## 28.6 Rollback

### 28.6.1 Explicit Rollback

```
BEGIN
  SPAWN t: Task { title = "Test" }
  -- Changed my mind
ROLLBACK
-- Node not created
```

### 28.6.2 Automatic Rollback

If any operation fails, automatic rollback:

```
BEGIN
  SPAWN t: Task { title = "Test" }       -- succeeds tentatively
  SET #nonexistent.status = "done"       -- fails
COMMIT
-- Automatic rollback: Task not created
-- ERROR: Node #nonexistent not found
```

## 28.7 Isolation Levels

### 28.7.1 READ COMMITTED (default)

```
BEGIN READ COMMITTED
  -- Sees committed data from other transactions
  -- No dirty reads
COMMIT
```

### 28.7.2 SERIALIZABLE

```
BEGIN SERIALIZABLE
  -- Full isolation
  -- Transactions appear to execute sequentially
COMMIT
```

## 28.8 Constraint Checking

Constraints are checked at COMMIT time (for cardinality) or operation time (for others):

| Constraint Type | When Checked |
|-----------------|--------------|
| Type validation | At operation |
| Required | At operation |
| Value constraints | At operation |
| Unique | At operation |
| no_self, acyclic | At operation |
| Cardinality | At COMMIT |

```
BEGIN
  SPAWN t: Task { title = "Test" }
  -- Cardinality [task -> 1] on belongs_to not yet checked
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(#t, #p)
  -- Now cardinality satisfied
COMMIT  -- Cardinality verified here
```

## 28.9 Rule Execution

Rules execute within the transaction:

```
BEGIN
  SPAWN t: Task { title = "Test", status = "done" }
  -- Rule 'auto_complete_timestamp' fires
  -- SET t.completed_at = now() happens within transaction
COMMIT
-- t has completed_at set
```

## 28.10 Nested Transactions (Savepoints)

```
BEGIN
  SPAWN p: Project { name = "Main" }
  
  SAVEPOINT sp1
    SPAWN t: Task { title = "Subtask" }
    -- Something goes wrong
  ROLLBACK TO sp1
  -- Task not created, Project still pending
  
  SPAWN t: Task { title = "Different task" }
COMMIT
-- Only Project and "Different task" committed
```

## 28.11 Result Format

```typescript
interface TransactionResult {
  action: "begin" | "commit" | "rollback"
  success: boolean
  transactionId: string | null
  errors?: string[]
}
```

## 28.12 AST

```typescript
interface BeginStmt {
  kind: "Begin"
  isolation: "read-committed" | "serializable"
}

interface CommitStmt {
  kind: "Commit"
}

interface RollbackStmt {
  kind: "Rollback"
  savepoint: string | null
}

interface SavepointStmt {
  kind: "Savepoint"
  name: string
}

interface RollbackToStmt {
  kind: "RollbackTo"
  savepoint: string
}
```

---

# 29. Administration

## 29.1 Overview

Administration statements manage schema, indexes, and engine state.

```
AdminStmt =
    LoadStmt
  | ExtendStmt
  | ShowStmt
  | IndexStmt
  | DropIndexStmt
```

## 29.2 LOAD ONTOLOGY

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

## 29.3 EXTEND ONTOLOGY

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

## 29.4 SHOW Statements

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

## 29.5 INDEX Management

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

## 29.6 Result Format

```typescript
interface AdminResult {
  success: boolean
  action: string
  data?: any
  errors?: string[]
}
```

## 29.7 AST

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

# 30. Versioning

## 30.1 Overview

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

## 30.2 SNAPSHOT

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

## 30.3 SHOW VERSIONS

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

## 30.4 CHECKOUT

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

-- Mutations are blocked
SPAWN t: Task { title = "Test" }
-- ERROR: Cannot mutate in readonly checkout mode

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

## 30.5 DIFF

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

## 30.6 Branching

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

## 30.7 MERGE

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

## 30.8 AST

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

# 31. Query Control and Safety

## 31.1 TIMEOUT Clause

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

## 31.2 Safety Limits and Warnings

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

## 31.3 DRY RUN Mode

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

# 32. Debug

## 31.1 Overview

Debug statements help understand query execution and performance.

```
DebugStmt =
    ExplainStmt
  | ProfileStmt
```

## 31.2 EXPLAIN

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

## 31.3 PROFILE

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

## 31.4 Optimization Hints

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

## 31.5 AST

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

# 33. Complete Grammar (HOHG Language)

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

(* Patterns - shared with Ontology DSL *)
Pattern          = PatternElem ("," PatternElem)* ("where" Expr)?
PatternElem      = NodePattern | EdgePattern
NodePattern      = Identifier ":" TypeExpr
EdgePattern      = Identifier TransMod? "(" Targets ")" ("as" Identifier)? DepthMod?
TransMod         = "+" | "*"
Targets          = Target ("," Target)*
Target           = Identifier | "_"
DepthMod         = "[" "depth:" IntLiteral "]"

(* Types - shared with Ontology DSL *)
TypeExpr         = UnionType
UnionType        = OptionalType ("|" OptionalType)*
OptionalType     = PrimaryType "?"?
PrimaryType      = QualIdent | EdgeRefType | "any" | ScalarType | "(" TypeExpr ")"
EdgeRefType      = "edge" "<" (QualIdent | "any") ">"
ScalarType       = "String" | "Int" | "Float" | "Bool" | "Timestamp" | "ID"

(* Expressions - shared with Ontology DSL *)
Expr             = OrExpr
OrExpr           = AndExpr ("or" AndExpr)*
AndExpr          = EqualityExpr ("and" EqualityExpr)*
EqualityExpr     = CompareExpr (("=" | "!=") CompareExpr)*
CompareExpr      = AddExpr (("<" | ">" | "<=" | ">=") AddExpr)*
AddExpr          = MulExpr (("+" | "-" | "++") MulExpr)*
MulExpr          = UnaryExpr (("*" | "/" | "%") UnaryExpr)*
UnaryExpr        = ("-" | "not")? PostfixExpr
PostfixExpr      = PrimaryExpr ("." Identifier | "(" (Expr ("," Expr)*)? ")")*
PrimaryExpr      = Literal | Identifier | IdRef | AggregateExpr | ExistsExpr 
                 | IfExpr | CaseExpr | CoalesceExpr | "(" Expr ")"
AggregateExpr    = ("count" | "sum" | "avg" | "min" | "max" | "collect") "(" "distinct"? Expr ")"
ExistsExpr       = "not"? "exists" "(" Pattern ")"
IfExpr           = "if" Expr "then" Expr "else" Expr
CaseExpr         = "case" Expr? WhenClause+ ElseClause? "end"
WhenClause       = "when" Expr "then" Expr
ElseClause       = "else" Expr
CoalesceExpr     = "coalesce" "(" Expr ("," Expr)+ ")" | Expr "??" Expr

(* Literals *)
Literal          = StringLiteral | IntLiteral | FloatLiteral | BoolLiteral | NullLiteral
Identifier       = [a-zA-Z_][a-zA-Z0-9_]*
QualIdent        = Identifier ("::" Identifier)*
```

---

# 34. Quick Reference

## 34.1 Observation

```
-- Pattern matching
MATCH pattern WHERE condition RETURN projection ORDER BY expr LIMIT n

-- Path traversal
WALK FROM start FOLLOW edge [depth: n] UNTIL condition RETURN nodes|edges|path|terminal

-- Direct access
INSPECT #id RETURN attrs
```

## 34.2 Transformation

```
-- Create node
SPAWN var: Type { attr = value } RETURNING id|*

-- Remove node
KILL #id|{match} CASCADE|NO CASCADE

-- Create edge (with optional inline spawn)
LINK EdgeType(target, target) AS alias { attr = value }
LINK EdgeType(SPAWN Type {...}, #existing) AS alias

-- Remove edge
UNLINK #id|pattern|{match}

-- Modify attribute (single or multiple)
SET #id.attr = value
SET #id { attr1 = value1, attr2 = value2 }
SET {match}.attr = value
```

## 34.3 Transaction

```
BEGIN [READ COMMITTED | SERIALIZABLE]
  -- operations
COMMIT | ROLLBACK

SAVEPOINT name
ROLLBACK TO name
```

## 34.4 Administration

```
LOAD ONTOLOGY FROM "file" | { source }
EXTEND ONTOLOGY name { declarations }

SHOW TYPES | EDGES | CONSTRAINTS | RULES | INDEXES | STATISTICS | STATUS
SHOW TYPE name | EDGE name | ...

CREATE INDEX name ON Type(attr)
DROP INDEX name
```

## 34.5 Versioning

```
SNAPSHOT "label"
SHOW VERSIONS
CHECKOUT "version" READONLY | RESTORE
DIFF "v1" "v2" DETAILED?

CREATE BRANCH "name" FROM "version"?
SWITCH BRANCH "name"
SHOW BRANCHES
DELETE BRANCH "name"
MERGE BRANCH "source" INTO "target"?
```

## 34.6 Debug & Safety

```
EXPLAIN statement
PROFILE statement
DRY RUN transformation_statement

-- Query timeout
MATCH ... RETURN ... TIMEOUT 5s

-- Safety overrides
KILL { MATCH ... } FORCE
CHECKOUT "version" RESTORE CONFIRM
```

---

# 35. Summary

## 35.1 Part III Contents

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
| 34. Quick Reference | Cheat sheet |

## 35.2 Design Principles Applied

| Principle | How Applied |
|-----------|-------------|
| Graph-native | Vocabulary: SPAWN/KILL, LINK/UNLINK, WALK |
| Observation vs Transformation | Clear separation of read/write operations |
| Explicit | All behavior declared, no hidden defaults |
| Composable | Match patterns reused across operations |
| Inspectable | SHOW, EXPLAIN, PROFILE for transparency |

## 35.3 Complete Language Map

```
┌─────────────────────────────────────────────────────────────────────┐
│                     HOHG LANGUAGE FAMILY                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ONTOLOGY DSL (Part II)              HOHG LANGUAGE (Part III)       │
│  ═══════════════════════             ════════════════════════        │
│  Compiled, defines schema            Interpreted, operates on graph │
│                                                                      │
│  • node declarations                 OBSERVATION                     │
│  • edge declarations                 • MATCH (pattern)              │
│  • constraint declarations           • WALK (traversal)             │
│  • rule declarations                 • INSPECT (direct)             │
│  • type aliases                                                      │
│                                      TRANSFORMATION                  │
│  Modifiers (sugar → constraints):    • SPAWN (create node)          │
│  • [required, unique]                • KILL (remove node)           │
│  • [>= N, <= M, in:[...]]            • LINK (create edge)           │
│  • [no_self, acyclic]                • UNLINK (remove edge)         │
│  • [task -> 0..1]                    • SET (modify attr)            │
│  • [on_kill: cascade]                                                │
│                                      CONTROL                         │
│                                      • BEGIN/COMMIT/ROLLBACK        │
│                                      • SAVEPOINT                     │
│                                                                      │
│                                      ADMIN                           │
│                                      • LOAD/EXTEND ONTOLOGY         │
│                                      • SHOW                          │
│                                      • CREATE/DROP INDEX            │
│                                                                      │
│                                      VERSION                         │
│                                      • SNAPSHOT/CHECKOUT            │
│                                      • DIFF/MERGE                   │
│                                      • BRANCH                        │
│                                                                      │
│                                      DEBUG                           │
│                                      • EXPLAIN/PROFILE              │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                     SHARED CONSTRUCTS (Part I)                       │
│  • Lexical structure    • Scalar types    • Type expressions        │
│  • Patterns             • Expressions     • Operators               │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of Part III: HOHG Language (Runtime)*

---

# Appendix A: Reserved Keywords (Complete)

All reserved keywords across both languages:

```
abstract    acyclic     and         any         as          asc
auto        begin       bool        branch      by          cascade
checkout    collect     commit      constraint  count       create
delete      depth       desc        detailed    diff        distinct
drop        edge        exists      explain     extend      false
float       follow      from        hard        id          in
inbound     index       indexed     inspect     int         into
kill        length      limit       link        load        manual
match       max         merge       message     min         no
node        not         null        offset      on          ontology
or          order       outbound    path        prevent     priority
profile     readonly    required    restore     return      returning
rollback    rule        savepoint   sealed      self        serializable
set         show        snapshot    soft        spawn       statistics
status      string      sum         switch      symmetric   terminal
timestamp   to          true        type        types       unique
unlink      until       using       versions    walk        where
```

---

# Appendix B: Operator Precedence (Complete)

From highest to lowest:

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `.` `()` | Left |
| 2 | unary `-` `not` | Right |
| 3 | `*` `/` `%` | Left |
| 4 | `+` `-` `++` | Left |
| 5 | `<` `>` `<=` `>=` | Left |
| 6 | `=` `!=` | Left |
| 7 | `and` | Left |
| 8 | `or` | Left |

---

# Appendix C: Built-in Functions (Complete)

## String Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `length(s)` | String → Int | Character count |
| `lower(s)` | String → String | Lower case |
| `upper(s)` | String → String | Upper case |
| `trim(s)` | String → String | Remove whitespace |
| `contains(s, sub)` | String × String → Bool | Substring test |
| `starts_with(s, pre)` | String × String → Bool | Prefix test |
| `ends_with(s, suf)` | String × String → Bool | Suffix test |
| `substring(s, start, len)` | String × Int × Int → String | Extract |
| `replace(s, old, new)` | String × String × String → String | Replace |
| `split(s, delim)` | String × String → String[] | Split |
| `matches(s, pattern)` | String × String → Bool | Regex match |

## Numeric Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs(n)` | Number → Number | Absolute value |
| `min(a, b)` | Number × Number → Number | Minimum |
| `max(a, b)` | Number × Number → Number | Maximum |
| `floor(f)` | Float → Int | Round down |
| `ceil(f)` | Float → Int | Round up |
| `round(f)` | Float → Int | Round nearest |

## Timestamp Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `now()` | () → Timestamp | Current time |
| `year(t)` | Timestamp → Int | Extract year |
| `month(t)` | Timestamp → Int | Extract month |
| `day(t)` | Timestamp → Int | Extract day |
| `hour(t)` | Timestamp → Int | Extract hour |
| `minute(t)` | Timestamp → Int | Extract minute |
| `second(t)` | Timestamp → Int | Extract second |

## General Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `coalesce(a, b)` | T? × T → T | First non-null |
| `is_null(x)` | T? → Bool | Null test |
| `type_of(x)` | Any → String | Type name |

## Aggregate Functions (MATCH only)

| Function | Signature | Description |
|----------|-----------|-------------|
| `count(x)` | Any → Int | Count matches |
| `count(distinct x)` | Any → Int | Count unique |
| `sum(x)` | Number → Number | Sum |
| `avg(x)` | Number → Float | Average |
| `min(x)` | Comparable → Same | Minimum |
| `max(x)` | Comparable → Same | Maximum |
| `collect(x)` | Any → List | Collect to list |

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

  Query result limit reached
  
  At: MATCH t: Task RETURN t
  
  Returned: 10,000 rows (limit)
  Total matching: 1,234,567 rows
  
  Hint: Add WHERE clause or LIMIT to reduce results.
```

---

*End of HOHG Language Specification*