# 13. SESSION

## PURPOSE

Provide external interface for interacting with MEW.

## RESPONSIBILITIES

- Accept statements (REPL, HTTP, embedded)
- Route statements to appropriate executor
- Track session state (current transaction)
- Format and return results
- Handle errors gracefully

## NON-RESPONSIBILITIES

- Parsing (that's Parser)
- Analysis (that's Analyzer)
- Execution (that's Query/Mutation)
- Transaction logic (that's Transaction)

## DEPENDS ON

- Parser: parses input
- Analyzer: analyzes AST
- Query: executes queries
- Mutation: executes mutations
- Transaction: manages transactions
- Compiler: loads ontologies

## DEPENDED ON BY

- (external clients)

## INVARIANTS

- Each session has independent transaction state
- Errors don't crash session
- Results are properly formatted

## ACCEPTANCE CRITERIA

- [ ] Accept statement string
- [ ] Parse and analyze
- [ ] Route to Query/Mutation/Transaction
- [ ] Return results as structured data
- [ ] Return errors with context
- [ ] Track current transaction
- [ ] REPL: interactive prompt
- [ ] REPL: multi-line input
- [ ] REPL: command history
- [ ] HTTP: accept POST with statement
- [ ] HTTP: return JSON response
- [ ] Handle concurrent sessions

## NOTES

- Session state: session_id, current_txn, auto_commit mode
- Result format: columns, types, rows for queries; affected counts for mutations
- REPL commands: .help, .schema, .quit
- HTTP endpoint: POST /execute with body { "statement": "..." }
