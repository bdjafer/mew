# Meta-Ontology Revision Protocol

## Document Purpose

This document defines the governance process for proposing, evaluating, and implementing changes to the **Layer 0 Meta-Ontology Specification**.

Layer 0 is the axiomatic foundation upon which all ontologies, all data, and all reasoning in the system are built. Changes to Layer 0 propagate to everything. Therefore, revisions must meet an exceptionally high bar.

**The default answer to any proposed revision is NO.**

A revision must earn its place through rigorous justification.

---

## Core Principles

### Principle 1: Immutability Preference

Layer 0 should be treated as **immutable by default**.

The specification was designed to be complete for its intended scope. If something seems missing, first ask:

- Can this be expressed using existing constructs?
- Can this be built as a Layer 1+ ontology on top of Layer 0?
- Is this actually necessary, or merely convenient?

Only when all three answers are definitively "no" should a revision be considered.

---

### Principle 2: Irreducibility Requirement

A proposed addition must be **irreducible** — it cannot be expressed as a combination of existing Layer 0 constructs.

**Test for irreducibility:**

```
For proposed construct X:
  1. Attempt to represent X using existing Layer 0 types and edges
  2. Attempt to represent X using a Layer 1 ontology
  3. Attempt to represent X using engine-level implementation without spec change
  
If ANY of these succeed → X is reducible → REJECT the proposal
```

**Examples of reducible proposals (reject):**

| Proposal | Why Reducible |
|----------|---------------|
| Add `_Comment` type for documentation | Use `doc: String?` attribute on existing types |
| Add `_Priority` edge for constraints | Use `priority: Int` attribute on `_ConstraintDef` (already exists) |
| Add `_Symmetric` edge type | Use `symmetric: Bool` attribute on `_EdgeType` (already exists) |
| Add `_Timestamp` type | Already have `Timestamp` scalar |
| Add `_Alias` for type renaming | Build as Layer 1 pattern with indirection |

---

### Principle 3: Orthogonality Requirement

A proposed addition must be **orthogonal** — it must not overlap in function with existing constructs.

**Test for orthogonality:**

```
For proposed construct X:
  1. List all existing constructs that relate to X's domain
  2. Define precisely what X does that they do not
  3. Verify X does not create redundant paths to the same outcome
  
If overlap exists → X violates orthogonality → REJECT or REFINE
```

**Example of non-orthogonal proposal (reject):**

> "Add `_MustExist` constraint type for mandatory relationships"

This overlaps with:
- `required: Bool` on `_AttributeDef`
- Constraint definitions with EXISTS patterns

The proposal creates a redundant mechanism.

---

### Principle 4: Minimal Footprint

Any accepted revision must have the **smallest possible footprint**.

```
Prefer:
  1 new edge type over 1 new node type
  1 new attribute over 1 new edge type
  Documentation clarification over any structural change
  
If a proposal requires N constructs, ask:
  Can the same capability be achieved with N-1 constructs?
  Repeat until minimal.
```

---

### Principle 5: Backward Compatibility

Revisions must be **backward compatible** or provide a **clear migration path**.

| Compatibility Level | Definition | Requirement |
|---------------------|------------|-------------|
| **Full** | Existing ontologies work unchanged | Preferred |
| **Soft deprecation** | Old constructs still work, new way preferred | Acceptable with justification |
| **Migration required** | Existing ontologies need updates | Requires migration tooling |
| **Breaking** | Existing ontologies become invalid | Exceptional circumstances only |

**Breaking changes require:**
- Unanimous approval from all maintainers
- Published migration guide
- Tooling to automate migration
- 6-month deprecation notice (if in production)

---

### Principle 6: Justification Burden

The burden of proof lies **entirely on the proposer**.

A proposal without complete justification is automatically rejected. "This would be nice to have" is not justification.

---

## Proposal Requirements

### Required Sections

Every revision proposal must include ALL of the following:

#### 1. Problem Statement

```
What specific problem does this revision solve?
- Concrete use case that cannot be addressed today
- Evidence that the problem is real (not hypothetical)
- Impact assessment: how many users/ontologies are affected
```

**Rejection trigger:** Vague problem statement, hypothetical use cases only

#### 2. Irreducibility Proof

```
Demonstrate that no existing mechanism can solve the problem:
- Attempted solution using existing Layer 0 constructs (show failure)
- Attempted solution using Layer 1 ontology (show failure)  
- Attempted solution using engine implementation (show failure)
- Explanation of WHY each attempt fails
```

**Rejection trigger:** Missing any attempt, superficial analysis

#### 3. Proposed Change

```
Precise specification of the change:
- Exact new types, edges, constraints, or modifications
- Complete syntax and semantics
- Integration with existing constructs
- No ambiguity — another implementer could build from this alone
```

**Rejection trigger:** Incomplete specification, ambiguity

#### 4. Minimality Argument

```
Demonstrate this is the smallest possible change:
- Alternatives considered and rejected (with reasons)
- Why each element of the proposal is necessary
- What happens if any element is removed
```

**Rejection trigger:** Unexplored alternatives, unnecessary elements

#### 5. Compatibility Analysis

```
Impact on existing system:
- Effect on existing Layer 0 constructs
- Effect on existing ontologies (test against all 9 reference ontologies)
- Effect on existing queries and rules
- Effect on engine implementation
- Migration path if not fully compatible
```

**Rejection trigger:** Breaking changes without migration path

#### 6. Test Cases

```
Validation criteria:
- At least 3 concrete examples using the new construct
- At least 1 edge case / boundary condition
- At least 1 negative test (what should NOT be allowed)
- Integration test with existing constructs
```

**Rejection trigger:** Insufficient test coverage

#### 7. Reversibility Assessment

```
What if this revision is wrong?
- Can it be removed in a future version?
- What would removal break?
- Is there a sunset path?
```

**Rejection trigger:** Irreversible changes without exceptional justification

---

## Evaluation Process

### Stage 1: Initial Screening (1 week)

```
Evaluator checklist:
□ All required sections present
□ Problem statement is concrete and evidenced
□ Irreducibility proof is complete
□ Proposal is precisely specified
□ Test cases are sufficient

Outcome: PASS to Stage 2 / REJECT with feedback
```

### Stage 2: Technical Review (2 weeks)

```
Review criteria:
□ Verify irreducibility claims (attempt alternatives)
□ Verify minimality (attempt reduction)
□ Verify compatibility (test against reference ontologies)
□ Verify implementation feasibility
□ Identify unintended consequences

Outcome: APPROVE / REQUEST CHANGES / REJECT
```

### Stage 3: Community Review (2 weeks)

```
Public review period:
□ Proposal published for community feedback
□ All feedback must receive response from proposer
□ Significant objections must be resolved

Outcome: PROCEED / REVISE / REJECT
```

### Stage 4: Final Approval

```
Approval requirements:
□ Technical review passed
□ Community review completed
□ No unresolved objections
□ Maintainer sign-off

For ADDITIONS: Majority maintainer approval
For MODIFICATIONS: Unanimous maintainer approval  
For REMOVALS: Unanimous approval + deprecation period
```

---

## Change Categories

### Category A: Clarification

**Definition:** No structural change; improves documentation or resolves ambiguity.

**Examples:**
- Adding examples to existing definitions
- Clarifying edge cases in semantics
- Fixing typos or inconsistencies

**Process:** Expedited review (Stage 1 + maintainer approval)

**Versioning:** Patch version (1.0.x)

---

### Category B: Addition

**Definition:** New construct that does not modify existing constructs.

**Examples:**
- New Layer 0 type
- New Layer 0 edge type
- New Layer 0 constraint
- New scalar type

**Process:** Full review (all stages)

**Versioning:** Minor version (1.x.0)

**Requirements:**
- Must not change semantics of existing constructs
- Must not require changes to existing ontologies
- Existing queries must continue to work

---

### Category C: Modification

**Definition:** Change to existing construct behavior or structure.

**Examples:**
- Adding attribute to existing type
- Changing constraint semantics
- Modifying type checking rules

**Process:** Full review + extended community period

**Versioning:** Minor version (1.x.0) if compatible, Major version (x.0.0) if breaking

**Requirements:**
- Backward compatibility strongly preferred
- If breaking, requires migration tooling
- Must demonstrate necessity (not just improvement)

---

### Category D: Deprecation

**Definition:** Marking existing construct for future removal.

**Examples:**
- Deprecating a type in favor of better alternative
- Deprecating an edge type

**Process:** Full review + deprecation period

**Versioning:** Minor version (marks deprecated)

**Requirements:**
- Replacement must exist
- Migration path documented
- Minimum 1 year deprecation period before removal
- Must continue to function during deprecation

---

### Category E: Removal

**Definition:** Removing existing construct from specification.

**Examples:**
- Removing deprecated type
- Removing deprecated edge type

**Process:** Requires prior deprecation + final review

**Versioning:** Major version (x.0.0)

**Requirements:**
- Must have been deprecated for minimum period
- Migration tooling available
- No active usage (verified)

---

## Rejection Reasons (Quick Reference)

A proposal MUST be rejected if ANY of the following apply:

| Code | Reason | Description |
|------|--------|-------------|
| **R1** | Reducible | Can be expressed with existing constructs |
| **R2** | Non-orthogonal | Overlaps with existing construct |
| **R3** | Not minimal | Smaller change could achieve same goal |
| **R4** | Breaking without path | Incompatible with no migration |
| **R5** | Incomplete proposal | Missing required sections |
| **R6** | Insufficient justification | Problem not evidenced |
| **R7** | Hypothetical only | No concrete use case |
| **R8** | Convenience only | Not necessary, just nice-to-have |
| **R9** | Unresolved objections | Community concerns not addressed |
| **R10** | Implementation infeasible | Cannot be reasonably implemented |

---

## Version Numbering

```
MAJOR.MINOR.PATCH

MAJOR: Breaking changes (requires migration)
MINOR: Additions or compatible modifications
PATCH: Clarifications and documentation
```

**Current version:** 1.0.0

**Version history must be maintained with:**
- Date of change
- Category of change
- Summary of change
- Link to full proposal
- Migration notes (if applicable)

---

## Proposal Template

```markdown
# Layer 0 Revision Proposal: [Title]

**Proposal ID:** PROP-YYYY-NNN
**Author:** [Name]
**Date:** [Date]
**Category:** [A/B/C/D/E]
**Status:** [Draft/Review/Approved/Rejected]

## 1. Problem Statement

[Concrete problem description]

### Evidence
[Proof that problem is real]

### Impact
[Who/what is affected]

## 2. Irreducibility Proof

### Attempt 1: Layer 0 Constructs
[What was tried, why it failed]

### Attempt 2: Layer 1 Ontology
[What was tried, why it failed]

### Attempt 3: Engine Implementation
[What was tried, why it failed]

### Conclusion
[Why new construct is necessary]

## 3. Proposed Change

### New Constructs

```
[Precise specification]
```

### Integration
[How it connects to existing constructs]

### Semantics
[Precise behavior definition]

## 4. Minimality Argument

### Alternatives Rejected
[Other approaches and why rejected]

### Necessity of Each Element
[Why each part is required]

## 5. Compatibility Analysis

### Layer 0 Impact
[Effect on existing Layer 0]

### Ontology Impact
[Effect on reference ontologies]

### Engine Impact
[Implementation changes needed]

### Migration Path
[If applicable]

## 6. Test Cases

### Positive Tests
[Examples that should work]

### Edge Cases
[Boundary conditions]

### Negative Tests
[What should fail]

## 7. Reversibility Assessment

### Removal Path
[How this could be undone]

### Removal Impact
[What removal would break]

---

## Review History

| Date | Reviewer | Outcome | Notes |
|------|----------|---------|-------|
| | | | |

## Approval

| Maintainer | Decision | Date |
|------------|----------|------|
| | | |
```

---

## Governance

### Maintainers

Layer 0 maintainers are responsible for:
- Screening proposals
- Conducting technical review
- Making final approval decisions
- Maintaining specification integrity

**Maintainer requirements:**
- Deep understanding of Layer 0 design
- Commitment to specification integrity
- No conflict of interest on proposals

### Conflict Resolution

If maintainers disagree:
1. Extended discussion period (1 week)
2. Formal written positions from each side
3. If still unresolved: proposal is rejected (default is no change)

### Appeals

Rejected proposals may be appealed ONCE if:
- New evidence is presented
- Significant revision addresses rejection reasons
- 30-day waiting period has passed

---

## Final Note

**The purpose of this protocol is to say NO.**

Layer 0 should change rarely. A specification that changes frequently is not a foundation — it's a moving target. Every addition is a commitment forever. Every modification risks breaking the ecosystem.

When in doubt, reject. When uncertain, wait. When pressured, resist.

The value of Layer 0 is its stability. Protect it.

---

*This revision protocol is itself subject to the revision protocol.*
*Changes to this document require Category C (Modification) process.*