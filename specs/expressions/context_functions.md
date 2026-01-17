---
spec: context-functions
version: "1.0"
status: draft
category: expression
capability: policy execution context
requires: [expressions]
priority: essential
---

# Spec: Context Functions

## Overview

Context functions provide access to the execution context (actor, operation, target) within policy conditions. These functions enable policies to make decisions based on who is performing an operation, what operation is being performed, and what entity is being operated on. Context functions are ONLY valid in policy conditions - they require execution context that does not exist in normal constraints or rules.

## Syntax

### Grammar

```ebnf
ContextFunctionCall =
    "current_actor" "(" ")"
  | "operation" "(" ")"
  | "target" "(" ")"
  | "target_type" "(" ")"
  | "target_attr" "(" ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `current_actor` | Expression - returns the session's actor node |
| `operation` | Expression - returns operation type string |
| `target` | Expression - returns target entity |
| `target_type` | Expression - returns type name of target |
| `target_attr` | Expression - returns attribute name for SET operations |

### Examples

```
-- Check if actor is the owner of the target
ALLOW IF EXISTS(owned_by(target(), current_actor()))

-- Check the type of operation being performed
ALLOW IF operation() = "MATCH"

-- Check the type of the target entity
ALLOW IF target_type() = "Task"

-- Check specific attribute for SET operations
ALLOW IF target_attr() = "status"
```

## Semantics

### Function Signatures and Behavior

| Function | Returns | Description | Example Result |
|----------|---------|-------------|----------------|
| `current_actor()` | Node? | The session's actor node | `#alice` |
| `operation()` | String | Operation type string | `"SET"` |
| `target()` | Node? | Target entity or null | `#task_123` |
| `target_type()` | String? | Type name of target | `"Task"` |
| `target_attr()` | String? | Attribute name (for SET) | `"status"` |

### Valid Operations

The `operation()` function returns one of the following strings:

| Operation String | Description |
|------------------|-------------|
| `"SPAWN"` | Node creation |
| `"KILL"` | Node deletion |
| `"LINK"` | Edge creation |
| `"UNLINK"` | Edge deletion |
| `"SET"` | Attribute modification |
| `"MATCH"` | Query/read operation |
| `"META SPAWN"` | Schema node creation |
| `"META KILL"` | Schema node deletion |
| `"META LINK"` | Schema edge creation |
| `"META UNLINK"` | Schema edge deletion |
| `"META SET"` | Schema attribute modification |
| `"META MATCH"` | Schema query operation |

### Return Value Semantics

**current_actor():**
- Returns the actor node bound to the current session
- Returns `null` in system context (sessions without an actor)
- The returned node can be used in pattern matching and edge traversal

**operation():**
- Always returns a non-null string
- Value corresponds to the operation type being evaluated

**target():**
- Returns the target entity for instance-level operations
- Returns `null` for type-level operations (e.g., SPAWN without a specific target)
- Returns `null` for wildcard operations (`*`)

**target_type():**
- Returns the type name of the target entity as a string
- Returns `null` when `target()` is null
- For edges, returns the edge type name

**target_attr():**
- Returns the attribute name for SET operations
- Returns `null` for non-SET operations
- Returns `null` when SET uses wildcard (`_`)

### Null Return Summary

| Function | Returns null when |
|----------|-------------------|
| `current_actor()` | Session has no bound actor (system context) |
| `operation()` | Never - always returns a valid operation string |
| `target()` | Type-level operations, wildcard patterns (`*`) |
| `target_type()` | `target()` is null |
| `target_attr()` | Non-SET operations, or SET with wildcard (`_`) |

Policies should use null-safe comparisons when checking nullable context functions:

```
-- Safe: explicit null check
ALLOW IF target() != null AND EXISTS(owned_by(target(), current_actor()))

-- Safe: null propagates to false in boolean context
ALLOW IF target_type() = "Task"  -- false if target_type() is null
```

### Context-Only Restriction

Context functions are ONLY valid in policy conditions. Using them elsewhere results in a compile-time error:

```
-- VALID: In policy condition
policy owner_access:
  ON SET(t: Task, _)
  ALLOW IF EXISTS(owned_by(t, current_actor()))

-- INVALID: In constraint (no execution context)
constraint: t: Task => owned_by(t, current_actor())  -- ERROR

-- INVALID: In rule condition (no execution context)
rule: t: Task WHERE owned_by(t, current_actor()) => ...  -- ERROR
```

### Purity

Context functions are pure with respect to a single policy evaluation:
- They return the same value for each invocation within a policy condition
- They do not modify any state
- They do not have side effects

The values come from the execution context which is fixed for the duration of the policy check.

## Layer 0

None.

Context functions are built-in primitives provided by the policy evaluation engine. They are not user-definable and have no graph representation.

## Examples

### Owner-Based Access

```
-- Only the owner can modify a resource
policy owner_modify:
  ON SET(r: Resource, _) | KILL(r: Resource)
  ALLOW IF EXISTS(owned_by(r, current_actor()))

-- Only the owner can delete, but anyone can read
policy owner_delete:
  ON KILL(t: Task)
  ALLOW IF EXISTS(owned_by(t, current_actor()))
```

### Operation-Specific Policies

```
-- Read operations are less restricted
policy read_access:
  ON MATCH(t: Task)
  ALLOW IF operation() = "MATCH"
    AND EXISTS(p: Project, belongs_to(t, p), member_of(current_actor(), p))

-- Write operations require editor role
policy write_access:
  ON SET(t: Task, _)
  ALLOW IF operation() = "SET"
    AND EXISTS(has_role(current_actor(), r) WHERE r.name = "editor")
```

### Attribute-Level Control

```
-- Anyone can modify status, but only admins can modify priority
policy status_update:
  ON SET(t: Task, "status")
  ALLOW IF target_attr() = "status"
    AND EXISTS(assigned_to(t, current_actor()))

policy priority_update:
  ON SET(t: Task, "priority")
  ALLOW IF target_attr() = "priority"
    AND EXISTS(has_role(current_actor(), r) WHERE r.name = "admin")
```

### Type-Based Access

```
-- Different rules for different types
policy task_access:
  ON MATCH(_)
  ALLOW IF target_type() = "Task"
    AND EXISTS(p: Project, belongs_to(target(), p), member_of(current_actor(), p))

policy document_access:
  ON MATCH(_)
  ALLOW IF target_type() = "Document"
    AND current_actor().department = target().department
```

### Combined Context Checks

```
-- Complex policy using multiple context functions
policy project_editor:
  ON SET(t: Task, _) | LINK(_, t: Task) | UNLINK(_, t: Task)
  ALLOW IF target_type() = "Task"
    AND target() != null
    AND current_actor() != null
    AND EXISTS(
      p: Project,
      belongs_to(target(), p),
      has_project_role(current_actor(), p, "editor")
    )
```

### System Context Handling

```
-- Allow system operations (no actor bound)
policy system_bypass [priority: 1000]:
  ON *
  ALLOW IF current_actor() = null

-- Require actor for user operations
policy require_actor:
  ON SPAWN(_) | SET(_, _) | KILL(_)
  DENY IF current_actor() = null
  MESSAGE "User operations require an authenticated actor"
```

## Errors

| Condition | Message |
|-----------|---------|
| Context function used outside policy condition | `current_actor` is only valid in policy conditions |
| Context function called without execution context | Cannot evaluate `operation()`: no execution context available |
| Invalid comparison with context function result | Type error: cannot compare Node with String: `current_actor() = "alice"` |
| Accessing target() when no target bound | `target()` returns null (no error, use null-safe comparison) |
