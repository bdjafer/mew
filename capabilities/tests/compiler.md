# 6. COMPILER

## 6.1 Sugar Expansion

### TEST: expand_required_modifier
```
INPUT: "node Task { title: String [required] }"
EXPECT: generates constraint:
  "constraint _Task_title_required on Task [hard, immediate]:
     t.title IS NOT NULL"
```

### TEST: expand_unique_modifier
```
INPUT: "node Task { code: String [unique] }"
EXPECT: generates constraint:
  "constraint _Task_code_unique on Task [hard, immediate]:
     NOT EXISTS t2: Task WHERE t != t2 AND t.code = t2.code"
```

### TEST: expand_range_modifier
```
INPUT: "node Task { priority: Int [>= 0, <= 10] }"
EXPECT: generates constraint:
  "constraint _Task_priority_range on Task [hard, immediate]:
     t.priority >= 0 AND t.priority <= 10"
```

### TEST: expand_acyclic_modifier
```
INPUT: "edge depends_on(a: Task, b: Task) [acyclic]"
EXPECT: generates constraint:
  "constraint _depends_on_acyclic on depends_on [hard, immediate]:
     NOT path(a, b) via depends_on"
```

### TEST: expand_cascade_modifier
```
INPUT: "edge owns(owner: Person, item: Item) [on_kill: cascade]"
EXPECT: generates rule:
  "rule _owns_cascade on KILL Person [auto]:
     MATCH owns(p, i) WHERE p = killed
     KILL i"
```

## 6.2 Validation

### TEST: detect_duplicate_type
```
INPUT: "node Task { ... } node Task { ... }"
EXPECT: error "duplicate type name 'Task'"
```

### TEST: detect_inheritance_cycle
```
INPUT: "node A extends B { } node B extends A { }"
EXPECT: error "inheritance cycle: A -> B -> A"
```

### TEST: detect_unknown_type_reference
```
INPUT: "edge owns(a: Person, b: Unknown)"
EXPECT: error "unknown type 'Unknown'"
```

### TEST: detect_unknown_parent_type
```
INPUT: "node Task extends Unknown { }"
EXPECT: error "unknown parent type 'Unknown'"
```

## 6.3 Layer 0 Generation

### TEST: generate_node_type
```
INPUT: "node Task { title: String }"
EXPECT in graph:
  - _NodeType node with name="Task"
  - _AttributeDef node with name="title", value_type="String"
  - _type_has_attribute edge connecting them
```

### TEST: generate_edge_type
```
INPUT: "edge owns(owner: Person, item: Item)"
EXPECT in graph:
  - _EdgeType node with name="owns", arity=2
  - _EdgePosition nodes with position=0, position=1
  - _edge_has_position edges connecting them
  - Position nodes reference Person and Item types
```

## 6.4 Registry Building

### TEST: build_registry_from_ontology
```
INPUT: "node Task { title: String } edge owns(p: Person, t: Task)"
WHEN compile
THEN registry.get_type("Task") returns TypeDef
AND registry.get_edge_type("owns") returns EdgeTypeDef
```
