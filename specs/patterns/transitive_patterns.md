---
feature: transitive_patterns
version: "1.0"
status: stable
category: pattern
requires: []
priority: common
---

# Feature: Transitive Patterns

## 1. Overview

Transitive patterns match paths of one or more (`+`) or zero or more (`*`) edges, enabling reachability queries and cycle detection without explicit recursion.

**Why needed:** Graph queries frequently need path-based reasoning. "Is A reachable from B?" "Are there any cycles?" "Find all ancestors." Without transitive patterns, users must implement iterative queries in application code.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
EdgePattern = Identifier TransitiveOp? "(" Targets ")" ("AS" Identifier)?

TransitiveOp = "+" | "*"

DepthModifier = "[" "depth" ":" IntLiteral "]"
```

Depth modifier attaches to edge pattern in WHERE:
```ebnf
-- Used in WHERE clause context
TransitiveConstraint = EdgePattern DepthModifier
```

### 2.2 Keywords Added

None. Uses existing `+` and `*` as postfix operators on edge type names.

### 2.3 Examples
```
-- One or more hops (transitive closure)
causes+(e1, e2)

-- Zero or more hops (reflexive transitive closure)
parent_of*(p1, p2)

-- With depth limit
MATCH parent_of+(ancestor, person) [depth: 10]

-- In constraint (cycle detection)
constraint no_cycle:
  p: Person, parent_of+(p, p)
  => false
```

---

## 3. Semantics

### 3.1 Operators

| Operator | Name | Matches |
|----------|------|---------|
| `E+(a, b)` | Transitive closure | Path of 1+ edges from a to b |
| `E*(a, b)` | Reflexive transitive | Path of 0+ edges (includes a = b) |

### 3.2 Evaluation

`causes+(a, b)` matches if there exists a sequence:
```
a → x₁ → x₂ → ... → xₙ → b   (n ≥ 0, so at least one edge)
```

`causes*(a, b)` matches if:
- `a = b` (zero edges), OR
- `causes+(a, b)` matches (one or more edges)

### 3.3 Depth Limits

**Default depth limit:** 100 hops.

Configurable per-pattern:
```
parent_of+(a, b) [depth: 20]
```

**Behavior at limit:**
- Paths within limit: returned
- Paths exceeding limit: truncated with warning
- Cycles: terminated when revisiting a node

### 3.4 Cycle Handling

Cycles are detected and handled gracefully:
```
-- Graph: A → B → C → A (cycle)
MATCH follows+(a, b) WHERE a.name = "A"
-- Returns: (A,B), (A,C), (A,A)
-- Path A→B→C→A terminates at A (already visited)
```

No infinite loops occur.

### 3.5 Typing

Both endpoints must be compatible with the edge type's signature:
```
edge parent_of(parent: Person, child: Person)

parent_of+(x, y)
-- x and y must both be Person (or subtype)
```

### 3.6 Implicit Variable Binding

Unlike regular edge patterns which require targets to reference previously declared node pattern variables, transitive patterns support **implicit variable binding**. When an undeclared variable appears as a target in a transitive pattern, it is automatically bound to the appropriate type based on the edge parameter at that position.

```
edge follows(follower: User, followed: User)

-- Explicit binding (always valid)
MATCH u: User, follows+(u, u)
RETURN u.username

-- Implicit binding (valid only in transitive patterns)
MATCH follows+(u, u)
RETURN u.username
-- u is implicitly bound to User based on edge signature
```

**Why implicit binding for transitive patterns?**

Cycle detection is a primary use case for transitive patterns. The pattern `edge+(x, x)` inherently constrains `x` to the edge's endpoint type. Requiring explicit declarations like `MATCH x: T, edge+(x, x)` would be redundant — the type is already determined by the edge definition.

**Rules:**
- Implicit binding applies only to transitive patterns (`+` or `*`)
- The variable type is inferred from the edge parameter type at the corresponding position
- If the edge parameter type is `any`, the variable has type `AnyNodeRef`
- Once implicitly bound, the variable is available in subsequent clauses (WHERE, RETURN)

### 3.7 Restrictions

- Cannot bind intermediate nodes (no path variable)
- Use sparingly in constraints due to performance cost
- For complex path queries, use WALK statement (separate feature)

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _TransitiveModifier [sealed] {
  closure_type: String [required],   -- "+" or "*"
  depth_limit: Int?                   -- null means default (100)
}
```

### 4.2 Edge Types
```
edge _edge_pattern_transitive(
  edge_pattern: _EdgePattern,
  modifier: _TransitiveModifier
) {}
```

### 4.3 Constraints

None.

---

## 5. Compilation

### 5.1 Transitive Pattern
```
parent_of+(ancestor, descendant)
```

Compiles to:
```
_EdgePattern node:
  negated: false

_edge_pattern_type edge:
  (edge_pattern, parent_of_edge_type)

_edge_pattern_target edges:
  (edge_pattern, ancestor_var) { position: 0 }
  (edge_pattern, descendant_var) { position: 1 }

_TransitiveModifier node:
  closure_type: "+"
  depth_limit: null

_edge_pattern_transitive edge:
  (edge_pattern, transitive_modifier)
```

### 5.2 With Depth Limit
```
parent_of+(a, b) [depth: 20]
```

Compiles to same structure with:
```
_TransitiveModifier node:
  closure_type: "+"
  depth_limit: 20
```

---

## 6. Examples

### 6.1 Cycle Detection Constraint
```
constraint no_dependency_cycle:
  t: Task, depends_on+(t, t)
  => false
```

### 6.2 Cycle Detection Query (Implicit Binding)
```
-- Find all users in a follow cycle (using implicit variable binding)
MATCH follows+(u, u)
RETURN u.username AS in_cycle
```

### 6.3 Ancestor Query
```
MATCH
  ancestor: Person,
  descendant: Person,
  parent_of+(ancestor, descendant)
WHERE descendant.name = "Alice"
RETURN ancestor.name
```

### 6.4 Reachability Check
```
MATCH
  start: Node,
  end: Node,
  connected_to*(start, end)
WHERE start.id = $startId AND end.id = $endId
RETURN start, end
```

### 6.5 Bounded Depth
```
-- Find managers up to 5 levels up
MATCH
  employee: Person,
  manager: Person,
  reports_to+(employee, manager) [depth: 5]
WHERE employee.name = "Bob"
RETURN manager.name
```

### 6.6 Reflexive Use Case
```
-- Find node and all its descendants (including self)
MATCH
  root: Category,
  node: Category,
  subcategory_of*(node, root)
WHERE root.name = "Electronics"
RETURN node.name
```

---

## 7. Errors

| Code | Condition | Message |
|------|-----------|---------|
| E3101 | Depth limit ≤ 0 | `"Depth limit must be positive"` |
| E3102 | Depth limit exceeds maximum | `"Depth limit N exceeds maximum M"` |
| E3103 | Transitive on non-binary edge | `"Transitive patterns require binary edges"` |
| W3104 | Depth limit reached during evaluation | `"Transitive pattern reached depth limit; results may be incomplete"` |

---

## 8. Performance Notes

Transitive pattern evaluation is O(V + E) per query in the worst case.

| Graph Size | Typical Cost | Recommendation |
|------------|--------------|----------------|
| < 1,000 nodes | < 1ms | Use freely |
| 1,000 – 10,000 | 1–10ms | Generally fine |
| 10,000 – 100,000 | 10–100ms | Consider depth limits |
| > 100,000 | 100ms+ | Use application-level or WALK |

For constraints with transitive patterns, prefer explicit depth limits.

---

*End of Feature: Transitive Patterns*