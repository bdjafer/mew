# Feature: Rules

**Version:** 1.0
**Status:** Essential
**Requires:** Core (Parts I, II, III), exists_patterns, transactions

---

## 1. Overview

Rules define reactive transformations: when a pattern matches, execute actions.

**Why essential:** Rules enable self-maintaining graphs where derived data is computed automatically and invariants are maintained through transformations rather than just validation.

**Note:** Rules are powerful but optional. A graph database with only constraints is fully functional for validation. Rules add reactivity.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
RuleDecl =
  DocComment?
  "rule" Identifier RuleModifiers? ":" Pattern "=>" Production

RuleModifiers = "[" RuleModifier ("," RuleModifier)* "]"

RuleModifier =
    "priority:" IntLiteral
  | "auto"
  | "manual"

Production = Action ("," Action)*

Action =
    SpawnAction
  | KillAction
  | LinkAction
  | UnlinkAction
  | SetAction

SpawnAction = "SPAWN" Identifier ":" TypeExpr ("{" AttrAssignments "}")?

KillAction = "KILL" Identifier

LinkAction = "LINK" Identifier "(" TargetList ")" ("{" AttrAssignments "}")?

UnlinkAction = "UNLINK" Identifier

SetAction = "SET" AttrAccess "=" Expr

AttrAssignments = (AttrAssignment ","?)*
AttrAssignment = Identifier "=" Expr
```

Add to Declaration:
```ebnf
Declaration = ... | RuleDecl
```

### 2.2 Keywords Added

| Keyword | Context |
|---------|---------|
| `rule` | Declaration |
| `priority` | Rule modifier |
| `auto` | Rule modifier |
| `manual` | Rule modifier |
| `spawn` | Action |
| `kill` | Action |
| `link` | Action |
| `unlink` | Action |
| `set` | Action |

### 2.3 Examples
```
-- Auto-set timestamp
rule auto_created_at [priority: 100]:
  t: Task WHERE t.created_at = null
  =>
  SET t.created_at = now()

-- Multi-action rule
rule on_task_complete:
  t: Task
  WHERE t.status = "done" AND t.completed_at = null
  =>
  SET t.completed_at = now(),
  SPAWN n: Notification { message = "Task done: " ++ t.title },
  LINK notifies(n, t)

-- Manual rule
rule archive_old [manual]:
  t: Task
  WHERE t.completed_at != null AND t.archived = false
  =>
  SET t.archived = true
```

---

## 3. Semantics

### 3.1 Rule Modifiers

| Modifier | Default | Meaning |
|----------|---------|---------|
| `priority: N` | 0 | Execution order (higher first) |
| `auto` | Yes | Fire when pattern matches |
| `manual` | No | Only fire when invoked |

### 3.2 Execution Order

1. Rules execute in **priority order** (highest first)
2. Same-priority rules execute in **declaration order**
3. Rules continue until **quiescence** (no new matches)

### 3.3 Actions

| Action | Syntax | Effect |
|--------|--------|--------|
| SPAWN | `SPAWN x: Type { attrs }` | Create node, bind to x |
| KILL | `KILL x` | Remove node |
| LINK | `LINK edge(a, b) { attrs }` | Create edge |
| UNLINK | `UNLINK e` | Remove edge (e must be edge variable) |
| SET | `SET x.attr = value` | Modify attribute |

### 3.4 Production Scoping

Variables are bound sequentially:
```
rule example:
  a: TypeA
  =>
  SPAWN b: TypeB { name = a.name },  -- a available, b bound here
  LINK connects(a, b),               -- a, b available
  SET b.processed = true             -- a, b available
```

Cannot reference a variable before its SPAWN.

### 3.5 Execution Limits

| Limit | Default | Description |
|-------|---------|-------------|
| Same-binding | 1 | (rule, bindings) pair executes once per transaction |
| Action limit | 10,000 | Maximum actions per transaction |
| Depth limit | 100 | Maximum nested rule triggers |

When exceeded, transaction fails and rolls back.

### 3.6 Cycle Detection

The engine tracks executed (rule, match) pairs. If the same pair is encountered twice in one transaction, a cycle is detected and the transaction fails.

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _RuleDef [sealed] {
  name: String [required],
  priority: Int = 0,
  auto: Bool = true,
  doc: String?
}

node _ProductionDef [sealed] {
  -- Container for actions
}

node _Action [abstract, sealed, extension-point] {
  order: Int = 0    -- Execution order within production
}

node _SpawnAction : _Action [sealed] {
  var_name: String [required]
  -- Type linked via _spawn_type edge
}

node _KillAction : _Action [sealed] {
  target_var: String [required]
}

node _LinkAction : _Action [sealed] {
  var_name: String?    -- Optional edge binding
  -- Edge type and targets via edges
}

node _UnlinkAction : _Action [sealed] {
  target_var: String [required]
}

node _SetAction : _Action [sealed] {
  target_var: String [required],
  attr_name: String [required]
  -- Value via _set_value edge
}
```

### 4.2 Edge Types
```
edge _rule_has_pattern(
  rule: _RuleDef,
  pattern: _PatternDef
) {}

edge _rule_has_production(
  rule: _RuleDef,
  production: _ProductionDef
) {}

edge _production_has_action(
  production: _ProductionDef,
  action: _Action
) {}

edge _spawn_type(
  action: _SpawnAction,
  type_expr: _TypeExpr
) {}

edge _spawn_attr(
  action: _SpawnAction,
  value: _Expr
) {
  attr_name: String [required]
}

edge _link_edge_type(
  action: _LinkAction,
  edge_type: _EdgeType
) {}

edge _link_target(
  action: _LinkAction
) {
  position: Int [required],
  var_name: String [required]
}

edge _link_attr(
  action: _LinkAction,
  value: _Expr
) {
  attr_name: String [required]
}

edge _set_value(
  action: _SetAction,
  value: _Expr
) {}

edge _ontology_declares_rule(
  ontology: _Ontology,
  rule: _RuleDef
) {}
```

### 4.3 Constraints
```
constraint _rule_has_pattern:
  r: _RuleDef
  => EXISTS(p: _PatternDef, _rule_has_pattern(r, p))

constraint _rule_has_production:
  r: _RuleDef
  => EXISTS(p: _ProductionDef, _rule_has_production(r, p))

constraint _spawn_has_type:
  a: _SpawnAction
  => EXISTS(t: _TypeExpr, _spawn_type(a, t))

constraint _set_has_value:
  a: _SetAction
  => EXISTS(v: _Expr, _set_value(a, v))
```

---

## 5. Compilation

### 5.1 Rule Declaration
```
rule auto_timestamp [priority: 100]:
  t: Task WHERE t.created_at = null
  =>
  SET t.created_at = now()
```

Compiles to:
```
_RuleDef node:
  name: "auto_timestamp"
  priority: 100
  auto: true

_PatternDef node:
  (node pattern for t, condition for created_at = null)

_rule_has_pattern edge:
  (rule, pattern)

_ProductionDef node

_rule_has_production edge:
  (rule, production)

_SetAction node:
  target_var: "t"
  attr_name: "created_at"
  order: 0

_production_has_action edge:
  (production, set_action)

_CallExpr node:
  function_name: "now"

_set_value edge:
  (set_action, call_expr)
```

---

## 6. Execution Model

### 6.1 Trigger Timing

Rules are evaluated **after** mutations, **before** constraint checking:
```
Transaction:
1. Apply user mutations
2. Find triggered rules
3. Execute rules in priority order
4. Repeat 2-3 until quiescence
5. Check constraints
6. If constraints pass: commit
7. If constraints fail: rollback entire transaction
```

### 6.2 Rule-Constraint Interaction

Rules can "fix" potential constraint violations:
```
constraint task_has_timestamp:
  t: Task => t.created_at != null

rule auto_timestamp [priority: 100]:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()
```

User spawns Task without `created_at`:
1. Task created (constraint would fail)
2. Rule fires, sets `created_at`
3. Constraint checked → passes
4. Commit succeeds

### 6.3 Manual Rule Invocation

Manual rules don't auto-fire. They're invoked explicitly:
```
-- In MEW runtime:
INVOKE archive_old
INVOKE archive_old WHERE t.project_id = $project
```

---

## 7. Examples

### 7.1 Auto-Populate Timestamps
```
rule auto_created [priority: 100]:
  e: Entity WHERE e.created_at = null
  =>
  SET e.created_at = now()

rule auto_updated [priority: 100]:
  e: Entity WHERE e.updated_at = null OR e.updated_at < e.created_at
  =>
  SET e.updated_at = now()
```

### 7.2 Cascade Status
```
rule cascade_project_archive:
  p: Project, t: Task,
  belongs_to(t, p)
  WHERE p.archived = true AND t.archived = false
  =>
  SET t.archived = true
```

### 7.3 Create Derived Edges
```
rule transitive_reports:
  a: Person, b: Person, c: Person,
  reports_to(a, b),
  reports_to(b, c)
  WHERE NOT EXISTS(indirectly_reports_to(a, c))
  =>
  LINK indirectly_reports_to(a, c)
```

### 7.4 Notification System
```
rule notify_assignment:
  t: Task, p: Person,
  assigned_to(t, p) AS a
  WHERE a.notified = null
  =>
  SPAWN n: Notification {
    message = "You were assigned: " ++ t.title,
    recipient_id = p.id
  },
  LINK notification_for(n, t),
  SET a.notified = true
```

---

*End of Feature: Rules*