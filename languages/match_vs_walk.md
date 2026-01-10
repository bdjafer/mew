## MATCH - Pattern Matching

MATCH is fundamentally about **finding all subgraphs that match a given pattern**. You describe a shape, and the engine finds all instances of that shape in the graph.

Key characteristics:
- **Declarative**: You describe WHAT you want, not HOW to find it
- **Set-based**: Returns all matches simultaneously (conceptually)
- **Fixed pattern**: The pattern shape is known at compile time
- **No starting point**: Searches the entire graph (though can be optimized)

What MATCH can do:
- Find all nodes of a type
- Find all edges of a type
- Find subgraph patterns (A connected to B connected to C)
- Filter with conditions
- Return projections and aggregations

What MATCH cannot easily do:
- Variable-length paths (without transitive `+` or `*`)
- Paths where you don't know the edge types ahead of time
- Collecting path information (intermediate nodes)
- Starting from a specific known node and exploring

## WALK - Path Traversal

WALK is fundamentally about **navigating from a starting point through the graph structure**. You start somewhere and follow edges.

Key characteristics:
- **Navigational/Imperative**: You describe HOW to traverse
- **Sequential**: Follows paths step by step
- **Dynamic path**: Can follow any edge types, variable depths
- **Starting point required**: Must start somewhere

What WALK can do:
- Start from specific node(s) and explore
- Follow edges dynamically (any edge type, any direction)
- Control depth precisely
- Stop at conditions (UNTIL)
- Return paths (the sequence of nodes/edges)
- Traverse unknown structure

What WALK cannot easily do:
- Find patterns without a starting point (need to scan first)
- Complex multi-way joins efficiently
- Aggregations across non-path patterns


- 
At their core, MATCH is about discovering all instances of a known pattern across the graph, while WALK is about exploring outward from a specific starting point. MATCH returns sets of bindings that satisfy the pattern; WALK returns the reachable subgraph or paths traversed. While MATCH can theoretically replicate WALK's behavior with fixed-depth patterns, it becomes verbose and loses the dynamic traversal capability that makes WALK efficient for exploring unknown graph structures. The transitive closure approach works for single edge types at fixed depths, but it falls short when you need to follow multiple relationship types dynamically, return the actual traversal path, control whether you're doing breadth-first or depth-first search, or apply conditions at each hop.

WALK has the opposite limitation—it requires you to already know your starting point. To find all causal pairs where one event precedes another, you'd need to first scan every event in the database, then walk from each one. This makes it impractical for pattern discovery, complex multi-node joins with constraints on distant nodes, or aggregations across disconnected subgraphs.

MATCH excels when you're searching without a known entry point, need to count or sum across results, or have a fixed pattern structure with multiple node types and conditions. WALK shines when you're exploring from a specific node, need the actual sequence of nodes and edges traversed, want to dynamically follow different relationship types, or need fine-grained control over how the traversal happens.

For performance, MATCH leverages indexes heavily through the query planner, which can optimize join order and push filters down, though transitive patterns become expensive. WALK starts from a known point so it skips planning overhead, offers predictable costs when depth-bounded, and can short-circuit with UNTIL conditions, but has fewer opportunities for global optimization since traversal is inherently procedural.

MATCH excels at multi-entity correlations like finding task-project pairs with matching creation dates, aggregations across statuses, or detecting cycles in graphs. WALK is essential for reachability queries within hop limits and retrieving actual paths between nodes, where the starting point and traversal direction matter more than pattern matching across multiple entity types. ations | Yes | Limited |
| Complex joins | Yes | No |
| Optimization | Heavy (query planning) | Light (procedural) |
| Best for | Finding patterns | Exploring from a point |

MATCH and WALK serve different purposes in graph querying. MATCH excels at identifying specific patterns across the entire graph through declarative specifications, while WALK is better suited for navigating outward from a known starting point. Neither can fully replace the other—a comprehensive graph query language requires both approaches to handle the full range of traversal and pattern-matching scenarios.
# MATCH vs WALK: Fundamental Differences

## The Core Distinction

| Aspect | MATCH | WALK |
|--------|-------|------|
| **Question asked** | "What subgraphs look like this pattern?" | "Starting here, what can I reach?" |
| **Paradigm** | Declarative pattern matching | Navigational traversal |
| **Starting point** | None required (searches entire graph) | Required (must start somewhere) |
| **Returns** | Set of variable bindings | Reachable nodes / paths |

---

## Mental Models

### MATCH: Pattern Recognition

You describe a **shape**, the engine finds all instances:

```
MATCH a: Person, b: Person, c: Person,
      follows(a, b), follows(b, c), follows(c, a)
RETURN a, b, c
```

"Find all triangles in the social graph."

The engine figures out HOW to find them (query planning, index selection, join order).

### WALK: Navigation

You provide a **starting point and directions**, the engine explores:

```
WALK FROM #alice
FOLLOW follows [depth: 1..6]
RETURN NODES
```

"Starting from Alice, who can she reach through follows within 6 hops?"

You control HOW to traverse (edge types, depth, direction, strategy).

---

## What Each Can Do Exclusively

### Only MATCH (or impractical with WALK)

**1. Find patterns without a starting point:**
```
-- All tasks without assignees
MATCH t: Task
WHERE NOT EXISTS(assigned_to(t, _))
RETURN t
```
WALK requires a starting point. You'd have to iterate all nodes.

**2. Complex multi-way joins:**
```
-- Tasks in active projects assigned to active people in the same team as the project owner
MATCH t: Task, p: Project, person: Person, team: Team, owner: Person,
      belongs_to(t, p),
      assigned_to(t, person),
      member_of(person, team),
      owns(owner, p),
      member_of(owner, team)
WHERE p.status = "active" AND person.active = true
RETURN t, person, team
```
WALK can't express this multi-pattern join.

**3. Aggregations:**
```
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, COUNT(t), AVG(t.priority)
```
WALK doesn't aggregate across patterns.

**4. Finding specific topologies:**
```
-- Find all cycles of length 3
MATCH a: Node, b: Node, c: Node,
      edge(a, b), edge(b, c), edge(c, a)
RETURN a, b, c
```

---

### Only WALK (or impractical with MATCH)

**1. Variable-depth with multiple edge types:**
```
WALK FROM #task_123
FOLLOW depends_on | subtask_of | blocks [depth: 1..20]
RETURN NODES
```
MATCH would require enumerating all combinations of edge types and depths.

**2. Return actual paths:**
```
WALK FROM #alice
FOLLOW follows
UNTIL node.name = "Bob"
RETURN PATH

-- Returns: Alice → Carol → Dave → Bob
```
MATCH finds that Alice connects to Bob, but not the path taken.

**3. Follow any edge type:**
```
WALK FROM #entity_123
FOLLOW * [depth: 3]
RETURN NODES
```
"Explore everything within 3 hops." MATCH can't express "any edge."

**4. Traversal control:**
```
WALK FROM #root [strategy: bfs]
FOLLOW child_of INBOUND
UNTIL depth > 10 OR node.type = "leaf"
RETURN TERMINAL
```
BFS vs DFS, early termination, direction control — MATCH doesn't have these.

**5. Efficient expansion from known point:**
```
-- Given a specific user, find their network
WALK FROM #user_12345
FOLLOW follows ANY [depth: 2]
RETURN NODES
```
No need to scan all users — start directly from known point.

---

## Can You Do Everything With Just One?

### MATCH Only?

**Mostly, but some things are painful or impossible:**

| Operation | With MATCH | Verdict |
|-----------|-----------|---------|
| Fixed pattern | ✅ Native | Perfect |
| Variable depth (known bound) | ⚠️ Use `+`/`*` | Works but limited |
| Multiple edge types | ⚠️ Enumerate or union types | Verbose |
| Return paths | ❌ Not possible | Can't do |
| Follow any edge | ❌ Not possible | Can't do |
| Traversal strategy | ❌ Not possible | Can't do |

```
-- MATCH can't express this:
WALK FROM #x FOLLOW * RETURN PATH
```

### WALK Only?

**No. Fundamental limitations:**

| Operation | With WALK | Verdict |
|-----------|----------|---------|
| Find all patterns | ❌ Requires starting point | Must scan all nodes first |
| Complex joins | ❌ Can't express | No join semantics |
| Aggregations | ❌ No GROUP BY | Can't do |
| No starting point | ❌ Impossible | Fundamental limitation |

```
-- WALK can't express this:
MATCH a: Person, b: Person
WHERE a.company = b.company AND a.id != b.id
RETURN a, b
-- (find all colleague pairs)
```

---

## Performance Characteristics

### MATCH Compilation

```
MATCH t: Task, p: Project, belongs_to(t, p)
WHERE t.status = "done" AND p.name = "Alpha"
RETURN t
```

**Query planner decides:**
- Start with Task (filter on status) or Project (filter on name)?
- Which indexes to use?
- Join order?

**Optimization opportunities:**
- Index selection
- Filter pushdown
- Join reordering
- Index-only scans

**Cost:** Planning overhead, but optimal execution.

### WALK Execution

```
WALK FROM #project_alpha
FOLLOW contains INBOUND [depth: 1..10]
RETURN NODES
```

**No planning needed:**
- Starting point is given
- Traversal is procedural
- Just follow edges

**Optimization opportunities:**
- Edge index for fast neighbor lookup
- Early termination (UNTIL)
- Depth pruning

**Cost:** No planning overhead, predictable execution.

### Performance Comparison

| Scenario | MATCH | WALK |
|----------|-------|------|
| Find all X where condition | ✅ Uses indexes | ❌ Must scan |
| Expand from known node | ⚠️ Still plans full query | ✅ Direct traversal |
| Variable depth exploration | ⚠️ Transitive can be expensive | ✅ Native, bounded |
| Complex joins | ✅ Optimized | ❌ Can't express |
| Path finding | ❌ Can't do | ✅ Native |

---

## When to Use Each

### Use MATCH When:

1. **No specific starting point** — "Find all tasks that..."
2. **Complex conditions across multiple nodes** — Joins with filters
3. **Aggregations needed** — COUNT, SUM, GROUP BY
4. **Pattern is fixed and known** — Specific shape to find
5. **Need query optimization** — Let planner choose best path

```
-- Classic MATCH use cases:
MATCH t: Task WHERE t.status = "blocked" RETURN t
MATCH t: Task, p: Person, assigned_to(t, p) WHERE p.active RETURN t, p
MATCH t: Task RETURN t.status, COUNT(t)
```

### Use WALK When:

1. **Starting from specific known node(s)** — "Given this user..."
2. **Variable or unknown depth** — "Find everything reachable within N hops"
3. **Need the actual path** — "How does A connect to B?"
4. **Dynamic edge following** — "Follow any relationship"
5. **Traversal control** — BFS, DFS, UNTIL conditions
6. **Graph exploration** — "What's around this node?"

```
-- Classic WALK use cases:
WALK FROM #user_123 FOLLOW follows [depth: 3] RETURN NODES
WALK FROM #task FOLLOW depends_on UNTIL node.status = "done" RETURN PATH
WALK FROM #root FOLLOW * [depth: 2] RETURN NODES
```

---

## Combined Usage

Often you'll combine them:

```
-- Find starting points with MATCH, explore with WALK
blocked_tasks = MATCH t: Task WHERE t.status = "blocked" RETURN t

-- Then for each blocked task, find its dependency chain
WALK FROM { MATCH t: Task WHERE t.status = "blocked" RETURN t }
FOLLOW depends_on
UNTIL node.status = "done"
RETURN PATH
```

Or use MATCH for the overall pattern, WALK for path details:

```
-- Find connected pairs
MATCH a: Person, b: Person, follows+(a, b)
WHERE a.department = "Engineering" AND b.department = "Sales"
RETURN a, b

-- Then get actual path for specific pair
WALK FROM #person_a
FOLLOW follows
UNTIL node.id = "person_b"
RETURN PATH
```

---

## Summary

| | MATCH | WALK |
|--|-------|------|
| **Core question** | What matches this pattern? | What's reachable from here? |
| **Starting point** | Not needed | Required |
| **Depth** | Fixed (or transitive with `+`/`*`) | Variable, controlled |
| **Edge types** | Must specify | Can be dynamic (`*`) |
| **Returns** | Bindings | Nodes, edges, or paths |
| **Joins** | Yes, complex | No |
| **Aggregations** | Yes | No |
| **Path info** | No | Yes |
| **Optimization** | Query planning | Procedural |
| **Best for** | Finding patterns | Exploring structure |

**They are complementary, not interchangeable.** A complete graph language needs both.