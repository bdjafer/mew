# REVISE SPEC

Your goal: update the specification to reflect a better understanding.

## The Weight of This Decision

Specs are not documentation. They are **contracts**. Everything downstream depends on them:

```
spec change → scenario changes → test changes → implementation changes
```

A wrong spec revision:
- Invalidates correct tests
- Breaks working code
- Misleads future implementers
- Compounds into architectural debt

Revise specs deliberately, not casually. This is a high-impact decision.

## When to Use

Use this when `/compare-spec` detected drift AND:
- Implementation discovered a cleaner approach than spec anticipated
- Spec was incomplete and implementation filled the gap correctly
- Spec had a genuine error that implementation correctly avoided
- Real-world usage revealed spec was impractical

## When NOT to Use

- **Implementation is simply wrong** → fix the code, not the spec
- **You're unsure which is correct** → escalate, don't guess
- **Implementation is easier but spec is right** → implement as specified
- **You disagree with design decision** → not an error, don't change it
- **Edge case not covered** → add coverage, don't change existing behavior

The spec is not a rubber stamp for whatever code happens to exist.

## Procedure

### 1. Verify This Is the Right Call

Before touching the spec, confirm:
- [ ] I have concrete evidence the spec is wrong (not just inconvenient)
- [ ] The implementation behavior is genuinely better (not just different)
- [ ] I've checked higher-level specs don't contradict my change
- [ ] I understand what downstream documents will need updates

If any checkbox is false, stop.

### 2. Document the Problem

Be explicit about what you're changing and why:

```markdown
**Document**: specs/specification/X_NAME.md
**Section**: §Y.Z [section title]

**Current spec says**:
> [exact quote]

**Implementation does**:
[precise description]

**Why implementation is correct**:
[concrete reasoning, not "it's simpler"]
```

### 3. Check the Hierarchy

Specs have precedence:

```
1_FOUNDATIONS.md    ← Principles (highest authority)
2_LAYER0.md         ← Must conform to FOUNDATIONS
3_SCHEMA.md         ← Must conform to above
4_QUERIES.md        ← Must conform to above
5_MUTATIONS.md      ← Must conform to above
6_SYSTEM.md         ← Must conform to all above
```

Your change must not contradict a higher-level spec. If it does, you must revise the higher spec first—or accept that your change is wrong.

### 4. Draft the Revision

Write the corrected text. Be minimal—change only what's necessary.

```markdown
**BEFORE**:
> [exact original text]

**AFTER**:
> [your revision]

**JUSTIFICATION**:
[why this is correct, what problem it solves]
```

### 5. Identify Downstream Impact

What else must change as a result?

```bash
# Find references to this concept
grep -r "[concept]" specs/
grep -r "[concept]" examples/
```

List affected items:
- [ ] Other spec sections that reference this
- [ ] Scenarios that test this behavior
- [ ] Coverage tables (LEVEL*_SPECS.md)

### 6. Apply the Change

Edit the spec file. Make exactly the change you documented—no drive-by improvements.

### 7. Propagate

For each affected downstream item:
1. Read it
2. Find assumptions that no longer hold
3. Update to match new spec
4. Verify consistency

### 8. Verify

```bash
# Ensure nothing broke
cargo check --workspace
cargo test --workspace
```

If tests fail, either:
- Your spec change was wrong (revert it)
- Tests need updating to match new spec (update them)

## Output

- Updated spec file(s)
- List of downstream updates made
- Summary: what changed, why, what behavior is now correctly documented

## Remember

You are not just editing a markdown file. You are changing the contract that the entire system is built against. Be certain. Be minimal. Be correct.
