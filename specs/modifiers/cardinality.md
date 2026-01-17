---
spec: cardinality
version: "1.0"
status: stable
category: modifier
capability: cardinality
requires: []
priority: common
---

# Spec: Cardinality Constraints

## 1. Overview

Cardinality modifiers constrain how many edges of a given type can connect to a node, enabling one-to-one, one-to-many, and many-to-many relationship modeling.

**Why needed:** Most data models have cardinality requirements. "A task has at most one assignee." "A project must have at least one task." Without cardinality modifiers, these require verbose explicit constraints.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
EdgeModifiers = "[" EdgeModifier ("," EdgeModifier)* "]"

EdgeModifier = ... | CardinalityModifier

CardinalityModifier = Identifier "->" Cardinality

Cardinality = 
    IntLiteral                      -- exactly N
  | IntLiteral ".." IntLiteral      -- N to M (inclusive)
  | IntLiteral ".." "*"             -- N or more
```

### 2.2 Keywords Added

None.

### 2.3 Examples
```
-- Each task has at most one assignee
edge assigned_to(task: Task, person: Person) [task -> 0..1]

-- Each task belongs to exactly one project
edge belongs_to(task: Task, project: Project) [task -> 1]

-- Each project has at least one task
edge contains(project: Project, task: Task) [project -> 1..*]

-- Bidirectional: one-to-one marriage
edge married_to(a: Person, b: Person) [a -> 0..1, b -> 0..1]

-- Each person manages 0-10 people
edge manages(manager: Person, report: Person) [manager -> 0..10]
```

---

## 3. Semantics

### 3.1 Cardinality Values

| Syntax | Meaning |
|--------|---------|
| `N` | Exactly N edges |
| `N..M` | Between N and M edges (inclusive) |
| `N..*` | N or more edges |
| `0..1` | Optional (at most one) |
| `1` | Required (exactly one) |
| `1..*` | At least one |

### 3.2 Which Parameter

The parameter name specifies which end is constrained:
```
edge assigned_to(task: Task, person: Person) [task -> 0..1]
```

This means: "For each task, there are 0 or 1 assigned_to edges."

Equivalently: "A task can be assigned to at most one person."

### 3.3 Multiple Cardinalities

Multiple parameters can be constrained:
```
edge belongs_to(task: Task, project: Project) [
  task -> 1,        -- each task belongs to exactly one project
  project -> 0..*   -- each project has any number of tasks (implicit)
]
```

### 3.4 Enforcement Timing

| Component | When Enforced |
|-----------|---------------|
| Maximum (`..M`) | Immediately on LINK |
| Minimum (`N..`) | At transaction COMMIT |

This allows staged creation within a transaction:
```
BEGIN
  SPAWN t: Task { title = "Test" }   -- OK: no project yet
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(t, p)              -- OK: now linked
COMMIT                               -- Cardinality checked here
```

### 3.5 Implicit Cardinality

Unspecified cardinalities default to `0..*` (no constraint).
```
edge tagged(task: Task, tag: Tag)
-- Equivalent to: [task -> 0..*, tag -> 0..*]
```

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _CardinalityConstraint [sealed] {
  param_name: String [required],    -- which parameter
  min_count: Int = 0,
  max_count: Int?,                  -- null means unbounded
}
```

### 4.2 Edge Types
```
edge _edge_type_cardinality(
  edge_type: _EdgeType,
  cardinality: _CardinalityConstraint
) {}
```

### 4.3 Constraints

None at Layer 0. Cardinality compiles to user-level constraints.

---

## 5. Compilation

### 5.1 Maximum Cardinality
```
edge assigned_to(task: Task, person: Person) [task -> 0..1]
```

Compiles to constraint:
```
constraint assigned_to_task_max_1:
  t: Task, p1: Person, p2: Person,
  assigned_to(t, p1), assigned_to(t, p2)
  WHERE p1.id != p2.id
  => false
```

### 5.2 Minimum Cardinality
```
edge belongs_to(task: Task, project: Project) [task -> 1]
```

Compiles to constraint (checked at commit):
```
constraint belongs_to_task_min_1:
  t: Task
  => EXISTS(p: Project, belongs_to(t, p))
```

### 5.3 Exact Cardinality

`[task -> 1]` (exactly one) compiles to both min and max constraints.

### 5.4 Layer 0 Structure
```
_CardinalityConstraint node:
  param_name: "task"
  min_count: 0
  max_count: 1

_edge_type_cardinality edge:
  (assigned_to_edge_type, cardinality_constraint)
```

---

## 6. Examples

### 6.1 One-to-Many: Project Tasks
```
edge belongs_to(task: Task, project: Project) [
  task -> 1           -- each task in exactly one project
]

-- project -> 0..* is implicit (any number of tasks)
```

### 6.2 One-to-One: User Profile
```
edge has_profile(user: User, profile: Profile) [
  user -> 0..1,       -- each user has at most one profile
  profile -> 1        -- each profile belongs to exactly one user
]
```

### 6.3 Many-to-Many with Limits
```
edge enrolled_in(student: Student, course: Course) [
  student -> 1..6,    -- each student takes 1-6 courses
  course -> 10..30    -- each course has 10-30 students
]
```

### 6.4 Self-Referential with Cardinality
```
edge reports_to(employee: Person, manager: Person) [
  employee -> 0..1    -- each employee has at most one manager
]
```

---

## 7. Errors

| Condition | Message |
|-----------|---------|
| Unknown parameter name | `"Parameter 'x' not in edge signature"` |
| min > max | `"Invalid cardinality: min (N) > max (M)"` |
| Negative cardinality | `"Cardinality cannot be negative"` |
| Maximum violated on LINK | `"Cardinality exceeded: 'task' already has M 'assigned_to' edges"` |
| Minimum violated at COMMIT | `"Cardinality not satisfied: 'task' requires at least N 'belongs_to' edges"` |

---

*End of Feature: Cardinality Constraints*