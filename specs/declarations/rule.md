---
spec: rule
version: "1.0"
status: draft
category: declaration
capability: rules
requires: [pattern, expression, spawn, kill, link, unlink, set]
priority: common
---

# Spec: Rule Declaration

## Overview

A rule declaration defines a reactive transformation: when a graph pattern matches, execute a sequence of actions. Rules enable self-maintaining graphs where derived data is computed automatically. Rules fire within transactions, after user mutations but before constraint checking.

## Syntax

### Grammar

```ebnf
RuleDecl = DocComment? "rule" Identifier RuleModifiers? ":" Pattern "=>" Production

RuleModifiers = "[" RuleModifier ("," RuleModifier)* "]"

RuleModifier = PriorityModifier | AutoModifier | ManualModifier

PriorityModifier = "priority" ":" IntLiteral

AutoModifier = "auto"

ManualModifier = "manual"

Production = Action ("," Action)*

Action = SpawnAction | KillAction | LinkAction | UnlinkAction | SetAction

SpawnAction = "SPAWN" Identifier ":" TypeExpr AttrBlock?

KillAction = "KILL" Identifier

LinkAction = "LINK" EdgePattern AttrBlock?

UnlinkAction = "UNLINK" Identifier

SetAction = "SET" AttrAccess "=" Expr
```

### Keywords

| Keyword | Context |
|---------|---------|
| `rule` | Declaration |
| `priority` | Rule modifier |
| `auto` | Rule modifier |
| `manual` | Rule modifier |
| `SPAWN` | Action |
| `KILL` | Action |
| `LINK` | Action |
| `UNLINK` | Action |
| `SET` | Action |

### Examples

```
-- Simple timestamp rule
rule auto_created_at [priority: 100]:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()

-- Multi-action rule
rule on_task_complete:
  t: Task WHERE t.status = "done" AND t.completed_at = null
  =>
  SET t.completed_at = now(),
  SPAWN n: Notification { message = "Task done: " ++ t.title },
  LINK notifies(n, t)

-- Manual rule
rule archive_old [manual]:
  t: Task WHERE t.archived = false AND t.completed_at != null
  => SET t.archived = true

-- Derived edge rule
rule transitive_reports:
  a: Person, b: Person, c: Person,
  reports_to(a, b), reports_to(b, c)
  WHERE NOT EXISTS(indirectly_reports_to(a, c))
  => LINK indirectly_reports_to(a, c)
```

## Semantics

### Rule Modifiers

| Modifier | Default | Effect |
|----------|---------|--------|
| `priority: N` | 0 | Higher priority rules fire first |
| `auto` | yes | Rule fires automatically when pattern matches |
| `manual` | no | Rule only fires when explicitly triggered |

### Execution Order

1. Rules execute in **priority order** (highest first)
2. Same-priority rules execute in **declaration order**
3. Rules continue until **quiescence** (no new matches)

### Variable Scoping

Variables bound in the pattern are available in all actions. Variables bound by SPAWN are available to subsequent actions:

```
rule example:
  a: TypeA                           -- a bound from pattern
  =>
  SPAWN b: TypeB { ref = a.name },   -- a available, b bound here
  LINK connects(a, b),               -- a, b available
  SET b.processed = true             -- a, b available
```

### Once-Per-Binding

A rule fires at most once per unique binding within a transaction. This prevents infinite loops.

### Actions

| Action | Effect |
|--------|--------|
| `SPAWN x: Type { attrs }` | Create node, bind to x |
| `KILL x` | Remove node bound to x |
| `LINK edge(a, b) { attrs }` | Create edge between a and b |
| `UNLINK e` | Remove edge bound to e |
| `SET x.attr = value` | Modify attribute |

## Layer 0

### Nodes

```
node _RuleDef [sealed] {
  name: String [required, unique],
  priority: Int = 0,
  auto: Bool = true,
  doc: String?
}

node _ProductionDef [sealed] {}

node _Action [abstract, sealed] {
  order: Int = 0
}

node _SpawnAction : _Action [sealed] {
  var_name: String [required]
}

node _KillAction : _Action [sealed] {
  target_var: String [required]
}

node _LinkAction : _Action [sealed] {
  var_name: String?
}

node _UnlinkAction : _Action [sealed] {
  target_var: String [required]
}

node _SetAction : _Action [sealed] {
  target_var: String [required],
  attr_name: String [required]
}
```

### Edges

```
edge _rule_has_pattern(rule: _RuleDef, pattern: _PatternDef) {}

edge _rule_has_production(rule: _RuleDef, production: _ProductionDef) {}

edge _production_has_action(production: _ProductionDef, action: _Action) {}

edge _spawn_type(action: _SpawnAction, type_expr: _TypeExpr) {}

edge _spawn_attr(action: _SpawnAction, value: _Expr) {
  attr_name: String [required]
}

edge _link_edge_type(action: _LinkAction, edge_type: _EdgeType) {}

edge _link_target(action: _LinkAction, target: _Expr) {
  position: Int [required],
  var_name: String [required]
}

edge _link_attr(action: _LinkAction, value: _Expr) {
  attr_name: String [required]
}

edge _set_value(action: _SetAction, value: _Expr) {}

edge _ontology_declares_rule(ontology: _Ontology, rule: _RuleDef) {}
```

### Constraints

```
constraint _rule_has_pattern:
  r: _RuleDef => EXISTS(p: _PatternDef, _rule_has_pattern(r, p))

constraint _rule_has_production:
  r: _RuleDef => EXISTS(p: _ProductionDef, _rule_has_production(r, p))

constraint _spawn_has_type:
  a: _SpawnAction => EXISTS(t: _TypeExpr, _spawn_type(a, t))

constraint _set_has_value:
  a: _SetAction => EXISTS(v: _Expr, _set_value(a, v))
```

## Examples

### Cascading Status

```
rule cascade_project_archive:
  p: Project, t: Task,
  belongs_to(t, p)
  WHERE p.archived = true AND t.archived = false
  =>
  SET t.archived = true
```

### Notification on Assignment

```
rule notify_on_assignment:
  t: Task, p: Person,
  assigned_to(t, p) AS a
  WHERE a.notified = false
  =>
  SPAWN n: Notification {
    message = "You were assigned: " ++ t.title,
    recipient_id = p._id
  },
  LINK notification_for(n, t),
  SET a.notified = true
```

### Ensure Default Owner

```
rule ensure_project_has_owner:
  p: Project, creator: Person,
  created_by(p, creator)
  WHERE NOT EXISTS(o: Person, owns(p, o))
  =>
  LINK owns(p, creator)
```

## Errors

| Condition | Message |
|-----------|---------|
| Duplicate rule name | Rule `{name}` already defined |
| Unknown type in SPAWN | Unknown type `{type}` in SPAWN action |
| Unknown variable in action | Variable `{var}` not bound in pattern or previous action |
| Invalid edge pattern in LINK | Edge `{edge}` does not connect types `{a}` and `{b}` |
| UNLINK target not an edge | Variable `{var}` is not bound to an edge |
| SET on non-existent attribute | Type `{type}` has no attribute `{attr}` |
| Conflicting modifiers | Rule cannot be both `auto` and `manual` |
| Max actions exceeded | Transaction exceeded maximum action limit |
| Max depth exceeded | Rule chain exceeded maximum depth limit |
