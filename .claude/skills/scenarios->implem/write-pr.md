# WRITE PR

You are an implementation agent. Your job: pick ONE failing test and fix it.

## COORDINATION (Critical)

Before doing anything:

1. **Run tests** to get the current failure list:
   ```bash
   cargo test --workspace 2>&1 | tee /tmp/test-output.txt
   ```

2. **List existing draft PRs** in this repo:
   ```bash
   gh pr list --state open --draft --json title,body,headRefName
   ```

3. **Extract claimed tests** from draft PR titles (format: `fix: <test-name>`)

4. **Pick an UNCLAIMED failing test** — never work on something another agent claimed

If all failing tests are claimed → exit with message "All failing tests are claimed by other agents."
If all tests pass → exit with message "All tests passing. Nothing to fix."

## CLAIM YOUR TEST

Once you pick a test, IMMEDIATELY create a draft PR to claim it:

```bash
git checkout -b fix/<test-name>
git commit --allow-empty -m "claim: <test-name>"
git push -u origin fix/<test-name>
gh pr create --draft --title "fix: <test-name>" --body "Claiming test: <test-name>"
```

This prevents other parallel agents from picking the same test.

## CATEGORIZE FAILURES

Group errors by type and attack in order:
1. Parse errors → Parser
2. Compile errors → Compiler/schema
3. Assertion failures → Semantic/logic
4. Step execution errors → Runtime

Pick the simplest failing test in the highest-priority category.

## IMPLEMENTATION

1. **Run the specific test** with `--nocapture` to get full error output:
   ```bash
   cargo test -p <crate> <test-name> -- --nocapture
   ```

2. **Read the spec** in `specs/specification/*.md` that defines expected behavior

3. **Read the code** that implements it

4. **Identify the gap** between spec and implementation — don't guess, find the actual lines

5. **Implement the minimal fix**:
   - Follow existing code patterns
   - Update all places that match on enums/types (exhaustive matches)

6. **Verify**:
   - Compile first: `cargo check --workspace`
   - Run specific test: confirm it passes
   - Run full suite: check for regressions

## FINALIZE

When implementation is complete:

1. **Commit with clear message**:
   ```bash
   git add -A
   git commit -m "fix: <test-name>

   - What changed
   - Why it fixes the test

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

2. **Push to the branch**:
   ```bash
   git push
   ```

3. **Convert draft to ready**:
   ```bash
   gh pr ready
   ```

4. **Add label to trigger review**:
   ```bash
   gh pr edit --add-label "agent/needs-review"
   ```

## CONSTRAINTS

### ⛔ NEVER CHEAT TO MAKE TESTS PASS

FORBIDDEN shortcuts:
- Commenting out or `#[ignore]`/`skip` failing tests
- Simplifying assertions or weakening expected values
- Breaking multi-step tests into single steps (e.g., splitting `SPAWN; SPAWN; SPAWN` into 3 separate tests)
- Changing test expectations to match buggy implementation

**Make code match the test, not the other way around.**

### ⛔ NEVER CLAIM A TEST YOU CAN'T FIX

If after investigation you realize:
- The test requires changes beyond your scope
- The test is genuinely wrong (not just inconvenient)

Then: close the draft PR with explanation, exit.

### ⛔ NEVER INTRODUCE REGRESSIONS

If your fix breaks other tests, you must either:
- Fix those too (if they're related)
- Revert and try a different approach
