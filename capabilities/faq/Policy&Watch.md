---

# MEW Design Insights: Policy & Watch

## The Question

Are "authorization" and "subscription" fundamental concepts, or arbitrary features bolted on?

## The Answer

They're **fundamental** — not because we invented them, but because they emerge necessarily from having **actors + boundaries**.

---

## The Core Insight

When you have multiple perspectives (actors, interiority), every boundary creates two questions:

| Direction | Question | Concept |
|-----------|----------|---------|
| **Outward** | What can actor push INTO the graph? | **Policy** |
| **Inward** | What can graph push TO the actor? | **Watch** |

Policy = how boundaries constrain action
Watch = how boundaries enable perception

No boundaries → no actors → no need for either.
Boundaries → these concepts emerge automatically.

---

## The Reduction (What They Actually Are)

Both reduce to simpler primitives:

```
Policy = predicate over (operation, state) that references current_actor()
Watch  = query that runs continuously and pushes deltas
```

But this reduction isn't useful to users. The concepts are the right abstraction level.

---

## The Naming

| Old | New | Why |
|-----|-----|-----|
| Authorization | **Policy** | Less enterprise baggage, more neutral |
| Subscription | **Watch** | Active, clear, mirrors MATCH |

The MATCH/WATCH symmetry:

```
MATCH  →  point-in-time  (what is)
WATCH  →  continuous     (what changes)
```

Same pattern language. M and W are mirror letters. One query, two modes.

---

## Policy Has Two Aspects

| Aspect | What It Does | On Failure |
|--------|--------------|------------|
| **Operation** | Gates mutations (can actor do X?) | Reject |
| **Visibility** | Filters reads (can actor see X?) | Exclude |

Both use same syntax, same pattern language:

```
policy owner_modifies:
  CAN SET(t: Task, _) IF current_actor() = t.owner

policy task_visibility:
  CAN MATCH(t: Task) IF EXISTS(
    p: Project,
    belongs_to(t, p),
    member_of(current_actor(), p)
  )
```

---

## Everything Stays Graph-Native

No special constructs. The actor is just a node. Policies are patterns over the graph.

```
current_actor()  -- just a binding in evaluation context
```

Policy conditions query relationships involving that node. Watch patterns are normal patterns. No magic.

---

## The Minimal DSL Concepts

| Concept | Purpose |
|---------|---------|
| **node/edge** | Structure |
| **constraint** | State invariant (must hold at commit) |
| **policy** | What actors can do/see |
| **rule** | Reactions to state/events |
| **MATCH** | Point-in-time query |
| **WATCH** | Continuous query |

Six concepts. All graph-native. Complete.

---