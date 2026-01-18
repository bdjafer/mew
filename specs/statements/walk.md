---
spec: walk
version: "1.0"
status: draft
category: statement
requires: [transitive_patterns]
priority: specialized
---

# Spec: Walk Statement

## Overview

The WALK statement traverses graph paths with fine-grained control: direction, depth limits, filtering, and path collection. While MATCH finds patterns, WALK explores connectivity.

**Why needed:** MATCH with transitive patterns (`+`, `*`) answers "is there a path?" WALK answers "what are the paths?", "how long?", "through which nodes?".

---

## Syntax

### Grammar
```ebnf
WalkStatement = 
  "WALK" "FROM" Expr
  "FOLLOW" EdgeSpec ("," EdgeSpec)*
  WalkModifiers?
  ReturnClause

EdgeSpec = Identifier Direction?

Direction = "->" | "<-" | "<->"

WalkModifiers = (DepthClause | UntilClause | CollectClause)*

DepthClause = "DEPTH" IntLiteral (".." IntLiteral)?

UntilClause = "UNTIL" Expr

CollectClause = "COLLECT" CollectTarget ("," CollectTarget)*

CollectTarget = "nodes" | "edges" | "path"

ReturnClause = "RETURN" ReturnType ("AS" Identifier)?
            | "RETURN" Projection ("," Projection)*

ReturnType = "PATH" | "NODES" | "EDGES" | "TERMINAL"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `WALK` | Statement |
| `FROM` | Walk clause |
| `FOLLOW` | Walk clause |
| `DEPTH` | Walk modifier |
| `UNTIL` | Walk modifier |
| `COLLECT` | Walk modifier |
| `NODES` | Return type |
| `EDGES` | Return type |
| `PATH` | Return type |
| `TERMINAL` | Return type |

### Examples
```
-- Basic: find all reachable nodes
WALK FROM start_node
FOLLOW causes
RETURN endpoint

-- With depth limit
WALK FROM task
FOLLOW depends_on
DEPTH 1..5
RETURN endpoint

-- Bidirectional traversal
WALK FROM person
FOLLOW knows <->
DEPTH 1..3
RETURN endpoint

-- Collect the path
WALK FROM source
FOLLOW causes
COLLECT path
RETURN endpoint, path

-- Stop at condition
WALK FROM root
FOLLOW parent_of <-
UNTIL endpoint.status = "archived"
RETURN endpoint
```

---

## Semantics

### Starting Point

`FROM expr` specifies the starting node(s):
```
WALK FROM some_node ...         -- single start
WALK FROM $node_id ...          -- from parameter
```

### Edge Direction

| Syntax | Meaning |
|--------|---------|
| `FOLLOW edge` | Forward (default) |
| `FOLLOW edge ->` | Forward (explicit) |
| `FOLLOW edge <-` | Backward (reverse) |
| `FOLLOW edge <->` | Both directions |
```
edge parent_of(parent: Person, child: Person)

WALK FROM person FOLLOW parent_of      -- find descendants
WALK FROM person FOLLOW parent_of <-   -- find ancestors
WALK FROM person FOLLOW parent_of <->  -- find all relatives
```

### Multiple Edge Types
```
WALK FROM node
FOLLOW edge1, edge2, edge3
-- Traverses any of these edge types at each step
```

### Depth Control

`DEPTH min..max` limits traversal depth:
```
DEPTH 3        -- exactly 3 hops
DEPTH 1..5     -- 1 to 5 hops
DEPTH 2..      -- 2 or more (up to engine limit)
```

Default depth limit: engine-configured (typically 100).

### Until Condition

`UNTIL expr` stops traversal when condition becomes true:
```
WALK FROM start
FOLLOW links
UNTIL endpoint.type = "terminal"
RETURN endpoint
-- Stops at terminal nodes, doesn't traverse beyond
```

The endpoint meeting the UNTIL condition IS included in results.

Use `RETURN TERMINAL` to return only the nodes where the UNTIL condition first becomes true (see Return Types).

### Return Types

WALK supports specialized return types for common traversal patterns:

| Return Type | Description |
|-------------|-------------|
| `RETURN NODES` | Returns all nodes visited during traversal |
| `RETURN EDGES` | Returns all edges traversed |
| `RETURN PATH` | Returns the complete path (nodes and edges) |
| `RETURN TERMINAL` | Returns only nodes where UNTIL condition matched |

```
-- Return all nodes in the chain
WALK FROM start FOLLOW links RETURN NODES AS chain

-- Return only the terminal node (where UNTIL matched)
WALK FROM employee FOLLOW reports_to
UNTIL node:Executive
RETURN TERMINAL AS executive

-- Return the path taken
WALK FROM source FOLLOW depends_on RETURN PATH AS dependency_path

-- Return edges traversed
WALK FROM task FOLLOW blocks RETURN EDGES AS blocking_edges
```

`RETURN TERMINAL` is specifically designed for UNTIL queries where you want only the stopping points, not all intermediate nodes. Without UNTIL, it returns all endpoints.

### Collection

`COLLECT` specifies what to gather during traversal:

| Target | Result |
|--------|--------|
| `nodes` | All visited nodes |
| `edges` | All traversed edges |
| `path` | Ordered list of (node, edge) pairs |
```
WALK FROM start
FOLLOW causes
COLLECT nodes, edges
RETURN endpoint, nodes, edges

-- nodes: [n1, n2, n3, endpoint]
-- edges: [e1, e2, e3]
```

### Return Variables

Special variables available in RETURN:

| Variable | Type | Description |
|----------|------|-------------|
| `endpoint` | Node | Final node of each path |
| `depth` | Int | Number of hops to reach endpoint |
| `nodes` | List<Node> | Collected nodes (if COLLECT nodes) |
| `edges` | List<Edge> | Collected edges (if COLLECT edges) |
| `path` | List<(Node,Edge)> | Full path (if COLLECT path) |

### Cycle Handling

Cycles are detected and terminated:
```
-- Graph: A → B → C → A
WALK FROM A FOLLOW edge
-- Returns: B (depth 1), C (depth 2), A (depth 3)
-- Does NOT continue: B again would be depth 4, but already visited
```

Each node appears at most once per path.

---

## Layer 0

None. WALK is a runtime statement, not an ontology construct.

---

## Examples

### Dependency Analysis
```
ontology Tasks {
  node Task { 
    title: String [required],
    status: String = "pending"
  }
  edge depends_on(downstream: Task, upstream: Task)
}

-- Find all upstream dependencies
WALK FROM my_task
FOLLOW depends_on
RETURN endpoint, depth

-- Find blocking tasks (incomplete upstream)
WALK FROM my_task
FOLLOW depends_on
UNTIL endpoint.status = "done"
RETURN endpoint
WHERE endpoint.status != "done"
```

### Social Network
```
ontology Social {
  node Person { name: String [required] }
  edge knows(a: Person, b: Person) [symmetric]
}

-- Friends of friends (2 degrees)
WALK FROM me
FOLLOW knows
DEPTH 2..2
RETURN endpoint

-- Shortest path to target (collect path)
WALK FROM me
FOLLOW knows
UNTIL endpoint.id = target_id
COLLECT path
RETURN path
```

### Organizational Hierarchy
```
ontology OrgChart {
  node Person { 
    name: String [required],
    level: String
  }
  edge reports_to(employee: Person, manager: Person)
}

-- Find all managers up the chain
WALK FROM employee
FOLLOW reports_to
COLLECT nodes
RETURN endpoint AS ceo, nodes AS chain

-- Find all reports (recursive)
WALK FROM manager
FOLLOW reports_to <-
RETURN endpoint, depth
```

### Category Traversal
```
ontology Taxonomy {
  node Category { name: String [required] }
  edge subcategory_of(child: Category, parent: Category)
}

-- All ancestors of a category
WALK FROM leaf_category
FOLLOW subcategory_of
COLLECT path
RETURN endpoint AS root, path

-- All descendants
WALK FROM root_category
FOLLOW subcategory_of <-
DEPTH 1..10
RETURN endpoint, depth
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Invalid depth range | `"Invalid depth range: min must be <= max"` |
| Depth limit exceeded | `"Walk exceeded maximum depth (N)"` |
| Unknown edge type | `"Unknown edge type 'X'"` |
| Non-node start | `"WALK FROM requires a node"` |

---

*End of Spec: Walk Statement*