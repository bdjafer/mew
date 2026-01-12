# MEW Implementation Meta-Roadmap

**Purpose:** A compass for autonomous implementation. Not a task list — a behavior repertoire with selection heuristics.

---

## 1. Terminal Condition

**You are done when:**

- All specifications are implemented
- All tests pass
- The system is production-ready (robust, recoverable, operable)

Production-ready means users can:
- Define and load ontologies
- Execute queries and mutations
- Rely on constraint enforcement and rule firing
- Trust ACID guarantees and crash recovery
- Understand errors and act on them
- Operate the system without reading source code

---

## 2. Canonical Behaviors

These are the activities that advance the project. At any moment, you should be doing exactly one of these.

### B1: Fix Bug

**What:** Make failing test pass, or make behavior match specification.

**Outputs:** Code change, test now passes.

**Quality signal:** The fix is minimal and doesn't break other tests.

---

### B2: Implement Feature

**What:** Write code for specified but unimplemented functionality.

**Outputs:** New code, new tests, capability now works end-to-end.

**Quality signal:** Feature works as specified, tests cover happy path and error cases.

---

### B3: Write Tests

**What:** Add test coverage for existing or planned behavior.

**Outputs:** New test cases in appropriate test files.

**Quality signal:** Tests are deterministic, fast, and test one thing each.

---

### B4: Improve Testgen

**What:** Enhance `mew-testgen` to produce better/more diverse test cases.

**Outputs:** Changes to testgen module, expanded generated test coverage.

**Quality signal:** Generated tests find real bugs or cover previously untested paths.

---

### B5: Create Ontology

**What:** Write new `.mew` ontology file exploring different patterns.

**Outputs:** New ontology in `ontologies/`, possibly new testgen seeds.

**Quality signal:** Ontology exercises features not well-covered by existing ontologies.

---

### B6: Discover Edge Cases

**What:** Adversarial exploration — find inputs that break assumptions.

**Outputs:** Bug reports, new test cases, spec clarifications.

**Quality signal:** Discoveries lead to concrete fixes or spec refinements.

---

### B7: Refine Specification

**What:** Clarify ambiguity, fix contradiction, add missing detail.

**Outputs:** Updated spec documents.

**Quality signal:** Refinement unblocks implementation or resolves discovered inconsistency.

---

### B8: Design/Plan

**What:** Think through approach before implementing.

**Outputs:** Notes, diagrams, or updated architecture docs.

**Quality signal:** Design answers "how" questions that were blocking progress.

---

### B9: Refactor

**What:** Restructure code without changing behavior.

**Outputs:** Cleaner code, same test results.

**Quality signal:** Change makes future work easier; motivated by concrete pain.

---

### B10: Optimize

**What:** Improve performance of correct code.

**Outputs:** Faster code, benchmarks proving improvement.

**Quality signal:** Optimization addresses measured bottleneck, not hypothetical one.

---

### B11: Document

**What:** Make implicit knowledge explicit.

**Outputs:** Comments, READMEs, architecture notes.

**Quality signal:** Documentation would help someone unfamiliar with the code.

---

## 3. Behavior Selection

Given current context, which behavior to execute?

```
┌─────────────────────────────────────────────────────────────────────┐
│                     BEHAVIOR SELECTION TREE                          │
└─────────────────────────────────────────────────────────────────────┘

START
  │
  ▼
┌─────────────────────────────────┐
│ Are any tests failing?          │
└─────────────────────────────────┘
  │ YES                      │ NO
  ▼                          ▼
B1: Fix Bug            ┌─────────────────────────────────┐
                       │ Is there a spec'd feature       │
                       │ that isn't implemented?         │
                       └─────────────────────────────────┘
                         │ YES                      │ NO
                         ▼                          ▼
                  ┌──────────────────┐    ┌─────────────────────────────────┐
                  │ Is the feature   │    │ Is test coverage adequate?      │
                  │ design clear?    │    │ (Can testgen find more bugs?)   │
                  └──────────────────┘    └─────────────────────────────────┘
                    │ YES       │ NO        │ NO                      │ YES
                    ▼           ▼           ▼                         ▼
              B2: Implement  B8: Design  B4: Improve Testgen    ┌─────────────────┐
                  Feature       /Plan    or B3: Write Tests     │ Is robustness   │
                                                                │ proven?         │
                                                                └─────────────────┘
                                                                  │ NO        │ YES
                                                                  ▼           ▼
                                                          B6: Discover    ┌─────────────┐
                                                              Edge Cases  │Performance  │
                                                                          │acceptable?  │
                                                                          └─────────────┘
                                                                           │ NO     │ YES
                                                                           ▼        ▼
                                                                     B10: Optimize  DONE
                                                                           or
                                                                     B11: Document
```

### Selection Shortcuts

| Context | Behavior |
|---------|----------|
| Test is red | B1: Fix Bug |
| Spec exists, code doesn't | B2: Implement Feature |
| Code exists, tests don't | B3: Write Tests |
| Manual test writing is bottleneck | B4: Improve Testgen |
| Need diverse inputs for testing | B5: Create Ontology |
| Happy path works, edge cases unknown | B6: Discover Edge Cases |
| Spec is ambiguous or contradictory | B7: Refine Specification |
| Don't know how to implement | B8: Design/Plan |
| Code is painful to modify | B9: Refactor |
| Correct but too slow | B10: Optimize |
| Knowledge is tribal/tacit | B11: Document |

### Priority When Multiple Apply

If multiple behaviors seem valid:

1. **B1 (Fix Bug)** — Always first. Broken code contaminates everything.
2. **B2 (Implement)** — Features are the point.
3. **B7 (Refine Spec)** — Unblock implementation.
4. **B3/B4 (Tests/Testgen)** — Verify implementation.
5. **B6 (Edge Cases)** — Harden implementation.
6. **B5 (Ontology)** — Expand test surface.
7. **B9 (Refactor)** — Only when pain is concrete.
8. **B10 (Optimize)** — Only when slowness is measured.
9. **B8 (Design)** — Only when stuck on "how."
10. **B11 (Document)** — Fill gaps opportunistically.

---

## 4. Behavior Execution Patterns

### B1: Fix Bug — Execution Pattern

```
1. Reproduce: Ensure test fails consistently
2. Isolate: Find minimal reproduction
3. Diagnose: Identify root cause (not just symptom)
4. Fix: Change code minimally
5. Verify: Test passes, no regressions
6. Reflect: Should this bug class be caught by testgen?
```

### B2: Implement Feature — Execution Pattern

```
1. Understand: Read relevant spec thoroughly
2. Scope: Identify exactly what's in/out of scope
3. Test first: Write tests that define "done"
4. Implement: Make tests pass
5. Integrate: Ensure feature works with adjacent features
6. Cover: Add edge case tests
```

### B4: Improve Testgen — Execution Pattern

```
1. Identify gap: What's not being generated that should be?
2. Analyze: Why isn't testgen producing this?
3. Enhance: Add generation capability
4. Validate: New generation finds bugs or covers new ground
5. Tune: Adjust distribution if needed
```

### B6: Discover Edge Cases — Execution Pattern

```
1. Target: Pick a component or feature
2. Attack: Try to break it
   - Boundary values
   - Empty/null inputs
   - Malformed inputs
   - Concurrent operations
   - Resource exhaustion
   - Unexpected sequences
3. Record: Document each finding
4. Convert: Turn discoveries into tests or spec refinements
```

### B7: Refine Spec — Execution Pattern

```
1. Identify: What's ambiguous or wrong?
2. Research: Check architecture, philosophy for guidance
3. Decide: Make a concrete choice
4. Document: Update spec with decision and rationale
5. Propagate: Update dependent specs/tests/code
```

---

## 5. Stuck Detection and Recovery

### You Are Stuck When

- Same bug persists across 3+ fix attempts
- Implementation contradicts spec (and both seem right)
- Spec contradicts spec
- Don't know which behavior applies
- All behaviors seem blocked

### Unstuck Procedures

**Stuck on B1 (can't fix bug):**
```
1. Question the test: Is it correct per spec?
2. Question the spec: Is it correct per architecture?
3. Isolate further: Make reproduction even more minimal
4. Trace execution: Print/debug every step
5. Sleep on it: Fresh eyes often see the obvious
```

**Stuck on B2 (can't implement):**
```
1. Reduce scope: Implement smallest possible slice
2. Switch to B8: Design on paper before code
3. Find analogous code: How do similar features work?
4. Switch to B7: Maybe spec needs refinement first
```

**Stuck on B4 (testgen not helping):**
```
1. Manual first: Write the test by hand
2. Analyze: What makes this hard to generate?
3. Simplify: Generate simpler version first
4. Expand ontologies: Maybe need richer input patterns
```

**Stuck on "which behavior?":**
```
1. Run all tests: Results will clarify (if failing → B1)
2. Read specs: Find unimplemented feature (if found → B2)
3. Run testgen: See if it finds anything (if bugs → B1)
4. Do B6: Manual exploration always produces signal
```

---

## 6. Invariants

These hold regardless of which behavior you're executing.

### Architectural Invariants

- Graph is single source of truth for data
- Layer 0 and Registry are consistent
- Transactions are the only write path to committed state
- Rules fire before constraints check

### Process Invariants

- Tests are never silently skipped
- Public API never panics (returns Result)
- Errors include context (what, where, why)
- No optimization without measurement
- Specs updated only when contracts change
- Working code not deleted without replacement

---

## 7. Document Hierarchy

When documents conflict, higher wins:

```
Philosophy (CONTEXT.md, core principles)
    ↓
Specification (specs/specification/*.md)
    ↓
Architecture (specs/architecture.md)
    ↓
Component Specs (specs/components/*.md)
    ↓
Test Specs (specs/tests/*.md)
    ↓
Implementation (mew/**/src/)
```

Fix at highest level where error exists, propagate down.

---

## 8. Progress Checkpoints

Periodically verify overall progress:

### Weekly Checkpoint

- [ ] All tests still pass?
- [ ] Any new specs implemented?
- [ ] Any new bugs found and fixed?
- [ ] Testgen coverage improved?
- [ ] Any specs refined?

### Phase Checkpoint

- [ ] Current phase's features complete?
- [ ] Test coverage adequate for phase?
- [ ] No known critical bugs?
- [ ] Ready to move to next phase?

### Pre-Release Checkpoint

- [ ] All specs implemented?
- [ ] All tests pass?
- [ ] Edge cases explored?
- [ ] Performance acceptable?
- [ ] Documentation current?
- [ ] Error messages actionable?

---

## 9. Context Recovery

After interruption, to resume:

```
1. Run: cargo test --workspace
   → If failures, current behavior is B1

2. Check: git status / git log
   → What was in progress?

3. Apply: Section 3 (Behavior Selection)
   → Resume appropriate behavior
```

---

## Summary

```
┌────────────────────────────────────────────────────────────────────┐
│                         META-COMPASS                                │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TERMINAL       All specs implemented, all tests pass,             │
│                 system is production-ready                          │
│                                                                     │
│  BEHAVIORS      B1:Bug  B2:Implement  B3:Test  B4:Testgen          │
│                 B5:Ontology  B6:EdgeCase  B7:Spec  B8:Design        │
│                 B9:Refactor  B10:Optimize  B11:Document             │
│                                                                     │
│  SELECTION      Test failing? → B1                                  │
│                 Spec unimplemented? → B2                            │
│                 Coverage gap? → B3/B4                               │
│                 Robustness unknown? → B6                            │
│                 Spec unclear? → B7                                  │
│                                                                     │
│  PRIORITY       B1 > B2 > B7 > B3/B4 > B6 > B5 > B9 > B10 > B8 > B11│
│                                                                     │
│  STUCK          Isolate → Question test → Question spec →          │
│                 Reduce scope → Sleep on it                          │
│                                                                     │
│  HIERARCHY      Philosophy > Architecture > Specs > Tests > Code    │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

**Now execute the appropriate behavior.**