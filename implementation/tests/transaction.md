# 12. TRANSACTION

## 12.1 Basic Operations

### TEST: begin_creates_transaction
```
WHEN BEGIN
THEN new transaction active
AND subsequent operations go to buffer
```

### TEST: commit_applies_changes
```
WHEN BEGIN
AND SPAWN t: Task { title = "X" }
AND COMMIT
THEN node exists in graph
```

### TEST: rollback_discards_changes
```
WHEN BEGIN
AND SPAWN t: Task { title = "X" }
AND ROLLBACK
THEN node does NOT exist in graph
```

## 12.2 Read Your Writes

### TEST: read_uncommitted_within_transaction
```
WHEN BEGIN
AND SPAWN t: Task { title = "X" }
AND MATCH t: Task WHERE t.title = "X"
THEN finds the uncommitted node
```

### TEST: other_session_cannot_read_uncommitted
```
SESSION 1: BEGIN, SPAWN Task
SESSION 2: MATCH Task WHERE title = "X"
THEN Session 2 does NOT see Session 1's uncommitted node
```

## 12.3 Constraint Integration

### TEST: constraint_violation_aborts_transaction
```
GIVEN hard constraint: priority >= 0
WHEN BEGIN
AND SPAWN t: Task { priority = 5 }
AND SPAWN t2: Task { priority = -1 }
THEN second SPAWN fails
AND entire transaction aborted
AND first node NOT committed
```

### TEST: deferred_constraint_at_commit
```
GIVEN deferred cardinality constraint
WHEN BEGIN
AND create nodes that violate at intermediate state
AND fix violation before commit
AND COMMIT
THEN success (only final state matters)
```

## 12.4 Rule Integration

### TEST: rules_fire_before_constraints
```
GIVEN:
  - required constraint: Task.created_at
  - auto rule: on SPAWN Task, SET created_at = NOW()
WHEN SPAWN Task { title = "X" }
THEN rule fires first (sets created_at)
THEN constraint checks (passes)
```

## 12.5 Auto-Commit

### TEST: auto_commit_single_statement
```
GIVEN auto-commit mode (no BEGIN)
WHEN SPAWN t: Task { title = "X" }
THEN immediately committed
AND visible to other sessions
```

## 12.6 Savepoints

### TEST: savepoint_partial_rollback
```
WHEN BEGIN
AND SPAWN A
AND SAVEPOINT s1
AND SPAWN B
AND ROLLBACK TO s1
AND COMMIT
THEN A exists
AND B does NOT exist
```