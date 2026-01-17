# 11. RULE

## 11.1 Basic Triggering

### TEST: rule_triggers_on_spawn
```
GIVEN rule: "on SPAWN Task: SET t.created_at = NOW()"
WHEN SPAWN t: Task { title = "X" }
THEN t.created_at is set
```

### TEST: rule_triggers_on_link
```
GIVEN rule: "on LINK owns(p, t): SET t.owner_name = p.name"
WHEN LINK owns(Alice, Task1)
THEN Task1.owner_name = "Alice"
```

### TEST: rule_triggers_on_set
```
GIVEN rule: "on SET Task.status: SET t.updated_at = NOW()"
WHEN SET task.status = "done"
THEN task.updated_at is updated
```

## 11.2 Priority Order

### TEST: rules_execute_in_priority_order
```
GIVEN:
  - rule R1 priority=10: append "A" to log
  - rule R2 priority=50: append "B" to log
  - rule R3 priority=20: append "C" to log
WHEN trigger all rules
THEN log = ["B", "C", "A"]
```

## 11.3 Cascading

### TEST: rule_actions_trigger_more_rules
```
GIVEN:
  - rule R1: on SPAWN Task, LINK default_owner(admin, t)
  - rule R2: on LINK default_owner, SET t.has_owner = true
WHEN SPAWN Task
THEN R1 fires (creates link)
AND R2 fires (from R1's link)
AND task.has_owner = true
```

## 11.4 Termination

### TEST: same_binding_executes_once
```
GIVEN rule: "on Task: SPAWN helper for t"
AND rule can match same task multiple ways
WHEN trigger
THEN rule executes once per unique task
```

### TEST: depth_limit_prevents_infinite_recursion
```
GIVEN rule that spawns another Task (recursive)
WHEN SPAWN Task
THEN stops at MAX_DEPTH (100)
AND error or warning about depth limit
```

### TEST: action_limit_prevents_runaway
```
GIVEN rule that spawns many nodes per trigger
WHEN trigger
THEN stops at MAX_ACTIONS (10,000)
AND error about action limit
```

## 11.5 Quiescence

### TEST: reach_quiescence
```
GIVEN rules that eventually stop producing new matches
WHEN initial mutation
THEN all rules fire until no new matches
AND transaction completes
```

## 11.6 Manual Rules

### TEST: manual_rule_does_not_auto_fire
```
GIVEN rule R1 [auto: false]: on Task, do something
WHEN SPAWN Task
THEN R1 does NOT fire
```

### TEST: manual_rule_fires_on_explicit_call
```
GIVEN rule R1 [auto: false]
WHEN FIRE R1 WITH { t: task1 }
THEN R1 fires with given bindings
```