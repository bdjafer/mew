# 8. MUTATION

## 8.1 SPAWN

### TEST: spawn_valid_node
```
GIVEN registry with type Task (attrs: title String, priority Int)
WHEN SPAWN t: Task { title = "Hello", priority = 1 }
THEN node created with correct type and attributes
AND returns created NodeId
```

### TEST: spawn_abstract_type_error
```
GIVEN registry with abstract type Entity
WHEN SPAWN e: Entity { }
THEN error: "cannot instantiate abstract type 'Entity'"
```

### TEST: spawn_missing_required_error
```
GIVEN registry with type Task, attr title [required]
WHEN SPAWN t: Task { priority = 1 }
THEN error: "missing required attribute 'title'"
```

### TEST: spawn_wrong_attr_type_error
```
GIVEN registry with type Task, attr priority: Int
WHEN SPAWN t: Task { priority = "high" }
THEN error: "attribute 'priority' expects Int, got String"
```

### TEST: spawn_applies_defaults
```
GIVEN registry with type Task, attr status: String [default: "pending"]
WHEN SPAWN t: Task { title = "Hello" }
THEN node has status = "pending"
```

## 8.2 KILL

### TEST: kill_existing_node
```
GIVEN graph with node A
WHEN KILL A
THEN node A deleted
```

### TEST: kill_nonexistent_error
```
GIVEN empty graph
WHEN KILL (id=999)
THEN error: "node not found"
```

### TEST: kill_cascades_edges
```
GIVEN graph with nodes A, B and edge E(A, B)
WHEN KILL A
THEN node A deleted
AND edge E deleted
```

## 8.3 LINK

### TEST: link_valid_edge
```
GIVEN graph with Person p, Task t
AND edge type owns(owner: Person, item: Task)
WHEN LINK e: owns(p, t)
THEN edge created connecting p to t
```

### TEST: link_wrong_target_type_error
```
GIVEN edge type owns(owner: Person, item: Task)
AND nodes: Person p1, Person p2
WHEN LINK owns(p1, p2)
THEN error: "owns position 2 expects Task, got Person"
```

### TEST: link_wrong_arity_error
```
GIVEN edge type owns(a, b) (arity 2)
WHEN LINK owns(x, y, z)
THEN error: "owns expects 2 targets, got 3"
```

## 8.4 UNLINK

### TEST: unlink_existing_edge
```
GIVEN graph with edge E
WHEN UNLINK E
THEN edge E deleted
```

### TEST: unlink_cascades_higher_order
```
GIVEN graph with edge E1, higher-order edge E2 about E1
WHEN UNLINK E1
THEN E1 deleted
AND E2 deleted
```

## 8.5 SET

### TEST: set_valid_attribute
```
GIVEN node t: Task with priority=1
WHEN SET t.priority = 5
THEN t.priority == 5
```

### TEST: set_wrong_type_error
```
GIVEN node t: Task with priority: Int
WHEN SET t.priority = "high"
THEN error: "attribute 'priority' expects Int, got String"
```

### TEST: set_unknown_attribute_error
```
GIVEN node t: Task (no attr "foo")
WHEN SET t.foo = 123
THEN error: "type 'Task' has no attribute 'foo'"
```