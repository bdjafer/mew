---
spec: match
version: "1.0"
status: draft
category: statement
capability: query
requires: [pattern, expression, return]
priority: essential
---

# Spec: MATCH

## Overview

MATCH finds all subgraphs matching a pattern. It is the primary read operation in MEW, enabling graph traversal and data retrieval through declarative pattern matching against the graph structure.

## Syntax

### Grammar

```ebnf
MatchStmt =
  "match" Pattern
  OptionalMatchClause*
  WhereClause?
  ReturnClause
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

OptionalMatchClause = "optional" "match" Pattern
```

### Keywords

| Keyword | Context |
|---------|---------|
| `match` | Statement - begins pattern matching |
| `where` | Clause - filters matched results |
| `return` | Clause - specifies output projection |
| `as` | Projection - aliases expressions |
| `*` | Projection - returns all bound variables |

### Examples

```
-- Find all events
MATCH e: Event
RETURN e

-- Find events with condition
MATCH e: Event
WHERE e.timestamp > 1000
RETURN e

-- Find causal pairs with edge binding
MATCH e1: Event, e2: Event, causes(e1, e2) AS c
WHERE c.strength > 0.5
RETURN e1.name, e2.name, c.strength

-- Return all bound variables
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN *
```

## Semantics

### RETURN Clause Requirement

The RETURN clause is **required** for query MATCH statements. MATCH without RETURN or mutation is a compile error:

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

### MATCH in Different Contexts

MATCH behavior varies by context:

| Context | RETURN Required? | Purpose |
|---------|------------------|---------|
| **Query Statement** | Yes | Specifies what to return to caller |
| **Compound Statement** | No | Followed by LINK/SET/KILL/UNLINK mutations |
| **Subquery** (in KILL, SET, etc.) | Yes | Specifies what to operate on |
| **EXISTS pattern** | No | EXISTS uses Pattern, not MATCH |

### Pattern Matching

#### Node Patterns

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

#### Edge Patterns

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

#### Higher-Order Patterns

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

#### Transitive Patterns

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

#### Negative Patterns

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

### WHERE Clause

Filters matched results based on expressions:

```
-- Basic condition
MATCH t: Task
WHERE t.priority > 5
RETURN t

-- Compound conditions
MATCH t: Task
WHERE t.status = "done" AND t.priority >= 8
RETURN t

-- Attribute comparisons
MATCH e1: Event, e2: Event, causes(e1, e2)
WHERE e1.timestamp < e2.timestamp
RETURN e1, e2

-- Null checks
MATCH t: Task
WHERE t.description != null
RETURN t
```

#### Aggregates in WHERE

Unlike SQL, MEW allows aggregate functions directly in WHERE clauses:

```
-- Find tasks with more than 2 assignees
MATCH t: Task
WHERE COUNT(p: Person, assigned_to(t, p)) > 2
RETURN t

-- Find projects with no tasks
MATCH p: Project
WHERE COUNT(t: Task, belongs_to(t, p)) = 0
RETURN p
```

### Projections

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
```

### Grouping

When mixing aggregations with non-aggregated values, non-aggregated values become grouping keys:

```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t), AVG(t.priority)
--     ^^^^^^
--     grouping key
```

Returns one row per unique `p.name`.

### Result Format

MATCH returns a stream of result rows:

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
  [column: string]: Value     // column name -> value
}

type Value =
  | string | number | boolean | null
  | Timestamp
  | NodeRef | EdgeRef
  | Value[]                   // for COLLECT
```

## Layer 0

None.

## Examples

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

-- Subquery MATCH: RETURN specifies targets
KILL { MATCH t: Task WHERE t.archived RETURN t }
SET { MATCH t: Task WHERE t.old RETURN t }.status = "archived"
```

## Compound MATCH Statements

Compound MATCH statements allow mutations to be performed on all nodes that match a pattern:

```ebnf
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

### Examples

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

## Errors

| Condition | Message |
|-----------|---------|
| Missing RETURN and no mutation | MATCH statement requires RETURN clause or mutation operation |
| Invalid pattern syntax | Expected pattern in MATCH statement |
| Unbound variable in projection | Variable `x` is not bound in MATCH pattern |
| Type mismatch in WHERE | Cannot compare `T1` with `T2` |
