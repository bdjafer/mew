---
spec: constraint-declaration
version: "1.0"
status: draft
category: declaration
capability: schema-definition
requires: [patterns, expressions]
priority: essential
---

# Spec: Constraint Declaration

## Overview

Constraint declarations define invariants that must always hold in the graph. Each constraint specifies a pattern to match and a condition that must be true for all matches. Constraints are the foundation of graph validation, enforcing business rules, data integrity, and structural requirements. Attribute modifiers on node and edge declarations (like `[required]`, `[unique]`, `[>= N]`) compile down to constraints, making constraints the universal validation mechanism.

## Syntax

### Grammar

```ebnf
ConstraintDecl   = DocComment? "constraint" Identifier ConstraintModifiers? ":" Pattern "=>" Expr

ConstraintModifiers = "[" ConstraintModifier ("," ConstraintModifier)* "]"

ConstraintModifier = "soft"
                   | "hard"
                   | "message:" StringLiteral

Pattern          = PatternElement ("," PatternElement)* WhereClause?

PatternElement   = NodePattern | EdgePattern

NodePattern      = Identifier ":" TypeExpr

EdgePattern      = EdgeType TransitiveModifier? "(" TargetList ")" EdgeAlias?

TransitiveModifier = "+" | "*"

TargetList       = Target ("," Target)*

Target           = Identifier | "_"

EdgeAlias        = "AS" Identifier

WhereClause      = "WHERE" Expr

Expr             = <boolean expression that must evaluate to true>
```

### Keywords

| Keyword | Context |
|---------|---------|
| `constraint` | Declaration - introduces a constraint |
| `hard` | Modifier - violation rejects operation (default) |
| `soft` | Modifier - violation logs warning only |
| `message` | Modifier - custom error/warning message |
| `WHERE` | Pattern clause - filters matched elements |
| `=>` | Separates pattern from condition |
| `exists` | Expression - existential quantification |
| `NOT EXISTS` | Expression - negated existence |

### Examples

```mew
-- Simple prohibition constraint
constraint no_self_cause:
  e: Event, causes(e, e)
  => false

-- Constraint with WHERE clause and message
constraint temporal_order [message: "Cause must precede effect"]:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp

-- Soft constraint (warning only)
constraint prefer_description [soft, message: "Tasks should have descriptions"]:
  t: Task
  => t.description != null

-- Existence requirement
constraint task_needs_project [message: "Every task must belong to a project"]:
  t: Task
  => exists(p: Project, belongs_to(t, p))

-- Uniqueness constraint
constraint unique_email:
  p1: Person, p2: Person
  WHERE p1.id != p2.id
  => p1.email != p2.email OR p1.email = null OR p2.email = null
```

## Semantics

### Reading Constraints

A constraint reads as: "For all matches of pattern, condition must hold."

```mew
constraint temporal_order:
  e1: Event, e2: Event, causes(e1, e2)
  => e1.timestamp < e2.timestamp

-- "For all (e1, e2) where e1 causes e2,
--  e1.timestamp must be less than e2.timestamp"
```

The constraint is violated if ANY match exists where the condition is false.

### Constraint Names Required

Every constraint must have a name. Anonymous constraints are not allowed:

```mew
-- Compile error: Missing constraint name
constraint:
  t: Task => t.priority >= 0
-- Error: Constraint name required.
```

**Rationale:**
- Error messages reference constraint names for debugging
- Named constraints are observable and inspectable
- Forces developers to think about what the constraint means

**Good naming conventions:**
```mew
constraint task_priority_valid: ...       -- <entity>_<attribute>_<condition>
constraint temporal_order: ...            -- semantic name
constraint no_self_loop: ...              -- describes what's prevented
constraint task_needs_project: ...        -- describes requirement
```

### Hard vs Soft Constraints

| Modifier | Default | Behavior on Violation |
|----------|---------|----------------------|
| `hard` | Yes | Operation rejected, transaction rolled back |
| `soft` | No | Warning logged, operation proceeds |

**Hard constraint (default):**
```mew
constraint valid_priority:
  t: Task
  => t.priority >= 0 AND t.priority <= 10

SPAWN t: Task { priority = 15 }  -- Error: constraint violated
```

**Soft constraint:**
```mew
constraint prefer_description [soft]:
  t: Task
  => t.description != null

SPAWN t: Task { title = "Test" }  -- Warning logged, task created
```

### Custom Messages

The `message:` modifier provides a human-readable error or warning:

```mew
constraint temporal_order [message: "Cause must precede effect"]:
  e1: Event, e2: Event, causes(e1, e2)
  => e1.timestamp < e2.timestamp

-- On violation:
-- Error: Constraint 'temporal_order' violated: Cause must precede effect
```

### Pattern Matching

Constraints use patterns to identify what to validate:

**Node patterns:** Bind variables to nodes of a type
```mew
constraint valid_age:
  p: Person
  => p.age = null OR p.age >= 0
```

**Edge patterns:** Match relationships
```mew
constraint no_self_assignment:
  t: Task, p: Person, assigned_to(t, p)
  => t.created_by != p.id  -- example logic
```

**Combined patterns:**
```mew
constraint subtask_same_project:
  child: Task, parent: Task, p1: Project, p2: Project,
  subtask_of(child, parent), belongs_to(child, p1), belongs_to(parent, p2)
  => p1.id = p2.id
```

### WHERE Clause

The WHERE clause filters which matches are validated:

```mew
constraint completed_has_timestamp:
  t: Task WHERE t.status = "completed"
  => t.completed_at != null
```

This reads: "For all tasks WHERE status is completed, completed_at must be non-null."

Tasks with other statuses are not validated by this constraint.

### Transitive Patterns in Constraints

Constraints can use transitive patterns for cycle detection:

```mew
constraint no_dependency_cycle:
  t: Task, depends_on+(t, t)
  => false

constraint no_management_cycle:
  p: Person, manages+(p, p)
  => false
```

**Depth limits:** Transitive patterns have a default depth limit of 100. Use explicit depth for bounded checks:

```mew
constraint shallow_acyclic:
  p: Person, parent_of+(p, p) [depth: 20]
  => false
```

### Common Constraint Patterns

**Prohibition (something must not exist):**
```mew
constraint no_self_cause:
  e: Event, causes(e, e)
  => false
```

**Requirement (something must exist):**
```mew
constraint task_has_project:
  t: Task
  => exists(p: Project, belongs_to(t, p))
```

**Implication (if X then Y):**
```mew
constraint completed_has_timestamp:
  t: Task WHERE t.status = "completed"
  => t.completed_at != null
```

**Uniqueness:**
```mew
constraint unique_email:
  p1: Person, p2: Person
  WHERE p1.id != p2.id AND p1.email != null AND p2.email != null
  => p1.email != p2.email
```

**Mutual exclusion:**
```mew
constraint not_both_admin_and_guest:
  p: Person
  => NOT (p.is_admin = true AND p.is_guest = true)
```

**Range validation:**
```mew
constraint valid_priority:
  t: Task WHERE t.priority != null
  => t.priority >= 0 AND t.priority <= 10
```

**Referential integrity:**
```mew
constraint assignment_valid:
  t: Task, p: Person, team: Team, proj: Project,
  assigned_to(t, p), belongs_to(t, proj),
  owns(team, proj)
  => exists(member_of(p, team))
```

### Constraint Evaluation Order

Constraints are checked AFTER rules execute within each transaction:

```
Transaction Execution Order:
1. User mutation (SPAWN, KILL, LINK, UNLINK, SET)
2. Find triggered rules
3. Execute rules in priority order
4. Check constraints (hard constraints fail -> rollback ALL)
5. If rules modified data, go to step 2 (until quiescence)
6. Commit
```

This means rules can "fix" constraint violations:

```mew
constraint task_has_timestamp:
  t: Task => t.created_at != null

rule auto_timestamp [priority: 100]:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()

-- User action:
SPAWN t: Task { title = "Test" }  -- No created_at provided

-- Execution:
-- 1. SPAWN creates task without created_at
-- 2. auto_timestamp rule fires, sets created_at = now()
-- 3. task_has_timestamp constraint checked -> PASSES
-- 4. Commit
```

### Restrictions

**No `now()` in constraints:** Using `now()` in constraint conditions is a compile-time error because constraints must be deterministic:

```mew
-- Compile error: now() not allowed in constraint conditions
constraint recent_only:
  t: Task
  => t.created_at > now() - 86400000
```

**Workarounds:**
1. Use manual rules triggered periodically
2. Handle in application code
3. Store a reference timestamp and compare against that

## Layer 0

### Nodes

```
_ConstraintDef:
  name: String          -- constraint name
  hard: Bool            -- true for hard, false for soft
  message: String?      -- custom error message
  doc: String?          -- documentation comment
```

### Edges

```
_constraint_has_pattern(constraint: _ConstraintDef, pattern: _Pattern)
  -- Associates constraint with its pattern

_constraint_has_condition(constraint: _ConstraintDef, condition: _Expr)
  -- Associates constraint with its condition expression

_pattern_has_element(pattern: _Pattern, element: _PatternElement) {
  position: Int         -- order in pattern
}

_pattern_has_where(pattern: _Pattern, where: _Expr)
  -- Associates pattern with its WHERE clause
```

### Constraints

Constraints on constraints (meta-constraints) are implicit in the type system:
- Constraint names must be unique within an ontology
- Pattern variables must be bound before use in conditions
- Condition expressions must evaluate to boolean

## Examples

### Data Validation Constraints

```mew
-- Required field validation
constraint person_name_required:
  p: Person
  => p.name != null

-- Unique field validation
constraint person_email_unique:
  p1: Person, p2: Person
  WHERE p1.id != p2.id AND p1.email != null AND p2.email != null
  => p1.email != p2.email

-- Range validation
constraint valid_age:
  p: Person WHERE p.age != null
  => p.age >= 0 AND p.age <= 150

-- Enum validation
constraint valid_status:
  t: Task WHERE t.status != null
  => t.status = "todo" OR t.status = "in_progress" OR t.status = "done"

-- Format validation (compiled from [format: email])
constraint person_email_format:
  p: Person WHERE p.email != null
  => is_email(p.email)

-- Length validation
constraint person_name_length:
  p: Person WHERE p.name != null
  => length(p.name) >= 1 AND length(p.name) <= 200
```

### Structural Constraints

```mew
-- No self-loops
constraint no_self_dependency:
  t: Task, depends_on(t, t)
  => false

-- Acyclicity
constraint no_dependency_cycle:
  t: Task, depends_on+(t, t)
  => false

-- Required relationships
constraint task_has_project [message: "Every task must belong to exactly one project"]:
  t: Task
  => exists(p: Project, belongs_to(t, p))

-- Relationship consistency
constraint subtask_same_project [message: "Subtask must be in same project as parent"]:
  child: Task, parent: Task, p1: Project, p2: Project,
  subtask_of(child, parent), belongs_to(child, p1), belongs_to(parent, p2)
  => p1.id = p2.id
```

### Business Rule Constraints

```mew
-- Temporal ordering
constraint cause_precedes_effect [message: "Cause must occur before effect"]:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp

-- State machine transitions
constraint valid_status_transition [message: "Invalid status transition"]:
  t: Task
  WHERE t.status = "done"
  => t.completed_at != null

-- Business logic
constraint manager_must_be_member:
  manager: Person, team: Team, manages(manager, team)
  => exists(member_of(manager, team))

-- Soft recommendation
constraint recommend_description [soft, message: "Tasks should have descriptions for clarity"]:
  t: Task
  => t.description != null AND length(t.description) >= 10
```

### Complete Constraint Example with Context

```mew
ontology TaskManagement {

  node Person {
    name: String [required],
    email: String [required, unique, format: email]
  }

  node Project {
    name: String [required],
    deadline: Timestamp?
  }

  node Task {
    title: String [required],
    status: String [in: ["todo", "in_progress", "done"]] = "todo",
    priority: Int [0..10] = 5,
    completed_at: Timestamp?
  }

  edge belongs_to(task: Task, project: Project) [task -> 1]
  edge assigned_to(task: Task, person: Person) [task -> 0..1]
  edge depends_on(downstream: Task, upstream: Task) [no_self, acyclic]
  edge subtask_of(child: Task, parent: Task) [no_self, acyclic, child -> 0..1]

  -- Explicit constraints (beyond what modifiers provide)

  constraint completed_has_timestamp [message: "Completed tasks must have a completion timestamp"]:
    t: Task WHERE t.status = "done"
    => t.completed_at != null

  constraint subtask_same_project [message: "Subtasks must be in the same project as their parent"]:
    child: Task, parent: Task, p1: Project, p2: Project,
    subtask_of(child, parent), belongs_to(child, p1), belongs_to(parent, p2)
    => p1.id = p2.id

  constraint dependency_same_project [message: "Task dependencies must be within the same project"]:
    t1: Task, t2: Task, p1: Project, p2: Project,
    depends_on(t1, t2), belongs_to(t1, p1), belongs_to(t2, p2)
    => p1.id = p2.id

  constraint blocked_has_dependency [soft, message: "Blocked tasks should have unfinished dependencies"]:
    t: Task WHERE t.status = "blocked"
    => exists(upstream: Task, depends_on(t, upstream) WHERE upstream.status != "done")

  constraint high_priority_needs_assignee [soft, message: "High priority tasks should be assigned"]:
    t: Task WHERE t.priority >= 8
    => exists(p: Person, assigned_to(t, p))

  constraint deadline_not_passed [soft, message: "Project deadline should not be in the past"]:
    p: Project WHERE p.deadline != null
    => p.deadline > p.created_at  -- assuming created_at exists
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Missing constraint name | Constraint name required. Add a name: `constraint <name>: ...` |
| Duplicate constraint name | Constraint `<name>` already defined in this ontology |
| Invalid pattern variable | Variable `<name>` not bound in pattern |
| Type mismatch in condition | Cannot compare `<type1>` with `<type2>` in constraint condition |
| Non-boolean condition | Constraint condition must evaluate to boolean, got `<type>` |
| now() in constraint | `now()` cannot appear in constraint conditions. Constraints must be deterministic |
| Unbound variable in condition | Variable `<name>` used in condition but not defined in pattern |
| Hard constraint violated | Constraint `<name>` violated: `<message or default>` |
| Soft constraint violated | Warning: Constraint `<name>` violated: `<message or default>` |
| Transitive pattern too deep | Transitive pattern `<pattern>` reached depth limit `<limit>` |
| Empty pattern | Constraint must have at least one pattern element |
| Conflicting modifiers | Cannot use both [hard] and [soft] on the same constraint |
