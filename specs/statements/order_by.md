---
spec: order_by
version: "1.0"
status: draft
category: statement
capability: query
requires: [match, expression]
priority: common
---

# Spec: ORDER BY

## Overview

ORDER BY sorts the result set of a query based on one or more expressions. It allows ascending or descending ordering and supports expressions, aliases, and multiple sort keys for complex ordering requirements.

## Syntax

### Grammar

```ebnf
OrderClause = "order" "by" OrderTerm ("," OrderTerm)*

OrderTerm = Expr ("asc" | "desc")?
```

### Keywords

| Keyword | Context |
|---------|---------|
| `order` | Clause - begins ordering specification |
| `by` | Clause - separates `order` from sort terms |
| `asc` | Modifier - ascending order (default) |
| `desc` | Modifier - descending order |

### Examples

```
-- Single order, descending
MATCH t: Task
RETURN t
ORDER BY t.priority DESC

-- Multiple sort keys
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

## Semantics

### Default Direction

When no direction is specified, the default is ASC (ascending):

```
MATCH t: Task
RETURN t
ORDER BY t.priority
-- equivalent to: ORDER BY t.priority ASC
```

### Sort Direction

- **ASC (ascending):** smallest to largest, A to Z, earliest to latest
- **DESC (descending):** largest to smallest, Z to A, latest to earliest

```
-- Numbers: 1, 2, 3, 4, 5
ORDER BY t.priority ASC

-- Numbers: 5, 4, 3, 2, 1
ORDER BY t.priority DESC

-- Strings: "apple", "banana", "cherry"
ORDER BY t.name ASC

-- Timestamps: oldest first
ORDER BY t.created_at ASC

-- Timestamps: newest first
ORDER BY t.created_at DESC
```

### Multiple Sort Keys

Multiple ORDER BY terms are evaluated left to right. Secondary keys break ties from primary keys:

```
MATCH t: Task
RETURN t
ORDER BY t.priority DESC, t.created_at ASC
```

1. First, sort by priority in descending order (highest priority first)
2. For tasks with the same priority, sort by created_at in ascending order (oldest first)

### Ordering by Expressions

Any valid expression can be used in ORDER BY:

```
-- Order by computed value
MATCH t: Task
RETURN t
ORDER BY t.priority * t.urgency DESC

-- Order by function result
MATCH p: Person
RETURN p
ORDER BY length(p.name)

-- Order by string transformation
MATCH p: Person
RETURN p
ORDER BY lower(p.last_name), lower(p.first_name)
```

### Ordering by Alias

Aliases defined in RETURN can be used in ORDER BY:

```
MATCH t: Task
RETURN t.title AS name, t.priority * 10 AS score
ORDER BY score DESC

-- Multiple aliases
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name AS project, COUNT(t) AS task_count
ORDER BY task_count DESC, project ASC
```

### NULL Handling

NULL values are sorted last in ASC order and first in DESC order (implementation-defined, may be configurable):

```
-- NULLs appear at the end
ORDER BY t.due_date ASC

-- NULLs appear at the beginning
ORDER BY t.due_date DESC
```

### Type Ordering

Values are ordered according to their type:

| Type | Ordering |
|------|----------|
| Int/Float | Numeric comparison |
| String | Lexicographic (Unicode) |
| Boolean | false < true |
| Timestamp | Chronological |
| NULL | Last (ASC) or First (DESC) |

Cross-type comparisons follow a defined type precedence (error if incompatible types).

### ORDER BY with Aggregations

ORDER BY works with aggregated results:

```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count, AVG(t.priority) AS avg_priority
ORDER BY task_count DESC, avg_priority DESC

-- Order by aggregate without alias
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t)
ORDER BY COUNT(t) DESC
```

## Layer 0

None.

## Examples

```
-- Top 10 highest priority tasks
MATCH t: Task
WHERE t.status != "done"
RETURN t.title, t.priority
ORDER BY t.priority DESC
LIMIT 10

-- Recent activity sorted by timestamp
MATCH e: Event
WHERE e.timestamp > now() - 86400000
RETURN e.type, e.description, e.timestamp
ORDER BY e.timestamp DESC

-- Alphabetical listing with secondary sort
MATCH p: Person
RETURN p.last_name, p.first_name, p.email
ORDER BY p.last_name ASC, p.first_name ASC

-- Complex ordering with expressions and aggregates
MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person) AS a
WHERE p.status = "active"
RETURN p.name AS project,
       person.name AS assignee,
       COUNT(t) AS task_count,
       AVG(t.priority) AS avg_priority
ORDER BY avg_priority DESC, task_count DESC, project ASC

-- Order by edge attribute
MATCH e1: Event, e2: Event, causes(e1, e2) AS c
RETURN e1.name, e2.name, c.strength
ORDER BY c.strength DESC

-- Order by date with NULL handling
MATCH t: Task
RETURN t.title, t.due_date
ORDER BY t.due_date ASC
-- Tasks without due dates appear at the end
```

## Errors

| Condition | Message |
|-----------|---------|
| ORDER BY with no RETURN | ORDER BY requires RETURN clause |
| Invalid expression in ORDER BY | Cannot order by expression of type `T` |
| Unknown alias in ORDER BY | Alias `x` is not defined in RETURN clause |
| ORDER BY on non-comparable type | Cannot compare values of type `T` |
