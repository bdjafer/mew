# 10. CONSTRAINT

## 10.1 Immediate Constraints

### TEST: immediate_constraint_checked_after_mutation
```
GIVEN constraint: Task.priority >= 0
WHEN SPAWN t: Task { priority = -1 }
THEN error immediately: "constraint violated: priority >= 0"
```

### TEST: immediate_constraint_passes
```
GIVEN constraint: Task.priority >= 0
WHEN SPAWN t: Task { priority = 5 }
THEN success
```

## 10.2 Deferred Constraints

### TEST: deferred_constraint_checked_at_commit
```
GIVEN deferred constraint: every Task must have owner
WHEN BEGIN
AND SPAWN t: Task { title = "X" }
(no owner assigned yet)
THEN no error yet
WHEN COMMIT
THEN error: "constraint violated: Task must have owner"
```

### TEST: deferred_constraint_satisfied_before_commit
```
GIVEN deferred constraint: every Task must have owner
WHEN BEGIN
AND SPAWN t: Task { title = "X" }
AND SPAWN p: Person { name = "Alice" }
AND LINK owns(p, t)
AND COMMIT
THEN success
```

## 10.3 Hard vs Soft

### TEST: hard_constraint_aborts
```
GIVEN hard constraint: priority >= 0
WHEN SPAWN t: Task { priority = -1 }
THEN transaction aborted
AND node NOT created
```

### TEST: soft_constraint_warns
```
GIVEN soft constraint: priority <= 10
WHEN SPAWN t: Task { priority = 100 }
THEN warning: "soft constraint violated"
AND node IS created
```

## 10.4 Constraint Patterns

### TEST: constraint_with_pattern
```
GIVEN constraint: "NOT EXISTS t2: Task WHERE t != t2 AND t.code = t2.code"
AND existing Task with code="ABC"
WHEN SPAWN Task { code = "ABC" }
THEN error: "uniqueness constraint violated"
```

## 10.5 Acyclicity

### TEST: acyclic_constraint_prevents_cycle
```
GIVEN acyclic constraint on depends_on edge
AND A depends_on B, B depends_on C
WHEN LINK depends_on(C, A)
THEN error: "cycle detected: A -> B -> C -> A"
```