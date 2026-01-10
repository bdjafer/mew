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

New query syntax: `IN SCOPE my_scope: MATCH ...`

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
