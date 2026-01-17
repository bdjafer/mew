---
spec: transactions
version: "1.0"
status: draft
category: statement
capability: transaction-control
requires: []
priority: essential
---

# Spec: Transactions

## Overview

Transaction statements group multiple operations into an atomic unit. Either all operations succeed and are committed together, or all fail and the graph remains unchanged. Transactions provide isolation between concurrent operations and ensure data consistency through constraint checking at commit time.

## Syntax

### Grammar

```ebnf
TransactionStmt  = BeginStmt | CommitStmt | RollbackStmt | SavepointStmt | RollbackToStmt

BeginStmt        = "begin" IsolationLevel?
IsolationLevel   = "read" "committed" | "serializable"

CommitStmt       = "commit"

RollbackStmt     = "rollback"

SavepointStmt    = "savepoint" Identifier

RollbackToStmt   = "rollback" "to" Identifier
```

### Keywords

| Keyword | Context |
|---------|---------|
| `begin` | Statement - starts a new transaction |
| `commit` | Statement - commits the current transaction |
| `rollback` | Statement - aborts and rolls back the transaction |
| `savepoint` | Statement - creates a named checkpoint within transaction |
| `read committed` | Modifier - default isolation level |
| `serializable` | Modifier - strict isolation level |

### Examples

```
-- Basic transaction
BEGIN
  SPAWN t: Task { title = "New task" }
  SET #existing.status = "done"
COMMIT

-- Transaction with isolation level
BEGIN SERIALIZABLE
  SET #counter.value = #counter.value + 1
COMMIT

-- Explicit rollback
BEGIN
  SPAWN t: Task { title = "Test" }
ROLLBACK

-- Savepoint usage
BEGIN
  SPAWN p: Project { name = "Main" }
  SAVEPOINT sp1
    SPAWN t: Task { title = "Subtask" }
  ROLLBACK TO sp1
  SPAWN t: Task { title = "Different task" }
COMMIT
```

## Semantics

### Auto-Commit Mode

Without explicit `BEGIN`, each statement executes as its own transaction:

```
SPAWN t: Task { title = "Task 1" }  -- commits immediately
SPAWN t: Task { title = "Task 2" }  -- commits immediately
```

### Explicit Transactions

Multiple operations grouped atomically:

```
BEGIN
  SPAWN t: Task { title = "Test" }
  SET #existing_task.status = "done"
  LINK depends_on(#t, #other_task)
COMMIT  -- All committed atomically
```

If any operation fails, automatic rollback occurs:

```
BEGIN
  SPAWN t: Task { title = "Test" }       -- succeeds tentatively
  SET #nonexistent.status = "done"       -- fails
COMMIT
-- Automatic rollback: Task not created
-- ERROR: Node #nonexistent not found
```

### Isolation Levels

**READ COMMITTED (default):**
- Sees committed data from other transactions
- No dirty reads
- Default if no isolation level specified

```
BEGIN READ COMMITTED
  -- Sees committed data from other transactions
COMMIT
```

**SERIALIZABLE:**
- Full isolation
- Transactions appear to execute sequentially
- May fail with serialization errors on conflict

```
BEGIN SERIALIZABLE
  -- Full isolation from concurrent transactions
COMMIT
```

### Constraint Checking Timing

| Constraint Type | When Checked |
|-----------------|--------------|
| Type validation | At operation |
| Required attributes | At operation |
| Value constraints | At operation |
| Unique | At operation |
| no_self, acyclic | At operation |
| Cardinality | At COMMIT |

Deferred cardinality checking allows temporary violations within a transaction:

```
BEGIN
  SPAWN t: Task { title = "Test" }
  -- Cardinality [task -> 1] on belongs_to not yet checked
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(#t, #p)
  -- Now cardinality satisfied
COMMIT  -- Cardinality verified here
```

### Savepoints (Nested Transactions)

Savepoints create checkpoints within a transaction for partial rollback:

```
BEGIN
  SPAWN p: Project { name = "Main" }

  SAVEPOINT sp1
    SPAWN t: Task { title = "Subtask" }
    -- Something goes wrong
  ROLLBACK TO sp1
  -- Task not created, Project still pending

  SPAWN t: Task { title = "Different task" }
COMMIT
-- Only Project and "Different task" committed
```

### Rule Execution

Rules execute within the transaction context:

```
BEGIN
  SPAWN t: Task { title = "Test", status = "done" }
  -- Rule 'auto_complete_timestamp' fires
  -- SET t.completed_at = now() happens within transaction
COMMIT
-- t has completed_at set
```

## Layer 0

None. Transaction control is a runtime execution concept with no graph representation.

## Examples

### Multi-Entity Creation

```
BEGIN
  SPAWN p: Project { name = "New Project", status = "active" }
  SPAWN t1: Task { title = "Setup environment" }
  SPAWN t2: Task { title = "Write tests" }
  LINK belongs_to(#t1, #p)
  LINK belongs_to(#t2, #p)
  LINK depends_on(#t2, #t1)
COMMIT
```

### Atomic Counter Update

```
BEGIN SERIALIZABLE
  SET #stats.view_count = #stats.view_count + 1
  SET #stats.last_viewed = now()
COMMIT
```

### Conditional Rollback

```
BEGIN
  SPAWN t: Task { title = $title }

  -- Validate business rule
  SAVEPOINT check
  LINK belongs_to(#t, #$project_id)
  -- If project is archived, rollback
ROLLBACK TO check
-- Continue with alternative logic
COMMIT
```

## Errors

| Condition | Message |
|-----------|---------|
| COMMIT outside transaction | No active transaction to commit |
| ROLLBACK outside transaction | No active transaction to rollback |
| BEGIN inside transaction | Transaction already active |
| Unknown savepoint | Savepoint 'name' does not exist |
| Serialization conflict | Transaction conflicts with concurrent transaction (SERIALIZABLE) |
| Constraint violation at commit | Cardinality constraint 'name' violated |
