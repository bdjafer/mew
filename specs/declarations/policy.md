---
spec: policy
version: "1.0"
status: draft
category: declaration
capability: policy
requires: [pattern, expression, context_functions]
priority: essential
---

# Spec: Policy Declaration

## Overview

Policy declarations define actor-based access control rules for graph operations. Each policy specifies when an operation is allowed or denied based on the actor performing it, the operation type, and the target. Policies are evaluated before any mutation is attempted, meaning denied operations never touch the graph. The system uses default-deny semantics: if no matching policy allows an operation, it is denied.

## Syntax

### Grammar

```ebnf
PolicyDecl       = "policy" Identifier PolicyMods? ":"
                   "ON" OpPattern
                   Decision "IF" Expr
                   Message?

PolicyMods       = "[" "priority:" IntLiteral "]"

OpPattern        = "*"
                 | OpType
                 | OpType "(" Pattern ")"
                 | OpType "(" Pattern "," AttrName ")"
                 | OpPattern "|" OpPattern

OpType           = "SPAWN" | "KILL" | "LINK" | "UNLINK" | "SET" | "MATCH"
                 | "META" OpType

Decision         = "ALLOW" | "DENY"

Message          = "MESSAGE" StringLiteral

(* Context Functions - valid only in policy conditions *)
ContextFunc      = "current_actor" "(" ")"
                 | "operation" "(" ")"
                 | "target" "(" ")"
                 | "target_type" "(" ")"
                 | "target_attr" "(" ")"

(* Session Binding *)
SessionStmt      = "BEGIN" "SESSION" "AS" NodeRef
                 | "END" "SESSION"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `policy` | Declaration - introduces a policy rule |
| `ON` | Declaration - specifies the operation pattern to match |
| `ALLOW` | Decision - permits the operation |
| `DENY` | Decision - rejects the operation |
| `IF` | Declaration - introduces the condition expression |
| `MESSAGE` | Declaration - custom rejection message |
| `priority` | Modifier - policy evaluation priority (higher wins) |
| `SPAWN` | OpType - node creation |
| `KILL` | OpType - node deletion |
| `LINK` | OpType - edge creation |
| `UNLINK` | OpType - edge deletion |
| `SET` | OpType - attribute modification |
| `MATCH` | OpType - query/read operation |
| `META` | OpType prefix - schema-level operation |
| `BEGIN` | Session - starts session binding |
| `SESSION` | Session - session binding statement |
| `AS` | Session - specifies actor |
| `END` | Session - ends session binding |

### Examples

```mew
-- Unconditional access for superadmins
policy superadmin_bypass [priority: 1000]:
  ON *
  ALLOW IF EXISTS(has_role(current_actor(), r) WHERE r.name = "superadmin")

-- Type-level: only admins can spawn Projects
policy admin_create_project:
  ON SPAWN(p: Project)
  ALLOW IF has_role(current_actor(), admin_role)

-- Attribute-level: only assignees can change task status
policy assignee_set_status:
  ON SET(t: Task, "status")
  ALLOW IF assigned_to(t, current_actor())

-- Resource-level: project members can read project tasks
policy member_read_tasks:
  ON MATCH(t: Task)
  ALLOW IF EXISTS(
    p: Project,
    belongs_to(t, p),
    project_member(current_actor(), p)
  )

-- Default deny (lowest priority)
policy default_deny [priority: -1000]:
  ON *
  DENY IF true
  MESSAGE "Permission denied"

-- Session binding
BEGIN SESSION AS #alice
  SPAWN t: Task { title = "Test" }
COMMIT
END SESSION
```

## Semantics

### Policy vs Constraints

Policies and constraints serve different purposes:

| Aspect | Constraint | Policy |
|--------|------------|--------|
| Question | "Is the graph state valid?" | "Is this actor allowed to perform this operation?" |
| Input | Graph state | Actor + Operation + Target + Graph state |
| Timing | After mutation, before commit | Before mutation attempt |
| On Failure | Rollback mutation | Reject immediately (no mutation attempted) |

### Default-Deny Semantics

When no policy matches an operation, the system denies it:

- No matching policies = DENY
- At least one ALLOW must match and evaluate to true for permission
- This is the secure default

### Priority Resolution

When multiple policies match an operation:

1. Collect all policies matching (operation, target_type)
2. Evaluate each policy's condition with execution context
3. Gather decisions with priorities
4. Resolution:
   - Highest priority wins
   - Equal priority: DENY wins (secure default)

```mew
-- Example resolution:
-- Policy A [priority: 100] -> ALLOW (wins, highest priority)
-- Policy B [priority: 50]  -> DENY
-- Policy C [priority: 50]  -> ALLOW
-- Final decision: ALLOW (Policy A)
```

### Pre-Check Semantics

Policy is evaluated BEFORE any mutation:

```
Operation received
      |
      v
+------------------+
|   POLICY CHECK   |
|                  |
|  Extract:        |
|    actor         |
|    operation     |
|    target        |
|    attribute     |
|                  |
|  Decision:       |
|    ALLOW -> continue to mutation
|    DENY  -> reject immediately
+------------------+
      |
      v (only if ALLOW)
+------------------+
| MUTATION FLOW    |
| Type -> Apply -> |
| Constraints ->   |
| Rules -> Commit  |
+------------------+
```

Key property: Denied operations never touch the graph.

### Context Functions

Policy conditions have access to execution context:

| Function | Returns | Example |
|----------|---------|---------|
| `current_actor()` | The session's actor | `#alice` |
| `operation()` | Operation type string | `"SET"` |
| `target()` | Target entity or null | `#task_123` |
| `target_type()` | Type name of target | `"Task"` |
| `target_attr()` | Attribute (for SET) | `"status"` |

These functions are **only valid in policy conditions** -- they require execution context that doesn't exist in normal constraints or rules.

### Operation Pattern Matching

The `_` wildcard matches any type or attribute in a pattern position.

| Pattern | Matches |
|---------|---------|
| `*` | Any operation |
| `SPAWN(_)` | Any node creation |
| `SPAWN(t: Task)` | Task creation |
| `KILL(t: Task)` | Task deletion |
| `SET(t: Task, _)` | Any Task attribute modification |
| `SET(t: Task, "status")` | Specific attribute modification |
| `LINK(e: assigned_to)` | Specific edge type creation |
| `UNLINK(e: assigned_to)` | Specific edge type deletion |
| `MATCH(t: Task)` | Reading tasks |
| `META SPAWN(_)` | Schema node creation |
| `META SET(_)` | Any schema attribute modification |
| `META KILL(_)` | Schema element deletion |

### Compound Operation Patterns

Multiple patterns can be combined with `|`:

```mew
-- Editors can modify tasks (set attributes, link, or unlink)
policy editor_modify:
  ON SET(t: Task, _) | LINK(_, t) | UNLINK(_, t)
  ALLOW IF has_project_role(current_actor(), task_project(t), "editor")
```

### META Operations

Schema-level operations use the `META` prefix:

```mew
-- Schema read access requires schema_read capability
policy meta_read:
  ON META MATCH(_)
  ALLOW IF has_capability(current_actor(), "schema_read")

-- Schema write access requires schema_write capability
policy meta_write:
  ON META SPAWN(_) | META SET(_) | META LINK(_) | META UNLINK(_) | META KILL(_)
  ALLOW IF has_capability(current_actor(), "schema_write")
```

### Session Actor Binding

Every session operates on behalf of an actor:

```mew
BEGIN SESSION AS #alice
  SPAWN t: Task { title = "Test" }    -- Policy checked as Alice
  SET t.status = "done"               -- Policy checked as Alice
COMMIT
```

Sessions without an actor operate in **system context** -- typically unrestricted, used for internal operations.

### Interaction with Other Systems

**With Constraints:** Policy runs BEFORE constraints. A denied operation never reaches constraint checking.

**With Rules:** Rules execute with system authority by default. Rules can optionally inherit session authority:

```mew
rule user_triggered_action [inherit_authority]:
  t: Task WHERE t.status = "approved"
  =>
  LINK published(t, ...)      -- Policy checked as session actor
```

**With Transactions:** Policy is checked per operation, not per transaction. Each operation is independently checked; transaction commits only if all pass.

## Layer 0

### Nodes

```mew
node _PolicyRule {
  name: String [required, unique],
  priority: Int = 0,
  decision: String [in: ["allow", "deny"]],
  message: String?,
  doc: String?
}

node _OperationPattern {
  operation: String,         -- "SPAWN", "SET", "*", etc.
  target_type: String?,      -- null = any
  target_attr: String?       -- for SET
}
```

### Edges

```mew
edge _policy_has_pattern(
  rule: _PolicyRule,
  pattern: _OperationPattern
)

edge _policy_has_condition(
  rule: _PolicyRule,
  condition: _Expr
)

edge _ontology_declares_policy(
  ontology: _Ontology,
  rule: _PolicyRule
)
```

### Constraints

```mew
constraint _policy_has_pattern:
  r: _PolicyRule => EXISTS(p: _OperationPattern, _policy_has_pattern(r, p))

constraint _policy_has_condition:
  r: _PolicyRule => EXISTS(c: _Expr, _policy_has_condition(r, c))
```

Policy rule name uniqueness is enforced through the `unique` modifier on `_PolicyRule.name`.

## Examples

### Role-Based Access Control (RBAC)

```mew
ontology TaskManagement {

  node Role { name: String [required, unique] }
  node Permission {
    operation: String,
    target_type: String?
  }
  edge has_role(actor: Person, role: Role)
  edge role_has_permission(role: Role, perm: Permission)

  -- Generic RBAC policy
  policy rbac [priority: 100]:
    ON *
    ALLOW IF EXISTS(
      r: Role, p: Permission,
      has_role(current_actor(), r),
      role_has_permission(r, p),
      WHERE p.operation = operation()
        AND (p.target_type = null OR p.target_type = target_type())
    )
}
```

### Relationship-Based Access Control

```mew
ontology ProjectTasks {

  node Person { name: String [required] }
  node Project { name: String [required] }
  node Task { title: String [required] }

  edge belongs_to(task: Task, project: Project)
  edge member_of(person: Person, project: Project)
  edge assigned_to(task: Task, person: Person)

  -- Project members can read and modify tasks in their projects
  policy project_member_access:
    ON MATCH(t: Task) | SET(t: Task, _)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(t, p),
      member_of(current_actor(), p)
    )

  -- Only assignees can change task status
  policy assignee_set_status:
    ON SET(t: Task, "status")
    ALLOW IF assigned_to(t, current_actor())
}
```

### Ownership Model

```mew
ontology OwnedResources {

  edge owned_by(resource: any, owner: Person)

  -- Owners have full access to their resources
  policy owner_full_access:
    ON *
    ALLOW IF target() != null
      AND EXISTS(owned_by(target(), current_actor()))

  -- Auto-assign ownership on creation
  rule auto_ownership:
    n: any WHERE NOT EXISTS(owned_by(n, _))
    =>
    LINK owned_by(n, current_actor())
}
```

### Complete Policy Example

```mew
ontology SecureTaskManagement {

  node Person { name: String [required] }
  node Project { name: String [required] }
  node Task {
    title: String [required],
    status: String [in: ["todo", "in_progress", "done"]] = "todo",
    priority: Int [0..10] = 5
  }
  node Role { name: String [required, unique] }

  edge belongs_to(task: Task, project: Project)
  edge member_of(person: Person, project: Project)
  edge assigned_to(task: Task, person: Person)
  edge has_role(person: Person, role: Role)
  edge project_role(person: Person, project: Project) { role: String }

  -- Superadmin bypass (highest priority)
  policy superadmin_bypass [priority: 1000]:
    ON *
    ALLOW IF EXISTS(has_role(current_actor(), r) WHERE r.name = "superadmin")

  -- Project admins can create tasks
  policy admin_create_task:
    ON SPAWN(t: Task)
    ALLOW IF EXISTS(
      p: Project,
      project_role(current_actor(), p) WHERE project_role.role = "admin"
    )

  -- Project members can view tasks in their projects
  policy member_view_tasks:
    ON MATCH(t: Task)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(t, p),
      member_of(current_actor(), p)
    )

  -- Assignees can update their task status
  policy assignee_update_status:
    ON SET(t: Task, "status")
    ALLOW IF assigned_to(t, current_actor())

  -- Editors can modify task attributes (except status)
  policy editor_modify_task:
    ON SET(t: Task, _)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(t, p),
      project_role(current_actor(), p) WHERE project_role.role = "editor"
    ) AND target_attr() != "status"

  -- Project admins can delete tasks
  policy admin_delete_task:
    ON KILL(t: Task)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(t, p),
      project_role(current_actor(), p) WHERE project_role.role = "admin"
    )

  -- Schema access for system operators
  policy meta_read:
    ON META MATCH(_)
    ALLOW IF has_role(current_actor(), r) WHERE r.name = "operator"

  policy meta_write:
    ON META SPAWN(_) | META SET(_) | META LINK(_) | META UNLINK(_) | META KILL(_)
    ALLOW IF has_role(current_actor(), r) WHERE r.name = "operator"

  -- Explicit default deny (lowest priority)
  policy default_deny [priority: -1000]:
    ON *
    DENY IF true
    MESSAGE "Permission denied"
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Missing policy name | Policy name required. Add a name: `policy <name>: ...` |
| Duplicate policy name | Policy `<name>` already defined in this ontology |
| Invalid operation type | Unknown operation type `<type>`. Expected: SPAWN, KILL, LINK, UNLINK, SET, MATCH, or META prefix |
| Invalid operation pattern | Invalid operation pattern syntax |
| Missing ON clause | Policy requires ON clause specifying operation pattern |
| Missing decision | Policy requires ALLOW or DENY decision |
| Missing condition | Policy requires IF clause with condition expression |
| Non-boolean condition | Policy condition must evaluate to boolean, got `<type>` |
| Invalid context function | Context function `<name>` is only valid in policy conditions |
| Context function outside policy | `current_actor()` can only be used in policy conditions |
| Invalid priority value | Priority must be an integer, got `<value>` |
| Unbound variable in condition | Variable `<name>` used in condition but not defined in operation pattern |
| Permission denied | Policy `<name>` denied: `<message or "Permission denied">` |
| No actor bound | Operation requires actor but session has none |
| Invalid actor | Bound actor `<ref>` does not exist or is not a valid actor type |
| Policy evaluation error | Policy `<name>` condition failed to evaluate: `<reason>` |
| Conflicting patterns | Operation pattern `<pattern>` conflicts with existing pattern |
