---
spec: link
version: "1.0"
status: draft
category: statement
capability: mutation
requires: [id_ref, match_stmt, spawn_expr, attr_block, returning_clause]
priority: essential
---

# Spec: LINK

## Overview

LINK creates a new edge connecting nodes in the graph. It supports multiple target resolution methods (ID, pattern, inline spawn), edge attributes, uniqueness constraints, and the idempotent `LINK IF NOT EXISTS` variant for safe edge creation.

## Syntax

### Grammar

```ebnf
LinkStmt           = "link" IfNotExists? EdgeType "(" TargetList ")" AliasClause? AttrBlock? ReturningClause? ;

IfNotExists        = "if" "not" "exists" ;

EdgeType           = Identifier ;

TargetList         = TargetRef ("," TargetRef)* ;

TargetRef          = IdRef
                   | "{" MatchStmt "}"
                   | SpawnExpr ;

SpawnExpr          = "spawn" TypeExpr AttrBlock? AsBinding? ;

AsBinding          = "as" Identifier ;

AliasClause        = "as" Identifier ;

AttrBlock          = "{" (AttrAssignment ("," AttrAssignment)*)? "}" ;

AttrAssignment     = Identifier "=" Expr ;

ReturningClause    = "returning" ("id" | "*" | "created" | Identifier ("," Identifier)*) ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `link` | Statement - initiates edge creation |
| `if not exists` | Modifier - makes operation idempotent |
| `as` | Clause - binds edge or spawned node to identifier |
| `spawn` | Expression - inline node creation within LINK |
| `returning` | Clause - specifies what to return |
| `created` | RETURNING keyword - indicates if edge was newly created |

### Examples

```
-- Basic link by ID
LINK causes(#event_123, #event_456)

-- With attributes
LINK assigned_to(#task_123, #person_456) {
  assigned_at = now(),
  role = "owner"
}

-- With alias for returning
LINK causes(#e1, #e2) AS c
RETURNING *

-- Idempotent link
LINK IF NOT EXISTS member_of(#person, #team)
```

## Semantics

### Target Resolution

**By ID:**
```
LINK causes(#node_a, #node_b)
```
Direct reference to existing nodes.

**By Pattern:**
```
LINK assigned_to(
  #task_123,
  { MATCH p: Person WHERE p.email = "alice#example.com" RETURN p }
)
```
Pattern must return exactly one node. Error if zero or multiple.

**Mixed:**
```
LINK belongs_to(
  #new_task,
  { MATCH p: Project WHERE p.id = $projectId RETURN p }
)
```

**Inline SPAWN:**
Create a node and link it in a single statement:
```
LINK belongs_to(
  SPAWN Task { title = "New task", priority = 5 },
  #project_123
)
RETURNING id
```

### AS Binding in Inline SPAWN

Use `AS` to bind spawned nodes for use in RETURNING:

```
LINK belongs_to(
  SPAWN Task { title = "New task" } AS t,
  #project_123
) AS e
RETURNING t.id AS task_id, e.id AS edge_id
```

**Multiple spawns:**
```
LINK collaboration(
  SPAWN Person { name = "Alice" } AS alice,
  SPAWN Person { name = "Bob" } AS bob
) AS collab
RETURNING alice.id, bob.id, collab.id
```

**Scope:** Bindings are available only in the RETURNING clause of the same statement.

**Semantics:**
- SPAWN expressions are evaluated first (left to right)
- Created nodes are available for the LINK
- If LINK fails (constraint violation), all SPAWNs are rolled back
- RETURNING can access both the edge and spawned node IDs via AS bindings

### Edge Attributes

```
LINK assigned_to(#task, #person) {
  assigned_at = now(),
  role = "reviewer",
  notes = "Urgent review needed"
}
```

Same rules as node attributes:
- Required attributes must be provided
- Defaults apply for omitted attributes
- Type checking enforced

### Validation

Before committing:

1. **Edge type exists:** Declared in ontology
2. **Target count:** Matches edge arity
3. **Target types:** Each target matches signature type
4. **Target exists:** All referenced nodes exist
5. **Constraints:** Edge modifiers satisfied (no_self, acyclic, cardinality, unique, etc.)

### Uniqueness and Duplicate Edges

**Default: Duplicates Allowed**

By default, edges are distinct entities. Multiple edges of the same type can connect the same nodes:

```
edge tagged(task: Task, tag: Tag) {}  -- no [unique]

LINK tagged(#task_1, #tag_urgent)     -- creates edge_001
LINK tagged(#task_1, #tag_urgent)     -- creates edge_002 (different ID!)
```

**With [unique]: Duplicates Prevented**

If edge type has `[unique]` modifier:

```
edge member_of(person: Person, team: Team) [unique] {}

-- First link succeeds
LINK member_of(#person_123, #team_456)

-- Duplicate link fails
LINK member_of(#person_123, #team_456)
-- ERROR: Duplicate edge: member_of(person_123, team_456) already exists
```

**When to Use [unique]:**

| Scenario | Use [unique]? | Reason |
|----------|---------------|--------|
| Membership (person -> team) | Yes | Person is either a member or not |
| Assignment (task -> person) | Yes | Task has at most one assignee |
| Tagging (item -> tag) | Usually | Unless tracking multiple tag events |
| Event causation | No | Multiple causal links may exist |
| Audit trail edges | No | Each event is distinct |

### LINK IF NOT EXISTS

Create an edge only if it doesn't already exist. Idempotent operation.

```
-- Create assignment if not already assigned
LINK IF NOT EXISTS assigned_to(#task, #person) { assigned_at = now() }
RETURNING CREATED

-- Result: { created: true, id: "edge_123" } or { created: false, id: "edge_456" }
```

**Behavior:**

| Scenario | Behavior | RETURNING CREATED |
|----------|----------|-------------------|
| Edge doesn't exist | Creates new edge | `true` |
| Edge already exists | No-op, returns existing | `false` |
| Edge type has `[unique]` | Same as without (redundant but allowed) | - |

**Matching criteria:** An edge "exists" if there's an edge of the same type connecting the same targets in the same order (or any order for symmetric edges).

**Attributes on existing edge:** If the edge exists, provided attributes are **ignored** (not updated). Use SET to update.

```
-- Edge exists with assigned_at = yesterday
LINK IF NOT EXISTS assigned_to(#task, #person) { assigned_at = now() }
-- Returns existing edge; assigned_at still = yesterday

-- To update if exists, use separate SET:
LINK IF NOT EXISTS assigned_to(#task, #person) AS a { assigned_at = now() }
SET a.last_checked = now()  -- Always updates
```

### Result Format

```typescript
interface LinkResult {
  success: boolean
  id: string                     // created edge ID
  data: Record<string, Value>    // based on RETURNING
  created?: boolean              // for IF NOT EXISTS
  errors?: string[]
}
```

## Layer 0

None. LINK operates on user-defined edge types declared in the ontology.

## Examples

### Task Assignment with Attributes

```
LINK assigned_to(#task_123, #person_456) {
  assigned_at = now(),
  role = "owner",
  due_date = $deadline
}
RETURNING id, assigned_at
```

### Create and Link in One Statement

```
-- Create task and assign in one statement
LINK assigned_to(
  SPAWN Task { title = "Review PR #123" },
  #person_alice
) AS assignment
RETURNING assignment.id

-- Create both endpoints
LINK causes(
  SPAWN Event { name = "User clicked", timestamp = now() } AS cause,
  SPAWN Event { name = "Page loaded", timestamp = now() + 100 } AS effect
) AS causation
RETURNING cause.id AS cause_id, effect.id AS effect_id, causation.id AS edge_id
```

### Higher-Order Edge

```
-- Base edge
LINK causes(#event_a, #event_b) AS c

-- Higher-order edge referencing the base
LINK confidence(#c, 0.9) {
  assessed_by = "expert"
}
```

### Idempotent API Operations

```
-- Idempotent follow operation
LINK IF NOT EXISTS follows(#user, #target)
RETURNING id, CREATED AS is_new

-- Ensure relationship (defensive programming)
LINK IF NOT EXISTS belongs_to(#task, #project)

-- Batch import (skip duplicates)
FOR item IN $items:
  LINK IF NOT EXISTS tagged(#item.entity, #item.tag)
```

## Errors

| Condition | Message |
|-----------|---------|
| Unknown edge type | `Edge type 'X' not declared in ontology` |
| Target not found | `Node 'X' not found` |
| Self-loop on no_self edge | `Edge 'X' does not allow self-loops` |
| Cardinality violation | `Cardinality constraint violated: X -> Y` |
| Acyclic violation | `Edge 'X' would create cycle` |
| Duplicate on unique edge | `Duplicate edge: X(a, b) already exists` |
| Pattern returns multiple | `Pattern must return exactly one node, got N` |
| Target type mismatch | `Target type mismatch: expected X, got Y` |
