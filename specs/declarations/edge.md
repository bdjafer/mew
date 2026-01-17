---
spec: edge-declaration
version: "1.0"
status: draft
category: declaration
capability: schema-definition
requires: [types, attributes, node-declaration]
priority: essential
---

# Spec: Edge Type Declaration

## Overview

Edge type declarations define the kinds of relationships that can exist between nodes in the graph. Each edge type specifies a name, a signature of typed parameters identifying the connected endpoints, optional modifiers for structural constraints and referential integrity, and optional attributes. Edge types enable typed relationship modeling, cardinality constraints, and higher-order relationships where edges can reference other edges.

## Syntax

### Grammar

```ebnf
EdgeTypeDecl     = DocComment? "edge" Identifier "(" SignatureParams ")" EdgeModifiers? ("{" AttributeDecl* "}")?

SignatureParams  = SignatureParam ("," SignatureParam)*

SignatureParam   = Identifier ":" TypeExpr

EdgeModifiers    = "[" EdgeModifier ("," EdgeModifier)* "]"

EdgeModifier     = "symmetric"
                 | "no_self"
                 | "acyclic"
                 | "unique"
                 | "indexed"
                 | CardinalityModifier
                 | ReferentialModifier

CardinalityModifier = Identifier "->" Cardinality

Cardinality      = IntLiteral                      -- exactly N
                 | IntLiteral ".." IntLiteral      -- N to M
                 | IntLiteral ".." "*"             -- N or more

ReferentialModifier = "on_kill_source:" ReferentialAction
                    | "on_kill_target:" ReferentialAction

ReferentialAction = "cascade" | "unlink" | "prevent"

AttributeDecl    = DocComment? Identifier ":" TypeExpr AttrModifiers? DefaultValue? ","?
```

### Keywords

| Keyword | Context |
|---------|---------|
| `edge` | Declaration - introduces an edge type |
| `symmetric` | Modifier - edge is order-independent |
| `no_self` | Modifier - prevents self-referencing edges |
| `acyclic` | Modifier - prevents cycles through this edge type |
| `unique` | Modifier - prevents duplicate edges with same targets |
| `indexed` | Modifier - creates indexes for traversal |
| `on_kill_source` | Modifier - action when source node is killed |
| `on_kill_target` | Modifier - action when target node is killed |
| `cascade` | Referential action - kill connected nodes |
| `unlink` | Referential action - remove edge (default) |
| `prevent` | Referential action - prevent kill operation |

### Examples

```mew
-- Simple binary edge
edge causes(from: Event, to: Event)

-- Edge with attributes
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required] = now(),
  role: String [in: ["owner", "reviewer", "observer"]] = "owner"
}

-- Polymorphic edge
edge tagged(entity: any, tag: Tag)

-- Higher-order edge (targets another edge)
edge confidence(about: edge<causes>) {
  level: Float [required],
  assessed_by: String?
}

-- N-ary edge (hyperedge)
edge meeting(organizer: Person, attendee1: Person, attendee2: Person, room: Room) {
  scheduled_at: Timestamp [required]
}

-- Edge with modifiers
edge friend_of(a: Person, b: Person) [symmetric]

edge depends_on(from: Task, to: Task) [no_self, acyclic]

edge belongs_to(task: Task, project: Project) [
  task -> 1,
  on_kill_target: cascade
]
```

## Semantics

### Edge Signature

An edge signature defines the parameters (endpoints) of the edge:
- Each parameter has a name and a type
- Parameter types can be node types, union types, `any`, or `edge<T>` for higher-order edges
- The arity of an edge is the number of parameters (typically 2 for binary edges)

### Symmetric Modifier

The `[symmetric]` modifier makes an edge order-independent: `edge(a, b)` equals `edge(b, a)`.

```mew
edge friend_of(a: Person, b: Person) [symmetric]
```

**Semantics:**
- Storage: One edge stored with targets in canonical order
- Matching: `friend_of(x, y)` matches regardless of storage order
- Uniqueness: `friend_of(a, b)` and `friend_of(b, a)` are the same edge

**Type requirement:** Symmetric edges require identical types on both endpoints:

```mew
-- Valid: Same type on both endpoints
edge friend_of(a: Person, b: Person) [symmetric]

-- Compile error: Different types
edge collaboration(person: Person, org: Organization) [symmetric]
-- Error: Cannot apply [symmetric] to edge with different parameter types.
```

**Cardinality on symmetric edges:** When a cardinality constraint is specified on one parameter, it applies symmetrically to both:

```mew
edge married_to(a: Person, b: Person) [symmetric, a -> 0..1]
-- Equivalent to: [symmetric, a -> 0..1, b -> 0..1]
```

Conflicting cardinality specifications on symmetric edges are a compile-time error.

### No Self Modifier

The `[no_self]` modifier prevents an edge from connecting a node to itself:

```mew
edge depends_on(from: Task, to: Task) [no_self]
```

**Compiles to:**
```mew
constraint depends_on_no_self:
  t: Task, depends_on(t, t)
  => false
```

### Acyclic Modifier

The `[acyclic]` modifier prevents cycles through this edge type:

```mew
edge parent_of(parent: Person, child: Person) [acyclic]
```

**Compiles to:**
```mew
constraint parent_of_acyclic:
  p: Person, parent_of+(p, p)
  => false
```

Where `parent_of+(a, b)` matches transitive closure (one or more hops).

**Performance warning:** Cycle detection is O(V+E) per LINK operation. For large graphs (> 10,000 reachable nodes), consider alternatives:
- Application-level cycle detection
- Depth-limited constraints
- Periodic batch validation

### Unique Modifier

The `[unique]` modifier prevents duplicate edges with the same targets:

```mew
edge member_of(person: Person, team: Team) [unique]
```

Only one `member_of(alice, engineering)` can exist.

### Indexed Modifier

The `[indexed]` modifier creates indexes for efficient edge traversal:

```mew
edge causes(from: Event, to: Event) [indexed]
```

Creates indexes for:
- Forward lookup: given `from`, find all `to`
- Reverse lookup: given `to`, find all `from`

### Cardinality Modifiers

Control how many edges a node can have through a particular parameter:

| Syntax | Meaning |
|--------|---------|
| `N` | Exactly N |
| `N..M` | Between N and M (inclusive) |
| `N..*` | N or more |
| `0..1` | At most one (optional) |
| `1` | Exactly one (required) |
| `1..*` | At least one |

**Examples:**
```mew
-- Each task belongs to exactly one project
edge belongs_to(task: Task, project: Project) [task -> 1]

-- Each task has at most one assignee
edge assigned_to(task: Task, person: Person) [task -> 0..1]

-- Each project has at least one task
edge contains(project: Project, task: Task) [project -> 1..*]

-- Each person can manage 0-10 people
edge manages(manager: Person, report: Person) [manager -> 0..10]
```

**Bidirectional cardinality:**
```mew
edge married_to(a: Person, b: Person) [
  symmetric,
  a -> 0..1,
  b -> 0..1
]
```

**Enforcement timing:**

| Component | When Enforced |
|-----------|---------------|
| Maximum | Immediately on LINK |
| Minimum | At transaction COMMIT |

This allows staged creation within a transaction:
```mew
BEGIN
  SPAWN t: Task { title = "Test" }   -- OK: no project yet
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(t, p)              -- OK: now linked
COMMIT                               -- Cardinality checked here
```

### Referential Actions

Control what happens when a connected node is killed. Only valid for binary edges (arity = 2).

| Action | Meaning |
|--------|---------|
| `unlink` | Remove the edge (default) |
| `cascade` | Kill the other endpoint |
| `prevent` | Prevent the kill operation |

**Examples:**
```mew
-- When project is killed, kill all its tasks
edge belongs_to(task: Task, project: Project) [
  on_kill_target: cascade
]

-- Cannot kill team if it has members
edge member_of(person: Person, team: Team) [
  on_kill_target: prevent
]

-- When user is killed, just remove edges (default)
edge created_by(entity: any, user: User) [
  on_kill_target: unlink
]
```

**N-ary edges:** Referential action modifiers are only valid for binary edges. For edges with 3+ parameters, use explicit rules:

```mew
edge meeting(a: Person, b: Person, c: Person)

rule meeting_cascade_on_kill:
  p: Person, meeting(p, _, _) AS m, _pending_kill(p)
  =>
  UNLINK m
```

**Cascade depth limits:**
- Default cascade depth limit: 100
- Default max cascade count: 10,000

### Higher-Order Edges

Edges can target other edges using `edge<T>` types:

```mew
edge confidence(about: edge<causes>) {
  level: Float [required]
}
edge provenance(about: edge<any>) {
  source: String [required]
}
```

**Creating higher-order edges:**
```mew
LINK causes(event1, event2) AS c
LINK confidence(c) { level = 0.85, assessed_by = "expert" }
```

**Lifecycle:** When a base edge is unlinked, all higher-order edges that reference it are automatically unlinked (cascaded). This is implicit and cannot be disabled.

### Edge Attributes

Edges can have attributes like nodes:

```mew
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required] = now(),
  role: String [in: ["owner", "reviewer"]] = "owner"
}
```

If all attributes have defaults or are optional, braces can be omitted at LINK time:
```mew
LINK assigned_to(t, p)                           -- both use defaults
LINK assigned_to(t, p) { role = "reviewer" }     -- one explicit
```

Required attributes without defaults must be provided:
```mew
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required]   -- no default!
}

LINK assigned_to(t, p)                 -- Error: assigned_at required
LINK assigned_to(t, p) { assigned_at = now() }  -- OK
```

## Layer 0

### Nodes

```
_EdgeType:
  name: String          -- the edge type name
  arity: Int            -- number of signature parameters
  symmetric: Bool       -- whether edge is symmetric
  doc: String?          -- documentation comment

_VarDef:
  name: String          -- parameter name
  is_edge_var: Bool     -- true if parameter type is edge<T>

_AttributeDef:
  name: String          -- attribute name
  required: Bool        -- whether required
  unique: Bool          -- whether unique
  indexed: String       -- "none" | "asc" | "desc"
  default_value: Any?   -- serialized default or null
  doc: String?          -- documentation comment
```

### Edges

```
_edge_has_position(edge_type: _EdgeType, var_def: _VarDef) {
  position: Int         -- 0-indexed position in signature
}

_var_has_type(var_def: _VarDef, type_expr: _TypeExpr)
  -- Associates parameter with its type

_edge_has_attribute(edge_type: _EdgeType, attr: _AttributeDef)
  -- Associates attribute with its owning edge type

_attr_has_type(attr: _AttributeDef, type: _TypeExpr)
  -- Associates attribute with its type expression
```

### Constraints

```
-- For [no_self] modifier:
constraint <edge>_no_self:
  n: <Type>, <edge>(n, n)
  => false

-- For [acyclic] modifier:
constraint <edge>_acyclic:
  n: <Type>, <edge>+(n, n)
  => false

-- For cardinality [param -> max]:
constraint <edge>_<param>_max_<N>:
  x: <Type>, y1: <OtherType>, y2: <OtherType>,
  <edge>(x, y1), <edge>(x, y2)
  WHERE y1.id != y2.id
  => false   -- (repeated for each excess over max)

-- For cardinality [param -> min..]:
constraint <edge>_<param>_min_<N>:
  x: <Type>
  => exists(y: <OtherType>, <edge>(x, y))  -- (with count >= N)

-- For [on_kill_target: cascade]:
rule <edge>_cascade_on_kill_target [priority: 1000]:
  s: <Source>, t: <Target>,
  <edge>(s, t),
  _pending_kill(t)
  =>
  KILL s

-- For [on_kill_target: prevent]:
constraint <edge>_prevent_kill_target:
  s: <Source>, t: <Target>,
  <edge>(s, t),
  _pending_kill(t)
  => false
```

## Examples

### Task Management Domain

```mew
-- Basic membership
edge member_of(person: Person, team: Team) [unique] {
  role: String [in: ["lead", "member", "observer"]] = "member",
  joined_at: Timestamp = now()
}

-- Project ownership with referential integrity
edge owns(team: Team, project: Project) [
  project -> 1,
  on_kill_source: prevent
]

-- Task hierarchy with acyclicity
edge belongs_to(task: Task, project: Project) [
  task -> 1,
  on_kill_target: cascade
]

edge subtask_of(child: Task, parent: Task) [
  no_self,
  acyclic,
  child -> 0..1
]

-- Task assignment
edge assigned_to(task: Task, person: Person) [
  task -> 0..1,
  on_kill_target: unlink
] {
  assigned_at: Timestamp = now(),
  assigned_by: String?
}

-- Task dependencies
edge depends_on(downstream: Task, upstream: Task) [no_self, acyclic]

-- Tagging
edge tagged(task: Task, tag: Tag) [unique]
```

### Social Network Domain

```mew
-- Symmetric friendship
edge friend_of(a: Person, b: Person) [symmetric, a -> 0..500]

-- Asymmetric following
edge follows(follower: Person, followed: Person) [
  no_self,
  unique,
  follower -> 0..1000
]

-- Blocking (prevents other edges)
edge blocked(blocker: Person, blocked: Person) [
  no_self,
  unique
]

-- Message edges
edge sent_message(from: Person, to: Person) {
  content: String [required, length: 1..10000],
  sent_at: Timestamp = now(),
  read: Bool = false
}
```

### Knowledge Graph with Higher-Order Edges

```mew
-- Base causal edge
edge causes(from: Event, to: Event)

-- Higher-order confidence
edge confidence(about: edge<causes>) {
  level: Float [required],
  assessed_by: String?,
  assessed_at: Timestamp = now()
}

-- Higher-order provenance
edge provenance(about: edge<any>) {
  source: String [required],
  retrieved_at: Timestamp = now()
}

-- Usage:
-- LINK causes(earthquake, tsunami) AS c
-- LINK confidence(c) { level = 0.95, assessed_by = "seismologist" }
-- LINK provenance(c) { source = "USGS database" }
```

### Comprehensive Edge Example

```mew
--- Task assignment with full constraints and attributes
edge assigned_to(task: Task, person: Person) [
  no_self,                        -- not applicable but showing syntax
  unique,                         -- no duplicate assignments
  task -> 0..1,                   -- each task has at most one assignee
  on_kill_target: unlink          -- kill person -> remove assignment
] {
  --- When the assignment was made
  assigned_at: Timestamp [required] = now(),

  --- Who made the assignment
  assigned_by: String?,

  --- Role in this assignment
  role: String [in: ["owner", "reviewer", "observer"]] = "owner",

  --- Optional notes
  notes: String? [length: 0..500]
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Duplicate edge type name | Edge type `<name>` already defined in this ontology |
| Unknown parameter type | Type `<name>` not found for parameter `<param>` |
| Empty signature | Edge type `<name>` must have at least one parameter |
| Symmetric with different types | Cannot apply [symmetric] to edge with different parameter types: `<type1>` vs `<type2>` |
| Symmetric cardinality conflict | Symmetric edge `<name>` has conflicting cardinality: `<param1>` -> `<card1>` vs `<param2>` -> `<card2>` |
| Referential action on n-ary edge | Referential actions only supported for binary edges (arity = 2). Edge `<name>` has arity `<n>`. Use explicit rules instead |
| Unknown parameter in cardinality | Cardinality constraint references unknown parameter `<param>` |
| Invalid cardinality range | Cardinality minimum `<min>` is greater than maximum `<max>` |
| Acyclic on non-homogeneous edge | [acyclic] requires edge endpoints to have compatible types for cycle detection |
| No_self on heterogeneous edge | [no_self] requires edge endpoints to have compatible types |
| Duplicate cardinality constraint | Cardinality for parameter `<param>` specified multiple times |
| Cascade depth exceeded | Cascade depth limit exceeded at `<node>`. Limit: `<limit>` |
| Cascade count exceeded | Cascade count limit exceeded. Affected: `<count>` entities. Limit: `<limit>` |
