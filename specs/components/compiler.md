# 6. COMPILER

## PURPOSE

Transform ontology source into populated Registry and Layer 0 graph.

## RESPONSIBILITIES

- Parse ontology DSL
- Expand syntactic sugar (modifiers → constraints/rules)
- Validate ontology consistency
- Generate Layer 0 nodes and edges
- Build and return Registry

## NON-RESPONSIBILITIES

- Runtime type checking (that's Mutation)
- Statement parsing (that's Parser)
- Schema modification at runtime (v2 META mode)

## DEPENDS ON

- Parser: parses ontology text
- Graph: stores Layer 0
- Registry: returned as output

## DEPENDED ON BY

- Session: LOAD statement triggers compilation

## INVARIANTS

- Valid ontology produces consistent Registry + Layer 0
- Layer 0 and Registry are equivalent representations
- Invalid ontology produces error before any graph modification

## ACCEPTANCE CRITERIA

- [ ] Parse node type definition
- [ ] Parse edge type definition
- [ ] Parse constraint definition
- [ ] Parse rule definition
- [ ] Expand [required] → required constraint
- [ ] Expand [unique] → uniqueness constraint
- [ ] Expand [>= N] → range constraint
- [ ] Expand [acyclic] → cycle detection constraint
- [ ] Expand [on_kill: cascade] → cascade rule
- [ ] Detect duplicate type names → error
- [ ] Detect inheritance cycles → error
- [ ] Detect unknown type references → error
- [ ] Generate _NodeType, _EdgeType, _AttributeDef nodes
- [ ] Generate _type_has_attribute, _edge_has_position edges
- [ ] Return populated Registry

## NOTES

- Compilation is atomic: all or nothing
- Layer 0 types are prefixed with underscore
- Ontology can extend existing schema (EXTEND vs LOAD)
- Future: incremental compilation for schema evolution
