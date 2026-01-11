---
name: revise-specification
description: Controlled procedure for revising top-level specifications (FOUNDATIONS.md, DSL.md, GQL.md, architecture.md). REQUIRED when these documents need correction. Do not modify these files without using this skill.
---

# Revise Specification

## When to Use

**REQUIRED** when you need to modify:
- `specification/1_FOUNDATIONS.md`
- `specification/2_DSL.md`
- `specification/3_GQL.md`
- `implementation/architecture.md`

**For `0_META_ONTOLOGY.md`, use `revise-meta-ontology` skill instead.**

## Why This Skill Exists

Specifications are the source of truth. Incorrect changes cascade into:
- Wrong component specs
- Wrong tests
- Wrong code
- Wasted implementation effort

This procedure ensures changes are deliberate, justified, and propagated.

## Pre-Revision Checklist

Before proceeding, verify:

- [ ] I have identified a specific error or inconsistency
- [ ] The error is in THIS document (not a lower-level doc)
- [ ] I have checked higher-level docs to confirm they don't contradict my fix
- [ ] I understand the impact on downstream documents

If any checkbox is false, stop and reconsider.

## Procedure

### 1. Document the Problem

Create a revision record:

```bash
cat >> .spec-revisions.log << 'EOF'
---
Date: [date]
Document: [which spec]
Section: [which section]
Problem: [what's wrong]
Evidence: [how you discovered it]
EOF
```

### 2. Check Hierarchy

The document you're fixing must not contradict a higher document:

```
1_FOUNDATIONS.md    ← Philosophical principles (highest)
2_DSL.md            ← Must conform to FOUNDATIONS
3_GQL.md            ← Must conform to FOUNDATIONS
architecture.md     ← Must conform to all specification/ docs
```

If higher doc contradicts your fix, you must fix the higher doc first.

### 3. Draft the Change

Write the corrected text. Be minimal — change only what's necessary.

```markdown
## BEFORE (quote exact text)
[original text]

## AFTER (your revision)
[corrected text]

## JUSTIFICATION
[why this is correct]
```

### 4. Check Downstream Impact

Identify what else must change:

```bash
# What references this concept?
grep -r "[concept]" implementation/
grep -r "[concept]" specification/
```

List affected documents:
- [ ] `implementation/architecture.md`
- [ ] `implementation/components/*.md`
- [ ] `implementation/tests/*.md`
- [ ] Other specification files

### 5. Apply the Change

```bash
# Edit the specification file
# Make the minimal change documented in step 3
```

### 6. Propagate Downward

For each affected downstream document:

1. Read the document
2. Find sections that assumed old behavior
3. Update to match new specification
4. If it's a component spec, update corresponding test file
5. If tests change, code may need to change

### 7. Record Completion

```bash
cat >> .spec-revisions.log << 'EOF'
Resolution: [what you changed]
Propagated to: [list of updated files]
---
EOF
```

### 8. Verify

```bash
# Check nothing is broken
cargo check --workspace
cargo test --workspace
```

## Example Revision

**Problem:** GQL.md says KILL cascades to edges, but architecture.md says it doesn't.

```
Document: specification/3_GQL.md
Section: KILL statement
Problem: Says "KILL removes node only", but implementation/architecture.md 
         says "KILL removes node AND all incident edges"
Evidence: Implementing KILL, unclear which behavior is correct

Hierarchy check: FOUNDATIONS.md doesn't specify cascade behavior.
                 This is a GQL-level decision.
                 Architecture should conform to GQL, not vice versa.

BEFORE (GQL.md):
"KILL removes the specified node from the graph."

AFTER (GQL.md):
"KILL removes the specified node and all incident edges from the graph.
Higher-order edges about deleted edges are also deleted (cascade)."

JUSTIFICATION:
Dangling edges (edges with missing endpoints) violate graph integrity.
Cascade deletion is the only consistent behavior.

Downstream impact:
- architecture.md: Already says cascade, no change needed
- components/mutation.md: Verify cascade is specified ✓
- components/graph.md: Verify delete_node cascades ✓
- tests/mutation.md: Verify kill_cascades_edges test exists ✓
```

## Pitfalls

- **Don't skip the hierarchy check** — you might be "fixing" something that's correct at a higher level
- **Don't make drive-by improvements** — only fix the identified problem
- **Don't forget propagation** — spec change without downstream update causes inconsistency
- **Don't skip the log** — future you needs to understand why this changed
- **Don't change specs to match buggy code** — specs define correct behavior

## When NOT to Revise

- Spec is ambiguous but your interpretation works → Add clarification, don't change meaning
- You disagree with design decision → Not an error, don't change it
- Implementation would be easier with different spec → Not an error, implement as specified
- Edge case not covered → Add coverage, don't change existing behavior