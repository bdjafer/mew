---
spec: id_references
version: "1.0"
status: draft
category: expression
capability: direct entity lookup
requires: []
priority: common
---

# Spec: ID References

## Overview

ID references provide a syntax for directly referencing nodes and edges by their unique identifiers. This enables direct entity lookup without pattern matching, which is useful for the INSPECT statement, WALK starting points, and any context where a specific known entity needs to be accessed. ID references use the `#` prefix followed by either a simple identifier or a quoted string for complex IDs like UUIDs.

## Syntax

### Grammar

```ebnf
IdRef = "#" (Identifier | StringLiteral)

Identifier = [a-zA-Z_][a-zA-Z0-9_]*

StringLiteral = '"' StringCharacter* '"'

StringCharacter = ~["\\\n\r] | EscapeSequence
```

### Keywords

| Keyword | Context |
|---------|---------|
| `#` | Expression prefix - denotes an ID reference |

### Examples

```
-- Simple ID (alphanumeric with underscores)
#node_abc123

-- Quoted ID for UUIDs
#"550e8400-e29b-41d4-a716-446655440000"

-- Quoted ID with hyphens
#"task-123-abc"

-- Quoted ID with special characters
#"node:type:123"
```

## Semantics

### ID Reference Syntax

The `#` prefix indicates that the following token is a node or edge identifier:

**Simple form:** `#identifier`
- Used for IDs that match the identifier pattern: alphanumeric characters and underscores
- Must start with a letter or underscore
- Examples: `#task_1`, `#Person_42`, `#_system_node`

**Quoted form:** `#"string"`
- Used for IDs containing characters not allowed in identifiers
- Required for UUIDs, hyphens, colons, or other special characters
- The string content is the exact ID value (excluding quotes)
- Examples: `#"550e8400-e29b-41d4"`, `#"task-123"`, `#"ns:type:id"`

### Type

An ID reference expression has type `ID`. The referenced entity may be either a node or an edge - the type is not known until runtime lookup.

### Resolution

ID references are resolved at runtime using a two-phase lookup:

1. **Session binding lookup**: If the identifier matches a session-bound variable name (e.g., from a prior SPAWN statement), the bound entity is returned
2. **Graph entity lookup**: Otherwise, the engine looks up the entity (node or edge) with that literal ID in the graph
3. If found by either method, the entity is returned
4. If not found, the behavior depends on context (see below)

**Why session bindings first?** Node IDs are auto-generated numeric values, not user-specified strings. When you write `SPAWN foo: Type { ... }`, the variable `foo` is bound to the created node's numeric ID. The ID reference `#foo` resolves via session bindings, enabling natural chaining:

```
SPAWN task: Task { title = "Example" }
WALK FROM #task FOLLOW depends_on    -- resolves via session binding
RETURN endpoint
```

For literal graph IDs (e.g., stored UUIDs or known identifiers), the reference resolves directly:

```
WALK FROM #"550e8400-e29b-41d4"    -- resolves via graph lookup
FOLLOW causes
RETURN path
```

### Usage Contexts

**INSPECT statement:**
```
INSPECT #node_123
INSPECT #"550e8400-e29b-41d4"
```
Returns the node or edge with full details if found, or `{found: false}` if not.

**WALK FROM clause:**
```
WALK FROM #task_root
FOLLOW depends_on
RETURN nodes

WALK FROM #"uuid-start-node"
FOLLOW causes
RETURN path
```
The walk begins from the referenced node. Error if node not found.

**Direct lookup in expressions:**
```
-- Reference in LINK/UNLINK
LINK assigned_to(#task_123, #person_456)

-- Reference in comparison
WHERE t.id = #task_123
```

**Subquery with known ID:**
```
MATCH t: Task
WHERE EXISTS(depends_on(t, #known_blocker))
RETURN t
```

### ID Format Considerations

**Simple IDs (unquoted):**
- Match pattern: `[a-zA-Z_][a-zA-Z0-9_]*`
- Cannot start with a digit
- Cannot contain hyphens or special characters
- Case-sensitive: `#Task_1` != `#task_1`

**Quoted IDs:**
- Can contain any characters except unescaped quotes and newlines
- Use escape sequences for special characters: `\"`, `\\`, etc.
- Leading/trailing whitespace is significant
- Case-sensitive

**Common ID formats:**
```
#simple_id_123          -- Simple alphanumeric
#"550e8400-e29b-41d4"   -- UUID (standard format)
#"task-123-abc"         -- Hyphenated IDs
#"node:type:123"        -- Namespaced IDs with colons
#"2024-01-15_task"      -- Date-prefixed (needs quotes due to hyphen)
```

### Distinction from @ Prefix

The `@` prefix is reserved for timestamp literals, not ID references:

```
#task_123       -- ID reference to a node/edge
@2024-01-15     -- Timestamp literal (January 15, 2024)
```

This distinction prevents ambiguity between entity references and temporal values.

### Null and Error Behavior

- An ID reference cannot be `null` - the syntax requires a value
- If the referenced entity does not exist:
  - In INSPECT: Returns `{found: false, type: null, data: null}`
  - In WALK FROM: Produces a runtime error
  - In LINK/UNLINK: Produces a runtime error
  - In comparison: Comparison fails (no match)

## Layer 0

None.

## Examples

### Basic INSPECT Usage

```
-- Inspect a node by simple ID
INSPECT #user_12345
-- Returns: {found: true, type: "node", data: {_id: "user_12345", _type: "User", name: "Alice", ...}}

-- Inspect with projection
INSPECT #task_abc
RETURN title, status, priority

-- Inspect an edge
INSPECT #edge_relationship_1
-- Returns edge details including _targets
```

### UUID-Based Systems

```
-- Inspect a node with UUID
INSPECT #"550e8400-e29b-41d4-a716-446655440000"

-- Walk from UUID-identified node
WALK FROM #"550e8400-e29b-41d4-a716-446655440000"
FOLLOW contains
RETURN nodes

-- Link to UUID-identified entities
LINK member_of(#"user-uuid-123", #"team-uuid-456")
```

### WALK Starting Points

```
-- Walk from known root node
WALK FROM #project_root
FOLLOW subtask_of INBOUND [depth: 10]
RETURN nodes

-- Multi-hop walk from specific person
WALK FROM #person_alice
FOLLOW follows [depth: 3]
FOLLOW member_of
RETURN terminal
```

### Direct Entity Operations

```
-- Kill a specific node
KILL #obsolete_task_123

-- Set attribute on known node
SET #task_456.status = "done"

-- Unlink specific relationship
UNLINK assigned_to(#task_789, #person_old)

-- Create link to known node
SPAWN t: Task { title: "New task" }
LINK depends_on(t, #existing_blocker)
```

### Conditional Logic with Known IDs

```
-- Find tasks blocked by a specific known issue
MATCH t: Task
WHERE EXISTS(depends_on(t, #known_blocker_123))
RETURN t.title, t.status

-- Find people not in a specific team
MATCH p: Person
WHERE NOT EXISTS(member_of(p, #team_alpha))
RETURN p.name
```

### Programmatic ID Construction

```
-- With parameter
INSPECT #$node_id

-- In dynamic contexts (pseudo-code showing intent)
WALK FROM #$start_node
FOLLOW causes
RETURN path
```

### Verifying Entity Existence

```
-- Check if a node exists (in application logic)
INSPECT #maybe_exists_123
-- Check result.found to determine existence

-- Pattern matching still works for batch operations
MATCH t: Task
WHERE t.id = #known_task
RETURN t
```

## Errors

| Condition | Message |
|-----------|---------|
| Missing ID after # | Syntax error: expected identifier or quoted string after '#' |
| Invalid characters in unquoted ID | Syntax error: invalid character '-' in ID. Use quoted form: #"id-with-hyphen" |
| Unterminated quoted ID | Syntax error: unterminated string in ID reference |
| Entity not found (in WALK FROM) | Runtime error: node with ID 'nonexistent_123' not found |
| Entity not found (in LINK/UNLINK) | Runtime error: cannot link - entity with ID 'missing_id' not found |
| Using @ instead of # | Syntax error: '@' is for timestamp literals. Use '#' for ID references |
