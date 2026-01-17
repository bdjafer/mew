---
spec: edge_symmetry
version: "1.0"
status: stable
category: modifier
requires: []
priority: specialized
---

# Spec: Edge Symmetry

## Overview

The `[symmetric]` modifier declares that an edge is order-independent: `edge(a, b)` equals `edge(b, a)`. Useful for mutual relationships like friendship, similarity, or collaboration where directionality is meaningless.

**Why needed:** Without symmetry, modeling "A is friend of B" requires either two edges (redundant) or application-level handling of both directions. Symmetric edges handle this at the storage and query level.

---

## Syntax

### Grammar
```ebnf
EdgeModifier = ... | "symmetric"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `symmetric` | Edge modifier |

### Examples
```
edge friend_of(a: Person, b: Person) [symmetric]

edge similar_to(x: Document, y: Document) [symmetric] {
  similarity_score: Float [required]
}

edge collaborates(p1: Person, p2: Person) [symmetric]
```

---

## Semantics

### Type Requirement

Symmetric edges require **identical types** on both parameters:
```
-- VALID: same type
edge friend_of(a: Person, b: Person) [symmetric]

-- INVALID: different types
edge owns(person: Person, item: Item) [symmetric]
-- ERROR: Cannot apply [symmetric] to edge with different parameter types
```

### Storage

Only one edge is stored, with targets in canonical order (by ID):
```
-- Creating friend_of(alice, bob) where alice.id > bob.id
-- Stored as: friend_of(bob, alice)
```

### Creation Deduplication

Creating a symmetric edge that already exists (in either order) returns the existing edge:
```
LINK friend_of(alice, bob)  -- Creates edge
LINK friend_of(bob, alice)  -- Returns existing edge ID, no duplicate
```

### Matching

Patterns match regardless of argument order:
```
edge friend_of(a: Person, b: Person) [symmetric]

MATCH friend_of(x, y) WHERE x.name = "Alice"
-- Finds edges where Alice is in either position
```

### Attributes

Attributes belong to the single stored edge:
```
edge similar_to(a: Doc, b: Doc) [symmetric] {
  score: Float
}

LINK similar_to(doc1, doc2) { score = 0.8 }
-- Accessing via either order returns same attribute
```

### Arity Restriction

`[symmetric]` only applies to binary edges (arity = 2):
```
-- INVALID
edge meeting(a: Person, b: Person, c: Person) [symmetric]
-- ERROR: [symmetric] only valid for binary edges
```

---

## Layer 0

### Nodes

Extends `_EdgeType` with a `symmetric` field:
```
node _EdgeType [sealed] {
  ...
  name: String [required, unique],
  arity: Int [required],
  symmetric: Bool = false,          -- Added by this feature
  doc: String?
  ...
}
```

### Edges

None.

### Constraints
```
constraint _symmetric_requires_same_types:
  e: _EdgeType,
  v1: _VarDef, v2: _VarDef,
  t1: _TypeExpr, t2: _TypeExpr,
  _edge_has_position(e, v1) AS p1,
  _edge_has_position(e, v2) AS p2,
  _var_has_type(v1, t1),
  _var_has_type(v2, t2)
  WHERE e.symmetric = true AND p1.position = 0 AND p2.position = 1
  => types_equal(t1, t2)

constraint _symmetric_requires_binary:
  e: _EdgeType WHERE e.symmetric = true
  => e.arity = 2
```

---

## Compilation
```
edge friend_of(a: Person, b: Person) [symmetric]
```

Compiles to:
```
_EdgeType node:
  name: "friend_of"
  arity: 2
  symmetric: true
```

---

## Examples

### Social Network
```
ontology Social {
  node Person {
    name: String [required]
  }
  
  edge friend_of(a: Person, b: Person) [symmetric] {
    since: Timestamp = now()
  }
  
  edge blocked(blocker: Person, blocked: Person)  -- NOT symmetric
}

-- Usage
LINK friend_of(alice, bob)
-- Both can query:
MATCH friend_of(alice, x) RETURN x  -- finds bob
MATCH friend_of(bob, x) RETURN x    -- finds alice
```

### Document Similarity
```
ontology Documents {
  node Document {
    title: String [required]
  }
  
  edge similar_to(a: Document, b: Document) [symmetric] {
    score: Float [>= 0.0, <= 1.0]
  }
  
  constraint similarity_threshold:
    similar_to(a, b) AS s
    => s.score >= 0.5
}
```

### Collaboration
```
ontology Workspace {
  node Person { name: String [required] }
  node Project { name: String [required] }
  
  edge collaborates(p1: Person, p2: Person) [symmetric] {
    project: String,
    started: Timestamp = now()
  }
  
  -- Find all collaborators of Alice
  -- MATCH collaborates(alice, other) RETURN other
}
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Different parameter types | `"[symmetric] requires identical parameter types"` |
| Non-binary edge | `"[symmetric] only valid for binary edges (arity 2)"` |

---

*End of Spec: Edge Symmetry*