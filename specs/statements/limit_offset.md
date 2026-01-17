---
spec: limit_offset
version: "1.0"
status: draft
category: statement
capability: query
requires: [match]
priority: common
---

# Spec: LIMIT and OFFSET

## Overview

LIMIT restricts the number of rows returned by a query. OFFSET skips a specified number of rows before returning results. Together, they enable pagination and top-N queries essential for handling large result sets efficiently.

## Syntax

### Grammar

```ebnf
LimitClause  = "limit" IntLiteral OffsetClause?
             | OffsetClause

OffsetClause = "offset" IntLiteral
```

### Keywords

| Keyword | Context |
|---------|---------|
| `limit` | Clause - maximum number of rows to return |
| `offset` | Clause - number of rows to skip |

### Examples

```
-- Limit results to 10
MATCH t: Task
RETURN t
LIMIT 10

-- Pagination: page 3 with 20 items per page
MATCH t: Task
RETURN t
ORDER BY t.created_at DESC
LIMIT 20 OFFSET 40

-- First result only
MATCH t: Task
ORDER BY t.priority DESC
RETURN t
LIMIT 1

-- Skip first 5, take next 10
MATCH t: Task
RETURN t
LIMIT 10 OFFSET 5
```

## Semantics

### LIMIT

LIMIT specifies the maximum number of rows to return:

```
MATCH t: Task
RETURN t
LIMIT 10
```

- Returns at most 10 rows
- If fewer than 10 matches exist, returns all matches
- LIMIT 0 returns no rows (but query is still executed)

### OFFSET

OFFSET skips a number of rows before returning results:

```
MATCH t: Task
RETURN t
LIMIT 10 OFFSET 20
```

- Skips the first 20 rows
- Then returns up to 10 rows
- If fewer than 20 rows exist, returns no rows

### Evaluation Order

LIMIT and OFFSET are applied after:
1. Pattern matching (MATCH)
2. Filtering (WHERE)
3. Projection (RETURN)
4. Sorting (ORDER BY)

```
MATCH t: Task
WHERE t.status = "open"
RETURN t.title, t.priority
ORDER BY t.priority DESC
LIMIT 10 OFFSET 5
```

Execution order:
1. Find all Task nodes
2. Filter to status = "open"
3. Project title and priority
4. Sort by priority descending
5. Skip first 5 rows
6. Return next 10 rows

### Without ORDER BY

LIMIT and OFFSET without ORDER BY return arbitrary rows:

```
-- Results are non-deterministic without ORDER BY
MATCH t: Task
RETURN t
LIMIT 10
```

For reproducible pagination, always use ORDER BY with LIMIT/OFFSET.

### Pagination Pattern

Standard pagination formula:

```
-- Page N (1-indexed) with PAGE_SIZE items
LIMIT PAGE_SIZE OFFSET (N - 1) * PAGE_SIZE
```

Examples:
```
-- Page 1, 20 items per page
LIMIT 20 OFFSET 0

-- Page 2, 20 items per page
LIMIT 20 OFFSET 20

-- Page 3, 20 items per page
LIMIT 20 OFFSET 40
```

### Top-N Queries

Get the first N results after sorting:

```
-- Highest priority task
MATCH t: Task
ORDER BY t.priority DESC
RETURN t
LIMIT 1

-- Top 5 most recent events
MATCH e: Event
ORDER BY e.timestamp DESC
RETURN e
LIMIT 5

-- Bottom 3 by score
MATCH p: Player
ORDER BY p.score ASC
RETURN p
LIMIT 3
```

### OFFSET without LIMIT

OFFSET can be used without LIMIT to skip initial rows:

```
-- Skip first 10, return rest
MATCH t: Task
RETURN t
OFFSET 10
```

This returns all rows after the first 10.

### Performance Considerations

- LIMIT enables early termination - query can stop after finding enough matches
- Large OFFSET values require scanning and discarding rows - can be expensive
- For large datasets, consider cursor-based pagination instead of OFFSET

```
-- Efficient: LIMIT stops early
MATCH t: Task
ORDER BY t.created_at DESC
RETURN t
LIMIT 10

-- Less efficient: must scan 10000 rows before returning
MATCH t: Task
ORDER BY t.created_at DESC
RETURN t
LIMIT 10 OFFSET 10000
```

### Integer Requirements

Both LIMIT and OFFSET require non-negative integer literals:

```
-- Valid
LIMIT 10
LIMIT 0
LIMIT 10 OFFSET 20
OFFSET 0

-- Invalid
LIMIT -1          -- negative not allowed
LIMIT 10.5        -- must be integer
LIMIT t.count     -- must be literal, not expression
OFFSET -5         -- negative not allowed
```

## Layer 0

None.

## Examples

```
-- Simple pagination with deterministic order
MATCH t: Task
WHERE t.status = "open"
RETURN t.title, t.priority, t.created_at
ORDER BY t.created_at DESC
LIMIT 25 OFFSET 50

-- First item only (top-1)
MATCH t: Task
ORDER BY t.priority DESC
RETURN t
LIMIT 1

-- Skip header rows equivalent
MATCH r: Record
ORDER BY r.row_number ASC
RETURN r
LIMIT 100 OFFSET 1

-- Complex query with all clauses
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person) AS a
WHERE t.status != "done"
  AND p.status = "active"
RETURN t.title AS task,
       p.name AS project,
       person.name AS assignee,
       a.assigned_at AS since
ORDER BY t.priority DESC, a.assigned_at ASC
LIMIT 50 OFFSET 100

-- Aggregation with limit
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name AS project, COUNT(t) AS task_count
ORDER BY task_count DESC
LIMIT 10

-- Path query with limit
MATCH start: Person, end: Person, follows+(start, end) [depth: 3]
WHERE start.name = "Alice"
RETURN end.name, end.email
ORDER BY end.name
LIMIT 20
```

## Errors

| Condition | Message |
|-----------|---------|
| Negative LIMIT | LIMIT must be a non-negative integer |
| Negative OFFSET | OFFSET must be a non-negative integer |
| Non-integer LIMIT | LIMIT requires an integer literal |
| Non-integer OFFSET | OFFSET requires an integer literal |
| LIMIT/OFFSET without RETURN | LIMIT and OFFSET require RETURN clause |
| Expression in LIMIT | LIMIT requires a literal value, not an expression |
