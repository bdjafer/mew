# 13. SESSION

## 13.1 Statement Processing

### TEST: accept_and_execute_query
```
INPUT: "MATCH t: Task RETURN t"
EXPECT: parsed, analyzed, executed
AND returns result rows
```

### TEST: accept_and_execute_mutation
```
INPUT: "SPAWN t: Task { title = \"Hello\" }"
EXPECT: node created
AND returns created ID
```

### TEST: syntax_error_returns_error
```
INPUT: "MATC t: Task"
EXPECT: error response with message and location
```

### TEST: analysis_error_returns_error
```
INPUT: "MATCH t: Unknown RETURN t"
EXPECT: error: "unknown type 'Unknown'"
```

## 13.2 Transaction State

### TEST: session_tracks_transaction
```
INPUT: "BEGIN"
THEN session.current_transaction is Some

INPUT: "COMMIT"
THEN session.current_transaction is None
```

### TEST: transaction_spans_statements
```
INPUT: "BEGIN"
INPUT: "SPAWN t: Task { title = \"A\" }"
INPUT: "SPAWN t: Task { title = \"B\" }"
INPUT: "COMMIT"
THEN both nodes committed together
```

## 13.3 REPL

### TEST: repl_interactive_prompt
```
START REPL
EXPECT: shows prompt "mew> "
```

### TEST: repl_multiline_input
```
INPUT: "MATCH t: Task"
EXPECT: continuation prompt "...> "
INPUT: "RETURN t"
EXPECT: executes complete statement
```

### TEST: repl_help_command
```
INPUT: ".help"
EXPECT: shows available commands
```

### TEST: repl_schema_command
```
INPUT: ".schema"
EXPECT: shows loaded types
```

### TEST: repl_quit_command
```
INPUT: ".quit"
EXPECT: REPL exits
```

## 13.5 Concurrent Sessions

### TEST: concurrent_sessions_isolated
```
SESSION A: BEGIN, SPAWN Task
SESSION B: MATCH Task
THEN B does not see A's uncommitted work
SESSION A: COMMIT
SESSION B: MATCH Task
THEN B now sees A's committed work
```