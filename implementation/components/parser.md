# 3. PARSER

## PURPOSE

Transform source text into abstract syntax trees.

## RESPONSIBILITIES

- Tokenize MEW source text (statements and ontology)
- Build AST from tokens
- Produce meaningful error messages with location

## NON-RESPONSIBILITIES

- Name resolution (that's Analyzer)
- Type checking (that's Analyzer)
- Execution (that's Query/Mutation)

## DEPENDS ON

- (none — takes raw strings)

## DEPENDED ON BY

- Analyzer: consumes statement AST
- Compiler: consumes ontology AST
- Session: entry point for user input

## INVARIANTS

- Valid syntax produces AST
- Invalid syntax produces error with line/column
- Parsing is deterministic (same input → same output)

## ACCEPTANCE CRITERIA

- [ ] Parse MATCH statement → MatchStmt AST
- [ ] Parse SPAWN statement → SpawnStmt AST
- [ ] Parse KILL statement → KillStmt AST
- [ ] Parse LINK statement → LinkStmt AST
- [ ] Parse UNLINK statement → UnlinkStmt AST
- [ ] Parse SET statement → SetStmt AST
- [ ] Parse WALK statement → WalkStmt AST
- [ ] Parse BEGIN/COMMIT/ROLLBACK → TxnStmt AST
- [ ] Parse ontology (node, edge, constraint, rule) → OntologyAST
- [ ] Parse expressions (arithmetic, comparison, function calls)
- [ ] Syntax error includes line number and column
- [ ] Syntax error includes what was expected

## NOTES

- Lexer is internal implementation detail
- Statement and ontology grammars share expression sublanguage
- Keywords are case-insensitive; identifiers are case-sensitive
- String literals support escape sequences
