# HOHG Language Specification

## Part IV: Meta Mode

**Version:** 1.0
**Status:** Draft
**Scope:** Reflection capabilities for dynamic typing, schema access, and self-modification

---

# 35. Meta Mode Overview

## 35.1 Purpose

Meta Mode enables **reflection** — the ability to reason about and manipulate structure when types are not known at compile time.

**Normal Mode:**
- Types known at compile time
- Full static optimization
- Fast, specialized execution

**Meta Mode:**
- Types discovered at runtime
- Generic patterns and introspection
- Schema access and modification
- Slower but essential for AGI

## 35.2 The Level Model

```
┌─────────────────────────────────────────────────────────────────────┐
│ LEVEL 0: Meta-Meta (Layer 0)                                        │
│                                                                      │
│ Fixed structures that define what schemas can contain:             │
│ • _NodeType, _EdgeType, _AttributeDef                              │
│ • _ConstraintDef, _RuleDef, _PatternDef                            │
│ • _VarDef, _Expr, etc.                                              │
│                                                                      │
│ Rarely modified. Defines the "physics" of ontologies.              │
├─────────────────────────────────────────────────────────────────────┤
│ LEVEL 1: Meta (User Ontologies)                                     │
│                                                                      │
│ User-defined types, edges, constraints, rules:                     │
│ • Person, Task, Project (node types)                               │
│ • causes, assigned_to, belongs_to (edge types)                     │
│ • temporal_order, unique_email (constraints)                       │
│                                                                      │
│ Instances of Level 0 types. Modified via META mode.                │
├─────────────────────────────────────────────────────────────────────┤
│ LEVEL 2: Instance (Data)                                            │
│                                                                      │
│ Actual nodes and edges, including higher-order:                    │
│ • alice: Person, task_123: Task                                    │
│ • causes(#event_a, #event_b)                                       │
│ • confidence(#causes_edge, 0.9) ← higher-order, same level        │
│                                                                      │
│ Queried and modified via Normal mode. Full optimization.           │
└─────────────────────────────────────────────────────────────────────┘
```

## 35.3 Higher-Order vs Meta

**These are different concepts:**

| Concept | Meaning | Mode Required |
|---------|---------|---------------|
| **Higher-order edges** | Edges targeting edges. Instance data at Level 2. | Normal (if types known) |
| **Meta operations** | Reflection, generic types, schema access. | META |

```
-- Higher-order WITHOUT meta (types known):
MATCH causes(e1, e2) AS c, confidence(c, level)
WHERE level > 0.8
RETURN e1, e2

-- Higher-order WITH meta (types unknown):
META MATCH meta: edge<any>(#some_edge, _)
RETURN meta, edge_type(meta)
```

## 35.4 Syntax

All META operations are prefixed with the `META` keyword:

```
MetaStatement =
    "meta" MatchStmt
  | "meta" WalkStmt
  | "meta" SpawnStmt
  | "meta" KillStmt
  | "meta" LinkStmt
  | "meta" UnlinkStmt
  | "meta" SetStmt
```

## 35.5 When META is Required

| Situation | META Required? |
|-----------|----------------|
| Query with known types | No |
| Query higher-order with known types | No |
| Query with `edge<any>` wildcard | Yes |
| Use introspection functions on unbound | Yes |
| Access Layer 0 types | Yes |
| Access Level 1 schema | Yes |
| Create new types at runtime | Yes |
| Modify rules/constraints | Yes |

---

# 36. Edge Introspection Functions

## 36.1 Overview

Introspection functions examine edge structure at runtime.

```
IntrospectionFunction =
    "arity" "(" Expr ")"
  | "targets" "(" Expr ")"
  | "target" "(" Expr "," IntLiteral ")"
  | "has_target" "(" Expr "," Expr ")"
  | "edge_type" "(" Expr ")"
  | "source_types" "(" Expr ")"
  | "is_higher_order" "(" Expr ")"
  | "edges_about" "(" Expr ")"
```

## 36.2 Function Reference

### 36.2.1 arity(e)

Returns the number of targets in an edge.

```
Signature: edge<any> → Int

MATCH causes(e1, e2) AS c
RETURN arity(c)  -- 2

META MATCH e: edge<any>
WHERE arity(e) > 2
RETURN e  -- all hyperedges with 3+ targets
```

### 36.2.2 targets(e)

Returns all targets of an edge as a list.

```
Signature: edge<any> → List<Node | Edge>

MATCH meeting(org, p1, p2, room) AS m
RETURN targets(m)  -- [org, p1, p2, room]

META MATCH e: edge<any>
RETURN e, targets(e)
```

### 36.2.3 target(e, n)

Returns the nth target (0-indexed).

```
Signature: edge<any> × Int → Node | Edge

MATCH causes(from, to) AS c
RETURN target(c, 0), target(c, 1)  -- from, to

META MATCH e: edge<any>
WHERE arity(e) >= 2
RETURN target(e, 0), target(e, 1)
```

Out-of-bounds returns `null`.

### 36.2.4 has_target(e, t)

Returns true if the edge includes the target.

```
Signature: edge<any> × (Node | Edge) → Bool

MATCH meeting(_, _, _, _) AS m
WHERE has_target(m, #alice)
RETURN m  -- meetings involving Alice (any position)

META MATCH e: edge<any>
WHERE has_target(e, #specific_node)
RETURN e
```

### 36.2.5 edge_type(e)

Returns the type name of an edge.

```
Signature: edge<any> → String

MATCH causes(e1, e2) AS c
RETURN edge_type(c)  -- "causes"

META MATCH e: edge<any>
WHERE edge_type(e) LIKE "learned_%"
RETURN e
```

### 36.2.6 source_types(e)

Returns the type signature of targets.

```
Signature: edge<any> → List<String>

MATCH assigned_to(t, p) AS a
RETURN source_types(a)  -- ["Task", "Person"]

META MATCH e: edge<any>
WHERE source_types(e) CONTAINS "Person"
RETURN e  -- all edges involving Person
```

### 36.2.7 is_higher_order(e)

Returns true if any target is an edge.

```
Signature: edge<any> → Bool

META MATCH e: edge<any>
WHERE is_higher_order(e)
RETURN e  -- edges about edges

MATCH confidence(c, level) AS conf
RETURN is_higher_order(conf)  -- true
```

### 36.2.8 edges_about(e)

Returns all edges that target this edge (higher-order edges referencing it).

```
Signature: edge<any> → List<Edge>

MATCH causes(e1, e2) AS c
RETURN edges_about(c)  -- [confidence_edge, provenance_edge, ...]

META MATCH e: edge<any>
WHERE COUNT(edges_about(e)) > 0
RETURN e  -- edges that have metadata
```

## 36.3 Usage Context

### 36.3.1 On Bound Variables (Normal Mode OK)

When used on already-bound edge variables, introspection is allowed without META:

```
-- OK: c is bound with known type
MATCH causes(e1, e2) AS c
RETURN arity(c), edge_type(c), targets(c)
```

The edge is already resolved; introspection is just attribute access.

### 36.3.2 On Generic Patterns (META Required)

When used with `edge<any>` or for filtering on structure:

```
-- Requires META: edge type unknown
META MATCH e: edge<any>
WHERE arity(e) > 2
RETURN e

-- Requires META: structural query
META MATCH e: edge<any>
WHERE has_target(e, #alice) AND edge_type(e) != "causes"
RETURN e
```

## 36.4 Node Introspection Functions

Node introspection provides symmetry with edge introspection, enabling dynamic attribute access and type inspection.

```
NodeIntrospectionFunction =
    "type_of" "(" Expr ")"
  | "type_node" "(" Expr ")"
  | "attributes" "(" Expr ")"
  | "attr" "(" Expr "," StringLiteral ")"
  | "has_attr" "(" Expr "," StringLiteral ")"
```

### 36.4.1 type_of(n)

Returns the type name of a node as a string.

```
Signature: Node → String

MATCH t: Task
RETURN type_of(t)  -- "Task"

META MATCH n: any
WHERE type_of(n) LIKE "Learned%"
RETURN n
```

### 36.4.2 type_node(n)

Returns the `_NodeType` node for this instance (not just the name string).

```
Signature: Node → _NodeType

MATCH t: Task
RETURN type_node(t)  -- #_NodeType_Task

-- Useful for schema navigation
META MATCH t: Task
LET tn = type_node(t)
MATCH _type_has_attribute(tn, attr)
RETURN attr.name
```

### 36.4.3 attributes(n)

Returns all attribute names defined on the node's type.

```
Signature: Node → List<String>

MATCH t: Task
RETURN attributes(t)  -- ["title", "status", "priority", ...]

-- Generic attribute enumeration
META MATCH n: any
FOR a IN attributes(n):
  RETURN n.id, a, attr(n, a)
```

### 36.4.4 attr(n, name)

Dynamic attribute access by string name.

```
Signature: Node × String → Any

MATCH t: Task
RETURN attr(t, "title")  -- equivalent to t.title

-- Dynamic access pattern
META MATCH n: any
WHERE attr(n, "confidence") > 0.8
RETURN n

-- Returns null if attribute doesn't exist
MATCH t: Task
RETURN attr(t, "nonexistent")  -- null
```

### 36.4.5 has_attr(n, name)

Checks if a node has an attribute (defined on its type).

```
Signature: Node × String → Bool

MATCH t: Task
RETURN has_attr(t, "priority")  -- true or false

-- Find all nodes with a specific attribute
META MATCH n: any
WHERE has_attr(n, "confidence")
RETURN n, attr(n, "confidence")
```

### 36.4.6 Usage Examples

```
-- Generic attribute copy between nodes
META MATCH src: any, dst: any
WHERE src.id = $srcId AND dst.id = $dstId
FOR a IN attributes(src):
  IF has_attr(dst, a):
    SET attr(dst, a) = attr(src, a)

-- Find nodes by dynamic attribute condition
META MATCH n: any
WHERE has_attr(n, $attrName) AND attr(n, $attrName) = $value
RETURN n

-- Compare type structures
META MATCH n1: any, n2: any
WHERE type_of(n1) = type_of(n2)
  AND n1.id != n2.id
RETURN n1, n2
```

## 36.5 Schema Navigation Helpers

Helper functions for navigating ontology structure without manual Layer 0 traversal.

```
SchemaNavigationFunction =
    "constraints_on" "(" TypeRef ")"
  | "rules_affecting" "(" TypeRef ")"
  | "attributes_of" "(" TypeRef ")"
  | "subtypes_of" "(" TypeRef ")"
  | "supertypes_of" "(" TypeRef ")"
  | "edges_from" "(" TypeRef ")"
  | "edges_to" "(" TypeRef ")"
  | "edges_involving" "(" TypeRef ")"
```

### 36.5.1 constraints_on(T)

Returns all constraints that could affect instances of type T.

```
Signature: TypeRef → List<_ConstraintDef>

META MATCH c IN constraints_on(Task)
RETURN c.name, c.hard

-- Equivalent to manual traversal:
META MATCH
  t: _NodeType,
  c: _ConstraintDef,
  p: _PatternDef,
  v: _VarDef,
  _constraint_has_pattern(c, p),
  _pattern_has_node_var(p, v),
  _var_has_type(v, type_ref)
WHERE type_ref.ref_name = "Task"
RETURN c
```

### 36.5.2 rules_affecting(T)

Returns all rules whose patterns involve type T.

```
Signature: TypeRef → List<_RuleDef>

META MATCH r IN rules_affecting(Task)
WHERE r.auto = true
RETURN r.name, r.priority
```

### 36.5.3 attributes_of(T)

Returns all attribute definitions for type T (including inherited).

```
Signature: TypeRef → List<_AttributeDef>

META MATCH a IN attributes_of(Task)
RETURN a.name, a.required, a.unique

-- Check if type has specific attribute
META MATCH a IN attributes_of(Task)
WHERE a.name = "priority"
RETURN a  -- or empty if not found
```

### 36.5.4 subtypes_of(T)

Returns all types that inherit from T (direct and transitive).

```
Signature: TypeRef → List<_NodeType>

META MATCH sub IN subtypes_of(Entity)
RETURN sub.name
-- ["Person", "Organization", "Bot", ...]
```

### 36.5.5 supertypes_of(T)

Returns all types that T inherits from (direct and transitive).

```
Signature: TypeRef → List<_NodeType>

META MATCH sup IN supertypes_of(Task)
RETURN sup.name
-- ["Entity", "Named", ...]
```

### 36.5.6 edges_from(T)

Returns edge types where T is the first (source) position.

```
Signature: TypeRef → List<_EdgeType>

META MATCH e IN edges_from(Task)
RETURN e.name
-- ["belongs_to", "assigned_to", "depends_on", ...]
```

### 36.5.7 edges_to(T)

Returns edge types where T appears in any non-first position.

```
Signature: TypeRef → List<_EdgeType>

META MATCH e IN edges_to(Person)
RETURN e.name
-- ["assigned_to", "created_by", ...]
```

### 36.5.8 edges_involving(T)

Returns all edge types where T appears in any position.

```
Signature: TypeRef → List<_EdgeType>

META MATCH e IN edges_involving(Task)
RETURN e.name, e.arity
```

---

# 37. Generic Edge Patterns

## 37.1 Syntax

```
GenericEdgePattern =
    "edge" "<" "any" ">" "(" TargetPattern ")" EdgeAlias?
  | "edge" "<" "any" ">" EdgeAlias?

TargetPattern =
    "*"                                    -- any targets
  | Target ("," Target)*                   -- specific positions
```

## 37.2 Syntax Clarification: `edge<T>` in Different Contexts

The `edge<T>` syntax appears in two different contexts with related but distinct meanings:

### 37.2.1 In Type Signatures (Ontology DSL)

In edge type signatures, `edge<T>` means "a reference to an edge of type T":

```
-- Ontology DSL: edge<causes> is a TYPE for edge references
edge confidence(about: edge<causes>, level: Float)
--              ^^^^^^^^^^^^^^^^^^^^
--              This position accepts edges of type "causes"
```

This is used for **higher-order edges** — edges that target other edges.

### 37.2.2 In META Patterns

In META patterns, `edge<any>` means "match any edge regardless of type":

```
-- META pattern: edge<any> is a PATTERN that matches any edge
META MATCH e: edge<any>
--          ^^^^^^^^^^
--          This matches any edge in the graph
```

### 37.2.3 Why the Same Syntax?

The syntax is intentionally similar because both describe edge references:

| Context | Syntax | Meaning |
|---------|--------|---------|
| Type signature | `edge<causes>` | "An edge of type causes" |
| Type signature | `edge<any>` | "Any edge type" (wildcard in signature) |
| META pattern | `edge<any>` | "Match any edge" (wildcard in pattern) |

The context (type signature vs pattern) disambiguates the meaning.

### 37.2.4 Specific Type in META Patterns

You can also use specific types in META patterns:

```
-- Match only edges of type "causes"
META MATCH e: edge<causes>
RETURN e

-- Equivalent to normal mode (but with META overhead):
MATCH causes(_, _) AS e
RETURN e
```

This is rarely useful since normal mode is faster for known types.

## 37.3 Examples

### 37.2.1 Any Edge, Any Targets

```
META MATCH e: edge<any>
RETURN e, edge_type(e), arity(e)
-- All edges in the graph
```

### 37.2.2 Any Edge With Specific Target Count

```
META MATCH e: edge<any>(a, b)
RETURN e, a, b
-- All binary edges

META MATCH e: edge<any>(a, b, c)
RETURN e, a, b, c
-- All ternary edges

META MATCH e: edge<any>(*)
WHERE arity(e) >= 3
RETURN e
-- All hyperedges (3+ targets)
```

### 37.2.3 Any Edge Involving Specific Node

```
META MATCH e: edge<any>
WHERE has_target(e, #alice)
RETURN e, edge_type(e)
-- All edges involving Alice (any position, any type)
```

### 37.2.4 Any Higher-Order Edge

```
META MATCH base: edge<any>, meta: edge<any>(base, _)
RETURN base, meta, edge_type(meta)
-- All edges that have other edges about them
```

### 37.2.5 Any Edge Matching Type Pattern

```
META MATCH e: edge<any>
WHERE edge_type(e) LIKE "learned_%"
RETURN e
-- Edges with types matching pattern (discovered at runtime)

META MATCH e: edge<any>
WHERE edge_type(e) IN ["causes", "leads_to", "triggers"]
RETURN e
-- Edges of specific types (but generic matching)
```

## 37.3 Performance Characteristics

| Pattern | Cost | Index Used |
|---------|------|------------|
| `edge<any>` | Full edge scan | None |
| `edge<any> WHERE has_target(e, #x)` | Node's edge index | Per-node edge list |
| `edge<any>(a, b)` | Full scan + arity filter | None |
| `edge<any> WHERE edge_type(e) = "X"` | Type-specific | Type index |

**Recommendation:** Add conditions to narrow the scan:

```
-- Slow: scans all edges
META MATCH e: edge<any>
RETURN e

-- Faster: uses node's edge list
META MATCH e: edge<any>
WHERE has_target(e, #known_node)
RETURN e

-- Fastest: use normal mode if you know the type
MATCH causes(_, _) AS e
RETURN e
```

---

# 38. META MATCH

## 38.1 Syntax

```
MetaMatchStmt = "meta" MatchStmt
```

All MATCH syntax is valid after META, plus:
- `edge<any>` patterns
- Layer 0 type access
- Introspection in WHERE

## 38.2 Instance-Level Meta Queries

Query instances with generic patterns:

```
-- All edges from a node
META MATCH e: edge<any>
WHERE target(e, 0) = #start_node  -- first position
RETURN e, edge_type(e)

-- All edges between two nodes
META MATCH e: edge<any>
WHERE has_target(e, #node_a) AND has_target(e, #node_b)
RETURN e

-- All metadata on an edge
META MATCH meta: edge<any>(#known_edge, _)
RETURN meta, edge_type(meta)

-- Edges with specific structural property
META MATCH e: edge<any>
WHERE arity(e) = 3 
  AND source_types(e) CONTAINS "Person"
  AND source_types(e) CONTAINS "Room"
RETURN e
```

## 38.3 Schema-Level Queries (Level 1)

Query the ontology structure:

```
-- All node types
META MATCH t: _NodeType
RETURN t.name, t.abstract

-- All edge types with their arities
META MATCH e: _EdgeType
RETURN e.name, e.arity, e.symmetric

-- Attributes of a type
META MATCH 
  t: _NodeType,
  a: _AttributeDef,
  _type_has_attribute(t, a)
WHERE t.name = "Task"
RETURN a.name, a.required, a.unique

-- Type inheritance hierarchy
META MATCH 
  child: _NodeType,
  parent: _NodeType,
  _type_inherits(child, parent)
RETURN child.name, parent.name

-- All rules
META MATCH r: _RuleDef
WHERE r.auto = true
RETURN r.name, r.priority

-- Constraints on a type
META MATCH
  t: _NodeType,
  c: _ConstraintDef,
  p: _PatternDef,
  v: _VarDef,
  _constraint_has_pattern(c, p),
  _pattern_has_node_var(p, v),
  _var_has_type(v, type_ref)
WHERE type_ref.name = t.name AND t.name = "Task"
RETURN c.name, c.hard
```

## 38.4 Meta-Schema Queries (Level 0)

Query the structure of Layer 0 itself:

```
-- All Layer 0 node types
META MATCH t: _NodeType
WHERE t.name LIKE "_%"  -- Layer 0 types start with underscore
RETURN t.name

-- Structure of _EdgeType
META MATCH
  t: _NodeType,
  a: _AttributeDef,
  _type_has_attribute(t, a)
WHERE t.name = "_EdgeType"
RETURN a.name
```

## 38.5 Combined Queries

Mix instance and schema queries:

```
-- Find instances of dynamically-discovered types
META MATCH t: _NodeType
WHERE t.name LIKE "Learned%"
-- Then for each type, query instances:
-- (This would be a programmatic pattern)

-- Find edges that match a schema pattern
META MATCH
  e_type: _EdgeType,
  e: edge<any>
WHERE e_type.name = edge_type(e)
  AND e_type.arity > 2
RETURN e, e_type.name
```

---

# 39. META WALK

## 39.1 Syntax

```
MetaWalkStmt = "meta" WalkStmt
```

META WALK enables:
- Generic edge following (`edge<any>`)
- Starting from edges (not just nodes)
- Schema-level traversal

## 39.2 Generic Edge Traversal

```
-- Follow any edge type from a node
META WALK FROM #alice
FOLLOW edge<any>
RETURN NODES

-- Follow any edge type with depth
META WALK FROM #start_node
FOLLOW edge<any> [depth: 3]
RETURN PATH

-- Follow edges matching pattern
META WALK FROM #node
FOLLOW edge<any> WHERE edge_type(edge) LIKE "relates_%"
RETURN NODES
```

## 39.3 Higher-Order Traversal

Start from an edge and follow edges-about-edges:

```
-- Find all metadata about an edge, recursively
META WALK FROM #causes_edge_123
FOLLOW edge<any>
RETURN PATH

-- Example graph:
-- causes(#e1, #e2) AS c
-- confidence(c, 0.9) AS conf
-- assessed_by(conf, #expert) AS assess
-- timestamp(assess, now()) AS ts
--
-- META WALK FROM #c FOLLOW edge<any>
-- Returns path: c → conf → assess → ts
```

```
-- Find the "metadata depth" of edges
META WALK FROM #some_edge
FOLLOW edge<any>
UNTIL NOT is_higher_order(node)  -- until we hit non-edge targets
RETURN depth, node
```

## 39.4 Schema Traversal

Walk the ontology structure:

```
-- Find type hierarchy
META WALK FROM #_NodeType_Task
FOLLOW _type_inherits
RETURN PATH

-- Find all types reachable from a type
META WALK FROM #_NodeType_Entity
FOLLOW _type_inherits INBOUND  -- subtypes
RETURN TERMINAL

-- Explore constraint dependencies
META WALK FROM #_ConstraintDef_temporal_order
FOLLOW _constraint_has_pattern | _pattern_has_node_var | _var_has_type
RETURN PATH
```

## 39.5 Direction with Generic Edges

```
-- Any outgoing edge (node is first target)
META WALK FROM #node
FOLLOW edge<any> OUTBOUND
RETURN NODES

-- Any incoming edge (node is in any later position)
META WALK FROM #node
FOLLOW edge<any> INBOUND
RETURN NODES

-- Any edge (any position)
META WALK FROM #node
FOLLOW edge<any> ANY
RETURN NODES
```

**Direction semantics for hyperedges:**

| Direction | Meaning for edge(t0, t1, t2, ...) |
|-----------|-----------------------------------|
| OUTBOUND | Current node is t0 (first position) |
| INBOUND | Current node is t1, t2, ... (any other position) |
| ANY | Current node is in any position |

## 39.6 FOLLOW Filtering

```
-- Follow edges matching runtime condition
META WALK FROM #start
FOLLOW edge<any> WHERE arity(edge) = 2
RETURN PATH

-- Follow edges of certain types
META WALK FROM #start
FOLLOW edge<any> WHERE edge_type(edge) IN ["causes", "leads_to"]
RETURN PATH

-- Follow only high-confidence edges
META WALK FROM #start
FOLLOW edge<any> AS e WHERE EXISTS(
  confidence(e, level) WHERE level > 0.8
)
RETURN PATH
```

---

# 39.7 META DESCRIBE

The DESCRIBE command provides quick access to type structures, edge signatures, and instance details. It's syntactic sugar over META MATCH queries but essential for tooling and REPL workflows.

## 39.7.1 Syntax

```
MetaDescribeStmt =
    "meta" "describe" TypeName
  | "meta" "describe" "edge" EdgeTypeName
  | "meta" "describe" NodeRef
  | "meta" "describe" EdgeRef
```

## 39.7.2 Describe Node Type

```
META DESCRIBE Task

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ Type: Task                                                          │
-- │ Parents: [Entity, Named]                                            │
-- │ Abstract: false                                                     │
-- │ Sealed: false                                                       │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Attributes:                                                         │
-- │   title: String [required]                                          │
-- │   status: String [in: ["todo", "in_progress", "done"]] = "todo"    │
-- │   priority: Int [>= 0, <= 10] = 5                                  │
-- │   created_at: Timestamp = now()                                    │
-- │   completed_at: Timestamp?                                          │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Constraints affecting:                                              │
-- │   • task_title_required (hard)                                     │
-- │   • task_priority_range (hard)                                     │
-- │   • task_completion_timestamp (soft)                               │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Rules affecting:                                                    │
-- │   • auto_complete_timestamp [priority: 10, auto]                   │
-- │   • notify_on_completion [priority: 5, auto]                       │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Outgoing edges: belongs_to, assigned_to, depends_on                │
-- │ Incoming edges: subtask_of, blocks                                 │
-- │ Instance count: 1,247                                               │
-- └─────────────────────────────────────────────────────────────────────┘
```

## 39.7.3 Describe Edge Type

```
META DESCRIBE EDGE causes

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ Edge: causes                                                        │
-- │ Arity: 2                                                            │
-- │ Symmetric: false                                                    │
-- │ Reflexive allowed: false                                            │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Signature:                                                          │
-- │   0: from: Event                                                    │
-- │   1: to: Event                                                      │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Attributes:                                                         │
-- │   mechanism: String?                                                │
-- │   strength: Float [>= 0, <= 1] = 1.0                               │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Modifiers: [indexed, acyclic]                                      │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Constraints:                                                        │
-- │   • temporal_order (hard)                                          │
-- │   • no_self_causation (hard)                                       │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Higher-order edges targeting this type:                            │
-- │   • confidence(edge<causes>, Float)                                │
-- │   • provenance(edge<causes>, Source)                               │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Instance count: 8,432                                               │
-- └─────────────────────────────────────────────────────────────────────┘
```

## 39.7.4 Describe Node Instance

```
META DESCRIBE #task_123

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ Node: Task (id: task_123)                                          │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Attributes:                                                         │
-- │   title: "Fix authentication bug"                                  │
-- │   status: "in_progress"                                            │
-- │   priority: 8                                                       │
-- │   created_at: 2024-01-15T10:30:00Z                                 │
-- │   completed_at: null                                                │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Outgoing edges:                                                     │
-- │   belongs_to → #project_456 (Project: "Auth System")              │
-- │   assigned_to → #alice (Person: "Alice Smith")                    │
-- │   depends_on → #task_100, #task_101                                │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Incoming edges:                                                     │
-- │   subtask_of ← #task_050                                           │
-- │   blocks ← #task_200, #task_201                                    │
-- └─────────────────────────────────────────────────────────────────────┘
```

## 39.7.5 Describe Edge Instance

```
META DESCRIBE #causes_edge_789

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ Edge: causes (id: causes_edge_789)                                 │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Targets:                                                            │
-- │   0 (from): #event_100 (Event: "Server restart")                  │
-- │   1 (to): #event_101 (Event: "Cache cleared")                     │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Attributes:                                                         │
-- │   mechanism: "automatic"                                            │
-- │   strength: 0.95                                                    │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Higher-order edges (edges about this edge):                        │
-- │   confidence(#causes_edge_789, 0.92) → #conf_001                  │
-- │   provenance(#causes_edge_789, #log_source) → #prov_001           │
-- │   assessed_by(#conf_001, #expert_alice) → #assess_001             │
-- └─────────────────────────────────────────────────────────────────────┘
```

## 39.7.6 Programmatic Access

DESCRIBE returns structured data that can be captured:

```
-- Capture as variable
LET info = META DESCRIBE Task
RETURN info.attributes, info.constraints

-- Use in conditions
LET type_info = META DESCRIBE $dynamic_type_name
WHERE type_info.instance_count > 1000
RETURN type_info.name, "high volume type"
```

## 39.7.7 Equivalent META MATCH Queries

DESCRIBE is sugar for common introspection patterns:

```
-- META DESCRIBE Task is equivalent to:
META MATCH t: _NodeType WHERE t.name = "Task"
LET attrs = (
  META MATCH a: _AttributeDef, _type_has_attribute(t, a)
  RETURN COLLECT(a)
)
LET constraints = constraints_on(Task)
LET rules = rules_affecting(Task)
LET outgoing = edges_from(Task)
LET incoming = edges_to(Task)
LET count = (MATCH n: Task RETURN COUNT(n))
RETURN {
  type: t,
  attributes: attrs,
  constraints: constraints,
  rules: rules,
  outgoing_edges: outgoing,
  incoming_edges: incoming,
  instance_count: count
}
```

---

# 39.8 META DRY RUN

Preview the effects of schema modifications without committing them.

## 39.8.1 Syntax

```
MetaDryRunStmt = "meta" "dry" "run" MetaModificationStmt
```

## 39.8.2 Usage

```
META DRY RUN CREATE NODE LearnedConcept {
  name: String [required],
  confidence: Float
}

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ DRY RUN COMPLETE (no changes made)                                  │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Would create:                                                       │
-- │   • _NodeType "LearnedConcept"                                     │
-- │   • _AttributeDef "name" (String, required)                        │
-- │   • _AttributeDef "confidence" (Float)                             │
-- │   • 2 _ScalarTypeExpr nodes                                        │
-- │   • 4 schema edges (_type_has_attribute, _attr_has_type, etc.)    │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Recompilation: Full                                                 │
-- │ Estimated time: ~50ms                                               │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Validation:                                                         │
-- │   ✓ No name conflicts                                              │
-- │   ✓ All referenced types exist                                     │
-- │   ✓ Attribute constraints valid                                    │
-- └─────────────────────────────────────────────────────────────────────┘
```

## 39.8.3 Detecting Conflicts

```
META DRY RUN CREATE NODE Task {
  title: String
}

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ DRY RUN FAILED                                                      │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Validation errors:                                                  │
-- │   ✗ Type "Task" already exists                                     │
-- │     Existing type has 5 attributes, 3 constraints                  │
-- │     Use META CREATE NODE Task : ExistingTask to extend             │
-- │     Or use a different name                                        │
-- └─────────────────────────────────────────────────────────────────────┘
```

## 39.8.4 Dry Run for Destructive Operations

```
META DRY RUN KILL #deprecated_type

-- Output:
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ DRY RUN COMPLETE (no changes made)                                  │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Would delete:                                                       │
-- │   • _NodeType "DeprecatedType"                                     │
-- │   • 3 _AttributeDef nodes                                          │
-- │   • 847 instances of DeprecatedType                                │
-- │   • 2,341 edges referencing those instances                        │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ Cascade impact:                                                     │
-- │   • 12 constraints would be orphaned (auto-removed)               │
-- │   • 3 rules would be orphaned (auto-removed)                      │
-- │   • 2 edge types would lose valid targets                         │
-- ├─────────────────────────────────────────────────────────────────────┤
-- │ ⚠️  WARNING: This operation is destructive                          │
-- │ To proceed: META KILL #deprecated_type CONFIRM                     │
-- └─────────────────────────────────────────────────────────────────────┘
```

---

# 40. META Transformations

## 40.1 Overview

META transformations modify the schema at runtime. They enable:
- Creating new types dynamically
- Modifying rules and constraints
- Self-modification for AGI

**These operations are dangerous.** They require explicit permission and trigger recompilation.

## 40.2 META SPAWN

Create new schema elements.

### 40.2.1 Create Node Type

```
META SPAWN t: _NodeType {
  name = "DiscoveredConcept",
  abstract = false,
  sealed = false,
  doc = "Automatically discovered concept"
}

-- Add attributes
META SPAWN attr: _AttributeDef {
  name = "confidence",
  required = false,
  unique = false
}
META SPAWN type_ref: _ScalarTypeRef {
  scalar_type = "Float"
}
META LINK _attr_has_type(#attr, #type_ref)
META LINK _type_has_attribute(#t, #attr)
```

### 40.2.2 Create Edge Type

```
-- Create edge type
META SPAWN e: _EdgeType {
  name = "potential_cause",
  arity = 2,
  symmetric = false,
  doc = "Learned causal relationship"
}

-- Define signature
META SPAWN v1: _VarDef { name = "from", is_edge_var = false }
META SPAWN v2: _VarDef { name = "to", is_edge_var = false }
META SPAWN t_event: _NamedTypeRef { type_name = "Event" }
META LINK _edge_has_position(#e, #v1) { position = 0 }
META LINK _edge_has_position(#e, #v2) { position = 1 }
META LINK _var_has_type(#v1, #t_event)
META LINK _var_has_type(#v2, #t_event)

-- Register with ontology
META LINK _ontology_declares_edge(#current_ontology, #e)
```

### 40.2.3 Create Constraint

```
META SPAWN c: _ConstraintDef {
  name = "learned_temporal",
  hard = true,
  message = "Learned temporal ordering"
}

META SPAWN p: _PatternDef { }
-- ... build pattern structure ...

META SPAWN cond: _CompareExpr {
  operator = "<"
}
-- ... build condition structure ...

META LINK _constraint_has_pattern(#c, #p)
META LINK _constraint_has_condition(#c, #cond)
META LINK _ontology_declares_constraint(#current_ontology, #c)
```

### 40.2.4 Create Rule

```
META SPAWN r: _RuleDef {
  name = "learned_inference",
  priority = 10,
  auto = true,
  doc = "Automatically learned inference rule"
}

META SPAWN pattern: _PatternDef { }
META SPAWN production: _ProductionDef { }
-- ... build pattern and production ...

META LINK _rule_has_pattern(#r, #pattern)
META LINK _rule_has_production(#r, #production)
META LINK _ontology_declares_rule(#current_ontology, #r)
```

## 40.3 META LINK

Create schema relationships.

```
-- Add attribute to existing type
META LINK _type_has_attribute(#existing_type, #new_attr)

-- Create inheritance
META LINK _type_inherits(#child_type, #parent_type)

-- Add edge position
META LINK _edge_has_position(#edge_type, #var_def) { position = 0 }
```

## 40.4 META SET

Modify schema elements.

```
-- Change rule priority
META SET #rule_123.priority = 100

-- Disable auto-fire
META SET #rule_456.auto = false

-- Change constraint to soft
META SET #constraint_789.hard = false

-- Update documentation
META SET #type_abc.doc = "Updated description"
```

## 40.5 META KILL

Remove schema elements.

```
-- Remove a rule
META KILL #rule_to_remove

-- Remove a constraint
META KILL #constraint_to_remove

-- Remove a type (DANGEROUS: affects all instances!)
META KILL #deprecated_type CONFIRM
-- Requires CONFIRM for types with instances
```

### 40.5.1 Cascade Behavior

| Target | Cascade |
|--------|---------|
| Rule | Just the rule |
| Constraint | Just the constraint |
| Attribute | Attribute removed from all instances |
| Edge type | All edges of that type removed |
| Node type | All nodes of that type removed |

```
-- Check impact before killing
META MATCH n: #type_to_remove
RETURN COUNT(n) AS instance_count

-- Kill with cascade acknowledgment
META KILL #type_to_remove CONFIRM
```

## 40.6 META UNLINK

Remove schema relationships.

```
-- Remove attribute from type (doesn't delete instances' values)
META UNLINK _type_has_attribute(#type, #attr)

-- Remove inheritance
META UNLINK _type_inherits(#child, #parent)
```

## 40.7 META CREATE (Runtime Ontology DSL)

The low-level SPAWN/LINK operations are verbose and error-prone. META CREATE provides familiar Ontology DSL syntax for runtime schema modification.

### 40.7.1 Syntax

```
MetaCreateStmt =
    "meta" "create" "node" NodeTypeDef
  | "meta" "create" "edge" EdgeTypeDef
  | "meta" "create" "constraint" ConstraintDef
  | "meta" "create" "rule" RuleDef
```

The syntax mirrors the Ontology DSL exactly. The engine parses and expands to Layer 0 operations internally.

### 40.7.2 Create Node Type

```
-- Simple type
META CREATE NODE DiscoveredConcept {
  name: String [required],
  confidence: Float [>= 0, <= 1]
}

-- With inheritance
META CREATE NODE LearnedEntity : Entity {
  source: String,
  discovered_at: Timestamp = now()
}

-- Abstract type
META CREATE NODE AbstractPattern [abstract] {
  pattern_id: String [required, unique]
}
```

**Equivalent to:**
```
META BEGIN
  META SPAWN t: _NodeType { name = "DiscoveredConcept" }
  META SPAWN a1: _AttributeDef { name = "name", required = true }
  META SPAWN a2: _AttributeDef { name = "confidence", required = false }
  META SPAWN t1: _ScalarTypeExpr { scalar_type = "String" }
  META SPAWN t2: _ScalarTypeExpr { scalar_type = "Float" }
  META LINK _attr_has_type(#a1, #t1)
  META LINK _attr_has_type(#a2, #t2)
  META LINK _type_has_attribute(#t, #a1)
  META LINK _type_has_attribute(#t, #a2)
  -- ... constraint expressions for >= 0, <= 1 ...
  META LINK _ontology_declares_type(#current_ontology, #t)
META COMMIT
```

### 40.7.3 Create Edge Type

```
-- Simple binary edge
META CREATE EDGE learned_cause(from: Event, to: Event)

-- With modifiers
META CREATE EDGE correlation(a: Event, b: Event) [symmetric, indexed]

-- With attributes
META CREATE EDGE potential_link(src: Entity, dst: Entity) {
  confidence: Float [>= 0, <= 1],
  discovered_at: Timestamp = now()
}

-- Higher-order edge
META CREATE EDGE evidence_for(evidence: Entity, claim: edge<any>) {
  strength: Float
}

-- Ternary edge
META CREATE EDGE meeting(organizer: Person, attendee: Person, room: Room)
```

### 40.7.4 Create Constraint

```
-- Simple constraint
META CREATE CONSTRAINT learned_temporal:
  e1: Event, e2: Event, learned_cause(e1, e2)
  => e1.timestamp < e2.timestamp

-- Soft constraint (warning only)
META CREATE CONSTRAINT [soft] confidence_threshold:
  c: DiscoveredConcept
  WHERE c.confidence < 0.5
  => false
  MESSAGE "Low confidence concept detected"

-- Existence constraint
META CREATE CONSTRAINT concept_requires_source:
  c: DiscoveredConcept
  => EXISTS(s: Source, derived_from(c, s))
```

### 40.7.5 Create Rule

```
-- Simple inference rule
META CREATE RULE infer_transitivity:
  a: Event, b: Event, c: Event,
  learned_cause(a, b),
  learned_cause(b, c)
  WHERE NOT EXISTS(learned_cause(a, c))
  =>
  CREATE learned_cause(a, c) { confidence = 0.5 }

-- Rule with priority
META CREATE RULE [priority: 10, auto: true] propagate_confidence:
  e1: Event, e2: Event,
  learned_cause(e1, e2) AS lc,
  confidence(lc, level)
  WHERE level > 0.8
  =>
  SET e2.high_confidence = true
```

### 40.7.6 Batching META CREATE

Multiple META CREATE statements can be batched:

```
META BEGIN
  META CREATE NODE Concept { name: String [required] }
  META CREATE NODE Relation { type: String }
  META CREATE EDGE has_concept(Entity, Concept)
  META CREATE EDGE has_relation(Concept, Relation, Concept)
  META CREATE CONSTRAINT concept_name_required:
    c: Concept => c.name != ""
META COMMIT
-- Single recompilation for all
```

### 40.7.7 Comparison: SPAWN vs CREATE

| Aspect | META SPAWN/LINK | META CREATE |
|--------|-----------------|-------------|
| Verbosity | ~10 operations per type | 1 statement |
| Error-prone | High (manual linking) | Low (validated) |
| Flexibility | Full Layer 0 access | DSL subset |
| Use case | Edge cases, repairs | Normal schema evolution |
| AGI-friendly | No | Yes |

**Recommendation:** Use META CREATE for all normal schema modifications. Reserve SPAWN/LINK for:
- Repairing corrupted schema
- Operations not expressible in DSL
- Fine-grained control over Layer 0 structure

## 40.8 META ENABLE / META DISABLE

Convenience syntax for toggling constraints and rules.

### 40.8.1 Syntax

```
MetaEnableStmt  = "meta" "enable" ("constraint" | "rule") Identifier
MetaDisableStmt = "meta" "disable" ("constraint" | "rule") Identifier
```

### 40.8.2 Constraints

```
-- Disable a constraint (make it soft, non-enforcing)
META DISABLE CONSTRAINT temporal_order

-- Re-enable
META ENABLE CONSTRAINT temporal_order

-- Equivalent to:
META SET #temporal_order.hard = false
META SET #temporal_order.hard = true
```

### 40.8.3 Rules

```
-- Disable auto-firing
META DISABLE RULE auto_complete_timestamp

-- Re-enable
META ENABLE RULE auto_complete_timestamp

-- Equivalent to:
META SET #auto_complete_timestamp.auto = false
META SET #auto_complete_timestamp.auto = true
```

### 40.8.4 Temporary Disable in Transaction

```
META BEGIN
  META DISABLE CONSTRAINT temporal_order
  
  -- Perform operations that would violate constraint
  SPAWN e1: Event { timestamp = 100 }
  SPAWN e2: Event { timestamp = 50 }
  LINK causes(e1, e2)  -- would normally fail
  
  -- Fix the data
  SET e1.timestamp = 40
  
  META ENABLE CONSTRAINT temporal_order
META COMMIT
-- Constraint re-checked at commit
```

---

# 41. Schema Recompilation

## 41.1 When Recompilation Occurs

Schema modifications trigger recompilation:

| Operation | Recompilation |
|-----------|---------------|
| META SPAWN type | Full |
| META SPAWN edge | Full |
| META SPAWN constraint | Incremental |
| META SPAWN rule | Incremental |
| META SET on type | Full |
| META SET on rule/constraint | Incremental |
| META KILL type | Full |
| META KILL rule/constraint | Incremental |

## 41.2 Recompilation Process

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. Acquire schema lock (blocks new queries)                         │
│ 2. Validate modification                                            │
│ 3. Update Layer 0 graph                                            │
│ 4. Rebuild affected registries                                     │
│    • Type registry                                                  │
│    • Edge signature registry                                        │
│    • Constraint index                                               │
│    • Rule trigger index                                             │
│ 5. Rebuild affected indexes                                        │
│ 6. Release schema lock                                              │
└─────────────────────────────────────────────────────────────────────┘
```

## 41.3 In-Progress Query Handling

Queries in progress when recompilation starts:
- **Read queries:** Complete with old schema
- **Write queries:** Block until recompilation complete
- **New queries:** Wait for recompilation

## 41.4 Batching Modifications

Multiple schema modifications should be batched:

```
META BEGIN
  META SPAWN t: _NodeType { name = "Concept" }
  META SPAWN a1: _AttributeDef { name = "name" }
  META SPAWN a2: _AttributeDef { name = "confidence" }
  META LINK _type_has_attribute(#t, #a1)
  META LINK _type_has_attribute(#t, #a2)
META COMMIT
-- Single recompilation for all changes
```

## 41.5 Mixed Normal/META Transactions

Normal mode and META mode operations can be mixed within a single transaction, but with specific semantics.

### 41.5.1 Allowed Combinations

```
BEGIN
  -- Normal operations
  SPAWN t: Task { title = "Test" }
  LINK belongs_to(#t, #project)
  
  -- META operations (schema queries only with META READ)
  LET type_info = META DESCRIBE Task
  
  -- More normal operations
  SET t.priority = 5
COMMIT
```

### 41.5.2 Schema Modification Timing

When META WRITE operations are mixed with normal operations:

```
BEGIN
  -- Normal operation using existing schema
  SPAWN t: Task { title = "Test" }
  
  -- Schema modification
  META CREATE NODE NewType { name: String }
  -- ⚠️ Recompilation is DEFERRED until commit
  
  -- This FAILS: NewType doesn't exist yet in this transaction
  SPAWN n: NewType { name = "Foo" }
  -- ERROR: Type "NewType" not found
COMMIT
```

**Rule:** Schema modifications take effect at transaction commit, not immediately. You cannot use newly created types within the same transaction.

### 41.5.3 Recommended Pattern

For operations that need new types:

```
-- Transaction 1: Create schema
META BEGIN
  META CREATE NODE NewType { name: String }
META COMMIT
-- Recompilation happens here

-- Transaction 2: Use new schema
BEGIN
  SPAWN n: NewType { name = "Foo" }
COMMIT
```

### 41.5.4 Exception: META CREATE with Immediate Use

A future enhancement may support immediate schema availability:

```
-- NOT YET SUPPORTED (v2 consideration)
BEGIN IMMEDIATE_SCHEMA
  META CREATE NODE NewType { name: String }
  -- Immediate recompilation
  SPAWN n: NewType { name = "Foo" }
  -- Works because schema was immediately applied
COMMIT
```

### 41.5.5 Transaction Isolation

| Scenario | Behavior |
|----------|----------|
| Read query during META transaction | Uses old schema |
| Write query during META transaction | Blocks until commit |
| META query during normal transaction | Uses current schema |
| META write during normal transaction | Deferred until commit |

---

# 42. Permission Model

## 42.1 Permission Levels

```
┌─────────────────────────────────────────────────────────────────────┐
│ PERMISSION LEVEL          │ CAPABILITIES                            │
├───────────────────────────┼─────────────────────────────────────────┤
│ NONE (default)            │ Normal mode only                        │
│                           │ No META operations                      │
├───────────────────────────┼─────────────────────────────────────────┤
│ META READ                 │ META MATCH (schema queries)             │
│                           │ META WALK (schema traversal)            │
│                           │ Introspection functions                 │
│                           │ Cannot modify schema                    │
├───────────────────────────┼─────────────────────────────────────────┤
│ META WRITE                │ META READ capabilities                  │
│                           │ META SPAWN (create types, rules)        │
│                           │ META LINK (schema relationships)        │
│                           │ META SET (modify schema)                │
│                           │ META UNLINK (remove relationships)      │
│                           │ Cannot kill types with instances        │
├───────────────────────────┼─────────────────────────────────────────┤
│ META ADMIN                │ META WRITE capabilities                 │
│                           │ META KILL (remove types)                │
│                           │ Can modify Layer 0 (dangerous!)         │
└───────────────────────────┴─────────────────────────────────────────┘
```

## 42.2 Granting Permissions

```
-- Grant to role
GRANT META READ TO role_analyst
GRANT META WRITE TO role_architect
GRANT META ADMIN TO role_system

-- Grant to specific user
GRANT META READ TO user_alice

-- Revoke
REVOKE META WRITE FROM role_architect
```

## 42.3 Permission Errors

```
-- Without permission:
META MATCH t: _NodeType RETURN t
-- ERROR [E4001] PERMISSION_DENIED
--   META READ permission required
--   Current permission: NONE

-- With META READ, trying to write:
META SPAWN t: _NodeType { name = "Test" }
-- ERROR [E4002] PERMISSION_DENIED
--   META WRITE permission required
--   Current permission: META READ
```

## 42.4 Audit Logging

All META operations are logged:

```typescript
interface MetaAuditEntry {
  timestamp: Timestamp
  user: string
  permission: "READ" | "WRITE" | "ADMIN"
  operation: "MATCH" | "WALK" | "SPAWN" | "LINK" | "SET" | "UNLINK" | "KILL"
  target: string           // affected schema element
  details: object          // operation-specific data
  success: boolean
  error?: string
}
```

## 42.5 Layer 0 Protection Invariants

Even with META ADMIN permission, certain Layer 0 invariants cannot be violated. These are the "physics" of the system.

### 42.5.1 Immutable Core Types

The following Layer 0 types cannot be modified or deleted:

```
PROTECTED_TYPES = [
  "_NodeType",
  "_EdgeType", 
  "_AttributeDef",
  "_ConstraintDef",
  "_RuleDef",
  "_PatternDef",
  "_VarDef",
  "_ProductionDef",
  "_Action",
  "_Expr",
  "_TypeExpr",
  "_Ontology"
]
```

Attempting to modify these results in:

```
META KILL #_NodeType
-- ERROR [E4010] LAYER0_INVARIANT_VIOLATION
--   Cannot delete protected Layer 0 type "_NodeType"
--   This type is fundamental to the system's operation

META SET #_EdgeType.arity = 5
-- ERROR [E4011] LAYER0_INVARIANT_VIOLATION
--   Cannot modify protected attribute on Layer 0 type
--   _EdgeType.arity is structurally fixed
```

### 42.5.2 Immutable Core Edges

The following Layer 0 edge types cannot be modified or deleted:

```
PROTECTED_EDGES = [
  "_type_inherits",
  "_type_has_attribute",
  "_attr_has_type",
  "_edge_has_position",
  "_var_has_type",
  "_constraint_has_pattern",
  "_constraint_has_condition",
  "_rule_has_pattern",
  "_rule_has_production",
  "_ontology_declares_type",
  "_ontology_declares_constraint",
  "_ontology_declares_rule"
]
```

### 42.5.3 Structural Invariants

These invariants are enforced even with META ADMIN:

| Invariant | Description | Error |
|-----------|-------------|-------|
| **Type name uniqueness** | No two `_NodeType` can have the same name | E4020 |
| **Edge name uniqueness** | No two `_EdgeType` can have the same name | E4021 |
| **Inheritance acyclicity** | `_type_inherits` cannot form cycles | E4022 |
| **Edge arity consistency** | `_edge_has_position` count must match `arity` | E4023 |
| **Pattern variable uniqueness** | Variables in a pattern must have unique names | E4024 |
| **Expression well-formedness** | Binary ops need two operands, etc. | E4025 |

```
-- Attempting to create inheritance cycle:
META LINK _type_inherits(#TypeA, #TypeB)
-- where TypeB already inherits from TypeA

-- ERROR [E4022] LAYER0_INVARIANT_VIOLATION
--   Cannot create inheritance cycle: TypeA → TypeB → ... → TypeA
--   Inheritance must form a DAG
```

### 42.5.4 Semantic Invariants

These ensure the system remains coherent:

| Invariant | Description |
|-----------|-------------|
| **Required attributes** | `_NodeType.name`, `_EdgeType.name`, `_EdgeType.arity` cannot be null |
| **Valid scalar types** | `_ScalarTypeExpr.scalar_type` must be one of: String, Int, Float, Bool, Timestamp |
| **Valid operators** | `_BinaryOpExpr.operator` must be a recognized operator |
| **Position bounds** | `_edge_has_position.position` must be in range [0, arity-1] |

### 42.5.5 What META ADMIN CAN Do

META ADMIN can still:

- Create new types that inherit from protected types (if not sealed)
- Add new attributes to user-defined types
- Modify user-defined constraints and rules
- Delete user-defined types (with CONFIRM)
- Modify non-structural attributes on Layer 0 instances (e.g., `doc` fields)

```
-- OK: Add documentation to Layer 0 type
META SET #_NodeType.doc = "Updated documentation"

-- OK: Create type inheriting from non-sealed Layer 0 type
META CREATE NODE CustomExpr : _Expr {
  custom_field: String
}

-- NOT OK: Modify structural attribute
META SET #_NodeType.name = "RenamedNodeType"
-- ERROR [E4011] LAYER0_INVARIANT_VIOLATION
```

### 42.5.6 Rationale

These invariants exist because:

1. **Bootstrap dependency:** The engine needs these types to function
2. **Query correctness:** Compiled queries assume these structures
3. **Constraint checking:** The constraint system depends on these types
4. **Self-consistency:** The system must be able to describe itself

Violating these invariants would corrupt the system beyond recovery.

---

# 43. Built-in Rule Metrics

## 43.1 System Attributes on _RuleDef

The engine automatically tracks execution metrics for all rules. These are system-managed attributes (prefixed with `_`) that cannot be directly modified.

### 43.1.1 Metric Attributes

| Attribute | Type | Description |
|-----------|------|-------------|
| `_invocation_count` | Int | Total number of times the rule has fired |
| `_success_count` | Int | Invocations that completed without error |
| `_error_count` | Int | Invocations that failed or were rolled back |
| `_last_invoked` | Timestamp | When the rule last fired |
| `_last_error` | Timestamp? | When the rule last failed (null if never) |
| `_avg_execution_time` | Float | Average execution time in milliseconds |
| `_total_execution_time` | Int | Cumulative execution time in milliseconds |
| `_matches_produced` | Int | Total number of graph changes made |

### 43.1.2 Derived Metrics

```
-- Success rate (computed)
success_rate(r) = r._success_count / r._invocation_count

-- Error rate (computed)
error_rate(r) = r._error_count / r._invocation_count

-- Throughput (matches per invocation)
throughput(r) = r._matches_produced / r._invocation_count
```

### 43.1.3 Observation Rule Metrics

```
-- Find most active rules
META MATCH r: _RuleDef
WHERE r._invocation_count > 0
RETURN r.name, r._invocation_count, r._avg_execution_time
ORDER BY r._invocation_count DESC
LIMIT 10

-- Find problematic rules (high error rate)
META MATCH r: _RuleDef
WHERE r._invocation_count > 100
  AND (r._error_count * 1.0 / r._invocation_count) > 0.1
RETURN r.name, r._error_count, r._invocation_count

-- Find slow rules
META MATCH r: _RuleDef
WHERE r._avg_execution_time > 100  -- > 100ms average
RETURN r.name, r._avg_execution_time, r._invocation_count

-- Rules that haven't fired recently
META MATCH r: _RuleDef
WHERE r.auto = true
  AND (r._last_invoked IS NULL OR r._last_invoked < now() - 86400000)
RETURN r.name, r._last_invoked
```

### 43.1.4 Metric Reset

Metrics can be reset (requires META ADMIN):

```
-- Reset metrics for a specific rule
META RESET METRICS #rule_name

-- Reset all rule metrics
META RESET METRICS ALL RULES
```

### 43.1.5 Constraint Metrics

Similar metrics exist for constraints:

| Attribute | Type | Description |
|-----------|------|-------------|
| `_check_count` | Int | Number of times constraint was evaluated |
| `_violation_count` | Int | Number of violations detected |
| `_last_checked` | Timestamp | When constraint was last evaluated |
| `_last_violation` | Timestamp? | When constraint last failed |
| `_avg_check_time` | Float | Average evaluation time in milliseconds |

```
-- Find frequently violated constraints
META MATCH c: _ConstraintDef
WHERE c._violation_count > 0
RETURN c.name, c._violation_count, c._check_count,
       (c._violation_count * 1.0 / c._check_count) AS violation_rate
ORDER BY violation_rate DESC
```

---

# 44. The AGI Learning Loop

## 43.1 Pattern

The typical AGI learning loop uses all three tiers:

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. OBSERVE (Normal Mode - Fast)                                     │
│                                                                      │
│    Observe instance data to find patterns                             │
│    Full optimization, GPU-acceleratable                             │
│                                                                      │
│    MATCH e1: Event, e2: Event                                       │
│    WHERE e1.timestamp < e2.timestamp                                │
│      AND time_diff(e1, e2) < 1000                                   │
│    RETURN e1.type, e2.type, COUNT(*) AS freq                       │
│    ORDER BY freq DESC                                               │
├─────────────────────────────────────────────────────────────────────┤
│ 2. REFLECT (Meta Mode - Observe)                                      │
│                                                                      │
│    Check if pattern is already known                                │
│    Schema queries, introspection                                    │
│                                                                      │
│    META MATCH e: _EdgeType                                          │
│    WHERE e.name = "learned_" ++ pattern_signature                  │
│    RETURN e                                                         │
├─────────────────────────────────────────────────────────────────────┤
│ 3. LEARN (Meta Mode - Modify)                                       │
│                                                                      │
│    Create new types/rules if pattern is novel                      │
│    Schema modification, triggers recompilation                     │
│                                                                      │
│    META SPAWN e: _EdgeType { name = "learned_xyz", ... }           │
│    META SPAWN r: _RuleDef { name = "infer_xyz", ... }              │
├─────────────────────────────────────────────────────────────────────┤
│ 4. APPLY (Normal Mode - Fast)                                       │
│                                                                      │
│    New types are now part of schema                                │
│    Full optimization applies                                        │
│                                                                      │
│    MATCH learned_xyz(e1, e2)                                       │
│    WHERE ...                                                        │
│    RETURN e1, e2                                                    │
└─────────────────────────────────────────────────────────────────────┘
```

## 43.2 Example: Concept Discovery

```
-- STEP 1: Observe co-occurrence patterns (Normal mode)
MATCH e1: Event, e2: Event
WHERE e1.source = e2.source
  AND ABS(e1.timestamp - e2.timestamp) < 5000
RETURN e1.type AS type_a, e2.type AS type_b, COUNT(*) AS co_occur
HAVING co_occur > 100
ORDER BY co_occur DESC
LIMIT 50

-- STEP 2: Check existing correlations (Meta mode)
META MATCH e: _EdgeType
WHERE e.name LIKE "correlation_%"
RETURN e.name

-- STEP 3: Create new correlation type (Meta mode)
-- (For each novel pattern)
META BEGIN
  -- Create edge type
  META SPAWN corr: _EdgeType {
    name = "correlation_" ++ type_a ++ "_" ++ type_b,
    arity = 2
  }
  -- Set up signature...
  
  -- Create maintenance rule
  META SPAWN rule: _RuleDef {
    name = "detect_" ++ type_a ++ "_" ++ type_b,
    priority = 5,
    auto = true
  }
  -- Set up pattern and production...
META COMMIT

-- STEP 4: Use new correlation (Normal mode)
MATCH correlation_typeA_typeB(e1, e2)
RETURN e1, e2
```

## 44.3 Example: Rule Evolution

```
-- Monitor rule effectiveness (Meta mode)
META MATCH r: _RuleDef
WHERE r._invocation_count > 0
RETURN r.name, 
       r._invocation_count,
       (r._success_count * 1.0 / r._invocation_count) AS success_rate
ORDER BY success_rate ASC
LIMIT 10

-- Disable ineffective rules
META DISABLE RULE low_success_rule

-- Boost effective rules
META MATCH r: _RuleDef
WHERE (r._success_count * 1.0 / r._invocation_count) > 0.95 
  AND r._invocation_count > 1000
  AND r.priority < 50
META SET r.priority = 50
```

## 44.4 Example: Self-Modeling

The system representing its own reasoning:

```
-- Create meta-edge for tracking reasoning steps
META SPAWN reasoning_step: _EdgeType {
  name = "_reasoning_step",
  arity = 3
}
-- signature: (from_state, to_state, via_rule)

-- Rule that tracks its own execution
META SPAWN tracker: _RuleDef {
  name = "_track_reasoning",
  priority = 1000,  -- runs after other rules
  auto = true
}
-- Pattern: any rule execution
-- Production: LINK _reasoning_step(before, after, rule_used)

-- Observe own reasoning
META MATCH _reasoning_step(before, after, rule) AS step
WHERE step.timestamp > now() - 60000  -- last minute
RETURN before, after, rule.name, step.timestamp
ORDER BY step.timestamp
```

---

# 45. Performance Characteristics

## 45.1 Cost Model

| Operation | Relative Cost | Notes |
|-----------|---------------|-------|
| Normal MATCH | 1x | Full optimization |
| META MATCH (typed L0) | 2-5x | Known Layer 0 types |
| META MATCH (edge<any>) | 10-100x | Full edge scan |
| META WALK (typed) | 2-5x | Known edge types |
| META WALK (edge<any>) | 10-100x | Generic traversal |
| META SPAWN | 100-1000x | Triggers recompilation |
| META SET (rule/constraint) | 10-100x | Incremental recompile |
| META SET (type) | 100-1000x | Full recompile |
| META KILL | 100-10000x | May cascade to instances |

## 45.2 Optimization Hints

```
-- BAD: Full edge scan
META MATCH e: edge<any>
RETURN e

-- BETTER: Constrain by target
META MATCH e: edge<any>
WHERE has_target(e, #known_node)
RETURN e

-- BEST: Use normal mode if types known
MATCH causes(_, #known_node) AS e
RETURN e
```

```
-- BAD: Multiple schema modifications
META SPAWN t1: _NodeType { ... }  -- recompile
META SPAWN t2: _NodeType { ... }  -- recompile again
META SPAWN t3: _NodeType { ... }  -- recompile again

-- BETTER: Batch modifications
META BEGIN
  META SPAWN t1: _NodeType { ... }
  META SPAWN t2: _NodeType { ... }
  META SPAWN t3: _NodeType { ... }
META COMMIT  -- single recompile
```

## 45.3 When to Use META

| Situation | Recommendation |
|-----------|----------------|
| Known types, instance queries | Normal mode |
| Known types, higher-order queries | Normal mode |
| Schema exploration, tooling | META mode |
| Dynamic type discovery | META mode |
| AGI learning loop | META mode (reflect/learn phases) |
| Performance-critical path | Avoid META if possible |

---

# 46. Complete Grammar (Meta Mode)

```ebnf
(* Meta Statements *)
MetaStatement    = "meta" Statement
                 | MetaCreateStmt
                 | MetaDescribeStmt
                 | MetaDryRunStmt
                 | MetaEnableStmt
                 | MetaDisableStmt
                 | MetaResetStmt

(* META CREATE - Runtime Ontology DSL *)
MetaCreateStmt   = "meta" "create" "node" NodeTypeDef
                 | "meta" "create" "edge" EdgeTypeDef
                 | "meta" "create" "constraint" ConstraintDef
                 | "meta" "create" "rule" RuleDef

(* META DESCRIBE *)
MetaDescribeStmt = "meta" "describe" TypeName
                 | "meta" "describe" "edge" EdgeTypeName
                 | "meta" "describe" NodeRef
                 | "meta" "describe" EdgeRef

(* META DRY RUN *)
MetaDryRunStmt   = "meta" "dry" "run" MetaModificationStmt

(* META ENABLE/DISABLE *)
MetaEnableStmt   = "meta" "enable" ("constraint" | "rule") Identifier
MetaDisableStmt  = "meta" "disable" ("constraint" | "rule") Identifier

(* META RESET *)
MetaResetStmt    = "meta" "reset" "metrics" (Identifier | "all" "rules" | "all" "constraints")

(* Extended for META context *)
GenericEdgePattern = "edge" "<" ("any" | TypeName) ">" ("(" TargetPattern ")")? ("as" Identifier)?
TargetPattern    = "*" | Target ("," Target)*

(* Edge Introspection Functions *)
EdgeIntrospectFunc = ArityFunc | TargetsFunc | TargetFunc | HasTargetFunc
                   | EdgeTypeFunc | SourceTypesFunc | IsHigherOrderFunc | EdgesAboutFunc

ArityFunc        = "arity" "(" Expr ")"
TargetsFunc      = "targets" "(" Expr ")"
TargetFunc       = "target" "(" Expr "," IntLiteral ")"
HasTargetFunc    = "has_target" "(" Expr "," Expr ")"
EdgeTypeFunc     = "edge_type" "(" Expr ")"
SourceTypesFunc  = "source_types" "(" Expr ")"
IsHigherOrderFunc = "is_higher_order" "(" Expr ")"
EdgesAboutFunc   = "edges_about" "(" Expr ")"

(* Node Introspection Functions *)
NodeIntrospectFunc = TypeOfFunc | TypeNodeFunc | AttributesFunc | AttrFunc | HasAttrFunc

TypeOfFunc       = "type_of" "(" Expr ")"
TypeNodeFunc     = "type_node" "(" Expr ")"
AttributesFunc   = "attributes" "(" Expr ")"
AttrFunc         = "attr" "(" Expr "," StringLiteral ")"
HasAttrFunc      = "has_attr" "(" Expr "," StringLiteral ")"

(* Schema Navigation Functions *)
SchemaNavFunc    = ConstraintsOnFunc | RulesAffectingFunc | AttributesOfFunc
                 | SubtypesOfFunc | SupertypesOfFunc
                 | EdgesFromFunc | EdgesToFunc | EdgesInvolvingFunc

ConstraintsOnFunc   = "constraints_on" "(" TypeRef ")"
RulesAffectingFunc  = "rules_affecting" "(" TypeRef ")"
AttributesOfFunc    = "attributes_of" "(" TypeRef ")"
SubtypesOfFunc      = "subtypes_of" "(" TypeRef ")"
SupertypesOfFunc    = "supertypes_of" "(" TypeRef ")"
EdgesFromFunc       = "edges_from" "(" TypeRef ")"
EdgesToFunc         = "edges_to" "(" TypeRef ")"
EdgesInvolvingFunc  = "edges_involving" "(" TypeRef ")"

(* Meta Transaction *)
MetaTransaction  = "meta" "begin" MetaStatement* "meta" "commit"

(* Permissions *)
GrantStmt        = "grant" MetaPermission "to" Identifier
RevokeStmt       = "revoke" MetaPermission "from" Identifier
MetaPermission   = "meta" ("read" | "write" | "admin")

(* Layer 0 Types - for reference *)
Layer0NodeTypes  = "_NodeType" | "_EdgeType" | "_AttributeDef" | "_ConstraintDef"
                 | "_RuleDef" | "_PatternDef" | "_VarDef" | "_Expr" | "_ProductionDef"
                 | "_Ontology" | "_ScalarTypeExpr" | "_NamedTypeExpr" | ...

Layer0EdgeTypes  = "_type_has_attribute" | "_type_inherits" | "_edge_has_position"
                 | "_var_has_type" | "_constraint_has_pattern" | "_constraint_has_condition"
                 | "_rule_has_pattern" | "_rule_has_production" | "_ontology_declares_type"
                 | "_ontology_declares_edge" | "_ontology_declares_constraint"
                 | "_ontology_declares_rule" | ...
```

---

# 47. Summary

## 47.1 Key Concepts

| Concept | Meaning |
|---------|---------|
| **META** | Keyword enabling reflection mode |
| **Reflection** | Reasoning about structure with unknown types |
| **Levels** | 0 (meta-meta), 1 (schema), 2 (instance) |
| **Higher-order** | Edges about edges (instance data, not meta) |
| **edge<any>** | Generic edge pattern (META required) |
| **Introspection** | Functions examining node/edge structure |
| **Recompilation** | Schema changes trigger registry rebuild |
| **Layer 0 Invariants** | Protected structures that cannot be violated |

## 47.2 Operations

| Operation | Purpose | Permission |
|-----------|---------|------------|
| META MATCH | Query with generic patterns / schema access | META READ |
| META WALK | Traverse with generic edges / from edges | META READ |
| META DESCRIBE | Inspect type/instance structure | META READ |
| META DRY RUN | Preview schema modification effects | META READ |
| META SPAWN | Create schema elements (low-level) | META WRITE |
| META LINK | Create schema relationships | META WRITE |
| META SET | Modify schema elements | META WRITE |
| META UNLINK | Remove schema relationships | META WRITE |
| META CREATE | Create schema elements (DSL syntax) | META WRITE |
| META ENABLE | Enable constraint/rule | META WRITE |
| META DISABLE | Disable constraint/rule | META WRITE |
| META KILL | Delete schema elements | META ADMIN |
| META RESET METRICS | Reset rule/constraint metrics | META ADMIN |

## 47.3 Edge Introspection Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| arity(e) | edge → Int | Target count |
| targets(e) | edge → List | All targets |
| target(e, n) | edge × Int → Node/Edge | Nth target |
| has_target(e, t) | edge × any → Bool | Membership test |
| edge_type(e) | edge → String | Type name |
| source_types(e) | edge → List<String> | Target types |
| is_higher_order(e) | edge → Bool | Targets edges? |
| edges_about(e) | edge → List<Edge> | Edges targeting this |

## 47.4 Node Introspection Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| type_of(n) | Node → String | Type name |
| type_node(n) | Node → _NodeType | Type definition node |
| attributes(n) | Node → List<String> | Attribute names |
| attr(n, name) | Node × String → Any | Dynamic attribute access |
| has_attr(n, name) | Node × String → Bool | Attribute existence |

## 47.5 Schema Navigation Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| constraints_on(T) | TypeRef → List<_ConstraintDef> | Constraints affecting type |
| rules_affecting(T) | TypeRef → List<_RuleDef> | Rules involving type |
| attributes_of(T) | TypeRef → List<_AttributeDef> | Type's attributes |
| subtypes_of(T) | TypeRef → List<_NodeType> | Types inheriting from T |
| supertypes_of(T) | TypeRef → List<_NodeType> | Types T inherits from |
| edges_from(T) | TypeRef → List<_EdgeType> | Edges where T is source |
| edges_to(T) | TypeRef → List<_EdgeType> | Edges where T is target |
| edges_involving(T) | TypeRef → List<_EdgeType> | All edges involving T |

## 47.6 Built-in Metrics

### Rule Metrics (_RuleDef)

| Attribute | Type | Description |
|-----------|------|-------------|
| _invocation_count | Int | Times fired |
| _success_count | Int | Successful invocations |
| _error_count | Int | Failed invocations |
| _last_invoked | Timestamp | Last fire time |
| _avg_execution_time | Float | Average ms per invocation |

### Constraint Metrics (_ConstraintDef)

| Attribute | Type | Description |
|-----------|------|-------------|
| _check_count | Int | Times evaluated |
| _violation_count | Int | Violations detected |
| _last_checked | Timestamp | Last evaluation time |
| _avg_check_time | Float | Average ms per check |

## 47.7 Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Explicit cost** | META keyword signals slower path |
| **Tiered access** | Permissions control capabilities |
| **AGI-ready** | Self-modification, learning loop supported |
| **Safety** | Audit logging, Layer 0 invariants, confirmation for destructive ops |
| **Performance** | Normal mode fully optimized, META accepts cost |
| **DX-friendly** | META CREATE DSL, DESCRIBE, DRY RUN for usability |

---

*End of Part IV: Meta Mode*