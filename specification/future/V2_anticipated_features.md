## 22. V2 Anticipated Features

These features are intentionally deferred from v1. The current design accommodates them as non-breaking additions.

### 22.1 Explicit Scopes

```
node _Scope [sealed] {
  name: String [required],
  isolated: Bool = false
}

edge _scope_ontology(_Scope, _Ontology) {}
edge _scope_parent(child: _Scope, parent: _Scope) {}
edge _in_scope(entity: any, scope: _Scope) {}
```

New observation syntax: `IN SCOPE my_scope: MATCH ...`

### 22.2 Scope-Qualified Names

```
MATCH t: TodoApp::Task, n: NotesApp::Note
```

Unqualified names continue to work (resolved by context).

### 22.3 Dynamic Type Creation

```
rule suggest_abstraction:
  ...
  =>
  PROPOSE TYPE AbstractedConcept { ... }
```

Proposals go to approval queue, not created directly.

### 22.4 Soft Constraints with Scoring

```
constraint soft_preference [soft, weight: 0.8]:
  t: Task, p: Project, belongs_to(t, p)
  => p.priority >= t.priority
```

Soft constraints contribute to fitness score rather than reject.

### 22.5 Index Hints

```
node Task {
  title: String,
  done: Bool
} [
  index by done
]
```

### 22.6 Virtual/Computed Edges

```
edge ancestor(Person, Person) [virtual] {
  MATCH path = (a)-[:parent*]->(b)
  RETURN a, b
}
```

---

## Automatic Edge Reification

Every edge `e` automatically has a shadow node `_reified(e)` accessible via:
```
-- Get reified node for edge
MATCH causes(a, b) AS c
LET c_node = reify(c)  -- returns the shadow node

-- Query edges via their reified nodes
MATCH r: _ReifiedEdge
WHERE r.edge_type = "causes" AND r.created_at > $threshold
RETURN r.edge  -- returns the actual edge
```

**Shadow node attributes:**
- `edge`: Reference to actual edge
- `edge_type`: String name
- `created_at`: Timestamp
- `targets`: List of target IDs

**Benefits:**
- Edges become first-class queryable entities
- Higher-order patterns simplified
- Provenance can attach to edge-nodes

**Implementation:** Lazy materialization — shadow nodes created on first access, cached.