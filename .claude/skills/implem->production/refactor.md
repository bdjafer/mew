# REFACTOR MODE

1. VERIFY GREEN BASELINE
   - Run full test suite - ALL tests must pass
   - If any fail, switch to FIX MODE first
   - Record baseline metrics: test count, build time, module count

2. DIAGNOSE STRUCTURAL PAIN
   Analyze the codebase for architectural debt:
   - Circular dependencies between modules
   - God modules that everything imports
   - Scattered responsibilities (same concept in 5+ files)
   - Wrong abstraction boundaries (leaky, too granular, too coarse)
   - High coupling / low cohesion clusters
   
   Find where small changes require touching many files (shotgun surgery).
   Find where unrelated changes collide in the same file (divergent change).

3. DEFINE THE TARGET ARCHITECTURE
   Before touching code, write out:
   - CURRENT STATE: Draw the actual module/dependency graph
   - PROBLEM: What specifically is unmaintainable and why
   - TARGET STATE: Draw the desired module/dependency graph
   - JUSTIFICATION: How this improves maintainability
     → Reduces coupling (fewer cross-module dependencies)
     → Increases cohesion (related code lives together)
     → Clarifies boundaries (explicit interfaces between concerns)
     → Enables future changes (what becomes easier)

4. INVENTORY THE MIGRATION
   List everything that moves:
   - Files to create / delete / relocate
   - Types/interfaces to extract or merge
   - Functions to move (source → destination for each)
   - Import paths that change across the codebase
   - Public API surface changes (if any)
   
   Estimate scope: N files affected, M functions relocated.
   If scope is massive (>30 files), split into phases.

5. ESTABLISH MIGRATION STRATEGY
   Choose approach:
   - BIG BANG: Coordinate all moves in one batch (only if scope is small)
   - BRIDGE PATTERN: Introduce adapters, swap internals, remove adapters
   
   For multi-phase refactors:
   - Define intermediate milestones (each must be green)
   - Each phase should be independently valuable
   - Never leave the codebase in a half-migrated state

6. EXECUTE PHASE BY PHASE
   For each phase:
   a) Create new structure (empty modules, interfaces)
   b) Move code to new locations (update imports as you go)
   c) Delete abandoned files/exports
   d) Run full test suite - must stay green throughout
   e) If tests break, STOP - diagnose before continuing
   
   Track progress: X/N files migrated, Y/M functions moved.

7. VERIFY ARCHITECTURAL IMPROVEMENT
   After completion:
   - Dependency graph is cleaner (draw it, compare to before)
   - Module boundaries match mental model
   - No new circular dependencies introduced
   - Public API unchanged OR migration guide written
   - All tests pass, no regressions

8. COMMIT AT PHASE BOUNDARIES
   - One commit per completed phase
   - Message: what was restructured, why, what's now easier
   - Include before/after of module structure if significant
   - Never commit mid-phase (half-migrated = broken)
