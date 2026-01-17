---
spec: acyclic_edges
version: "1.0"
status: stable
category: modifier
capability: acyclic_edges
requires: [transitive_patterns]
priority: specialized
---

# Spec: Acyclic Edges

## Overview

The `[acyclic]` modifier prevents cycles through an edge type. If `parent_of(a, b)` and `parent_of(b, c)` exist, then `parent_of(c, a)` is forbidden. Essential for hierarchies, dependencies, and directed acyclic graphs (DAGs).

**Why needed:** Many domains require acyclic structures: org charts, task dependencies, category trees, inheritance. Without engine-level enforcement, cycles must be detected in application code.

---

## Syntax

### Grammar
```ebnf
EdgeModifier = ... | "acyclic"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `acyclic` | Edge modifier |

### Examples
```
edge parent_of(parent: Person, child: Person) [acyclic]

edge depends_on(downstream: Task, upstream: Task) [acyclic]

edge subcategory_of(child: Category, parent: Category) [acyclic]
```

---

## Semantics

### Constraint

`[acyclic]` compiles to a constraint using transitive closure:
```
constraint <edge>_acyclic:
  x: <Type>, <edge>+(x, x)
  => false
```

This forbids any path from a node back to itself.

### Cycle Detection

Cycles are detected at LINK time:
```
edge parent_of(parent: Person, child: Person) [acyclic]

LINK parent_of(alice, bob)   -- OK
LINK parent_of(bob, charlie) -- OK
LINK parent_of(charlie, alice) -- ERROR: would create cycle
```

### Transitivity

The check follows transitive paths:
```
-- Existing: A → B → C → D
LINK parent_of(D, A)  -- ERROR: creates cycle D → A → B → C → D
```

### Type Restriction

`[acyclic]` requires both parameters to have the same type (or compatible types through inheritance):
```
-- VALID: same type
edge parent_of(parent: Person, child: Person) [acyclic]

-- VALID: compatible types (Employee inherits Person)
edge manages(manager: Person, report: Employee) [acyclic]

-- INVALID: unrelated types (no cycle possible anyway)
edge owns(person: Person, item: Item) [acyclic]
-- WARNING: [acyclic] has no effect on edges between different types
```

### Performance Warning

Cycle detection is O(V+E) per LINK in the worst case. The compiler emits a warning:
```
WARNING: Edge 'depends_on' uses [acyclic].
         Cycle detection may be expensive for large graphs.
         Consider depth limits for performance.
```

### Depth Limit Configuration

Engine configuration controls cycle detection limits:
```
-- Maximum nodes to traverse for acyclic check
engine.acyclic_check_limit = 10000  -- default

-- Behavior when limit exceeded
engine.acyclic_check_overflow = "error"  -- default: fail safely
                              | "skip"   -- dangerous: allow potential cycles
```

---

## Layer 0

### Nodes

No new node types.

### Edges

None.

### Constraints

The `[acyclic]` modifier compiles to a constraint using transitive patterns (from `transitive_patterns` spec):
```
-- For: edge parent_of(parent: Person, child: Person) [acyclic]
-- Generates:

constraint parent_of_acyclic:
  p: Person, parent_of+(p, p)
  => false
```

---

## Compilation
```
edge parent_of(parent: Person, child: Person) [acyclic]
```

Compiles to:
```
_EdgeType node:
  name: "parent_of"
  arity: 2

_ConstraintDef node:
  name: "parent_of_acyclic"
  hard: true
  message: "Cycle detected in 'parent_of'"

-- Pattern with transitive edge
_PatternDef with _EdgePattern:
  edge_type: "parent_of"
  transitive: "+"
  targets: [p, p]  -- same variable both positions

-- Condition: false (any match violates)
_LiteralExpr:
  value_type: "Bool"
  value_string: "false"
```

---

## Examples

### Organizational Hierarchy
```
ontology OrgChart {
  node Person {
    name: String [required],
    title: String
  }
  
  edge reports_to(employee: Person, manager: Person) [acyclic] {
    since: Timestamp = now()
  }
  
  -- CEO reports to no one
  -- Manager cannot report to their own reports
}
```

### Task Dependencies
```
ontology TaskManagement {
  node Task {
    title: String [required],
    status: String = "todo"
  }
  
  edge depends_on(downstream: Task, upstream: Task) [acyclic]
  
  constraint dependency_not_done:
    t1: Task, t2: Task, depends_on(t1, t2)
    WHERE t1.status = "done"
    => t2.status = "done"
}
```

### Category Tree
```
ontology Taxonomy {
  node Category {
    name: String [required, unique]
  }
  
  edge subcategory_of(child: Category, parent: Category) [acyclic]
  
  -- Single root constraint
  constraint single_root:
    c: Category
    => c.name = "Root" OR EXISTS(subcategory_of(c, _))
}
```

### Combining with No Self
```
edge parent_of(parent: Person, child: Person) [acyclic]
-- [acyclic] implies no self-loops (a cycle of length 1)
-- No need for separate [no_self_loops]
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Cycle detected | `"Cycle detected in 'edge_name': A → B → ... → A"` |
| Check limit exceeded | `"Acyclic check limit exceeded (10000 nodes)"` |
| Incompatible types | `"[acyclic] requires compatible parameter types"` |

---

*End of Spec: Acyclic Edges*