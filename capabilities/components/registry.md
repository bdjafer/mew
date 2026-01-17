# 4. REGISTRY

## PURPOSE

Provide runtime lookup of schema definitions.

## RESPONSIBILITIES

- Store type definitions (nodes and edges)
- Store constraint definitions
- Store rule definitions
- Provide fast lookup by name and ID
- Track inheritance relationships
- Index constraints/rules by affected types

## NON-RESPONSIBILITIES

- Building itself (that's Compiler)
- Persistence (rebuilt from Layer 0 on load)
- Modification at runtime (immutable after build)

## DEPENDS ON

- (none — populated by Compiler)

## DEPENDED ON BY

- Analyzer: resolves names
- Compiler: populates registry
- Pattern: gets type info for matching
- Mutation: validates types
- Query: gets type info for planning
- Constraint: finds affected constraints
- Rule: finds triggered rules

## INVARIANTS

- Type names are unique
- Edge type names are unique
- All type references resolve
- Inheritance forms a DAG (no cycles)
- Registry is immutable after construction

## ACCEPTANCE CRITERIA

- [ ] Get type by name → TypeDef or None
- [ ] Get type by ID → TypeDef
- [ ] Get edge type by name → EdgeTypeDef or None
- [ ] Get edge type by ID → EdgeTypeDef
- [ ] Check subtype relationship → bool
- [ ] Get all subtypes of type → Set<TypeId>
- [ ] Get constraints for type → Vec<ConstraintDef>
- [ ] Get constraints for edge type → Vec<ConstraintDef>
- [ ] Get rules for type (sorted by priority) → Vec<RuleDef>
- [ ] Get rules for edge type → Vec<RuleDef>
- [ ] Get deferred constraints → Vec<ConstraintDef>

## NOTES

- TypeDef includes: id, name, parent_ids, attributes, abstract, sealed
- EdgeTypeDef includes: id, name, arity, signature, attributes, symmetric
- ConstraintDef includes: id, name, hard, pattern, condition, deferred
- RuleDef includes: id, name, priority, auto, pattern, production
- Precompute subtype sets for fast subtype queries
