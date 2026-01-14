# MEW Language Specification

## Part IV: Query Operations

**Version:** 1.0
**Status:** Draft
**Scope:** Read-only observation operations

---

# 1. GQL Overview

## 1.1 Purpose

The MEW Language is the runtime interface to the graph. It provides:

| Category | Operations | Purpose |
|----------|------------|---------|
| **Observation** | MATCH, WALK, INSPECT | Read without changing |
| **Transformation** | SPAWN, KILL, LINK, UNLINK, SET | Change the graph |
| **Transaction** | BEGIN, COMMIT, ROLLBACK | Group operations atomically |
| **Administration** | LOAD, EXTEND, SHOW, INDEX | Manage schema and engine |
| **Versioning** | SNAPSHOT, CHECKOUT, DIFF | Time travel and branching |
| **Debug** | EXPLAIN, PROFILE | Understand performance |

## 1.2 Execution Model

MEW Language statements are **interpreted** against a running engine:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Statement   │────▶│   Engine     │────▶│   Result     │
│  (text)      │     │   (runtime)  │     │   (data)     │
└──────────────┘     └──────────────┘     └──────────────┘
```

Unlike the Ontology DSL (compiled once), GQL statements execute immediately.

## 1.3 Statement Structure

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

## 1.4 Result Types

| Statement Type | Returns |
|----------------|---------|
| Observation | Stream of matches/nodes/edges |
| Transformation | Affected IDs + count |
| Transaction | Success/failure |
| Admin | Status/metadata |
| Version | Version info |
| Debug | Execution plan/statistics |

---

# 2. Observation: MATCH

## 2.1 Purpose

MATCH finds all subgraphs matching a pattern. It is the primary read operation.

## 2.2 Syntax

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

**Note:** The RETURN clause is **required** for query MATCH statements. MATCH without RETURN or mutation is a compile error:

```
-- INVALID: Missing RETURN and no mutation
MATCH t: Task WHERE t.priority > 5
-- ERROR: MATCH statement requires RETURN clause or mutation operation.
--        Did you mean: MATCH t: Task WHERE t.priority > 5 RETURN t

-- VALID: Query with RETURN
MATCH t: Task WHERE t.priority > 5
RETURN t

-- VALID: Compound statement with mutation
MATCH t: Task WHERE t.priority > 5
SET t.reviewed = true
```

### 20.2.1 MATCH in Different Contexts

MATCH behavior varies by context:

| Context | RETURN Required? | Purpose |
|---------|------------------|---------|
| **Query Statement** | ✅ Yes | Specifies what to return to caller |
| **Compound Statement** | ❌ No | Followed by LINK/SET/KILL/UNLINK mutations |
| **Subquery** (in KILL, SET, etc.) | ✅ Yes | Specifies what to operate on |
| **EXISTS pattern** | ❌ No | EXISTS uses Pattern, not MATCH |

**Examples:**
```
-- Query Statement: RETURN required
MATCH t: Task WHERE t.status = "done"
RETURN t

-- Compound Statement: Mutation instead of RETURN
MATCH t: Task WHERE t.status = "done"
SET t.archived = true

-- Subquery MATCH: RETURN specifies targets
KILL { MATCH t: Task WHERE t.archived RETURN t }
SET { MATCH t: Task WHERE t.old RETURN t }.status = "archived"

-- EXISTS: Uses Pattern syntax, not MATCH
WHERE EXISTS(p: Person, assigned_to(t, p))
-- Note: This is a Pattern, not a MATCH statement
```

## 2.3 Basic Examples

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

## 2.4 Pattern Matching

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

## 2.4.6 OPTIONAL MATCH

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

## 2.5 WHERE Clause

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

**Unlike SQL, MEW allows aggregate functions directly in WHERE clauses:**

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

-- MEW allows inline:
MATCH t: Task
WHERE COUNT(p: Person, assigned_to(t, p)) > 2
RETURN t
```

**Performance note:** Aggregates in WHERE may require per-row computation. For large result sets, consider restructuring with explicit aggregation and filtering.

## 2.6 RETURN Clause

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

## 2.7 ORDER BY Clause

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

## 2.8 LIMIT and OFFSET

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

## 2.9 Complete Examples

```
-- Complex observation with all clauses
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

-- Path observation
MATCH start: Person, end: Person, follows+(start, end) [depth: 3]
WHERE start.name = "Alice"
RETURN end.name, end.email
```

## 2.10 Result Format

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

## 2.11 Compound MATCH Statements

### 2.11.1 Purpose

Compound MATCH statements allow mutations to be performed on all nodes that match a pattern, combining pattern matching with mutation operations in a single statement.

### 2.11.2 Syntax

```
CompoundMatchStmt =
  "match" Pattern
  ("where" Expr)?
  MutationOp+

MutationOp =
    "spawn" SpawnExpr
  | "link" EdgeExpr
  | "unlink" EdgeExpr
  | "set" AttributeAssignment
  | "kill" Identifier
```

### 2.11.3 Semantics

1. The MATCH pattern is evaluated to find all matching bindings
2. Each mutation operation is applied to every matching binding
3. All mutations execute within the current transaction context
4. No RETURN clause is allowed (compound statements don't return results)

### 2.11.4 Examples

```
-- Update all high-priority tasks
MATCH t: Task WHERE t.priority > 8
SET t.reviewed = true

-- Archive completed tasks and link them to an archive
MATCH t: Task WHERE t.status = "done"
SET t.archived_at = now()
LINK archived_in(t, $archive_id)

-- Delete all expired sessions
MATCH s: Session WHERE s.expires < now()
KILL s

-- Multiple mutations on same matches
MATCH p: Person WHERE p.inactive_days > 90
SET p.status = "suspended"
UNLINK member_of(p, *)
```

### 2.11.5 Error Handling

If any mutation fails (e.g., type mismatch, constraint violation), the entire compound statement fails and the transaction should be rolled back if in a transaction context.

### 2.11.6 Difference from Subquery Mutations

Compound MATCH statements differ from subquery-based mutations:

```
-- Compound statement (new syntax)
MATCH t: Task WHERE t.done
SET t.archived = true

-- Subquery mutation (existing syntax)
SET { MATCH t: Task WHERE t.done RETURN t }.archived = true
```

The compound statement syntax is more concise and clearly expresses the intent of "find and mutate."

## 2.12 AST

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

interface MatchMutateStmt {
  kind: "MatchMutate"
  pattern: Pattern
  where: Expr | null
  mutations: MutationOp[]
}

interface MutationOp {
  kind: "Spawn" | "Link" | "Unlink" | "Set" | "Kill"
  // Specific fields depend on mutation type
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

# 3. Observation: WALK

## 3.1 Purpose

WALK performs path traversal from a starting point, following edges according to specified rules. Unlike MATCH (which finds patterns), WALK navigates structure.

## 3.2 Syntax

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

## 3.3 Basic Examples

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

## 3.4 Starting Points

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

## 3.5 FOLLOW Clause

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

## 3.6 UNTIL Clause

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

## 3.7 RETURN Options

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

## 3.8 Traversal Semantics

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

## 3.9 Complete Examples

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

## 3.10 Result Format

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

## 3.11 AST

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

# 4. Observation: INSPECT

## 4.1 Purpose

INSPECT retrieves a specific node or edge by ID. It's direct access without pattern matching.

## 4.2 Syntax

```
InspectStmt = 
  "inspect" IdRef
  ReturnClause?

IdRef = "#" (Identifier | StringLiteral)
```

## 4.2.1 ID Reference Syntax

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

## 4.3 Examples

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

## 4.4 Result

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

## 4.5 Not Found

If ID doesn't exist, returns `found: false`:

```
INSPECT #nonexistent_id
-- { found: false, type: null, data: null }
```

No error is thrown; caller checks `found`.

## 4.6 AST

```typescript
interface InspectStmt {
  kind: "Inspect"
  id: string
  return: Projection[] | null
}
```

---

# 5. Debug

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

