# 5. ANALYZER

## PURPOSE

Resolve names and check types in parsed AST.

## RESPONSIBILITIES

- Resolve type names to TypeId
- Resolve attribute names to AttrId
- Resolve variable references to declarations
- Check expression types
- Check operator compatibility
- Annotate AST with type information

## NON-RESPONSIBILITIES

- Parsing (that's Parser)
- Execution (that's Query/Mutation)
- Schema definition (that's Compiler)

## DEPENDS ON

- Parser: provides AST
- Registry: provides schema for resolution

## DEPENDED ON BY

- Query: receives analyzed AST
- Mutation: receives analyzed AST
- Session: orchestrates analysis

## INVARIANTS

- All names resolve or produce error
- All expressions have known types
- Type errors are caught before execution

## ACCEPTANCE CRITERIA

- [ ] Resolve type name → TypeId or error "unknown type"
- [ ] Resolve attribute on type → AttrId or error "unknown attribute"
- [ ] Resolve variable reference → declaration or error "undefined variable"
- [ ] Check Int + Int → Int
- [ ] Check Int + String → type error
- [ ] Check comparison operand types match
- [ ] Check attribute access type matches context
- [ ] Produce annotated AST with all types resolved

## NOTES

- Variable scope: pattern introduces variables, WHERE/RETURN can reference them
- EXISTS introduces nested scope
- Type widening: subtype values accepted where supertype expected
- Edge targets checked against signature types
