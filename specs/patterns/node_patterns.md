---
spec: node_patterns
version: "1.0"
status: stable
category: pattern
capability: node_matching
requires: []
priority: essential
---

# Spec: Node Patterns

## Overview

Node patterns are the fundamental building blocks for matching nodes in graph queries and constraints. They bind variables to nodes that satisfy type constraints, enabling subsequent filtering, projection, and mutation operations. Without node patterns, queries cannot reference specific nodes in the graph. Node patterns support typed matching, union types, anonymous nodes, and the universal `any` type for maximum flexibility.

## Syntax

### Grammar

```ebnf
Pattern         = PatternElement ("," PatternElement)*

PatternElement  = NodePattern | EdgePattern

NodePattern     = Identifier ":" TypeConstraint

TypeConstraint  = TypeName
                | TypeName "|" TypeConstraint     -- union type
                | "any"                           -- any node type

TypeName        = Identifier                      -- simple type
                | Identifier "::" Identifier      -- qualified type

AnonymousNode   = "_"                             -- wildcard in edge targets
```

### Keywords

| Keyword | Context |
|---------|---------|
| `any` | Type constraint - matches any node type |
| `_` | Anonymous node - matches any node without binding |

### Examples

```
-- Basic typed node pattern
p: Person

-- Union type pattern (matches Person OR Organization)
entity: Person | Organization

-- Multiple node patterns
p: Person, t: Team, proj: Project

-- Any type pattern
n: any

-- Anonymous node in edge context
assigned_to(task, _)
```

## Semantics

### Variable Binding

A node pattern `var: Type` introduces a variable `var` bound to each node of type `Type` (or its subtypes) that exists in the graph. The variable remains in scope for the duration of the statement.

**Binding rules:**
- Each variable name MUST be unique within the pattern scope
- Variables are bound lazily during pattern evaluation
- A pattern with no matches evaluates to an empty result set

### Type Matching

Node patterns match based on the declared type constraint:

| Pattern | Matches |
|---------|---------|
| `p: Person` | Nodes of type `Person` and all subtypes |
| `e: Task \| Bug` | Nodes of type `Task` OR `Bug` (and subtypes) |
| `n: any` | Any node regardless of type |

**Subtype semantics:**
```
node Employee : Person { ... }

MATCH p: Person RETURN p
-- Returns both Person nodes AND Employee nodes
```

### Anonymous Nodes

The underscore `_` acts as a wildcard that matches any node without creating a binding:

```
-- Match tasks assigned to someone (don't care who)
MATCH t: Task, assigned_to(t, _)
RETURN t

-- The _ position matches any Person node but doesn't bind it
```

**Usage rules:**
- `_` can only appear as an edge target, not as a standalone pattern
- Multiple `_` in the same pattern are independent (not correlated)
- Cannot access attributes of `_` (it's unbound)

### Union Types

Union type constraints match nodes of any listed type:

```
entity: Person | Organization | Team
```

**Semantics:**
- Acts as logical OR across types
- Each branch includes subtypes
- Order is irrelevant (commutative)
- Duplicate types are collapsed

### The `any` Type

The `any` keyword matches nodes of any type:

```
MATCH n: any
RETURN n._type, n._id
```

**Use cases:**
- Schema exploration
- Generic traversal utilities
- Debugging queries

**Restrictions:**
- Attribute access on `any` typed nodes requires runtime type checking
- Only universal attributes (`_id`, `_type`) are guaranteed available

### Typing Rules

Node pattern variables have static types derived from their constraints:

| Pattern | Variable Type |
|---------|---------------|
| `p: Person` | `Person` |
| `e: Task \| Bug` | `Task \| Bug` |
| `n: any` | `any` |

Type information flows to subsequent clauses (WHERE, RETURN) for type checking.

## Layer 0

### Nodes

```
node _NodePattern [sealed] {
  variable_name: String [required]
}

node _TypeConstraint [sealed] {
  is_any: Bool = false        -- true for 'any' type
}

node _UnionTypeConstraint : _TypeConstraint [sealed] {}
```

### Edges

```
edge _pattern_has_node(
  pattern: _PatternDef,
  node_pattern: _NodePattern
) {}

edge _node_pattern_type(
  node_pattern: _NodePattern,
  type_constraint: _TypeConstraint
) {}

edge _type_constraint_member(
  union: _UnionTypeConstraint,
  type: _TypeDef
) { position: Int }

edge _type_constraint_type(
  constraint: _TypeConstraint,
  type: _TypeDef
) {}
```

### Constraints

```
constraint _node_pattern_has_type:
  np: _NodePattern
  => EXISTS(tc: _TypeConstraint, _node_pattern_type(np, tc))

constraint _node_pattern_unique_name:
  np1: _NodePattern, np2: _NodePattern, p: _PatternDef,
  _pattern_has_node(p, np1), _pattern_has_node(p, np2)
  WHERE np1 != np2
  => np1.variable_name != np2.variable_name
```

## Examples

### Basic Query with Node Patterns

```
-- Find all active users
MATCH u: User
WHERE u.status = "active"
RETURN u.name, u.email
```

### Multiple Node Patterns with Edge

```
-- Find person-team relationships
MATCH p: Person, t: Team, member_of(p, t)
RETURN p.name, t.name
```

### Union Type Pattern

```
-- Find all assignable entities
MATCH entity: Person | Team | Bot
WHERE entity.available = true
RETURN entity._type, entity._id
```

### Using any Type for Schema Exploration

```
-- Count nodes by type
MATCH n: any
RETURN n._type, COUNT(n)
```

### Anonymous Nodes for Existence Checks

```
-- Find tasks that have at least one dependency
MATCH t: Task, depends_on(t, _)
RETURN DISTINCT t
```

### Nested Pattern with Constraints

```
-- Find projects with tasks assigned to active people
MATCH proj: Project, t: Task, p: Person,
      belongs_to(t, proj),
      assigned_to(t, p)
WHERE p.active = true
RETURN proj.name, COUNT(t) AS active_tasks
```

### Pattern in Constraint Definition

```
constraint person_has_email:
  p: Person
  => p.email != null

constraint task_in_project:
  t: Task
  => EXISTS(proj: Project, belongs_to(t, proj))
```

## Errors

| Condition | Message |
|-----------|---------|
| Duplicate variable name | `Variable 'x' already declared in pattern` |
| Unknown type in constraint | `Unknown type 'TypeName'` |
| Attribute access on `_` | `Cannot access attributes of anonymous node` |
| `_` used as standalone pattern | `Anonymous node '_' can only appear as edge target` |
| Invalid union type syntax | `Expected type name in union type` |
| Reserved identifier used | `Identifier '_x' is reserved for Layer 0` |
| Empty pattern | `Pattern must contain at least one element` |
