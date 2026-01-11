# 7. PATTERN

## 7.1 Pattern Compilation

### TEST: compile_single_node_pattern
```
INPUT pattern: "t: Task"
EXPECT: compiled pattern that scans Task nodes
```

### TEST: compile_multi_node_with_edge
```
INPUT pattern: "a: Person, b: Person, knows(a, b)"
EXPECT: compiled pattern that:
  1. Scans Person nodes for a
  2. For each a, finds knows edges from a
  3. Checks target is Person
```

### TEST: compile_pattern_with_edge_alias
```
INPUT pattern: "e: knows(a, b)"
EXPECT: compiled pattern that binds edge to variable e
```

## 7.2 Pattern Matching

### TEST: match_single_type
```
GIVEN graph with:
  - Task A, Task B, Person C
PATTERN: "t: Task"
EXPECT matches: [{t: A}, {t: B}]
```

### TEST: match_with_edge
```
GIVEN graph with:
  - Person Alice, Person Bob, Person Carol
  - knows(Alice, Bob)
  - knows(Bob, Carol)
PATTERN: "a: Person, b: Person, knows(a, b)"
EXPECT matches: [
  {a: Alice, b: Bob},
  {a: Bob, b: Carol}
]
```

### TEST: match_with_where_filter
```
GIVEN graph with:
  - Task A (priority=1)
  - Task B (priority=5)
  - Task C (priority=10)
PATTERN: "t: Task WHERE t.priority > 3"
EXPECT matches: [{t: B}, {t: C}]
```

### TEST: match_transitive_closure
```
GIVEN graph with:
  - A, B, C
  - edge(A, B), edge(B, C)
PATTERN: "a, c, edge+(a, c)"
EXPECT matches include {a: A, c: C}
```

### TEST: match_reflexive_transitive_closure
```
GIVEN graph with:
  - A, B
  - edge(A, B)
PATTERN: "a, b, edge*(a, b)"
EXPECT matches: [{a: A, b: A}, {a: A, b: B}, {a: B, b: B}]
```

### TEST: match_not_exists
```
GIVEN graph with:
  - Person Alice (has tasks)
  - Person Bob (no tasks)
  - owns(Alice, Task1)
PATTERN: "p: Person WHERE NOT EXISTS owns(p, _)"
EXPECT matches: [{p: Bob}]
```

## 7.3 Expression Evaluation

### TEST: eval_arithmetic
```
BINDINGS: {x: 10, y: 3}
EXPR: "x + y * 2"
EXPECT: 16
```

### TEST: eval_comparison
```
BINDINGS: {x: 10}
EXPR: "x >= 5 AND x < 20"
EXPECT: true
```

### TEST: eval_attribute_access
```
BINDINGS: {t: Task(priority=5)}
EXPR: "t.priority"
EXPECT: 5
```

### TEST: eval_aggregate_count
```
BINDINGS: {items: [A, B, C]}
EXPR: "COUNT(items)"
EXPECT: 3
```

### TEST: eval_unbound_variable_error
```
BINDINGS: {x: 10}
EXPR: "y + 1"
EXPECT: error "unbound variable 'y'"
```