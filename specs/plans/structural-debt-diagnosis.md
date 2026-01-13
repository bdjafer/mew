# Structural Debt Diagnosis

**Date:** 2026-01-13
**Status:** Remediation Complete
**Scope:** Full MEW workspace (17 crates)

## Completed Remediation

### Phase 1: Type Renames ✓
- `mutation::result::MutationResult` → `MutationOutcome`
- `session::result::MutationResult` → `MutationSummary`
- Removed confusing `MutationOutput` alias

### Phase 1b: Error Message Constants ✓
- Created `mew-core/src/messages.rs` with shared constants
- Updated session and REPL to use shared constants
- No more duplicated magic strings

### Phase 2: Shared Target Resolution ✓
- Created `mew-pattern/src/target.rs` with:
  - `resolve_target()` - full resolution with edge patterns
  - `resolve_target_ref()` - reference resolution
  - `resolve_var_target()` - variable-only (for REPL)
  - `TargetError` enum with Display impl
- Session and REPL now use shared module
- Reduced code duplication by ~100 lines

### Phase 3: Partial Session Extraction ✓
- Extracted `session/src/transaction.rs` for transaction control logic
- Extracted `session/src/query.rs` for query result conversion helpers
- Session struct now uses `TransactionState` for cleaner state management
- session.rs reduced from 953 → 897 lines (includes tests)

### Phase 4: Query Executor Extraction ✓
- Extracted `query/src/operators.rs` (625 lines) for plan operator execution
  - `OperatorContext` struct encapsulates execution dependencies
  - All PlanOp variant handlers moved to focused module
  - Value comparison and grouping helpers
- Extracted `query/src/aggregates.rs` (253 lines) for aggregate computation
  - COUNT, SUM, AVG, MIN, MAX implementations
  - Unit tests for aggregate functions
- executor.rs reduced from 1063 → 514 lines (52% reduction)

---

---

## 1. Current State: Module/Dependency Graph

### 1.1 Crate Organization (7 Layers)

```
LAYER 0 - Foundation (0 internal deps)
├── mew-core         # Identity types, Value, Node/Edge entities
└── mew-parser       # Lexer + AST (independent, 0 deps)

LAYER 1 - Storage & Schema
├── mew-graph        # Multi-dimensional indexed graph storage
├── mew-registry     # Immutable schema repository
└── mew-journal      # Write-ahead logging

LAYER 2 - Parsing & Analysis
├── mew-analyzer     # Name resolution + type checking
└── mew-compiler     # Ontology compilation

LAYER 3 - Execution Engines
├── mew-pattern      # Pattern compilation, matching, expression evaluation
├── mew-query        # Query planning & execution
└── mew-mutation     # Write operations

LAYER 4 - Constraints & Rules
├── mew-constraint   # Constraint validation
└── mew-rule         # Reactive rule engine

LAYER 5 - Transaction Coordination
└── mew-transaction  # ACID transactions, orchestration

LAYER 6 - External Interfaces
├── mew-session      # Core orchestration (10 deps)
└── mew-repl         # Interactive CLI (14 deps)

LAYER 7 - Testing
├── mew-testgen      # Generative testing
└── mew-tests        # Integration test framework
```

### 1.2 Critical Dependency Statistics

| Module | Dependents | Role | Risk Level |
|--------|------------|------|------------|
| mew-core | 16/17 (94%) | Foundation types | CRITICAL |
| mew-registry | 13/17 (76%) | Schema lookups | CRITICAL |
| mew-graph | 11/17 (65%) | Storage layer | HIGH |
| mew-pattern | 7/17 (41%) | Universal abstraction | HIGH |
| mew-parser | 6/17 (35%) | AST processing | MEDIUM-HIGH |

### 1.3 Good News

- **No circular dependencies** - Clean DAG structure
- **Clear layer separation** - Dependencies flow upward only
- **Parser independence** - Zero internal dependencies, fully testable

---

## 2. Problem: Identified Anti-Patterns

### 2.1 SHOTGUN SURGERY - Statement Type Changes

Adding a new statement type (e.g., `SUBSCRIBE`) requires synchronized changes to **4 files**:

| File | Lines | What Changes |
|------|-------|--------------|
| `parser/src/parser/stmt.rs:20-59` | 742 | Parser dispatch (10 match arms) |
| `session/src/session.rs:147-194` | 1040 | Execute dispatcher (10 match arms) |
| `analyzer/src/analyzer.rs:32-43` | 936 | Type checker (10 match arms) |
| `repl/src/repl.rs` | 634 | REPL handler |

**Root cause:** Statement handling spread across layers without abstraction.

### 2.2 SHOTGUN SURGERY - Mutation Action Changes

Adding a new mutation action requires changes to **4 files**:

| File | Lines | Location |
|------|-------|----------|
| `analyzer/src/analyzer.rs:63-76` | 936 | 4 match arms |
| `session/src/session.rs:259-323` | 1040 | 4 match arms |
| `repl/src/executor.rs:422-489` | 497 | 4 match arms |
| `parser/src/parser/stmt.rs:222-225` | 742 | Parser dispatch |

### 2.3 SHOTGUN SURGERY - Value Type Changes

The `Value` enum (9 variants) is matched in **15+ locations**:

```
mew-core/src/value.rs:145-215     # type_name(), cmp_sortable(), Display
mew-query/src/executor.rs:594-626 # Numeric coercion
mew-pattern/src/eval.rs           # 868 lines, extensive Value matching
mew-testgen/src/types.rs:167-175  # Conversion
mew-repl/src/format.rs            # Formatting
```

**Impact:** Adding a new Value type (e.g., `List`, `Map`) cascades to 15+ files.

### 2.4 DIVERGENT CHANGE - session.rs (1040 lines)

This file has **8+ distinct responsibilities** that change for different reasons:

1. Session lifecycle management (id, registry, graph access)
2. Transaction state tracking (in_transaction flag)
3. Statement dispatching (10 statement types)
4. Query execution (execute_match, execute_walk, execute_inspect)
5. Mutation execution (execute_spawn, execute_kill, etc.)
6. Compound statement execution (execute_match_mutate - 200+ lines)
7. Target resolution (3 methods, 60+ lines)
8. Variable binding management

**Symptom:** Changes to query logic, mutation semantics, or transaction behavior all require modifying this same file.

### 2.5 DIVERGENT CHANGE - query/executor.rs (1063 lines)

Handles **11+ operator types** in a massive execute_op method:

- NodeScan, IndexScan, EdgeJoin, Filter, Projection
- Sorting, Limit/offset, Aggregation, Cross joins
- Transitive closure, Distinct

Each operator type is a separate concern that evolves independently.

### 2.6 DIVERGENT CHANGE - Other Large Files

| File | Lines | Responsibilities |
|------|-------|------------------|
| `tests/src/assertion.rs` | 989 | 6+ assertion types mixed together |
| `analyzer/src/analyzer.rs` | 936 | 5+ analysis domains |
| `pattern/src/eval.rs` | 868 | 7+ expression types |
| `registry/src/builder.rs` | 746 | 5+ builder classes |

### 2.7 CODE DUPLICATION - Session vs REPL

Nearly identical logic exists in both layers:

**Target resolution:**
```
session/src/session.rs:336-361   # resolve_target_with_bindings()
repl/src/executor.rs:326-350     # resolve_target()
```

**Magic strings duplicated:**
```
"Only variable targets are supported" - appears 6 times
"SET requires a node target" - appears 2 times
"KILL requires a node target" - appears 2 times
"UNLINK requires an edge target" - appears 2 times
```

### 2.8 TYPE NAME COLLISION - MutationResult

Three different types named `MutationResult`:

| Location | Type | Purpose |
|----------|------|---------|
| `mutation/src/error.rs:7` | `type MutationResult<T> = Result<T, MutationError>` | Error result type |
| `mutation/src/result.rs:8` | `enum MutationResult { Created, Deleted, ... }` | Operation outcome |
| `session/src/result.rs:61` | `struct MutationResult { nodes_affected, ... }` | Session result |

The collision is partially worked around by re-exporting:
```rust
pub use result::MutationResult as MutationOutput;
```

But this creates confusion when reading code.

---

## 3. Severity Assessment

### Critical (Must Fix)

| Issue | Impact | Files Affected |
|-------|--------|----------------|
| session.rs god file | Every feature touches it | 1 file, but blocks all changes |
| Statement dispatch shotgun surgery | Adding statements requires 4+ file changes | 4 files |

### High (Should Fix)

| Issue | Impact | Files Affected |
|-------|--------|----------------|
| query/executor.rs god file | All query operator changes here | 1 file |
| Session/REPL duplication | Bugs fixed twice, divergence risk | 2 files |
| MutationResult name collision | Confusing API | 3 files |

### Medium (Consider Fixing)

| Issue | Impact | Files Affected |
|-------|--------|----------------|
| Value type matching spread | Adding value types is expensive | 15+ files |
| analyzer.rs mixed responsibilities | Multiple change reasons | 1 file |
| Magic string duplication | Inconsistent error messages | 4+ files |

---

## 4. Target Architecture

### 4.1 Statement Handler Trait (Fix Shotgun Surgery)

**Current:** Match statements across 4 files.

**Target:** Trait-based dispatch with co-located handlers.

```rust
// In mew-core or new mew-dispatch crate
trait StatementHandler {
    fn analyze(&self, stmt: &Stmt, ctx: &AnalysisContext) -> AnalyzerResult<AnalyzedStmt>;
    fn execute(&self, stmt: &AnalyzedStmt, ctx: &mut ExecutionContext) -> SessionResult<StatementResult>;
}

// Each handler in its relevant crate
struct MatchHandler; // in mew-query
struct SpawnHandler; // in mew-mutation
// etc.
```

**Benefit:** Adding new statements = adding one handler struct + registering it.

### 4.2 Split session.rs by Responsibility

**Current:** 1040-line god file.

**Target:** Focused modules:

```
session/src/
├── lib.rs           # Public API
├── state.rs         # SessionState struct only (~50 lines)
├── dispatch.rs      # Statement routing (~100 lines)
├── query/
│   ├── mod.rs       # Query execution
│   └── match_mutate.rs  # Compound statement
├── mutation.rs      # Mutation execution
├── transaction.rs   # Transaction control
└── target.rs        # Target resolution (shared with REPL)
```

### 4.3 Extract Shared Target Resolution

**Current:** Duplicated in session and REPL.

**Target:** Shared module in mew-core or mew-pattern.

```rust
// In mew-pattern
pub mod target {
    pub fn resolve_target(
        target: &Target,
        bindings: &HashMap<String, EntityId>
    ) -> Result<EntityId, TargetError>;

    pub fn resolve_target_ref(
        target_ref: &TargetRef,
        bindings: &HashMap<String, EntityId>
    ) -> Result<EntityId, TargetError>;
}
```

### 4.4 Split query/executor.rs by Operator

**Current:** 1063-line file with 11+ operators.

**Target:** Operator modules:

```
query/src/
├── executor.rs      # Main executor, orchestration (~200 lines)
├── ops/
│   ├── mod.rs       # Op trait + registration
│   ├── scan.rs      # NodeScan, IndexScan
│   ├── join.rs      # EdgeJoin, CrossJoin
│   ├── filter.rs    # Filter, Distinct
│   ├── transform.rs # Project, Sort, LimitOffset
│   ├── aggregate.rs # Aggregate operator
│   └── walk.rs      # TransitiveClosure
```

### 4.5 Rename Colliding Types

| Current | Proposed |
|---------|----------|
| `mutation::MutationResult<T>` | Keep (standard pattern) |
| `mutation::result::MutationResult` | `MutationOutcome` |
| `session::MutationResult` | `MutationSummary` |

---

## 5. Migration Inventory

### Phase 1: Type Renames (Low Risk)

| Action | Files | Risk |
|--------|-------|------|
| Rename `mutation::result::MutationResult` → `MutationOutcome` | 3 | Low |
| Rename `session::MutationResult` → `MutationSummary` | 2 | Low |
| Extract error message constants | 4 | Low |

### Phase 2: Extract Target Resolution (Medium Risk)

| Action | Files | Risk |
|--------|-------|------|
| Create `mew-pattern/src/target.rs` | 1 new | Low |
| Update `session/src/session.rs` to use shared | 1 | Medium |
| Update `repl/src/executor.rs` to use shared | 1 | Medium |

### Phase 3: Split session.rs (Medium-High Risk)

| Action | Files | Risk |
|--------|-------|------|
| Extract `session/src/state.rs` | 1 new | Low |
| Extract `session/src/mutation.rs` | 1 new | Medium |
| Extract `session/src/query/match_mutate.rs` | 1 new | Medium |
| Update main `session.rs` | 1 | Medium |

### Phase 4: Split query/executor.rs (Medium-High Risk)

| Action | Files | Risk |
|--------|-------|------|
| Create `query/src/ops/` module structure | 7 new | Medium |
| Extract operators one at a time | 1 each | Medium |
| Update main executor | 1 | Medium |

### Phase 5: Statement Handler Trait (High Risk)

This is a larger refactor that should be considered separately.
It would require changes across parser, analyzer, session, and REPL.

---

## 6. Recommended Priorities

### Immediate (Before New Features)

1. **Rename MutationResult types** - 30 minutes, prevents confusion
2. **Extract error message constants** - 1 hour, prevents divergence

### Short Term (Next Sprint)

3. **Extract shared target resolution** - 2 hours, removes duplication
4. **Split session.rs** - 4-6 hours, improves maintainability

### Medium Term (When Modifying Query)

5. **Split query/executor.rs** - 4-6 hours, do when adding operators

### Long Term (Before Major Statement Changes)

6. **Statement Handler trait** - Multi-day refactor, do before adding new statement types

---

## 7. What NOT to Change

### Working Well

- **Layer structure** - Clean 7-layer DAG, don't flatten
- **Parser independence** - Keep zero dependencies
- **Per-crate error types** - Good separation, don't unify
- **Registry immutability** - Correct design choice

### Acceptable Coupling

- **mew-repl has 14 deps** - Expected for integration point
- **mew-session has 10 deps** - Expected for orchestration
- **mew-core used everywhere** - Foundation types should be universal

---

## 8. Metrics to Track

After refactoring, verify:

| Metric | Before | Target |
|--------|--------|--------|
| session.rs lines | 1040 | < 200 |
| query/executor.rs lines | 1063 | < 300 |
| Files touched to add statement | 4 | 1-2 |
| Duplicate code blocks | 4+ | 0 |
| MutationResult types | 3 | 3 (renamed) |

---

## Appendix: File Size Reference

```
1063 query/src/executor.rs       <- DIVERGENT CHANGE
1040 session/src/session.rs      <- DIVERGENT CHANGE
 989 tests/src/assertion.rs      <- DIVERGENT CHANGE
 936 analyzer/src/analyzer.rs    <- DIVERGENT CHANGE
 901 parser/src/lexer.rs         (acceptable - single responsibility)
 868 pattern/src/eval.rs         <- DIVERGENT CHANGE
 746 registry/src/builder.rs     <- Minor concern
 743 parser/src/parser/ontology.rs (acceptable)
 742 parser/src/parser/stmt.rs   <- SHOTGUN SURGERY point
 730 compiler/src/compiler.rs    (acceptable)
```

---

*End of Diagnosis*
