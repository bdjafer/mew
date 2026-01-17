---
spec: kill
version: "1.0"
status: draft
category: statement
capability: mutation
requires: [id_ref, match_stmt, returning_clause]
priority: essential
---

# Spec: KILL

## Overview

KILL removes a node from the graph. It handles connected edges according to ontology-defined `on_kill_*` modifiers, supports cascade behavior, and can operate on single nodes by ID or bulk nodes via pattern matching.

## Syntax

### Grammar

```ebnf
KillStmt       = "kill" Target CascadeClause? ReturningClause? ;

Target         = IdRef
               | "{" MatchStmt "}" ;

CascadeClause  = "cascade" | "no" "cascade" ;

ReturningClause = "returning" ("id" | "*" | Identifier ("," Identifier)*) ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `kill` | Statement - initiates node deletion |
| `cascade` | Clause - forces cascade deletion of connected nodes |
| `no cascade` | Clause - prevents cascade, only unlinks edges |
| `returning` | Clause - specifies what to return after deletion |

### Examples

```
-- Kill by ID
KILL #task_123

-- Kill by pattern
KILL { MATCH t: Task WHERE t.status = "archived" RETURN t }

-- With cascade behavior
KILL #project_456 CASCADE

-- Prevent cascade explicitly
KILL #person_789 NO CASCADE
```

## Semantics

### Target Resolution

**By ID:**
```
KILL #task_123
```
Direct reference to a specific node.

**By Pattern:**
```
KILL { MATCH t: Task WHERE t.status = "archived" RETURN t }
```
Pattern-based kill operates on all matching nodes.

### Edge Handling

When a node is killed, connected edges are handled according to their `on_kill_*` modifiers:

| Modifier | Behavior |
|----------|----------|
| `unlink` (default) | Edge is removed |
| `cascade` | Connected node is killed |
| `prevent` | Kill operation fails |

**Default: Unlink**
```
-- Edge: assigned_to(task, person) [on_kill_target: unlink]

KILL #person_123
-- All assigned_to edges pointing to person are removed
-- Tasks remain (just unassigned)
```

**Cascade**
```
-- Edge: belongs_to(task, project) [on_kill_target: cascade]

KILL #project_456
-- All tasks belonging to project are also killed
-- (and their edges, recursively)
```

**Prevent**
```
-- Edge: member_of(person, team) [on_kill_target: prevent]

KILL #team_789
-- ERROR: Cannot kill Team: has 5 member_of edges
-- Operation fails, nothing changes
```

### CASCADE Override

The CASCADE clause can override ontology-defined behavior:

```
-- Force cascade even if ontology says unlink
KILL #project_123 CASCADE

-- Prevent cascade even if ontology says cascade
KILL #project_123 NO CASCADE
-- Note: unlinks edges but doesn't cascade
```

**Important:** Cannot override `prevent`. If ontology says prevent, the only option is to first remove blocking edges.

### Bulk Kill

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

### Result Format

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

## Layer 0

None. KILL operates on user-defined node types declared in the ontology.

## Examples

### Delete with Return

```
KILL #task_123
RETURNING id
```

### Bulk Delete with Safety Limit

```
-- Archive cleanup with limit
KILL {
  MATCH t: Task
  WHERE t.status = "archived"
    AND t.archived_at < now() - 2592000000
  RETURN t
  LIMIT 1000
}
```

### Cascade Project Deletion

```
-- Delete project and all its tasks
-- Assumes: edge belongs_to(task, project) [on_kill_target: cascade]
KILL #project_456 CASCADE
```

### Prevent Violation Handling

```
-- First remove blocking edges, then kill
UNLINK {
  MATCH member_of(_, #team_789) AS e
  RETURN e
}
KILL #team_789
```

## Errors

| Condition | Message |
|-----------|---------|
| Node not found | `Node 'X' not found` |
| Prevent constraint | `Cannot kill X: has N Y edges` |
| Pattern returns non-node | `KILL pattern must return nodes` |
| Pattern returns no results | No nodes matched the KILL pattern (warning, not error) |
