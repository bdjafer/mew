# 1. GRAPH

## 1.1 Node Operations

### TEST: create_node_returns_unique_id
```
GIVEN empty graph
WHEN create node with type=1, attrs={name: "Alice"}
THEN returns NodeId
AND get_node(id) returns node with type=1, name="Alice"
```

### TEST: create_multiple_nodes_unique_ids
```
GIVEN empty graph
WHEN create node A
AND create node B
THEN A.id ≠ B.id
```

### TEST: get_nonexistent_node_returns_none
```
GIVEN empty graph
WHEN get_node(NodeId(999))
THEN returns None
```

### TEST: delete_node_removes_it
```
GIVEN graph with node A
WHEN delete_node(A.id)
THEN get_node(A.id) returns None
```

### TEST: delete_node_cascades_to_edges
```
GIVEN graph with nodes A, B
AND edge E from A to B
WHEN delete_node(A.id)
THEN get_node(A.id) returns None
AND get_edge(E.id) returns None
```

### TEST: set_attribute_updates_value
```
GIVEN graph with node A where name="Alice"
WHEN set_attr(A.id, "name", "Bob")
THEN get_node(A.id).attrs["name"] == "Bob"
```

## 1.2 Edge Operations

### TEST: create_edge_returns_unique_id
```
GIVEN graph with nodes A, B
WHEN create edge with type=1, targets=[A.id, B.id]
THEN returns EdgeId
AND get_edge(id) returns edge with targets=[A.id, B.id]
```

### TEST: create_higher_order_edge
```
GIVEN graph with nodes A, B
AND edge E1 connecting A to B
WHEN create edge E2 with targets=[E1.id]
THEN get_edge(E2.id).targets == [E1.id]
AND E2 is marked as higher_order
```

### TEST: delete_edge_removes_it
```
GIVEN graph with edge E
WHEN delete_edge(E.id)
THEN get_edge(E.id) returns None
```

### TEST: delete_edge_cascades_to_higher_order
```
GIVEN graph with edge E1
AND higher-order edge E2 about E1
WHEN delete_edge(E1.id)
THEN get_edge(E1.id) returns None
AND get_edge(E2.id) returns None
```

## 1.3 Type Index

### TEST: find_nodes_by_type
```
GIVEN graph with:
  - node A type=1
  - node B type=1
  - node C type=2
WHEN find_by_type(1)
THEN returns [A, B] (order unspecified)
```

### TEST: find_nodes_by_type_empty
```
GIVEN graph with node A type=1
WHEN find_by_type(2)
THEN returns []
```

## 1.4 Attribute Index

### TEST: find_nodes_by_attribute_value
```
GIVEN graph with:
  - node A where status="active"
  - node B where status="inactive"
  - node C where status="active"
WHEN find_by_attr(type=*, attr="status", value="active")
THEN returns [A, C]
```

### TEST: find_nodes_by_attribute_range
```
GIVEN graph with:
  - node A where priority=1
  - node B where priority=5
  - node C where priority=10
WHEN find_by_attr_range(attr="priority", min=3, max=7)
THEN returns [B]
```

## 1.5 Adjacency Index

### TEST: find_edges_from_node
```
GIVEN graph with:
  - nodes A, B, C
  - edge E1 from A to B
  - edge E2 from A to C
  - edge E3 from B to C
WHEN find_edges_from(A.id)
THEN returns [E1, E2]
```

### TEST: find_edges_to_node
```
GIVEN graph with:
  - nodes A, B, C
  - edge E1 from A to C
  - edge E2 from B to C
WHEN find_edges_to(C.id)
THEN returns [E1, E2]
```

## 1.6 Higher-Order Index

### TEST: find_edges_about_edge
```
GIVEN graph with:
  - edge E1 (base)
  - edge E2 about E1
  - edge E3 about E1
  - edge E4 about E2
WHEN find_edges_about(E1.id)
THEN returns [E2, E3]
```