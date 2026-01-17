---
spec: session
version: "1.0"
status: draft
category: statement
capability: policy
requires: [node_reference]
priority: essential
---

# Spec: SESSION

## Overview

Session statements bind an actor to a session, establishing the identity context for all subsequent operations. The actor determines what policy checks apply. Sessions without an explicit actor binding operate in system context, which is unrestricted and used for internal operations, migrations, and bootstrapping.

## Syntax

### Grammar

```ebnf
SessionStmt     = "BEGIN" "SESSION" "AS" NodeRef
                | "END" "SESSION" ;

NodeRef         = "#" Identifier
                | Identifier ;
```

Note: This grammar matches POLICY.md Appendix A (Session Binding).

### Keywords

| Keyword | Context |
|---------|---------|
| `BEGIN` | Statement - initiates session with actor binding |
| `SESSION` | Statement - identifies session context |
| `AS` | Clause - introduces the actor reference |
| `END` | Statement - terminates the current session |

### Examples

```
-- Bind session to a specific actor by ID
BEGIN SESSION AS #alice

-- Bind session to an actor via variable reference
BEGIN SESSION AS current_user

-- End the current session
END SESSION
```

## Semantics

### Actor Binding

When `BEGIN SESSION AS` executes:
1. The referenced node is resolved (by ID or variable)
2. The node is validated as existing in the graph
3. The node is validated as a valid actor type (Person, Agent, or Service)
4. If a session is already active, the new actor is pushed onto the actor stack
5. All subsequent operations are attributed to the bound actor
6. Policy checks use the session's actor for `current_actor()`

### System Context

Sessions without an explicit actor binding operate in **system context**:
- Unrestricted access to all operations (bypasses policy checks)
- Used for internal operations, migrations, and bootstrapping
- No actor attribution on operations
- The `current_actor()` function returns `null` in system context
- Policy conditions that require a non-null actor will fail with E7002 NO_ACTOR_BOUND

### Session Nesting

Sessions can be nested. Each `BEGIN SESSION` pushes a new actor onto a stack; each `END SESSION` pops the top actor, restoring the previous actor (or system context if outermost).

```
BEGIN SESSION AS #admin
  -- Operations as admin
  BEGIN SESSION AS #alice
    -- Operations as alice (admin temporarily suspended)
  END SESSION
  -- Back to admin
END SESSION
-- Back to system context (no actor bound)
```

The nested session fully replaces the active actor for policy purposes. The outer actor is not accessible until `END SESSION` restores it.

### Actor Types

Valid actor types are nodes representing entities capable of performing operations:

| Type | Description |
|------|-------------|
| `Person` | Human users |
| `Agent` | AI agents or automated actors |
| `Service` | System services or external integrations |

The ontology may define additional actor types. Custom actor types must be declared as valid in the ontology configuration.

### Session Lifetime

A session persists until:
1. Explicit `END SESSION` is called
2. The transaction commits or rolls back
3. The connection closes

## Layer 0

None.

Sessions are a **runtime concept**, not persisted graph state. There is no `_Session` Layer 0 type. Session state (including the actor stack) is maintained by the execution engine in memory and is not stored in the graph.

This means:
- Session state does not survive process restarts
- Sessions cannot be queried via MATCH
- Session history is not recorded in the journal (though operations within sessions may be)

## Examples

### Basic Session Usage

```
-- Authenticate and bind session
BEGIN SESSION AS #alice

-- All operations are now attributed to alice
SPAWN t: Task { title = "My task" }
SET t.status = "in_progress"

-- Policy checks use alice as current_actor()
-- e.g., "ALLOW IF assigned_to(t, current_actor())"

END SESSION
```

### Nested Session for Delegation

```
BEGIN SESSION AS #system_admin

-- Admin creates project structure
SPAWN p: Project { name = "Critical Project" }

-- Temporarily act as user to set up their workspace
BEGIN SESSION AS #new_user
  SPAWN t: Task { title = "Welcome task", project = p }
  -- This task is attributed to new_user, not admin
END SESSION

-- Back to admin context
LINK manages(#system_admin, p)

END SESSION
```

### Variable-Based Actor Binding

```
-- Resolve actor from query result
MATCH u: Person WHERE u.email = $email
BEGIN SESSION AS u

-- Operations as the matched user
SPAWN n: Notification { message = "Welcome back!" }

END SESSION
```

### Service Context

```
-- Background job running as service
BEGIN SESSION AS #notification_service

-- Service has specific policy permissions
MATCH t: Task WHERE t.due_date < now()
FOR EACH task IN t:
  SPAWN n: Notification {
    target = task.assignee,
    message = "Task overdue: " ++ task.title
  }

END SESSION
```

## Errors

| Condition | Message |
|-----------|---------|
| END SESSION without active session | `SESSION_NOT_STARTED: Cannot end session - no active session` |
| Policy requires actor but session has none | `NO_ACTOR_BOUND: Operation requires an active session actor` |
| Bound actor does not exist | `INVALID_ACTOR: Actor node '#X' does not exist` |
| Bound actor is not a valid actor type | `INVALID_ACTOR: Node '#X' is not a valid actor type (expected Person, Agent, or Service)` |
