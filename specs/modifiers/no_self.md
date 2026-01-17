---
spec: no_self
version: "1.0"
status: draft
category: modifier
capability: no_self
requires: []
priority: common
---

# Spec: No Self

## Overview

The `[no_self]` modifier prevents an edge from connecting a node to itself. It ensures that self-loops are forbidden for edges where such connections would be semantically invalid.

**Why needed:** Many relationships are inherently non-reflexive: a task cannot depend on itself, a person cannot be their own friend, a node cannot be its own parent. The no_self modifier enforces this structural constraint.

---

## Syntax

### Grammar
```ebnf
EdgeModifier = ... | "no_self"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `no_self` | Edge modifier |

### Examples
```
edge depends_on(from: Task, to: Task) [no_self]

edge parent_of(parent: Person, child: Person) [no_self, acyclic]

edge follows(follower: User, followee: User) [no_self]
```

---

## Semantics

### Self-Loop Detection

A self-loop exists when the same node appears in multiple positions where the types are compatible:
```
edge depends_on(from: Task, to: Task) [no_self]

SPAWN t: Task { title = "Test" }
LINK depends_on(t, t)  -- ERROR: self-loop detected
```

### Applicability

The `[no_self]` modifier is meaningful only for edges where multiple parameters have compatible types:

```
-- Applicable: same type on both ends
edge depends_on(from: Task, to: Task) [no_self]
edge friend_of(a: Person, b: Person) [no_self]
edge links_to(source: Page, target: Page) [no_self]

-- NOT applicable: different types (no possibility of self-loop)
edge assigned_to(task: Task, person: Person) [no_self]  -- Warning: no_self has no effect
```

### With Type Inheritance

Self-loop detection considers type inheritance:
```
node Entity {}
node Person : Entity {}

edge related_to(a: Entity, b: Entity) [no_self]

SPAWN p: Person { name = "Alice" }
LINK related_to(p, p)  -- ERROR: self-loop (Person is an Entity)
```

### N-ary Edges

For edges with more than two parameters, self-loop detection applies to any pair of compatible-type parameters:
```
edge meeting(a: Person, b: Person, c: Person) [no_self]

SPAWN p1: Person { name = "Alice" }
SPAWN p2: Person { name = "Bob" }

LINK meeting(p1, p1, p2)  -- ERROR: a = b (self-loop)
LINK meeting(p1, p2, p1)  -- ERROR: a = c (self-loop)
LINK meeting(p1, p2, p2)  -- ERROR: b = c (self-loop)
LINK meeting(p1, p2, p1)  -- ERROR: multiple self-loops
```

### Compilation to Constraint
```
edge depends_on(from: Task, to: Task) [no_self]
```

Compiles to:
```
constraint depends_on_no_self:
  t: Task, depends_on(t, t)
  => false
```

For n-ary edges, a constraint is generated for each pair of compatible parameters.

---

## Layer 0

### Constraints

For binary edge `edge E(a: T, b: T) [no_self]`:
```
constraint E_no_self:
  x: T, E(x, x)
  => false
```

For n-ary edge `edge E(a: T, b: T, c: T) [no_self]`:
```
constraint E_no_self_a_b:
  x: T, E(x, x, _)
  => false

constraint E_no_self_a_c:
  x: T, E(x, _, x)
  => false

constraint E_no_self_b_c:
  x: T, E(_, x, x)
  => false
```

---

## Examples

### Task Dependencies
```
ontology Tasks {
  node Task {
    title: String [required],
    status: String [in: ["todo", "in_progress", "done"]]
  }

  -- Tasks cannot depend on themselves
  edge depends_on(downstream: Task, upstream: Task) [no_self, acyclic]
}

SPAWN t1: Task { title = "Task 1" }
SPAWN t2: Task { title = "Task 2" }

LINK depends_on(t1, t2)  -- OK
LINK depends_on(t1, t1)  -- ERROR: Task cannot depend on itself
```

### Social Network
```
ontology Social {
  node User {
    username: String [required, unique]
  }

  -- Cannot follow yourself
  edge follows(follower: User, followee: User) [no_self]

  -- Cannot friend yourself
  edge friend_of(a: User, b: User) [no_self, symmetric]

  -- Cannot block yourself
  edge blocks(blocker: User, blocked: User) [no_self]
}
```

### Organizational Hierarchy
```
ontology Organization {
  node Employee {
    name: String [required],
    title: String
  }

  -- Cannot manage yourself
  edge manages(manager: Employee, report: Employee) [no_self, acyclic]

  -- Cannot mentor yourself
  edge mentors(mentor: Employee, mentee: Employee) [no_self]
}
```

### Document Links
```
ontology Wiki {
  node Page {
    title: String [required, unique],
    content: String
  }

  -- Page cannot link to itself
  edge links_to(from_page: Page, to_page: Page) [no_self]

  -- Page cannot be its own parent
  edge parent_of(parent: Page, child: Page) [no_self, acyclic, child -> 0..1]
}
```

### Combining with Other Modifiers
```
edge depends_on(a: Task, b: Task) [
  no_self,          -- Cannot self-depend
  acyclic,          -- No circular dependencies
  unique            -- No duplicate edges
]

edge married_to(a: Person, b: Person) [
  no_self,          -- Cannot marry yourself
  symmetric,        -- Marriage is symmetric
  a -> 0..1         -- At most one spouse
]
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Self-loop on LINK | `"Cannot create self-loop: <edge>(<node>, <node>)"` |
| Inapplicable modifier | `"Warning: [no_self] has no effect on edge '<edge>' with different parameter types"` |

---

*End of Spec: No Self*
