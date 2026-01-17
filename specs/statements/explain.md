---
spec: explain
version: "1.0"
status: draft
category: statement
capability: debug
requires: []
priority: convenience
---

# Spec: EXPLAIN and PROFILE

## Overview

Debug statements help understand query execution and performance. EXPLAIN shows the execution plan for a statement without executing it, revealing how the query optimizer plans to retrieve data. PROFILE executes the statement and returns actual performance metrics alongside results, enabling performance analysis and optimization.

## Syntax

### Grammar

```ebnf
DebugStmt    = ExplainStmt | ProfileStmt

ExplainStmt  = "explain" Statement

ProfileStmt  = "profile" Statement
```

### Keywords

| Keyword | Context |
|---------|---------|
| `explain` | Statement - shows execution plan without running |
| `profile` | Statement - executes and shows performance metrics |

### Examples

```
-- Show execution plan
EXPLAIN MATCH t: Task WHERE t.status = "done" RETURN t

-- Show execution plan for complex query
EXPLAIN MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name

-- Profile actual execution
PROFILE MATCH t: Task WHERE t.priority > 5 RETURN t

-- Profile with full metrics
PROFILE MATCH t: Task, p: Person, assigned_to(t, p)
RETURN t.title, p.name
```

## Semantics

### EXPLAIN

EXPLAIN analyzes a statement and outputs its execution plan without executing it. The plan shows:
- Operations to be performed
- Estimated row counts at each step
- Indexes to be used
- Join strategies

**Output format:**

```
EXPLAIN MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name

-- Result:
Query Plan:
+-- Scan: Task (estimated: 8,234 rows)
|   +-- Filter: status = "done" (estimated: 2,000 rows)
+-- Edge Lookup: belongs_to (from task)
|   +-- Index: belongs_to_task_idx
+-- Node Fetch: Project
+-- Project: t.title, p.name

Estimated cost: 2,450
Indexes used: belongs_to_task_idx
```

### Plan Components

| Component | Description |
|-----------|-------------|
| Scan | Full table scan of type |
| Index Scan | Scan using index |
| Filter | Apply WHERE condition |
| Edge Lookup | Find edges from/to node |
| Node Fetch | Retrieve node by ID |
| Join | Combine multiple sources |
| Project | Select output columns |
| Sort | ORDER BY operation |
| Limit | LIMIT/OFFSET operation |
| Aggregate | COUNT, SUM, etc. |

### PROFILE

PROFILE executes the statement and returns actual performance metrics:

```
PROFILE MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done"
RETURN t.title, p.name

-- Result:
Execution Profile:
+-- Scan: Task
|   +-- Rows scanned: 8,234
|   +-- Rows passed: 2,156
|   +-- Time: 12ms
|   +-- Filter: status = "done"
+-- Edge Lookup: belongs_to
|   +-- Lookups: 2,156
|   +-- Edges found: 2,156
|   +-- Time: 3ms
|   +-- Index: belongs_to_task_idx (hit rate: 100%)
+-- Node Fetch: Project
|   +-- Fetches: 234 (deduplicated)
|   +-- Time: 1ms
+-- Project
    +-- Time: <1ms

Total time: 16ms
Rows returned: 2,156
Memory used: 4.2 MB
```

### Profile Metrics

| Metric | Description |
|--------|-------------|
| Rows scanned | Total rows examined |
| Rows passed | Rows passing filter |
| Time | Execution time for step |
| Index hit rate | Cache/index effectiveness |
| Memory used | Peak memory consumption |
| Disk reads | I/O operations (if applicable) |

### Optimization Hints

Based on EXPLAIN/PROFILE results, common optimizations:

**Add missing index:**
```
-- EXPLAIN shows full scan on status
CREATE INDEX task_status ON Task(status)
```

**Rewrite query to use index:**
```
-- Before (full scan, no index on title):
MATCH t: Task WHERE contains(t.title, "urgent") RETURN t

-- After (uses indexed field):
MATCH t: Task WHERE t.priority >= 8 RETURN t
```

**Add LIMIT for large result sets:**
```
-- PROFILE shows 50,000 rows returned
MATCH t: Task WHERE t.status = "archived" RETURN t LIMIT 100
```

## Layer 0

None. EXPLAIN and PROFILE are runtime debug operations with no graph representation.

## Examples

### Analyzing a Complex Query

```
EXPLAIN MATCH t: Task, p: Project, person: Person,
      belongs_to(t, p),
      assigned_to(t, person) AS a
WHERE t.status != "done"
  AND p.status = "active"
  AND person.active = true
RETURN t.title, p.name, person.name

-- Result:
Query Plan:
+-- Scan: Task (estimated: 8,234 rows)
|   +-- Filter: status != "done" (estimated: 6,500 rows)
+-- Edge Lookup: belongs_to (from task)
|   +-- Index: belongs_to_task_idx
+-- Node Fetch: Project
|   +-- Filter: status = "active" (estimated: 4,000 rows)
+-- Edge Lookup: assigned_to (from task)
|   +-- Index: assigned_to_task_idx
+-- Node Fetch: Person
|   +-- Filter: active = true (estimated: 3,500 rows)
+-- Project: t.title, p.name, person.name

Estimated cost: 8,750
Indexes used: belongs_to_task_idx, assigned_to_task_idx
```

### Profiling Aggregation Query

```
PROFILE MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count
ORDER BY task_count DESC
LIMIT 10

-- Result:
Execution Profile:
+-- Scan: Task
|   +-- Rows scanned: 8,234
|   +-- Time: 8ms
+-- Edge Lookup: belongs_to
|   +-- Time: 5ms
+-- Node Fetch: Project
|   +-- Time: 2ms
+-- Aggregate: GROUP BY p.name, COUNT
|   +-- Groups: 234
|   +-- Time: 3ms
+-- Sort: task_count DESC
|   +-- Time: 1ms
+-- Limit: 10
|   +-- Time: <1ms

Total time: 19ms
Rows returned: 10
Memory used: 1.8 MB
```

### Comparing Query Variations

```
-- Version 1: Filter on indexed attribute
PROFILE MATCH t: Task
WHERE t.priority > 8
RETURN t
-- Total time: 5ms, Rows: 150

-- Version 2: Filter on non-indexed attribute
PROFILE MATCH t: Task
WHERE t.category = "urgent"
RETURN t
-- Total time: 25ms, Rows: 150

-- Version 1 is faster due to index usage
```

## Errors

| Condition | Message |
|-----------|---------|
| Invalid statement | Cannot explain/profile: syntax error in statement |
| Statement causes side effects in EXPLAIN | EXPLAIN does not execute transformations |
