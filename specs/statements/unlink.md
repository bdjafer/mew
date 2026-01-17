---
spec: unlink
version: "1.0"
status: draft
category: statement
capability: mutation
requires: [id_ref, edge_pattern, match_stmt, returning_clause]
priority: essential
---

# Spec: UNLINK

## Overview

UNLINK removes an edge from the graph. It supports multiple identification methods (by edge ID, by endpoints, by pattern, or by match), handles higher-order edge cascade automatically, and can operate on single or bulk edges.

## Syntax

### Grammar

```ebnf
UnlinkStmt      = "unlink" Target ReturningClause? ;

Target          = IdRef
                | EdgePattern
                | "{" MatchStmt "}" ;

EdgePattern     = EdgeType "(" TargetRef "," TargetRef ")" ;

TargetRef       = IdRef | "_" ;

ReturningClause = "returning" ("id" | "*" | "count" "(" "*" ")" | Identifier ("," Identifier)*) ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `unlink` | Statement - initiates edge removal |
| `_` | Wildcard - matches any node in pattern |
| `returning` | Clause - specifies what to return |

### Examples

```
-- Unlink by edge ID
UNLINK #edge_abc123

-- Unlink by endpoints
UNLINK causes(#event_a, #event_b)

-- Unlink by partial pattern (all matching)
UNLINK assigned_to(#task_123, _)

-- Unlink via match
UNLINK {
  MATCH t: Task, p: Person, assigned_to(t, p) AS e
  WHERE p.active = false
  RETURN e
}
```

## Semantics

### Edge Identification

**By ID:**
```
UNLINK #edge_xyz789
```
Direct reference to a specific edge by its ID.

**By Endpoints:**
```
UNLINK causes(#node_a, #node_b)
```
Removes the edge connecting these specific nodes. If multiple edges of the same type exist between the nodes (when not `[unique]`), all are removed.

**By Partial Pattern:**
```
-- Remove all causes edges from event_a
UNLINK causes(#event_a, _)

-- Remove all assignments for a task
UNLINK assigned_to(#task_123, _)
```

The wildcard `_` matches any node. Must specify the edge type.

**By Match:**
```
UNLINK {
  MATCH t: Task, assigned_to(t, _) AS e
  WHERE t.status = "done"
  RETURN e
}
-- Removes all assignments from completed tasks
```

### Higher-Order Edge Cascade

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

This cascade is automatic and cannot be prevented - higher-order edges cannot exist without their referenced base edge.

### Bulk Unlink

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

### Result Format

```typescript
interface UnlinkResult {
  success: boolean
  unlinkedCount: number
  unlinkedIds: string[]
  cascadeCount: number           // higher-order edges removed
  errors?: string[]
}
```

## Layer 0

None. UNLINK operates on user-defined edge types declared in the ontology.

## Examples

### Remove Single Assignment

```
UNLINK assigned_to(#task_123, #person_456)
RETURNING id
```

### Clear All Assignments for a Task

```
UNLINK assigned_to(#task_123, _)
```

### Bulk Cleanup of Stale Edges

```
-- Remove all old assignments
UNLINK {
  MATCH assigned_to(_, _) AS e
  WHERE e.assigned_at < now() - 2592000000
  RETURN e
}
RETURNING COUNT(*)
```

### Remove Edges Based on Node Condition

```
-- Remove assignments from inactive people
UNLINK {
  MATCH t: Task, p: Person, assigned_to(t, p) AS e
  WHERE p.active = false
  RETURN e
}
```

### Higher-Order Edge Cleanup

```
-- When base edge is unlinked, higher-order edges cascade
LINK causes(#event_1, #event_2) AS c1
LINK confidence(#c1, 0.8) AS conf
LINK annotated(#c1, "review needed") AS ann

UNLINK #c1
-- Both confidence and annotated edges are automatically removed
```

### Unlink in Transaction

```
BEGIN
  -- Reassign task: remove old, add new
  UNLINK assigned_to(#task_123, _)
  LINK assigned_to(#task_123, #new_person) {
    assigned_at = now()
  }
COMMIT
```

## Errors

| Condition | Message |
|-----------|---------|
| Edge not found | `Edge 'X' not found` |
| Must specify edge type | `Must specify edge type for pattern unlink` |
| Pattern returns non-edge | `UNLINK pattern must return edges` |
