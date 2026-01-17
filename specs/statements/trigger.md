---
spec: trigger
version: "1.0"
status: draft
category: statement
capability: rules
requires: [rule, pattern]
priority: common
---

# Spec: Trigger Statement

## Overview

The TRIGGER statement explicitly fires a manual rule. Manual rules (declared with `[manual]` modifier) do not fire automatically during transactions; they must be triggered explicitly. This enables batch operations, maintenance tasks, and on-demand transformations that should not run on every transaction.

## Syntax

### Grammar

```ebnf
TriggerStmt = "TRIGGER" Identifier WhereClause?

WhereClause = "WHERE" Condition
```

### Keywords

| Keyword | Context |
|---------|---------|
| `TRIGGER` | Statement |
| `WHERE` | Clause (optional filter) |

### Examples

```
-- Trigger a rule on all matching bindings
TRIGGER archive_old_tasks

-- Trigger with filter
TRIGGER archive_old_tasks WHERE t.project_id = $project_id

-- Trigger cleanup rule
TRIGGER remove_orphaned_nodes

-- Trigger migration rule
TRIGGER migrate_legacy_format WHERE t.version < 2
```

## Semantics

### Execution

1. The named rule is looked up in the registry
2. The rule's pattern is matched against the current graph state
3. If a WHERE clause is provided, it further filters the bindings
4. For each matching binding, the rule's production is executed
5. The rule fires to quiescence (same as auto rules)

### Behavior

| Aspect | Description |
|--------|-------------|
| Target | Must be a declared rule (manual or auto) |
| Filtering | WHERE clause restricts which bindings fire |
| Quiescence | Rule fires repeatedly until no new matches |
| Transaction | Executes within current transaction |
| Once-per-binding | Same binding fires at most once |

### Manual vs Auto Rules

- **Manual rules** (`[manual]`): Only fire when explicitly triggered
- **Auto rules** (default): Can also be triggered explicitly

TRIGGER works on both, but is primarily designed for manual rules.

### WHERE Clause

The WHERE clause references variables from the rule's pattern:

```
rule archive_tasks [manual]:
  t: Task WHERE t.completed = true AND t.archived = false
  => SET t.archived = true
```

```
-- Trigger only for a specific project
TRIGGER archive_tasks WHERE t.project_id = "proj-123"
```

The WHERE condition is combined with the rule's pattern condition using AND.

### Return Value

TRIGGER returns the count of bindings that fired:

```
TRIGGER cleanup_orphans
-- Returns: { fired: 42 }
```

## Layer 0

### Nodes

None.

### Edges

None.

### Constraints

None.

## Examples

### Batch Archival

```
-- Rule definition
rule archive_completed_tasks [manual]:
  t: Task
  WHERE t.status = "done"
    AND t.completed_at < now() - 30.days
    AND t.archived = false
  => SET t.archived = true

-- Trigger for all matching tasks
TRIGGER archive_completed_tasks

-- Trigger for specific project
TRIGGER archive_completed_tasks WHERE t.project_id = $project
```

### Data Migration

```
-- Rule to migrate old format
rule migrate_v1_to_v2 [manual]:
  d: Document WHERE d.schema_version = 1
  =>
  SET d.schema_version = 2,
  SET d.metadata = parse_legacy_metadata(d.raw_data)

-- Run migration
TRIGGER migrate_v1_to_v2
```

### Scheduled Cleanup

```
-- Rule for orphan removal
rule cleanup_orphan_attachments [manual]:
  a: Attachment
  WHERE NOT EXISTS(d: Document, has_attachment(d, a))
  => KILL a

-- Run cleanup (typically scheduled)
TRIGGER cleanup_orphan_attachments
```

### Parameterized Triggering

```
-- Rule for recalculating derived data
rule recalculate_project_stats [manual]:
  p: Project
  =>
  SET p.task_count = COUNT(t: Task, belongs_to(t, p)),
  SET p.completed_count = COUNT(t: Task, belongs_to(t, p) WHERE t.status = "done")

-- Recalculate for specific project
TRIGGER recalculate_project_stats WHERE p._id = $project_id

-- Recalculate for all projects
TRIGGER recalculate_project_stats
```

## Errors

| Condition | Message |
|-----------|---------|
| Unknown rule | Rule `{name}` not found |
| Invalid WHERE variable | Variable `{var}` not defined in rule pattern |
| WHERE type mismatch | Cannot compare `{type1}` with `{type2}` |
| Max actions exceeded | Transaction exceeded maximum action limit |
| Max depth exceeded | Rule chain exceeded maximum depth limit |
