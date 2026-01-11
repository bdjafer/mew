# AGENTS.md

How to operate autonomously in the MEW implementation environment.

---

## Operating Context

You run in a **closed micro-vm** with:
- Full filesystem access
- Rust toolchain (cargo, rustc, rustfmt, clippy)
- Git, standard Unix tools
- No internet, no human supervision

**You operate continuously until terminal condition is met.**

---

## Prime Directive

```
WHILE NOT terminal_condition_met:
    assess_state()
    decide_next_action()
    execute_action()
    verify_result()
```

**Never stop.** If you think you're done, verify. If stuck, apply unstuck procedures. If procedures fail, debug them.

---

## Using Skills

Skills are reusable procedures in `.claude/skills/`. Use them.

**Available skills:**
- `context-recovery` — After restart or confusion
- `implement-component` — Starting a new component
- `debug-failing-test` — When tests fail
- `revise-specification` — When specs need correction (REQUIRED for spec changes)
- `revise-meta-ontology` — When Layer 0 needs correction (REQUIRED for meta-ontology changes)

**Create new skills** when you solve a problem you'll face again:
```bash
mkdir -p .claude/skills/skill-name
# Write SKILL.md with: name, description, when to use, procedure, pitfalls
```

---

## Startup Protocol

Execute on every fresh start or context loss:

```bash
# 1. Where am I?
pwd && ls -la

# 2. What exists?
find mew -name "*.rs" -type f 2>/dev/null | wc -l

# 3. What compiles?
cd mew && cargo check --workspace 2>&1 | tail -20

# 4. What passes?
cd mew && cargo test --workspace 2>&1 | grep -E "test result|FAILED"

# 5. Recent context
git log --oneline -5
tail -20 .agent-journal.log 2>/dev/null
```

Then apply decision procedure.

---

## Decision Procedure

```
1. Failing test in working code?     → Fix it (bugs compound)
2. Component blocking others?        → Unblock it
3. Test almost passing?              → Finish it
4. Otherwise                         → Lowest-dependency unimplemented test
```

Reference: `implementation/meta-roadmap.md` section 3.

---

## Progress Measurement

```bash
# Count passing tests
cargo test --workspace 2>&1 | grep "test result"

# Progress = passing / 158
```

This is the only metric that matters.

---

## Work Journal

Maintain context across restarts:

```bash
# After significant work
echo "$(date): [component] [X/158 tests] [what you did] [next action]" >> .agent-journal.log

# On startup
tail -20 .agent-journal.log
```

---

## Stuck Detection

**You are stuck if:**
- Same test failing 3+ attempts with different approaches
- Circular dependency between components
- Test requires undocumented behavior
- Implementation contradicts spec
- Two specs contradict each other

**Stuck is not:**
- Test is hard
- Work is tedious
- Need to learn something

---

## Unstuck Procedures

### Test Won't Pass

```
1. Isolate: minimal reproduction
2. Is test correct? → Check component spec
3. Is spec correct? → Check architecture
4. Is architecture correct? → Check specification
5. Documents align, code fails → Debug harder
6. Documents conflict → Fix at highest level using required skill
```

### Spec Seems Wrong

**You MUST use the skill.** Do not modify specs without it.

```
1. Identify which spec is wrong
2. Invoke appropriate skill:
   - revise-specification (for foundations, DSL, GQL, architecture)
   - revise-meta-ontology (for Layer 0)
3. Follow skill procedure exactly
4. Propagate changes downward
```

---

## Anti-Patterns

| Don't | Do Instead |
|-------|------------|
| Stop without terminal condition | Verify explicitly, continue if not met |
| Ask for guidance | Apply decision procedure |
| Skip or ignore tests | Fix them or fix the spec (using skill) |
| Implement undocumented features | Only what's in acceptance tests |
| Optimize before correctness | Make it work first |
| Large uncommitted changes | Commit every working state |
| Modify specs without skill | Always use required skill |

---

## Recovery Protocol

After crash, restart, or confusion:

1. **Don't panic** — state is in files
2. **Read breadcrumbs:**
   ```bash
   git log --oneline -5
   git status
   tail -20 .agent-journal.log
   ```
3. **Assess state** (startup protocol)
4. **Apply decision procedure**
5. **Continue**

---

## Continuous Operation

```
Components: 13
Tests: 158
Estimated LOC: 10,000-20,000
```

### After Each Component

- [ ] All component tests pass
- [ ] Integrates with dependencies
- [ ] Committed with clear message
- [ ] Journal updated

### Self-Check (Periodically)

- [ ] Am I making progress? (test count increasing)
- [ ] Following decision procedure?
- [ ] Working on highest priority?
- [ ] Recent work committed?
- [ ] Current approach working?

If any "no": stop, reassess, adjust.

---

## Terminal Verification

Before claiming done:

```bash
# 1. All tests pass
cargo test --workspace  # Must be 158/158

# 2. Terminal session works
# Run the session from meta-roadmap.md section 1

# 3. Ontologies load
# Start REPL, load each ontology from ontologies/

# 4. Crash recovery works
# Start transaction, kill -9, restart, verify recovery
```

**"I think I'm done" is not done. Verify explicitly.**

---

## Final Notes

- You have everything you need
- The documents are complete
- The tests define done
- The meta-roadmap tells you what to do
- Skills exist — use them
- You can create more skills

**Start. Continue. Finish.**