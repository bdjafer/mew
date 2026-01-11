# 3. PARSER

## 3.1 Statement Parsing

### TEST: parse_match_single_type
```
INPUT: "MATCH t: Task RETURN t"
EXPECT: MatchStmt {
  pattern: [NodePattern { var: "t", type: "Task" }],
  where: None,
  return: [Var("t")]
}
```

### TEST: parse_match_with_edge
```
INPUT: "MATCH a: Person, b: Person, knows(a, b) RETURN a, b"
EXPECT: MatchStmt {
  pattern: [
    NodePattern { var: "a", type: "Person" },
    NodePattern { var: "b", type: "Person" },
    EdgePattern { type: "knows", targets: ["a", "b"] }
  ],
  return: [Var("a"), Var("b")]
}
```

### TEST: parse_match_with_where
```
INPUT: "MATCH t: Task WHERE t.priority > 5 RETURN t.title"
EXPECT: MatchStmt {
  pattern: [...],
  where: BinaryOp(GT, AttrAccess("t", "priority"), Int(5)),
  return: [AttrAccess("t", "title")]
}
```

### TEST: parse_spawn
```
INPUT: "SPAWN t: Task { title = \"Hello\", priority = 1 }"
EXPECT: SpawnStmt {
  var: "t",
  type: "Task",
  attrs: { "title": String("Hello"), "priority": Int(1) }
}
```

### TEST: parse_kill
```
INPUT: "KILL t"
EXPECT: KillStmt { var: "t" }
```

### TEST: parse_link
```
INPUT: "LINK e: owns(p, t)"
EXPECT: LinkStmt {
  var: "e",
  edge_type: "owns",
  targets: ["p", "t"]
}
```

### TEST: parse_unlink
```
INPUT: "UNLINK e"
EXPECT: UnlinkStmt { var: "e" }
```

### TEST: parse_set
```
INPUT: "SET t.status = \"done\""
EXPECT: SetStmt {
  target: AttrAccess("t", "status"),
  value: String("done")
}
```

### TEST: parse_transaction_statements
```
INPUT: "BEGIN"
EXPECT: TxnStmt::Begin

INPUT: "COMMIT"
EXPECT: TxnStmt::Commit

INPUT: "ROLLBACK"
EXPECT: TxnStmt::Rollback
```

## 3.2 Expression Parsing

### TEST: parse_arithmetic
```
INPUT: "a + b * 2"
EXPECT: BinaryOp(Add, Var("a"), BinaryOp(Mul, Var("b"), Int(2)))
(multiplication has higher precedence)
```

### TEST: parse_comparison
```
INPUT: "x >= 10 AND y < 20"
EXPECT: BinaryOp(And,
  BinaryOp(GtEq, Var("x"), Int(10)),
  BinaryOp(Lt, Var("y"), Int(20))
)
```

### TEST: parse_function_call
```
INPUT: "COUNT(items)"
EXPECT: FnCall { name: "COUNT", args: [Var("items")] }
```

## 3.3 Ontology Parsing

### TEST: parse_node_type
```
INPUT: "node Task { title: String [required], priority: Int [>= 0] }"
EXPECT: NodeTypeDef {
  name: "Task",
  attrs: [
    { name: "title", type: String, modifiers: [Required] },
    { name: "priority", type: Int, modifiers: [Range(min=0)] }
  ]
}
```

### TEST: parse_edge_type
```
INPUT: "edge owns(owner: Person, item: Item) [acyclic]"
EXPECT: EdgeTypeDef {
  name: "owns",
  params: [("owner", "Person"), ("item", "Item")],
  modifiers: [Acyclic]
}
```

### TEST: parse_constraint
```
INPUT: "constraint unique_email on Person: NOT EXISTS p2: Person WHERE p1 != p2 AND p1.email = p2.email"
EXPECT: ConstraintDef { ... }
```

### TEST: parse_rule
```
INPUT: "rule auto_timestamp on Task [auto, priority: 100]: SET t.updated_at = NOW()"
EXPECT: RuleDef {
  name: "auto_timestamp",
  on_type: "Task",
  auto: true,
  priority: 100,
  production: [SetStmt { ... }]
}
```

## 3.4 Error Handling

### TEST: syntax_error_has_location
```
INPUT: "MATCH t: Task WHER t.x > 1"
                      ^^^^
EXPECT: ParseError {
  message: "expected WHERE, found WHER",
  line: 1,
  column: 16
}
```

### TEST: unexpected_eof
```
INPUT: "MATCH t:"
EXPECT: ParseError { message: "unexpected end of input, expected type name" }
```
