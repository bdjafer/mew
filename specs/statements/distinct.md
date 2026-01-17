---
spec: distinct
version: "1.0"
status: draft
category: statement
capability: query
requires: [match, return]
priority: common
---

# Spec: DISTINCT

## Overview

DISTINCT removes duplicate rows from the result set. It can be applied to the entire RETURN clause to deduplicate rows, or used within aggregate functions like COUNT(DISTINCT x) to count unique values.

## Syntax

### Grammar

```ebnf
ReturnClause = "return" "distinct"? Projection ("," Projection)*

CountDistinct = "count" "(" "distinct" Expr ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `distinct` | Modifier - removes duplicate rows or values |

### Examples

```
-- Remove duplicate rows
MATCH t: Task, p: Person, assigned_to(t, p)
RETURN DISTINCT p

-- Distinct on specific projection
MATCH t: Task
RETURN DISTINCT t.status

-- Count distinct values
MATCH t: Task
RETURN COUNT(DISTINCT t.status) AS unique_statuses

-- Distinct with multiple columns
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN DISTINCT t.status, p.name
```

## Semantics

### RETURN DISTINCT

RETURN DISTINCT removes duplicate rows from the result set:

```
MATCH t: Task, p: Person, assigned_to(t, p)
RETURN DISTINCT p
```

- Evaluates the projection for all matches
- Compares complete rows for equality
- Removes rows that are duplicates of earlier rows
- Preserves order of first occurrence

### Row Equality

Two rows are considered equal if all projected values are equal:

```
MATCH t: Task
RETURN DISTINCT t.status, t.priority
```

| t.status | t.priority | Kept? |
|----------|------------|-------|
| "open" | 5 | Yes |
| "open" | 5 | No (duplicate) |
| "open" | 8 | Yes (different priority) |
| "done" | 5 | Yes (different status) |

### DISTINCT on Single Value

When projecting a single value, DISTINCT returns unique values:

```
-- Get all unique statuses
MATCH t: Task
RETURN DISTINCT t.status
```

If tasks have statuses ["open", "open", "done", "open", "done"], returns ["open", "done"].

### DISTINCT on Multiple Values

With multiple projections, the combination must be unique:

```
MATCH t: Task
RETURN DISTINCT t.status, t.priority
```

Returns unique (status, priority) pairs.

### DISTINCT with Nodes/Edges

DISTINCT works with node and edge references:

```
-- Get unique persons who are assigned to any task
MATCH t: Task, p: Person, assigned_to(t, p)
RETURN DISTINCT p
```

Two node references are equal if they refer to the same node (same ID).

### COUNT(DISTINCT)

COUNT(DISTINCT expr) counts unique values of an expression:

```
-- Count unique statuses
MATCH t: Task
RETURN COUNT(DISTINCT t.status) AS unique_statuses

-- Count distinct assignees per project
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person)
RETURN p.name, COUNT(DISTINCT person) AS unique_assignees
```

### DISTINCT vs Regular COUNT

```
-- COUNT includes duplicates
MATCH t: Task
RETURN COUNT(t.status)  -- counts all tasks

-- COUNT DISTINCT excludes duplicates
MATCH t: Task
RETURN COUNT(DISTINCT t.status)  -- counts unique status values
```

Example with 5 tasks having statuses ["open", "open", "done", "open", "done"]:
- COUNT(t.status) = 5
- COUNT(DISTINCT t.status) = 2

### NULL Handling

- DISTINCT treats NULL as a value - multiple NULLs are deduplicated to one
- COUNT(DISTINCT) excludes NULL values from the count

```
-- Statuses: ["open", NULL, "done", NULL, "open"]

MATCH t: Task
RETURN DISTINCT t.status
-- Returns: ["open", NULL, "done"]

MATCH t: Task
RETURN COUNT(DISTINCT t.status)
-- Returns: 2 (excludes NULLs)
```

### DISTINCT with ORDER BY

DISTINCT is applied before ORDER BY:

```
MATCH t: Task
RETURN DISTINCT t.status
ORDER BY t.status
```

1. Find all tasks
2. Project status values
3. Remove duplicates (DISTINCT)
4. Sort (ORDER BY)

### DISTINCT with Aggregations

DISTINCT can appear alongside aggregations:

```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN DISTINCT p.name, COUNT(t) AS task_count
```

The DISTINCT applies to the entire row (p.name, task_count), not just p.name.

For distinct values within an aggregation, use COUNT(DISTINCT):

```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(DISTINCT t.status) AS unique_statuses
```

## Layer 0

None.

## Examples

```
-- Get all unique values of a field
MATCH t: Task
RETURN DISTINCT t.status

-- Get unique entities through relationships
MATCH t: Task, p: Person, assigned_to(t, p)
RETURN DISTINCT p.name, p.email

-- Count distinct values
MATCH t: Task
RETURN COUNT(DISTINCT t.priority) AS priority_levels

-- Distinct per group
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person)
RETURN p.name AS project,
       COUNT(t) AS total_tasks,
       COUNT(DISTINCT person) AS unique_assignees,
       COUNT(DISTINCT t.status) AS unique_statuses

-- Distinct with ordering and limit
MATCH e: Event
RETURN DISTINCT e.type
ORDER BY e.type
LIMIT 10

-- Unique combinations
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN DISTINCT t.status, p.status

-- Distinct nodes
MATCH p: Person, t1: Task, t2: Task,
      assigned_to(t1, p),
      assigned_to(t2, p)
WHERE t1 != t2
RETURN DISTINCT p

-- Distinct with expression
MATCH t: Task
RETURN DISTINCT lower(t.category) AS category

-- Complex distinct with optional match
MATCH p: Project
OPTIONAL MATCH t: Task, belongs_to(t, p)
RETURN DISTINCT p.name, t.status
-- Returns all project/status combinations including NULL
```

## Errors

| Condition | Message |
|-----------|---------|
| DISTINCT without RETURN | DISTINCT must be used with RETURN clause |
| COUNT(DISTINCT *) | COUNT(DISTINCT) requires a specific expression |
| DISTINCT in non-projection context | DISTINCT can only appear in RETURN or COUNT |
