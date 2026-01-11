# MEW Implementation Meta-Roadmap

**Purpose:** A compass for autonomous implementation. Not a plan — a methodology.

---

## 1. Terminal Condition

**You are done when:**

All 158 acceptance tests pass, AND you can execute this session without error:

```
LOAD ONTOLOGY {
  node Task { 
    title: String [required]
    priority: Int [>= 0, default: 0]
    status: String [default: "pending"]
    created_at: Timestamp
  }
  
  node Person { name: String [required, unique] }
  
  edge owns(owner: Person, task: Task) [on_kill: cascade]
  edge depends_on(a: Task, b: Task) [acyclic]
  
  rule auto_timestamp on Task [auto, priority: 100] {
    SET t.created_at = NOW()
  }
}

BEGIN

SPAWN alice: Person { name = "Alice" }
SPAWN bob: Person { name = "Bob" }

SPAWN t1: Task { title = "Design API" }
SPAWN t2: Task { title = "Implement API", priority = 5 }
SPAWN t3: Task { title = "Write tests" }

LINK owns(alice, t1)
LINK owns(alice, t2)
LINK owns(bob, t3)

LINK depends_on(t2, t1)
LINK depends_on(t3, t2)

COMMIT

-- This should return 2 tasks
MATCH t: Task, owns(alice, t) 
WHERE t.priority >= 0 
RETURN t.title, t.priority 
ORDER BY t.priority DESC

-- This should fail (cycle)
BEGIN
LINK depends_on(t1, t3)
COMMIT

-- This should cascade delete tasks
BEGIN
KILL alice
COMMIT

-- Verify cascade
MATCH t: Task RETURN COUNT(t)  -- should be 1 (bob's task)
```

**Until then, you are not done.**

---

## 2. Progress Measurement

### 2.1 Primary Signal: Tests Passing

```
Progress = (passing acceptance tests) / 158
```

This is the only metric that matters. Lines of code, components "finished," or features "implemented" are vanity metrics.

### 2.2 Secondary Signal: Integration Depth

How many components work together end-to-end?

| Level | Description |
|-------|-------------|
| 0 | Nothing runs |
| 1 | Graph stores/retrieves data |
| 2 | Parser produces AST |
| 3 | Mutations modify graph |
| 4 | Queries read graph |
| 5 | Ontology loads and types work |
| 6 | Constraints enforce invariants |
| 7 | Rules fire reactively |
| 8 | Transactions provide ACID |
| 9 | Journal provides durability |
| 10 | Session provides interface |

**Prioritize depth over breadth.** A thin vertical slice that works end-to-end beats ten components that don't connect.

---

## 3. What To Work On Next

### Decision Procedure

```
1. Is there a failing test for already-written code?
   → Fix it. Bugs compound.

2. Is there a component blocking multiple other components?
   → Unblock it. Maximize parallelism potential.

3. Is there a test that's "almost passing"?
   → Finish it. Momentum matters.

4. Otherwise:
   → Pick the lowest-dependency unimplemented test.
```

### Anti-Patterns

- **Do not** implement features not covered by acceptance tests
- **Do not** optimize before correctness
- **Do not** refactor working code without failing test motivating it
- **Do not** add abstractions "for future flexibility"
- **Do not** work on component N+2 when component N has failing tests

---

## 4. Stuck Detection

**You are stuck if:**

- Same test failing for 3+ attempts with different approaches
- Circular dependency discovered between components
- Acceptance test appears to require undocumented behavior
- Implementation contradicts spec
- Two specs contradict each other

**Stuck is not:**

- Test is hard (that's normal)
- Implementation is tedious (that's normal)
- You need to learn something (that's normal)

---

## 5. Unstuck Procedures

### 5.1 Test Failing Repeatedly

```
1. Isolate: Write minimal reproduction
2. Question: Is the test correct? Check against spec.
3. Question: Is the spec correct? Check against architecture.
4. Question: Is the architecture correct? Check against philosophy.
5. If all documents align and code doesn't work:
   → Implementation bug. Debug harder.
6. If documents conflict:
   → Resolve conflict at highest level, propagate down.
```

### 5.2 Circular Dependency

```
1. Draw the actual dependency (not what spec says)
2. Identify which direction is "essential"
3. Break the cycle by:
   - Introducing an interface/trait
   - Passing data instead of component reference
   - Merging the components
4. Update spec if architecture changed
```

### 5.3 Spec Unclear or Contradictory

```
1. Check component spec's NOTES section
2. Check architecture document
3. Check acceptance tests for implicit clarification
4. If still unclear:
   → Make a decision
   → Document the decision in spec
   → Proceed
```

### 5.4 Acceptance Test Seems Wrong

```
1. Re-read the test. Are you misunderstanding?
2. Check if related tests clarify intent
3. Check component spec acceptance criteria
4. If test is genuinely wrong:
   → Fix the test
   → Document why
   → Do not silently skip tests
```

---

## 6. Quality Gates

### Before Considering a Component "Working"

- [ ] All acceptance tests for this component pass
- [ ] No panics on malformed input (returns errors)
- [ ] No unsafe code without comment justifying it
- [ ] Integrates with at least one dependent component

### Before Considering the System "Working"

- [ ] All 158 acceptance tests pass
- [ ] Terminal condition session runs without error
- [ ] Kill -9 during transaction → recovers correctly
- [ ] Can load, query, mutate in REPL
- [ ] No memory leaks on extended operation

---

## 7. Invariants (Must Always Hold)

These are non-negotiable throughout implementation:

### 7.1 Architectural Invariants

- Graph is the single source of truth for data
- Layer 0 is always consistent with Registry
- Transactions are the only write path to committed state
- Rules fire before constraints check

### 7.2 Implementation Invariants

- Tests never silently skipped
- Public API never panics (returns Result)
- Every error has context (what, where, why)
- No premature optimization

### 7.3 Process Invariants

- Specs updated if and only if contract changes
- Tests updated if and only if spec changes
- Working code is never deleted without replacement

---

## 8. Adaptation Protocol

When reality contradicts documentation:

```
                ┌─────────────────────────────────────┐
                │ Reality contradicts documentation   │
                └─────────────────────────────────────┘
                                  │
                                  ▼
                ┌─────────────────────────────────────┐
                │ Is reality correct or document?     │
                └─────────────────────────────────────┘
                         │                    │
                    Reality               Document
                         │                    │
                         ▼                    ▼
              ┌─────────────────┐   ┌─────────────────┐
              │ Fix document    │   │ Fix code        │
              │ Propagate to    │   │                 │
              │ dependent docs  │   │                 │
              └─────────────────┘   └─────────────────┘
```

**Document hierarchy (higher overrides lower):**

1. Philosophy (From Without, From Within)
2. Architecture (mew_architecture_v1.md)
3. Component Specs (mew_component_specs.md)
4. Acceptance Tests (mew_acceptance_tests.md)
5. Implementation

When conflict exists, defer to higher-level document. If higher-level is wrong, fix it explicitly — never silently override.

---

## 9. Recovery From Major Errors

### 9.1 Fundamental Design Flaw Discovered

```
1. Stop implementing
2. Identify the flaw precisely
3. Trace to which document level it originates
4. Fix at that level
5. Propagate changes downward
6. Identify which tests need revision
7. Identify which code needs revision
8. Resume implementing
```

### 9.2 Scope Creep Detected

```
Signs:
- Implementing feature not in any test
- Adding "nice to have" functionality
- Optimizing without benchmark showing need

Response:
- Stop
- Delete the uncommitted work
- Return to section 3 (What To Work On Next)
```

### 9.3 Lost Context (After Break/Interruption)

```
1. Run all tests. Note what passes/fails.
2. Read terminal condition. Assess distance.
3. Read last modified files.
4. Apply section 3 decision procedure.
5. Continue.
```

---

## 10. Communication Protocol

When in doubt, prefer:

- **Explicit over implicit** — State assumptions
- **Simple over clever** — Clarity over optimization  
- **Working over complete** — Iterate toward completeness
- **Tested over untested** — If it's not tested, it's broken

---

## 11. Meta-Compass Calibration

This document itself may need revision if:

- Terminal condition is achieved but system doesn't work
- Progress metric shows 100% but integration fails
- Stuck procedures don't unstick
- Quality gates pass but bugs exist in production use

If meta-roadmap fails, debug it like code:

```
1. What did it predict?
2. What actually happened?
3. Which section was wrong?
4. Fix that section.
5. Continue.
```

---

## Summary

```
┌────────────────────────────────────────────────────────────────┐
│                         META-COMPASS                           │
├────────────────────────────────────────────────────────────────┤
│  DIRECTION:  All 158 tests pass + terminal session works       │
│                                                                │
│  PROGRESS:   Count passing tests. Nothing else matters.        │
│                                                                │
│  NEXT STEP:  Fix failing → Unblock others → Near-done → Low-dep│
│                                                                │
│  STUCK:      Isolate → Question tests → Question specs →       │
│              Question architecture → Decide & document         │
│                                                                │
│  CONFLICT:   Philosophy > Architecture > Specs > Tests > Code  │
│                                                                │
│  ALWAYS:     Tests never skipped. Errors have context.         │
│              No premature optimization. Depth over breadth.    │
└────────────────────────────────────────────────────────────────┘
```

**Now implement.**