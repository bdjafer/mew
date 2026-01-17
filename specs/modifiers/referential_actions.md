---
spec: referential_actions
version: "1.0"
status: stable
category: modifier
capability: referential_actions
requires: [rules]
priority: common
---

# Spec: Referential Actions

## 1. Overview

Referential actions specify what happens to edges when a connected node is killed, enabling cascade delete, unlinking, or prevention.

**Why needed:** Data integrity often requires coordinated deletion. "When a project is deleted, delete all its tasks." "Cannot delete a team that has members." Without referential actions, users must implement these patterns manually.

**Note:** This feature requires the `rules` feature because referential actions compile to rules.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
EdgeModifier = ... | ReferentialModifier

ReferentialModifier =
    "on_kill_source" ":" ReferentialAction
  | "on_kill_target" ":" ReferentialAction

ReferentialAction = "cascade" | "unlink" | "prevent"
```

### 2.2 Keywords Added

| Keyword | Context |
|---------|---------|
| `on_kill_source` | Edge modifier |
| `on_kill_target` | Edge modifier |
| `cascade` | Referential action |
| `unlink` | Referential action |
| `prevent` | Referential action |

### 2.3 Examples
```
-- When project is killed, kill all its tasks
edge belongs_to(task: Task, project: Project) [
  on_kill_target: cascade
]

-- When person is killed, just remove assignment edges
edge assigned_to(task: Task, person: Person) [
  on_kill_target: unlink
]

-- Cannot kill team if it has members
edge member_of(person: Person, team: Team) [
  on_kill_target: prevent
]

-- Killing parent kills children
edge parent_of(parent: Node, child: Node) [
  on_kill_source: cascade
]
```

---

## 3. Semantics

### 3.1 Actions

| Action | Meaning |
|--------|---------|
| `unlink` | Remove the edge (default) |
| `cascade` | Kill the node at the other end |
| `prevent` | Prevent the kill operation |

### 3.2 Source vs Target

For `edge E(source: A, target: B)`:

| Modifier | Trigger | Affected |
|----------|---------|----------|
| `on_kill_source` | KILL source node | target node / edge |
| `on_kill_target` | KILL target node | source node / edge |
```
edge belongs_to(task: Task, project: Project) [on_kill_target: cascade]
--              source       target
-- When project (target) is killed, cascade to task (source)
```

### 3.3 Default Behavior

If no referential action is specified, `unlink` is the default:
- Killing a node silently removes all connected edges
- No cascading, no prevention

### 3.4 Binary Edges Only

Referential actions are only valid for **binary edges** (arity = 2).

For n-ary edges, use explicit rules.

### 3.5 Cascade Semantics

Cascade is recursive:
```
edge parent_of(parent: Org, child: Org) [on_kill_source: cascade]

-- Graph: Root → Dept1 → Dept2 → Dept3
KILL Root
-- Cascades: Root kills Dept1, Dept1 kills Dept2, Dept2 kills Dept3
```

### 3.6 Cascade Limits

| Limit | Default | Description |
|-------|---------|-------------|
| Depth limit | 100 | Maximum cascade chain length |
| Count limit | 10,000 | Maximum entities per cascade |

Exceeding limits fails the transaction.

### 3.7 Prevent Semantics

`prevent` blocks the KILL operation entirely:
```
edge member_of(person: Person, team: Team) [on_kill_target: prevent]

KILL team  -- ERROR if any member_of edges exist
```

### 3.8 Multiple Actions

When multiple edges have referential actions on the same node:

1. All `prevent` actions are checked first
2. If any prevents, KILL fails
3. Otherwise, all `cascade` and `unlink` actions execute

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _ReferentialAction [sealed] {
  trigger: String [required],       -- "on_kill_source" | "on_kill_target"
  action: String [required]         -- "cascade" | "unlink" | "prevent"
}
```

### 4.2 Edge Types
```
edge _edge_type_referential(
  edge_type: _EdgeType,
  referential: _ReferentialAction
) {}
```

### 4.3 Constraints
```
constraint _referential_binary_only:
  e: _EdgeType, r: _ReferentialAction,
  _edge_type_referential(e, r)
  => e.arity = 2
```

---

## 5. Compilation

Referential actions compile to rules.

### 5.1 Cascade
```
edge belongs_to(task: Task, project: Project) [on_kill_target: cascade]
```

Compiles to rule:
```
rule _belongs_to_cascade_target [priority: 1000]:
  t: Task, p: Project,
  belongs_to(t, p),
  _pending_kill(p)
  =>
  KILL t
```

Note: `_pending_kill` is an internal marker set by the engine during KILL processing.

### 5.2 Unlink
```
edge assigned_to(task: Task, person: Person) [on_kill_target: unlink]
```

Compiles to rule:
```
rule _assigned_to_unlink_target [priority: 1000]:
  t: Task, p: Person,
  assigned_to(t, p) AS e,
  _pending_kill(p)
  =>
  UNLINK e
```

### 5.3 Prevent
```
edge member_of(person: Person, team: Team) [on_kill_target: prevent]
```

Compiles to constraint:
```
constraint _member_of_prevent_kill_target:
  t: Team, p: Person,
  member_of(p, t),
  _pending_kill(t)
  => false
```

---

## 6. Examples

### 6.1 Project with Cascading Tasks
```
ontology ProjectManagement {
  node Project { name: String [required] }
  node Task { title: String [required] }
  
  edge belongs_to(task: Task, project: Project) [
    task -> 1,
    on_kill_target: cascade
  ]
}

-- Usage:
SPAWN p: Project { name = "Alpha" }
SPAWN t1: Task { title = "Task 1" }
SPAWN t2: Task { title = "Task 2" }
LINK belongs_to(t1, p)
LINK belongs_to(t2, p)

KILL p  -- Also kills t1 and t2
```

### 6.2 Protected Team
```
ontology Teams {
  node Person { name: String [required] }
  node Team { name: String [required] }
  
  edge member_of(person: Person, team: Team) [
    on_kill_target: prevent
  ]
}

-- Usage:
SPAWN t: Team { name = "Engineering" }
SPAWN p: Person { name = "Alice" }
LINK member_of(p, t)

KILL t  -- ERROR: Cannot kill team with members
```

### 6.3 Mixed Actions
```
ontology Organization {
  node Department { name: String }
  node Employee { name: String }
  node Asset { serial: String }
  
  -- Killing department kills its assets
  edge owns(dept: Department, asset: Asset) [
    on_kill_source: cascade
  ]
  
  -- Killing department just removes employee assignments
  edge works_in(employee: Employee, dept: Department) [
    on_kill_target: unlink
  ]
}

-- KILL department:
-- 1. All owned assets are killed (cascade)
-- 2. All works_in edges are removed (unlink)
-- 3. Employees remain (just unlinked from department)
```

---

## 7. Errors

| Code | Condition | Message |
|------|-----------|---------|
| E3301 | Referential on non-binary edge | `"Referential actions only supported for binary edges"` |
| E3302 | KILL prevented | `"Cannot kill 'X': referenced by 'edge_type' with prevent action"` |
| E3303 | Cascade depth exceeded | `"Cascade depth limit exceeded (N)"` |
| E3304 | Cascade count exceeded | `"Cascade count limit exceeded (N entities)"` |

---

*End of Feature: Referential Actions*