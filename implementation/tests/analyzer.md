# 5. ANALYZER

## 5.1 Name Resolution

### TEST: resolve_type_name
```
GIVEN registry with type Task (id=1)
INPUT AST: SpawnStmt { type: "Task", ... }
WHEN analyze
THEN annotated AST has type_id: 1
```

### TEST: unknown_type_error
```
GIVEN registry with type Task
INPUT AST: SpawnStmt { type: "Unknown", ... }
WHEN analyze
THEN error: "unknown type 'Unknown'"
```

### TEST: resolve_attribute
```
GIVEN registry with type Task having attr title (id=1)
INPUT AST: AttrAccess { var: "t", attr: "title" }
  (where t: Task in scope)
WHEN analyze
THEN annotated AST has attr_id: 1
```

### TEST: unknown_attribute_error
```
GIVEN registry with type Task (no attr "foo")
INPUT AST: AttrAccess { var: "t", attr: "foo" } where t: Task
WHEN analyze
THEN error: "type 'Task' has no attribute 'foo'"
```

### TEST: resolve_variable_reference
```
INPUT: "MATCH t: Task WHERE t.priority > 5 RETURN t"
WHEN analyze
THEN all references to "t" resolve to the pattern declaration
```

### TEST: undefined_variable_error
```
INPUT: "MATCH t: Task RETURN x"
WHEN analyze
THEN error: "undefined variable 'x'"
```

## 5.2 Type Checking

### TEST: type_check_arithmetic_valid
```
INPUT: "t.priority + 5"
  (where t.priority: Int)
WHEN analyze
THEN expression type is Int
```

### TEST: type_check_arithmetic_invalid
```
INPUT: "t.title + 5"
  (where t.title: String)
WHEN analyze
THEN error: "cannot add String and Int"
```

### TEST: type_check_comparison_valid
```
INPUT: "t.priority > 5"
WHEN analyze
THEN expression type is Bool
```

### TEST: type_check_comparison_type_mismatch
```
INPUT: "t.priority > \"high\""
WHEN analyze
THEN error: "cannot compare Int with String"
```

### TEST: type_check_edge_targets
```
GIVEN edge type owns(owner: Person, item: Item)
INPUT: "LINK owns(p, t)" where p: Person, t: Task
WHEN analyze
THEN error: "owns expects Item at position 2, got Task"
```

### TEST: type_check_subtype_accepted
```
GIVEN:
  - type Item
  - type Task extends Item
  - edge type owns(owner: Person, item: Item)
INPUT: "LINK owns(p, t)" where p: Person, t: Task
WHEN analyze
THEN OK (Task is subtype of Item)
```
