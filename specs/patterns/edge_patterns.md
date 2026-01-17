---
spec: edge_patterns
version: "1.0"
status: stable
category: pattern
capability: edge_matching
requires: [node_patterns]
priority: essential
---

# Spec: Edge Patterns

## Overview

Edge patterns match relationships between nodes in the graph. They specify edge type, source and target nodes, and optionally bind the edge itself to a variable for attribute access. Edge patterns are the primary mechanism for expressing structural relationships in queries and constraints. Without edge patterns, queries can only operate on isolated nodes without considering connectivity.

## Syntax

### Grammar

```ebnf
EdgePattern     = EdgeType "(" Targets ")" EdgeBinding?

EdgeType        = Identifier                      -- edge type name
                | Identifier "::" Identifier      -- qualified edge type

Targets         = Target "," Target               -- binary edge (source, target)
                | Target ("," Target)+            -- n-ary edge (multiple targets)

Target          = Identifier                      -- bound variable reference
                | "_"                             -- anonymous node

EdgeBinding     = "AS" Identifier                 -- bind edge to variable

HigherOrderEdge = EdgeType "(" EdgeRef "," Target ")"
                | EdgeType "(" Target "," EdgeRef ")"

EdgeRef         = Identifier                      -- reference to bound edge variable
```

### Keywords

| Keyword | Context |
|---------|---------|
| `AS` | Edge binding - binds matched edge to variable |
| `_` | Anonymous target - matches without binding |

### Examples

```
-- Basic edge pattern
member_of(person, team)

-- Edge with binding
assigned_to(task, person) AS assignment

-- Anonymous target
depends_on(task, _)

-- N-ary edge (hypothetical)
collaboration(person1, person2, project)

-- Higher-order edge (edge about edge)
confidence(causes_edge, level)
```

## Semantics

### Edge Matching

An edge pattern `edge_type(source, target)` matches all edges in the graph where:
1. The edge is of type `edge_type` (or a subtype)
2. The source node matches the first target
3. The target node matches the second target

**Variable correlation:**
```
MATCH p: Person, t: Team, member_of(p, t)
-- 'p' and 't' in edge pattern refer to nodes bound earlier
-- Only matches where actual member_of edges exist
```

### Edge Binding with AS

The `AS` keyword binds the matched edge to a variable for attribute access:

```
MATCH p: Person, t: Team, member_of(p, t) AS m
RETURN p.name, t.name, m.role, m.joined_at
```

**Binding semantics:**
- The bound variable has the edge's type
- All edge attributes are accessible via dot notation
- Edge bindings are in scope for WHERE and RETURN clauses

### Target References

Edge targets reference previously declared node variables:

```
MATCH p: Person, t: Team, member_of(p, t)
--    ^^^^^^^^  ^^^^^^   binding declarations
--                        ^^^^^^^^^^^^^^^^^^^^ references
```

**Rules:**
- Targets MUST reference declared node pattern variables
- The same variable MAY appear in multiple edge patterns
- Target order matches edge type signature order

### Anonymous Targets

The underscore `_` matches any node in that position:

```
-- Tasks assigned to anyone
MATCH t: Task, assigned_to(t, _)
RETURN t

-- Tasks that something depends on
MATCH t: Task, depends_on(_, t)
RETURN t
```

**Semantics:**
- `_` represents an existential quantification
- Multiple `_` in the same pattern are independent
- Cannot access `_` in subsequent clauses

### Higher-Order Edges

Edges can have other edges as targets, enabling meta-level relationships:

```
edge causes(e1: Event, e2: Event) { strength: Float }
edge confidence(edge: causes, value: Float) {}

MATCH e1: Event, e2: Event,
      causes(e1, e2) AS c,
      confidence(c, level)
WHERE level > 0.8
RETURN e1, e2, c.strength
```

**Higher-order pattern syntax:**
```
-- Edge binding is required to reference in higher-order pattern
causes(e1, e2) AS c,       -- bind the edge
confidence(c, level)        -- use edge variable as target
```

### Edge Attribute Access

With binding, edge attributes are accessible:

```
MATCH t: Task, p: Person, assigned_to(t, p) AS a
WHERE a.priority > 5
RETURN t.title, p.name, a.assigned_at
```

**Without binding**, edge attributes are inaccessible:

```
MATCH t: Task, p: Person, assigned_to(t, p)
WHERE ?.priority > 5  -- ERROR: no edge variable to access
```

### Type Constraints on Edges

Edge patterns enforce type constraints from the schema:

```
edge member_of(person: Person, team: Team) { ... }

MATCH p: Person, t: Team, member_of(p, t)   -- valid
MATCH t: Task, p: Project, member_of(t, p)   -- ERROR: type mismatch
```

### Multiple Edge Patterns

Patterns can include multiple edges creating complex structural requirements:

```
MATCH p: Person, t1: Team, t2: Team,
      member_of(p, t1),
      member_of(p, t2)
WHERE t1 != t2
RETURN p.name, t1.name, t2.name
-- People who belong to multiple teams
```

### Edge Direction

Edges are inherently directed; source and target positions have meaning:

```
edge follows(follower: Person, followed: Person)

-- Find who Alice follows
MATCH alice: Person, other: Person, follows(alice, other)
WHERE alice.name = "Alice"
RETURN other.name

-- Find who follows Alice
MATCH alice: Person, other: Person, follows(other, alice)
WHERE alice.name = "Alice"
RETURN other.name
```

## Layer 0

### Nodes

```
node _EdgePattern [sealed] {
  negated: Bool = false       -- for NOT EXISTS context
}

node _EdgeBinding [sealed] {
  variable_name: String [required]
}
```

### Edges

```
edge _pattern_has_edge(
  pattern: _PatternDef,
  edge_pattern: _EdgePattern
) {}

edge _edge_pattern_type(
  edge_pattern: _EdgePattern,
  edge_type: _EdgeTypeDef
) {}

edge _edge_pattern_target(
  edge_pattern: _EdgePattern,
  target: _NodePattern | _EdgeBinding
) {
  position: Int [required]    -- 0 for source, 1 for target, etc.
  is_anonymous: Bool = false  -- true for '_'
}

edge _edge_pattern_binding(
  edge_pattern: _EdgePattern,
  binding: _EdgeBinding
) {}
```

### Constraints

```
constraint _edge_pattern_has_type:
  ep: _EdgePattern
  => EXISTS(et: _EdgeTypeDef, _edge_pattern_type(ep, et))

constraint _edge_pattern_target_count:
  ep: _EdgePattern, et: _EdgeTypeDef,
  _edge_pattern_type(ep, et)
  => COUNT(t: _NodePattern | _EdgeBinding, _edge_pattern_target(ep, t))
     = et.arity

constraint _edge_binding_unique:
  eb1: _EdgeBinding, eb2: _EdgeBinding, p: _PatternDef,
  _pattern_has_edge(p, ep1), _edge_pattern_binding(ep1, eb1),
  _pattern_has_edge(p, ep2), _edge_pattern_binding(ep2, eb2)
  WHERE eb1 != eb2
  => eb1.variable_name != eb2.variable_name
```

## Examples

### Basic Edge Query

```
-- Find all team memberships
MATCH p: Person, t: Team, member_of(p, t)
RETURN p.name AS person, t.name AS team
```

### Edge with Attribute Access

```
-- Find recent assignments with their priority
MATCH t: Task, p: Person, assigned_to(t, p) AS a
WHERE a.assigned_at > @2024-01-01
RETURN t.title, p.name, a.priority
ORDER BY a.priority DESC
```

### Multiple Edges - Complex Relationships

```
-- Find tasks in projects owned by the assignee's team
MATCH t: Task, proj: Project, p: Person, team: Team,
      belongs_to(t, proj),
      assigned_to(t, p),
      owns(team, proj),
      member_of(p, team)
RETURN t.title, proj.name, p.name, team.name
```

### Higher-Order Edge Pattern

```
-- Find high-confidence causal relationships
MATCH e1: Event, e2: Event,
      causes(e1, e2) AS c,
      confidence(c, conf_node)
WHERE conf_node.value > 0.9
RETURN e1.description, e2.description, c.strength
```

### Anonymous Target for Existence

```
-- Find all tasks that have dependencies
MATCH t: Task, depends_on(t, _)
RETURN DISTINCT t.title

-- Find all people who are followed by someone
MATCH p: Person, follows(_, p)
RETURN DISTINCT p.name
```

### Edge Pattern in Constraints

```
-- Every task must be in a project
constraint task_belongs_to_project:
  t: Task
  => EXISTS(p: Project, belongs_to(t, p))

-- Managers can only manage their team members
constraint manager_manages_team_member:
  manager: Person, employee: Person, t: Team,
  manages(manager, employee)
  => EXISTS(
    member_of(manager, t),
    member_of(employee, t)
  )
```

### Self-Referential Patterns

```
-- Find people who follow themselves (narcissists)
MATCH p: Person, follows(p, p)
RETURN p.name

-- Find mutual follows
MATCH p1: Person, p2: Person,
      follows(p1, p2),
      follows(p2, p1)
WHERE p1._id < p2._id  -- avoid duplicates
RETURN p1.name, p2.name
```

### Edge Chaining

```
-- Find all entities connected to a task
MATCH t: Task,
      p: Person, assigned_to(t, p) AS assign,
      proj: Project, belongs_to(t, proj),
      team: Team, owns(team, proj)
RETURN t.title,
       p.name AS assignee,
       proj.name AS project,
       team.name AS owning_team,
       assign.priority
```

## Errors

| Condition | Message |
|-----------|---------|
| Unknown edge type | `Unknown edge type 'edge_name'` |
| Target variable not declared | `Variable 'x' not declared in pattern` |
| Wrong number of targets | `Edge type 'E' expects N targets, got M` |
| Type mismatch in target | `Edge 'E' expects 'Type1' at position N, got 'Type2'` |
| Duplicate edge binding name | `Edge binding 'x' already declared` |
| Edge binding shadows node variable | `Edge binding 'x' conflicts with node variable` |
| Attribute access on unbound edge | `Cannot access edge attributes without AS binding` |
| Invalid higher-order reference | `Variable 'x' is not an edge binding` |
| Qualified type not found | `Cannot resolve edge type 'A::B'` |
