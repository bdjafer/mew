# 4. REGISTRY

## 4.1 Type Lookup

### TEST: get_type_by_name
```
GIVEN registry with type Task (id=1)
WHEN get_type("Task")
THEN returns TypeDef { id: 1, name: "Task", ... }
```

### TEST: get_type_by_name_not_found
```
GIVEN registry with type Task
WHEN get_type("Unknown")
THEN returns None
```

### TEST: get_type_by_id
```
GIVEN registry with type Task (id=1)
WHEN get_type_by_id(1)
THEN returns TypeDef { name: "Task", ... }
```

## 4.2 Edge Type Lookup

### TEST: get_edge_type_by_name
```
GIVEN registry with edge type owns
WHEN get_edge_type("owns")
THEN returns EdgeTypeDef { name: "owns", ... }
```

## 4.3 Inheritance

### TEST: is_subtype_direct
```
GIVEN registry with:
  - type Entity (id=1)
  - type Task (id=2) extends Entity
WHEN is_subtype(Task, Entity)
THEN returns true
```

### TEST: is_subtype_transitive
```
GIVEN registry with:
  - type Entity
  - type Task extends Entity
  - type BugReport extends Task
WHEN is_subtype(BugReport, Entity)
THEN returns true
```

### TEST: is_subtype_self
```
GIVEN registry with type Task
WHEN is_subtype(Task, Task)
THEN returns true
```

### TEST: get_all_subtypes
```
GIVEN registry with:
  - type Entity
  - type Task extends Entity
  - type BugReport extends Task
  - type Feature extends Task
WHEN get_subtypes(Entity)
THEN returns {Task, BugReport, Feature}
```

## 4.4 Constraint Lookup

### TEST: get_constraints_for_type
```
GIVEN registry with:
  - type Task
  - constraint C1 affects Task
  - constraint C2 affects Task
  - constraint C3 affects Person
WHEN get_constraints(Task)
THEN returns [C1, C2]
```

### TEST: get_deferred_constraints
```
GIVEN registry with:
  - constraint C1 (immediate)
  - constraint C2 (deferred)
  - constraint C3 (deferred)
WHEN get_deferred_constraints()
THEN returns [C2, C3]
```

## 4.5 Rule Lookup

### TEST: get_rules_for_type_sorted_by_priority
```
GIVEN registry with:
  - rule R1 on Task, priority=10
  - rule R2 on Task, priority=50
  - rule R3 on Task, priority=20
WHEN get_rules(Task)
THEN returns [R2, R3, R1] (descending priority)
```
