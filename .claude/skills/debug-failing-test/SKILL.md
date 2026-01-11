---
name: debug-failing-test
description: Systematic approach to debugging a failing Rust test. Use when a test fails and you're not sure why, or when stuck on the same test for multiple attempts.
---

# Debug Failing Test

## When to Use

- Test fails and cause isn't obvious
- Same test failing after multiple fix attempts
- Assertion failure with unclear reason
- Need to determine if bug is in code, test, or spec

## Quick Diagnosis

```bash
# Run single test with output
cargo test -p mew-[component] [test_name] -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p mew-[component] [test_name] -- --nocapture
```

## Systematic Procedure

### 1. Read the Error

```
assertion failed: left == right
  left: X
  right: Y
```

- What was expected? (right)
- What was actual? (left)
- What's the difference?

### 2. Read the Test

```bash
grep -n "fn [test_name]" mew/[component]/src/*.rs
```

Identify:
- GIVEN (setup)
- WHEN (action)
- THEN (expected)

### 3. Read the Spec

```bash
grep -A 20 "[test_name]" implementation/tests/[component].md
```

### 4. Determine Where Bug Is

The bug is in ONE of:

| Location | Symptom | Action |
|----------|---------|--------|
| **Code** | Code doesn't match spec behavior | Fix code |
| **Test** | Test doesn't match spec | Fix test |
| **Component Spec** | Spec is ambiguous or wrong | Check architecture, then fix spec |
| **Architecture** | Architecture contradicts philosophy | Check specification/, then fix |
| **Specification** | Fundamental design flaw | Use `revise-specification` skill |

**Work up the hierarchy** until you find the source.

### 5. If Bug Is In Spec

**You MUST use the appropriate skill:**

- For `1_FOUNDATIONS.md`, `2_DSL.md`, `3_GQL.md`, `architecture.md`:
  → Use `revise-specification` skill

- For `0_META_ONTOLOGY.md`:
  → Use `revise-meta-ontology` skill

**Do not modify specs without using the skill.**

### 6. If Bug Is In Code

Add debug prints:

```rust
#[test]
fn failing_test() {
    let x = setup();
    dbg!(&x);
    
    let result = action(&x);
    dbg!(&result);
    
    assert_eq!(result, expected);
}
```

Run with `-- --nocapture`.

### 7. Binary Search (Complex Tests)

```rust
#[test]
fn complex_test() {
    let a = step1();
    assert!(check_a(&a), "failed at step1");
    
    let b = step2(&a);
    assert!(check_b(&b), "failed at step2");
    
    let c = step3(&b);
    assert!(check_c(&c), "failed at step3");
}
```

### 8. Minimal Reproduction

```rust
#[test]
fn minimal_repro() {
    let x = Thing::new();
    let y = x.problematic_method();
    assert!(y.is_some());
}
```

## Common Bug Locations

| Symptom | Likely Location |
|---------|-----------------|
| Off-by-one | Code (index handling) |
| None when Some expected | Code (missing insert or wrong lookup) |
| Test passes alone, fails with others | Test (shared state) |
| Behavior doesn't match any reasonable interpretation | Spec (ambiguous or wrong) |
| Two specs say different things | Higher-level spec (needs revision) |
| Implementation would require impossible semantics | Specification (fundamental issue) |

## Decision Tree

```
Test fails
    │
    ├─ Is the test correct per component spec?
    │   ├─ No → Fix test
    │   └─ Yes ↓
    │
    ├─ Is the code correct per component spec?
    │   ├─ No → Fix code
    │   └─ Yes ↓
    │
    ├─ Is the component spec correct per architecture?
    │   ├─ No → Fix component spec
    │   └─ Yes ↓
    │
    ├─ Is the architecture correct per specification/?
    │   ├─ No → Use revise-specification skill
    │   └─ Yes ↓
    │
    └─ Debug harder. The bug is in the code.
```

## Pitfalls

- **Don't change test to match buggy code** — verify test is correct first
- **Don't add #[ignore]** — fix it or fix the spec
- **Don't modify specs directly** — use the required skill
- **Don't assume bug is where error appears** — trace back through the hierarchy
- **Don't make random changes** — be systematic