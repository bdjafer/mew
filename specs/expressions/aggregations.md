---
spec: aggregations
version: "1.0"
status: draft
category: expression
capability: aggregation
requires: [literals, types, match]
priority: essential
---

# Spec: Aggregations

## Overview

Aggregation functions compute summary values over sets of matched data. They enable counting, summing, averaging, and collecting results from pattern matches. Aggregations can appear in RETURN clauses for result computation and directly in WHERE clauses for filtering (a MEW extension beyond standard SQL).

## Syntax

### Grammar

```ebnf
AggregateExpr     = AggregateFunc "(" AggregateArg ")" CollectLimit?

AggregateFunc     = "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT"

AggregateArg      = "DISTINCT"? Expr
                  | "*"
                  | Pattern                    (* for WHERE clause aggregates *)

CollectLimit      = "[" "limit" ":" (IntLiteral | "none") "]"

(* In WHERE clause *)
AggregateInWhere  = AggregateFunc "(" Pattern ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `COUNT` | Expression - count matches |
| `SUM` | Expression - sum numeric values |
| `AVG` | Expression - average numeric values |
| `MIN` | Expression - minimum value |
| `MAX` | Expression - maximum value |
| `COLLECT` | Expression - collect into list |
| `DISTINCT` | Modifier - unique values only |

### Examples

```
-- Count all tasks
MATCH t: Task RETURN COUNT(t)

-- Sum priorities
MATCH t: Task RETURN SUM(t.priority)

-- Average with grouping
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, AVG(t.priority)

-- Collect into list with limit
MATCH t: Task RETURN COLLECT(t) [limit: 100]
```

## Semantics

### Aggregation Functions

| Function | Description | Input | Output |
|----------|-------------|-------|--------|
| `COUNT(x)` | Count matches | Any | Int |
| `COUNT(DISTINCT x)` | Count unique values | Any | Int |
| `SUM(x)` | Sum values | Int/Float | Same |
| `AVG(x)` | Average | Int/Float | Float |
| `MIN(x)` | Minimum | Comparable | Same |
| `MAX(x)` | Maximum | Comparable | Same |
| `COLLECT(x)` | Collect into list | Any | List |

### Grouping Behavior

When mixing aggregations with non-aggregated values in RETURN, non-aggregated values become grouping keys:

```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t), AVG(t.priority)
--     ^^^^^^
--     grouping key
```

Returns one row per unique `p.name`.

### DISTINCT Modifier

```
-- Count unique statuses
MATCH t: Task
RETURN COUNT(DISTINCT t.status)

-- Count unique assignees per project
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p), assigned_to(t, person)
RETURN p.name, COUNT(DISTINCT person)
```

### COLLECT Limits

COLLECT has a configurable size limit to prevent memory issues:

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

### Aggregates in WHERE Clause

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

-- Find people following more than they're followed by
MATCH p: Person
WHERE COUNT(f: Person, follows(p, f)) > COUNT(f: Person, follows(f, p))
RETURN p
```

**Semantics for WHERE aggregates:**
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

### NULL Handling

- `COUNT(*)` counts all rows including those with NULL values
- `COUNT(x)` excludes NULL values
- `SUM`, `AVG`, `MIN`, `MAX` ignore NULL values
- `COLLECT` includes NULL values in the list

## Layer 0

### Nodes

None.

### Edges

None.

### Constraints

None.

Aggregations are computed at query execution time and do not create graph structures.

## Examples

### Basic Aggregations

```
-- Count all tasks
MATCH t: Task
RETURN COUNT(t)

-- Count completed tasks
MATCH t: Task
WHERE t.status = "done"
RETURN COUNT(t) AS completed

-- Multiple aggregations
MATCH t: Task
RETURN COUNT(t) AS total,
       MIN(t.priority) AS min_prio,
       MAX(t.priority) AS max_prio,
       AVG(t.priority) AS avg_prio
```

### Grouped Aggregations

```
-- Tasks per project
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count

-- Tasks per status
MATCH t: Task
RETURN t.status, COUNT(t) AS count
ORDER BY count DESC

-- Aggregation with grouping
MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.created_at > now() - 604800000  -- last 7 days
RETURN p.name AS project,
       COUNT(t) AS total,
       COUNT(t WHERE t.status = "done") AS completed
ORDER BY total DESC
LIMIT 10
```

### COLLECT Examples

```
-- Get first 10 tasks per project
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COLLECT(t) [limit: 10] AS sample_tasks

-- Get all critical tasks (when you know it's bounded)
MATCH t: Task WHERE t.status = "critical"
RETURN COLLECT(t) [limit: none] AS critical_tasks

-- Collect specific attributes
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COLLECT(t.title) AS task_titles
```

### Aggregates in WHERE

```
-- Projects with more than 10 tasks
MATCH p: Project
WHERE COUNT(t: Task, belongs_to(t, p)) > 10
RETURN p

-- Unassigned tasks
MATCH t: Task
WHERE COUNT(p: Person, assigned_to(t, p)) = 0
RETURN t

-- Highly connected nodes
MATCH n: any
WHERE COUNT(e: any, causes(n, e)) > 5
RETURN n
```

### Aggregation with OPTIONAL MATCH

```
-- Projects with task counts (including 0)
MATCH p: Project
OPTIONAL MATCH t: Task, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count
-- Projects with 0 tasks return task_count = 0
```

## Errors

| Condition | Message |
|-----------|---------|
| Non-numeric SUM/AVG | `SUM/AVG requires numeric type, got Type` |
| Non-comparable MIN/MAX | `MIN/MAX requires comparable type, got Type` |
| COLLECT limit exceeded | `COLLECT exceeded size limit` |
| Invalid aggregate context | `Aggregate function not allowed in this context` |
