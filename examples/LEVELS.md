# MEW Ontology Levels

Complexity tiers for ontologies. Each level adds features. Higher levels include all lower-level features.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Level 5: Meta-Systems                                                      │
│  ───────────────────────────────────────────────────────────────────────── │
│  META mutations, self-modeling, cognitive architectures, deterministic time │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 4: Reflection                                                        │
│  ───────────────────────────────────────────────────────────────────────── │
│  META queries, schema introspection, edge<any>, dynamic typing              │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 3: Dynamics                                                          │
│  ───────────────────────────────────────────────────────────────────────── │
│  Constraints, rules, higher-order edges, subscriptions, advanced auth       │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 2: Structure                                                         │
│  ───────────────────────────────────────────────────────────────────────── │
│  Inheritance, validation, WALK, simple auth, wall_time()                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 1: Fundamentals                                                      │
│  ───────────────────────────────────────────────────────────────────────── │
│  Nodes, edges, CRUD, basic queries                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Level 1: Fundamentals

Basic graph operations. Entry point for learning MEW.

| Category | Features |
|----------|----------|
| **Schema** | `node`, `edge`, scalar types (`String`, `Int`, `Float`, `Bool`, `Timestamp`, `Duration`), `[required]`, default values (`= value`), optional types (`T?`), doc comments (`---`) |
| **Mutations** | SPAWN, KILL, LINK, UNLINK, SET, RETURNING clause |
| **Queries** | MATCH, WHERE, RETURN, ORDER BY, LIMIT, OFFSET, DISTINCT |
| **Aggregations** | COUNT, SUM, AVG, MIN, MAX, COLLECT |
| **Expressions** | Arithmetic (`+`, `-`, `*`, `/`), comparison (`=`, `!=`, `<`, `>`, `<=`, `>=`), logical (`AND`, `OR`, `NOT`), string (`++` concat, `length()`, `starts_with()`, `ends_with()`) |
| **Transactions** | BEGIN, COMMIT, ROLLBACK, auto-commit mode |
| **Debug** | EXPLAIN, PROFILE |

**Example Ontologies:**

- **Bookmarks** — URL storage with folders and tags. Demonstrates basic node/edge relationships, simple categorization, and timestamp defaults for tracking when bookmarks were created.

- **Contacts** — People, organizations, phone numbers, and email addresses. Shows multi-type relationships (person-to-person, person-to-org), optional fields, and basic contact information modeling.

- **Library** — Books, authors, members, and loans. Illustrates many-to-many relationships (books ↔ authors), temporal data (loan dates, due dates), and simple status tracking.

---

## Level 2: Structure

Type system sophistication, validation, and traversal.

| Category | Features |
|----------|----------|
| **Schema** | `type` aliases, inheritance (`:`), multiple inheritance, `[unique]`, `[indexed]`, `[format: email\|url\|uuid\|...]`, `[match: "regex"]`, `[in: [...]]`, `[>= N]`/`[<= M]`, `[length: N..M]`, `[no_self]`, edge attributes |
| **Queries** | WALK traversal (FOLLOW, UNTIL, RETURN NODES/EDGES/PATH/TERMINAL), OPTIONAL MATCH, polymorphic queries, anonymous targets (`_`) |
| **Mutations** | Bulk operations (KILL/SET with MATCH subquery), LINK IF NOT EXISTS, inline SPAWN in LINK |
| **Time** | `now()`, `wall_time()` for real-world timestamps |
| **Authorization** | Simple ownership-based access control, `current_actor()` |
| **Parameters** | `$param` syntax for parameterized queries |

**Example Ontologies:**

- **Ecommerce** — Products with inheritance hierarchy (Product → PhysicalProduct/DigitalProduct), SKU validation via regex, price constraints, categories with parent-child relationships, and inventory tracking with indexed lookups.

- **HumanResources** — Deep inheritance chain (Person → Employee → Manager → Executive), email format validation, unique employee IDs, department assignments with edge attributes (role, start_date), and organizational hierarchy traversal.

- **Tasks** — Task/SubTask modeling with priority validation (1-5 range), status enums, blocking relationships with `[no_self]`, due date tracking, and WALK-based dependency chain analysis.

---

## Level 3: Dynamics

Active schemas with reactive behavior, higher-order modeling, and real-time features.

| Category | Features |
|----------|----------|
| **Schema** | `constraint` (hard/soft, custom messages), `rule` (priority, auto/manual), `[acyclic]`, `[symmetric]`, cardinality `[a -> N..M]`, `[on_kill_source: cascade\|unlink\|prevent]`, `[on_kill_target: ...]` |
| **Queries** | Transitive patterns (`edge+`, `edge*`), depth limits (`[depth: N]`), EXISTS / NOT EXISTS, aggregates in WHERE |
| **Higher-Order** | `edge<T>` type references, edges targeting edges, edge binding with `AS`, querying edges about edges |
| **Subscriptions** | SUBSCRIBE with watch/consume modes, competing consumers (`group`), windowing, buffering, ACK/NACK delivery protocol |
| **Time** | `logical_time()`, TICK advancement, tick-based rule execution, tick triggers (manual, per-transaction, periodic) |
| **Authorization** | Role-based (RBAC), relationship-based, attribute-based (ABAC), `operation()`, `target()`, `target_type()`, `target_attr()` |
| **Transactions** | SAVEPOINT, ROLLBACK TO savepoint, isolation levels (READ COMMITTED, SERIALIZABLE) |

**Example Ontologies:**

- **EventChain** — Causal event modeling with temporal ordering constraints (cause must precede effect), transitive causation queries (`causes+`), confidence edges about causal links, and acyclic enforcement to prevent causal loops.

- **ProjectManagement** — Task dependencies with automatic blocking/unblocking rules, milestone tracking, auto-timestamp rules (set `completed_at` when status changes to "done"), cardinality constraints (each task belongs to exactly one project), and cascade deletion (kill project → kill tasks).

- **Workflow** — State machine modeling with transition rules, history tracking via higher-order edges (who approved which transition), symmetric collaboration edges, subscription-based notifications on state changes, and RBAC for transition authorization.

- **Argumentation** — Arguments attacking/supporting other arguments, confidence scoring on attack/support edges, symmetric "related_to" edges, constraint ensuring arguments can't attack themselves, and transitive rebuttal chains.

---

## Level 4: Reflection

Schema introspection, dynamic typing, and meta-level queries.

| Category | Features |
|----------|----------|
| **META Queries** | META MATCH on Layer 0 (`_NodeType`, `_EdgeType`, `_AttributeDef`, `_ConstraintDef`, `_RuleDef`), META WALK on schema graph |
| **Introspection Functions** | `type_of()`, `edge_type()`, `attributes()`, `attr()`, `has_attr()`, `arity()`, `targets()`, `target()`, `has_target()`, `is_higher_order()`, `edges_about()`, `type_node()` |
| **Generic Patterns** | `edge<any>` wildcard (match any edge type), `any` node type, dynamic attribute access |
| **Schema Navigation** | `constraints_on()`, `rules_affecting()`, `subtypes_of()`, `supertypes_of()`, `edges_involving()`, `source_types()`, `target_types()` |
| **Authorization** | META operation authorization (control who can read/query schema), `META MATCH` policies |

**Example Ontologies:**

- **FactBase** — Generic entity-relation storage with confidence scores, dynamic attribute access for heterogeneous facts, schema queries to discover available relation types, and introspection-based validation rules.

- **Scientific** — Hypotheses, experiments, and observations with symmetric correlation edges between concepts, meta-queries to find all constraints affecting a hypothesis, schema navigation to discover experimental methodologies, and confidence propagation across evidence chains.

---

## Level 5: Meta-Systems

Self-modifying systems, cognitive architectures, and advanced temporal control.

| Category | Features |
|----------|----------|
| **META Mutations** | META SPAWN (create types at runtime), META LINK (add attributes/edges to types), META SET (modify type definitions), META KILL (remove types), META UNLINK |
| **Self-Modeling** | Agent/Belief/Goal/Plan patterns, self-reference structures, belief revision, goal conflict detection |
| **Time** | Domain ticks (named tick domains), deterministic replay, configurable `now()` binding, time-travel queries |
| **Authorization** | Self-modifying authorization rules, META authorization for schema mutations, dynamic policy creation |
| **Advanced Patterns** | Learned type creation, dynamic constraint generation, runtime ontology evolution |

**Example Ontologies:**

- **BDI** — Belief-Desire-Intention agent architecture with explicit belief stores, desire prioritization, intention commitment tracking, plan libraries, and belief revision rules that update agent state based on new observations.

- **CognitiveAgent** — Full cognitive system with perception (sensory input nodes), memory (short-term/long-term with decay rules), attention (focus tracking), meta-cognition (self-model nodes), and learning (runtime type creation for new concepts).

- **ConceptNet** — Learned concepts with dynamically typed relations, runtime schema evolution as new concept types are discovered, confidence-weighted edges, symmetric similarity relations, and meta-rules that propose new type abstractions.

---

## Feature Matrix

| Feature | L1 | L2 | L3 | L4 | L5 |
|---------|:--:|:--:|:--:|:--:|:--:|
| **Schema** |||||
| `node`, `edge`, scalars | ✓ | ✓ | ✓ | ✓ | ✓ |
| `[required]`, defaults | ✓ | ✓ | ✓ | ✓ | ✓ |
| `type` aliases | | ✓ | ✓ | ✓ | ✓ |
| Inheritance (`:`) | | ✓ | ✓ | ✓ | ✓ |
| `[unique]`, `[indexed]` | | ✓ | ✓ | ✓ | ✓ |
| `[format]`, `[match]`, `[in]` | | ✓ | ✓ | ✓ | ✓ |
| `[no_self]` | | ✓ | ✓ | ✓ | ✓ |
| Edge attributes | | ✓ | ✓ | ✓ | ✓ |
| `constraint` | | | ✓ | ✓ | ✓ |
| `rule` | | | ✓ | ✓ | ✓ |
| `[acyclic]`, `[symmetric]` | | | ✓ | ✓ | ✓ |
| Cardinality `[a -> N..M]` | | | ✓ | ✓ | ✓ |
| `[on_kill_*]` | | | ✓ | ✓ | ✓ |
| **Queries** |||||
| MATCH, WHERE, RETURN | ✓ | ✓ | ✓ | ✓ | ✓ |
| Aggregations (COUNT, SUM, ...) | ✓ | ✓ | ✓ | ✓ | ✓ |
| WALK traversal | | ✓ | ✓ | ✓ | ✓ |
| OPTIONAL MATCH | | ✓ | ✓ | ✓ | ✓ |
| Transitive `+`/`*` | | | ✓ | ✓ | ✓ |
| EXISTS / NOT EXISTS | | | ✓ | ✓ | ✓ |
| META MATCH | | | | ✓ | ✓ |
| `edge<any>`, introspection | | | | ✓ | ✓ |
| **Mutations** |||||
| SPAWN, KILL, LINK, UNLINK, SET | ✓ | ✓ | ✓ | ✓ | ✓ |
| Bulk operations | | ✓ | ✓ | ✓ | ✓ |
| LINK IF NOT EXISTS | | ✓ | ✓ | ✓ | ✓ |
| META SPAWN/LINK/SET | | | | | ✓ |
| **Higher-Order** |||||
| `edge<T>` types | | | ✓ | ✓ | ✓ |
| Edges about edges | | | ✓ | ✓ | ✓ |
| **Time** |||||
| `now()` / `wall_time()` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `logical_time()`, TICK | | | ✓ | ✓ | ✓ |
| Domain ticks | | | | | ✓ |
| Deterministic replay | | | | | ✓ |
| **Subscriptions** |||||
| SUBSCRIBE watch mode | | | ✓ | ✓ | ✓ |
| SUBSCRIBE consume mode | | | ✓ | ✓ | ✓ |
| Windowing, grouping | | | ✓ | ✓ | ✓ |
| **Transactions** |||||
| BEGIN, COMMIT, ROLLBACK | ✓ | ✓ | ✓ | ✓ | ✓ |
| SAVEPOINT | | | ✓ | ✓ | ✓ |
| Isolation levels | | | ✓ | ✓ | ✓ |
| **Authorization** |||||
| Simple ownership | | ✓ | ✓ | ✓ | ✓ |
| RBAC, relationship-based | | | ✓ | ✓ | ✓ |
| ABAC | | | ✓ | ✓ | ✓ |
| META authorization | | | | ✓ | ✓ |
| Self-modifying auth | | | | | ✓ |
| **Debug** |||||
| EXPLAIN, PROFILE | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## Choosing a Level

| Need | Level |
|------|-------|
| Simple data storage, learning MEW | 1 |
| Validated, typed data with traversal | 2 |
| Business rules, reactive behavior, real-time messaging | 3 |
| Schema introspection, dynamic typing, generic tools | 4 |
| Self-modifying systems, cognitive architectures, AGI foundations | 5 |

**Start at Level 1.** Add features as needed. Each level adds complexity and runtime cost.
