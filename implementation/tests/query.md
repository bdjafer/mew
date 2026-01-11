# 9. QUERY

## 9.1 Basic Queries

### TEST: query_single_type_scan
```
GIVEN graph with Task A, Task B, Person C
QUERY: "MATCH t: Task RETURN t"
EXPECT: rows [{t: A}, {t: B}]
```

### TEST: query_with_filter
```
GIVEN graph with Task A (p=1), Task B (p=5)
QUERY: "MATCH t: Task WHERE t.priority > 3 RETURN t"
EXPECT: rows [{t: B}]
```

### TEST: query_multiple_variables
```
GIVEN graph with knows(Alice, Bob)
QUERY: "MATCH a: Person, b: Person, knows(a, b) RETURN a.name, b.name"
EXPECT: rows [{a.name: "Alice", b.name: "Bob"}]
```

## 9.2 Sorting and Pagination

### TEST: query_order_by
```
GIVEN Tasks with priorities 5, 1, 3
QUERY: "MATCH t: Task RETURN t ORDER BY t.priority"
EXPECT: rows in order [p=1, p=3, p=5]
```

### TEST: query_order_by_desc
```
GIVEN Tasks with priorities 5, 1, 3
QUERY: "MATCH t: Task RETURN t ORDER BY t.priority DESC"
EXPECT: rows in order [p=5, p=3, p=1]
```

### TEST: query_limit
```
GIVEN 10 Task nodes
QUERY: "MATCH t: Task RETURN t LIMIT 3"
EXPECT: exactly 3 rows
```

### TEST: query_offset
```
GIVEN Tasks A, B, C, D, E (in insertion order)
QUERY: "MATCH t: Task RETURN t LIMIT 2 OFFSET 2"
EXPECT: rows [C, D]
```

## 9.3 Aggregation

### TEST: query_count
```
GIVEN 5 Task nodes
QUERY: "MATCH t: Task RETURN COUNT(t)"
EXPECT: 5
```

### TEST: query_sum
```
GIVEN Tasks with priorities 1, 2, 3
QUERY: "MATCH t: Task RETURN SUM(t.priority)"
EXPECT: 6
```

### TEST: query_group_by
```
GIVEN Tasks: 2 with status="open", 1 with status="closed"
QUERY: "MATCH t: Task RETURN t.status, COUNT(t) GROUP BY t.status"
EXPECT: rows [{"open", 2}, {"closed", 1}]
```

## 9.4 WALK

### TEST: walk_transitive
```
GIVEN: A->B->C->D via parent edges
QUERY: "WALK FROM A VIA parent"
EXPECT: [B, C, D]
```

## 9.5 INSPECT

### TEST: inspect_types
```
GIVEN loaded ontology with Task, Person
QUERY: "INSPECT TYPES"
EXPECT: returns Layer 0 _NodeType nodes for Task, Person
```
