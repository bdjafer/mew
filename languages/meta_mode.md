The user wants me to write the full spec for the META mode additions. This should cover:

1. META MATCH - generic patterns, edge<any>, introspection
2. META WALK - generic traversal, starting from edges
3. META transformations - SPAWN, LINK, SET, KILL, UNLINK on Layer 0
4. Permission model
5. Edge introspection functions
6. The conceptual model (levels, reflection)

I'll add this as a new Part IV or as an addition to Part III. Let me write it as Part IV: Meta Mode.
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
  | "meta_edges" "(" Expr ")"
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

### 36.2.8 meta_edges(e)

Returns all edges that target this edge.

```
Signature: edge<any> → List<Edge>

MATCH causes(e1, e2) AS c
RETURN meta_edges(c)  -- [confidence_edge, provenance_edge, ...]

META MATCH e: edge<any>
WHERE COUNT(meta_edges(e)) > 0
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

## 37.2 Examples

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

---

# 43. The AGI Learning Loop

## 43.1 Pattern

The typical AGI learning loop uses all three tiers:

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. OBSERVE (Normal Mode - Fast)                                     │
│                                                                      │
│    Query instance data to find patterns                             │
│    Full optimization, GPU-acceleratable                             │
│                                                                      │
│    MATCH e1: Event, e2: Event                                       │
│    WHERE e1.timestamp < e2.timestamp                                │
│      AND time_diff(e1, e2) < 1000                                   │
│    RETURN e1.type, e2.type, COUNT(*) AS freq                       │
│    ORDER BY freq DESC                                               │
├─────────────────────────────────────────────────────────────────────┤
│ 2. REFLECT (Meta Mode - Query)                                      │
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

## 43.3 Example: Rule Evolution

```
-- Monitor rule effectiveness (Meta mode)
META MATCH r: _RuleDef
RETURN r.name, r.invocation_count, r.success_rate
ORDER BY r.success_rate ASC
LIMIT 10

-- Disable ineffective rules
META SET #low_success_rule.auto = false

-- Boost effective rules
META MATCH r: _RuleDef
WHERE r.success_rate > 0.95 AND r.invocation_count > 1000
  AND r.priority < 50
META SET r.priority = 50
```

## 43.4 Example: Self-Modeling

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

-- Query own reasoning
META MATCH _reasoning_step(before, after, rule) AS step
WHERE step.timestamp > now() - 60000  -- last minute
RETURN before, after, rule.name, step.timestamp
ORDER BY step.timestamp
```

---

# 44. Performance Characteristics

## 44.1 Cost Model

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

## 44.2 Optimization Hints

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

## 44.3 When to Use META

| Situation | Recommendation |
|-----------|----------------|
| Known types, instance queries | Normal mode |
| Known types, higher-order queries | Normal mode |
| Schema exploration, tooling | META mode |
| Dynamic type discovery | META mode |
| AGI learning loop | META mode (reflect/learn phases) |
| Performance-critical path | Avoid META if possible |

---

# 45. Complete Grammar (Meta Mode)

```ebnf
(* Meta Statements *)
MetaStatement    = "meta" Statement

(* Extended for META context *)
GenericEdgePattern = "edge" "<" "any" ">" ("(" TargetPattern ")")? ("as" Identifier)?
TargetPattern    = "*" | Target ("," Target)*

(* Introspection Functions *)
IntrospectFunc   = ArityFunc | TargetsFunc | TargetFunc | HasTargetFunc
                 | EdgeTypeFunc | SourceTypesFunc | IsHigherOrderFunc | MetaEdgesFunc

ArityFunc        = "arity" "(" Expr ")"
TargetsFunc      = "targets" "(" Expr ")"
TargetFunc       = "target" "(" Expr "," IntLiteral ")"
HasTargetFunc    = "has_target" "(" Expr "," Expr ")"
EdgeTypeFunc     = "edge_type" "(" Expr ")"
SourceTypesFunc  = "source_types" "(" Expr ")"
IsHigherOrderFunc = "is_higher_order" "(" Expr ")"
MetaEdgesFunc    = "meta_edges" "(" Expr ")"

(* Meta Transaction *)
MetaTransaction  = "meta" "begin" MetaStatement* "meta" "commit"

(* Permissions *)
GrantStmt        = "grant" MetaPermission "to" Identifier
RevokeStmt       = "revoke" MetaPermission "from" Identifier
MetaPermission   = "meta" ("read" | "write" | "admin")

(* Layer 0 Types - for reference *)
Layer0NodeTypes  = "_NodeType" | "_EdgeType" | "_AttributeDef" | "_ConstraintDef"
                 | "_RuleDef" | "_PatternDef" | "_VarDef" | "_Expr" | "_ProductionDef"
                 | "_Ontology" | "_ScalarTypeRef" | "_NamedTypeRef" | ...

Layer0EdgeTypes  = "_type_has_attribute" | "_type_inherits" | "_edge_has_position"
                 | "_var_has_type" | "_constraint_has_pattern" | "_constraint_has_condition"
                 | "_rule_has_pattern" | "_rule_has_production" | "_ontology_declares_type"
                 | "_ontology_declares_edge" | "_ontology_declares_constraint"
                 | "_ontology_declares_rule" | ...
```

---

# 46. Summary

## 46.1 Key Concepts

| Concept | Meaning |
|---------|---------|
| **META** | Keyword enabling reflection mode |
| **Reflection** | Reasoning about structure with unknown types |
| **Levels** | 0 (meta-meta), 1 (schema), 2 (instance) |
| **Higher-order** | Edges about edges (instance data, not meta) |
| **edge<any>** | Generic edge pattern (META required) |
| **Introspection** | Functions examining edge structure |
| **Recompilation** | Schema changes trigger registry rebuild |

## 46.2 Operations

| Operation | Purpose | Permission |
|-----------|---------|------------|
| META MATCH | Query with generic patterns / schema access | META READ |
| META WALK | Traverse with generic edges / from edges | META READ |
| META SPAWN | Create schema elements | META WRITE |
| META LINK | Create schema relationships | META WRITE |
| META SET | Modify schema elements | META WRITE |
| META UNLINK | Remove schema relationships | META WRITE |
| META KILL | Delete schema elements | META ADMIN |

## 46.3 Introspection Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| arity(e) | edge → Int | Target count |
| targets(e) | edge → List | All targets |
| target(e, n) | edge × Int → Node/Edge | Nth target |
| has_target(e, t) | edge × any → Bool | Membership test |
| edge_type(e) | edge → String | Type name |
| source_types(e) | edge → List<String> | Target types |
| is_higher_order(e) | edge → Bool | Targets edges? |
| meta_edges(e) | edge → List<Edge> | Edges about this |

## 46.4 Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Explicit cost** | META keyword signals slower path |
| **Tiered access** | Permissions control capabilities |
| **AGI-ready** | Self-modification, learning loop supported |
| **Safety** | Audit logging, confirmation for destructive ops |
| **Performance** | Normal mode fully optimized, META accepts cost |

---

*End of Part IV: Meta Mode*