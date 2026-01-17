# MEW Language Specification

## Part I: Foundations

**Version:** 2.1
**Status:** Stable Core
**Scope:** Lexical structure, scalar types, core type expressions, core expressions

---

# 1. Introduction

## 1.1 Purpose

This specification defines the foundational constructs shared by both MEW languages:

- **Ontology DSL**: A declarative language for defining graph schemas (node types, edge types, constraints). Compiled into Layer 0 structures.

- **MEW Language**: A runtime language for operating on graphs (queries, mutations). Interpreted against a running engine.

This document covers the **stable kernel** — constructs unlikely to change.

## 1.2 Modular Specification Structure

| Document | Contents | Stability |
|----------|----------|-----------|
| `core/1_FOUNDATIONS.md` | Lexical, types, expressions (this document) | Stable |
| `core/2_LAYER0_CORE.md` | Meta-schema kernel | Stable |
| `core/3_ONTOLOGY_DSL.md` | Base schema definition syntax | Stable |
| `features/*.md` | Optional/extended functionality | Evolving |
| `runtime/*.md` | Query and mutation languages | Evolving |

**Extension points** are marked throughout. Feature modules may add:
- Keywords
- Expression types
- Type expression forms
- Operators
- Literal forms

## 1.3 Conformance

An implementation conforms to this specification if it:

1. Accepts all valid programs as defined by the grammar
2. Rejects all invalid programs with appropriate errors
3. Executes valid programs with the semantics defined herein
4. Implements all core constructs; features may be optional

## 1.4 Notation

### 1.4.1 Grammar Notation

Extended Backus-Naur Form (EBNF):

| Notation | Meaning |
|----------|---------|
| `rule = expr` | Rule definition |
| `"text"` | Literal text |
| `A B` | Sequence |
| `A \| B` | Alternative |
| `A?` | Optional (zero or one) |
| `A*` | Zero or more |
| `A+` | One or more |
| `(A B)` | Grouping |
| `[a-z]` | Character range |
| `~[x]` | Any character except x |

### 1.4.2 Requirement Levels

- **MUST** / **MUST NOT**: Absolute requirement
- **SHOULD** / **SHOULD NOT**: Recommended
- **MAY**: Optional

---

# 2. Lexical Structure

## 2.1 Source Text

### 2.1.1 Character Set

Source text MUST be encoded as UTF-8.

### 2.1.2 Line Terminators

```ebnf
LineTerminator = "\n" | "\r\n" | "\r"
```

### 2.1.3 Input Elements

```ebnf
InputElement = Whitespace | Comment | Token
```

Whitespace and comments are discarded after lexing.

## 2.2 Whitespace

```ebnf
Whitespace = (" " | "\t" | LineTerminator)+
```

## 2.3 Comments

### 2.3.1 Line Comments

```ebnf
LineComment = "--" ~[\n\r]* LineTerminator?
```

### 2.3.2 Block Comments

```ebnf
BlockComment = "/*" (~[*] | "*" ~[/])* "*/"
```

Block comments do not nest.

### 2.3.3 Documentation Comments

```ebnf
DocComment = "---" ~[\n\r]* LineTerminator?
```

Preserved in AST and attached to subsequent declarations.

```
--- Represents a point in time.
node Event {
  --- Milliseconds since Unix epoch
  timestamp: Timestamp
}
```

## 2.4 Tokens

```ebnf
Token = Keyword | Identifier | Literal | Operator | Punctuation
```

## 2.5 Keywords

### 2.5.1 Case Sensitivity

**Keywords** are case-insensitive:
```
MATCH t: Task RETURN t
match t: Task return t    -- equivalent
```

**Identifiers** are case-sensitive:
```
Task ≠ task
```

### 2.5.2 Core Keywords

Reserved in all MEW programs:

```
and         as          bool        by
constraint  edge        false       float
from        id          in          indexed
int         is          kill        link
match       node        not         null
ontology    or          order       required
return      set         spawn       string
timestamp   true        type        unique
unlink      where
```

## 2.6 Identifiers

```ebnf
Identifier = [a-zA-Z_] [a-zA-Z0-9_]*
```

### 2.6.1 Rules

1. MUST begin with letter or underscore
2. MAY contain letters, digits, underscores
3. Case-sensitive
4. MUST NOT be a keyword
5. Identifiers starting with `_` are reserved for Layer 0

### 2.6.2 Reserved Prefixes

| Prefix | Reserved For |
|--------|--------------|
| `_` | Layer 0 types, edges, attributes |
| `__` | Engine internal use |

### 2.6.3 Qualified Identifiers

```ebnf
QualifiedIdentifier = Identifier ("::" Identifier)*
```

```
Event                    -- simple
Physics::Event           -- qualified
```

## 2.7 Literals

```ebnf
Literal = StringLiteral | IntLiteral | FloatLiteral | BoolLiteral | NullLiteral
```

### 2.7.1 String Literals

```ebnf
StringLiteral = '"' StringCharacter* '"'

StringCharacter = ~["\\\n\r] | EscapeSequence

EscapeSequence = "\\" ["\\/bfnrt] | "\\u" HexDigit{4}

HexDigit = [0-9a-fA-F]
```

**Escape sequences:**

| Sequence | Character |
|----------|-----------|
| `\"` | Double quote |
| `\\` | Backslash |
| `\/` | Forward slash |
| `\b` | Backspace |
| `\f` | Form feed |
| `\n` | Line feed |
| `\r` | Carriage return |
| `\t` | Tab |
| `\uXXXX` | Unicode BMP code point |

### 2.7.2 Integer Literals

```ebnf
IntLiteral = "-"? [0-9]+
```

64-bit signed. Range: −2⁶³ to 2⁶³−1.

### 2.7.3 Float Literals

```ebnf
FloatLiteral = "-"? [0-9]+ "." [0-9]+ ([eE] [+-]? [0-9]+)?
             | "-"? [0-9]+ [eE] [+-]? [0-9]+
```

64-bit IEEE 754 double-precision.

### 2.7.4 Boolean Literals

```ebnf
BoolLiteral = "true" | "false"
```

### 2.7.5 Null Literal

```ebnf
NullLiteral = "null"
```

## 2.8 Operators

### 2.8.1 Core Operators

```
-- Comparison
=   !=   <   >   <=   >=

-- Arithmetic
+   -   *   /   %

-- Logical (also keywords)
and   or   not

-- String
++              -- concatenation

-- Access
.               -- attribute access

-- Type
:               -- type annotation
|               -- union type
?               -- optional type suffix
```

### 2.8.2 Operator Precedence

From highest to lowest:

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `.` | Left |
| 2 | unary `-`, `not` | Right |
| 3 | `*`, `/`, `%` | Left |
| 4 | `+`, `-`, `++` | Left |
| 5 | `<`, `>`, `<=`, `>=` | Left |
| 6 | `=`, `!=` | Left |
| 7 | `and` | Left |
| 8 | `or` | Left |

Parentheses override precedence.

## 2.9 Punctuation

```
(   )           -- grouping, signatures
{   }           -- blocks, objects
[   ]           -- modifiers
,               -- separator
;               -- statement terminator (optional)
:               -- type annotation
::              -- namespace qualifier
=>              -- implication
```

---

# 3. Scalar Types

## 3.1 Overview

MEW has five scalar (primitive) types:

| Type | Description | Size |
|------|-------------|------|
| `String` | UTF-8 text | Variable |
| `Int` | Signed integer | 64 bits |
| `Float` | IEEE 754 double | 64 bits |
| `Bool` | Boolean | 1 bit |
| `Timestamp` | Unix epoch milliseconds | 64 bits |

Additionally, `ID` is an opaque identifier type used internally.

## 3.2 String

A sequence of Unicode code points encoded as UTF-8.

**Operations:**

| Operation | Syntax | Result |
|-----------|--------|--------|
| Equality | `a = b` | Bool |
| Comparison | `a < b`, etc. | Bool (lexicographic) |
| Concatenation | `a ++ b` | String |

**Built-in functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `length(s)` | String → Int | Character count |
| `lower(s)` | String → String | Lowercase |
| `upper(s)` | String → String | Uppercase |
| `trim(s)` | String → String | Remove surrounding whitespace |
| `contains(s, sub)` | String × String → Bool | Substring test |
| `starts_with(s, pre)` | String × String → Bool | Prefix test |
| `ends_with(s, suf)` | String × String → Bool | Suffix test |
| `substring(s, start, len)` | String × Int × Int → String | Extract substring |
| `replace(s, old, new)` | String × String × String → String | Replace occurrences |

## 3.3 Int

64-bit signed two's complement integer.

**Range:** −9,223,372,036,854,775,808 to 9,223,372,036,854,775,807

**Operations:** `=`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`, `*`, `/`, `%`, unary `-`

**Semantics:**
- Division truncates toward zero: `7 / 3 = 2`, `-7 / 3 = -2`
- Division by zero: runtime error
- Overflow: wraps (two's complement)

**Built-in functions:** `abs(n)`, `min(a, b)`, `max(a, b)`

## 3.4 Float

64-bit IEEE 754 double-precision floating point.

**Special values:** `±0.0`, `±∞`, `NaN`

**Operations:** Same as Int, plus:
- `NaN = NaN` → `false`
- `NaN != NaN` → `true`
- `x / 0.0` → `±∞`
- `0.0 / 0.0` → `NaN`

**Built-in functions:** `abs(f)`, `floor(f)`, `ceil(f)`, `round(f)`, `min(a, b)`, `max(a, b)`, `is_nan(f)`

**Coercion:** Int → Float is implicit. Float → Int requires explicit `floor`/`ceil`/`round`.

## 3.5 Bool

**Values:** `true`, `false`

**Operations:** `=`, `!=`, `and`, `or`, `not`

**Short-circuit evaluation:**
- `false and x` → `false` (x not evaluated)
- `true or x` → `true` (x not evaluated)

## 3.6 Timestamp

Milliseconds since Unix epoch (1970-01-01T00:00:00Z).

**Operations:**
- Comparison: `=`, `!=`, `<`, `>`, `<=`, `>=`
- Arithmetic: `t + n` (add ms), `t - n`, `t1 - t2` (difference in ms)

**Built-in functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `now()` | () → Timestamp | Current time |
| `year(t)` | Timestamp → Int | Extract year |
| `month(t)` | Timestamp → Int | Month (1-12) |
| `day(t)` | Timestamp → Int | Day (1-31) |
| `hour(t)` | Timestamp → Int | Hour (0-23) |
| `minute(t)` | Timestamp → Int | Minute (0-59) |
| `second(t)` | Timestamp → Int | Second (0-59) |

All operations assume UTC.

## 3.7 ID

Opaque identifier for nodes and edges.

**Properties:**
- Immutable, unique, opaque
- Comparable only for equality: `=`, `!=`
- No ordering

Every node and edge has an implicit `id` attribute.

## 3.8 Null

`null` represents absence of value.

**Valid only for optional types** (`T?`).

**Propagation:** Most operations with null return null:
- `null + 1` → `null`
- `length(null)` → `null`

**Comparison:**
- `null = null` → `true`
- `null = x` (non-null x) → `false`
- `null < x` → `false` (for any x)

**Boolean context:**
- `null and x` → `false`
- `null or x` → `x`

---

# 4. Type Expressions

## 4.1 Overview

Type expressions describe types of attributes, variables, and parameters.

```ebnf
TypeExpr = UnionType
UnionType = OptionalType ("|" OptionalType)*
OptionalType = PrimaryType "?"?
PrimaryType = ScalarType | NamedType | "(" TypeExpr ")"
```

## 4.2 Scalar Types

```ebnf
ScalarType = "String" | "Int" | "Float" | "Bool" | "Timestamp"
```

## 4.3 Named Types

```ebnf
NamedType = QualifiedIdentifier
```

Reference to a declared node type:

```
Event
Physics::Event
```

**Resolution order:**
1. Current ontology's declared types
2. Layer 0 types (if permitted)

## 4.4 Optional Types

```ebnf
OptionalType = PrimaryType "?"
```

Allows `null` in addition to base type values:

```
String?                 -- String or null
Event?                  -- Event or null
(Task | Project)?       -- union, then optional
```

## 4.5 Union Types

```ebnf
UnionType = OptionalType ("|" OptionalType)*
```

Accepts values of any constituent type:

```
Task | Project
String | Int
```

**Precedence:** Union binds looser than optional:
```
Task | Project?         -- Task | (Project?)
(Task | Project)?       -- (Task | Project) or null
```

## 4.7 Type Compatibility

### 4.7.1 Subtyping Rules

```
T <: T                                    -- reflexive
T <: U, U <: V  ⟹  T <: V                -- transitive
Child <: Parent  (if inheritance)         -- inheritance
T <: T | U                                -- union left
U <: T | U                                -- union right
T <: T?                                   -- optional
null <: T?                                -- null in optional
```

### 4.7.2 Assignment Compatibility

Value of type S assignable to location of type T iff S <: T.

---

# 5. Expressions

## 5.1 Overview

Expressions compute values. They appear in:
- Constraint conditions
- WHERE clauses
- RETURN clauses
- Attribute assignments

```ebnf
Expr = OrExpr
OrExpr = AndExpr ("or" AndExpr)*
AndExpr = EqualityExpr ("and" EqualityExpr)*
EqualityExpr = CompareExpr (("=" | "!=") CompareExpr)*
CompareExpr = AddExpr (("<" | ">" | "<=" | ">=") AddExpr)*
AddExpr = MulExpr (("+" | "-" | "++") MulExpr)*
MulExpr = UnaryExpr (("*" | "/" | "%") UnaryExpr)*
UnaryExpr = ("-" | "not")? PostfixExpr
PostfixExpr = PrimaryExpr ("." Identifier)*
PrimaryExpr = Literal | Identifier | CallExpr | "(" Expr ")"
CallExpr = Identifier "(" (Expr ("," Expr)*)? ")"
```

## 5.2 Literal Expressions

Constant values:

```
42                      -- Int
3.14                    -- Float
"hello"                 -- String
true                    -- Bool
null                    -- Null
```

## 5.3 Variable References

```ebnf
VarRefExpr = Identifier
```

Retrieves value bound to a variable:

```
MATCH e: Event
WHERE e.timestamp > 1000
```

## 5.4 Attribute Access

```ebnf
AttrAccessExpr = Expr "." Identifier
```

Retrieves an attribute value:

```
e.timestamp
e.id                    -- built-in
x.y.z                   -- chained
```

**Built-in attributes** (all nodes and edges):

| Attribute | Type | Description |
|-----------|------|-------------|
| `id` | ID | Unique identifier |
| `_type` | String | Type name |

**Union type access:** When base has type `A | B`:
- Attribute on all members with compatible types → valid
- Attribute on some members only → compile error
- Different types → result is union

## 5.5 Binary Expressions

```ebnf
BinaryExpr = Expr BinaryOp Expr
```

### 5.5.1 Comparison Operators

| Operator | Valid For | Result |
|----------|-----------|--------|
| `=`, `!=` | Compatible types | Bool |
| `<`, `>`, `<=`, `>=` | Int, Float, String, Timestamp | Bool |

**Equality type rules:**
- Same type: always valid (`Int = Int`, `String = String`, etc.)
- Numeric coercion: `Int` and `Float` are compatible (`3 = 3.0` → `true`)
- Null: `null = null` → `true`; `null = x` (non-null) → `false`
- Incompatible types: type error (`Int = String` → error)

### 5.5.2 Arithmetic Operators

| Operator | Valid For | Result |
|----------|-----------|--------|
| `+`, `-` | Int, Float | Same type |
| `+`, `-` | Timestamp ± Int | Timestamp |
| `-` | Timestamp − Timestamp | Int |
| `*`, `/`, `%` | Int, Float | Same type |

### 5.5.3 String Concatenation

| Operator | Valid For | Result |
|----------|-----------|--------|
| `++` | String | String |

### 5.5.4 Logical Operators

| Operator | Valid For | Result |
|----------|-----------|--------|
| `and`, `or` | Bool | Bool |

Short-circuit evaluation applies.

## 5.6 Unary Expressions

```ebnf
UnaryExpr = ("-" | "not") Expr
```

| Operator | Valid For | Result |
|----------|-----------|--------|
| `-` | Int, Float | Same type |
| `not` | Bool | Bool |

## 5.7 Call Expressions

```ebnf
CallExpr = Identifier "(" (Expr ("," Expr)*)? ")"
```

Invokes a built-in function:

```
length(name)
now()
min(a, b)
```

**General functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `type_of(x)` | any → String | Runtime type name |

See §3 for type-specific functions.

## 5.8 Parenthesized Expressions

```ebnf
ParenExpr = "(" Expr ")"
```

Overrides operator precedence.

## 5.9 Type Inference

| Expression | Type |
|------------|------|
| `42` | Int |
| `3.14` | Float |
| `"text"` | String |
| `true`, `false` | Bool |
| `null` | Null (compatible with T?) |
| Variable `x` | Declared type |
| `x.attr` | Type of attr |
| `a + b` | Common numeric type |
| `a = b` | Bool |
| `a and b` | Bool |
| `f(args)` | Return type of f |

**Type errors** are reported at compile time.

---

# 6. Design Principles

## 6.1 No Conditional Expressions

**This language intentionally omits IF/THEN/ELSE and CASE/WHEN expressions.**

**Rationale — GPU/parallel execution:**

Conditional expressions cause thread divergence in SIMD execution, degrading performance.

**Alternative:** Use constraints with disjoint patterns, or handle in application code.

## 6.2 Null Handling

The `null_handling` feature module provides:
- `COALESCE(x, y, ...)`
- `??` operator
- `IS NULL` / `IS NOT NULL`

These are retained because null-handling has predictable branching patterns.

---

# 7. Grammar Summary

```ebnf
(* Lexical *)
Whitespace       = (" " | "\t" | "\n" | "\r")+
LineComment      = "--" ~[\n\r]*
BlockComment     = "/*" (~[*] | "*" ~[/])* "*/"
DocComment       = "---" ~[\n\r]*

Identifier       = [a-zA-Z_][a-zA-Z0-9_]*
QualifiedIdent   = Identifier ("::" Identifier)*

StringLiteral    = '"' (StringChar | EscapeSeq)* '"'
IntLiteral       = "-"? [0-9]+
FloatLiteral     = "-"? [0-9]+ "." [0-9]+ ([eE][+-]?[0-9]+)?
                 | "-"? [0-9]+ [eE][+-]?[0-9]+
BoolLiteral      = "true" | "false"
NullLiteral      = "null"
Literal          = StringLiteral | IntLiteral | FloatLiteral | BoolLiteral | NullLiteral

(* Type Expressions *)
TypeExpr         = UnionType
UnionType        = OptionalType ("|" OptionalType)*
OptionalType     = PrimaryType "?"?
PrimaryType      = ScalarType | NamedType | "(" TypeExpr ")"
ScalarType       = "String" | "Int" | "Float" | "Bool" | "Timestamp"
NamedType        = QualifiedIdent

(* Expressions *)
Expr             = OrExpr
OrExpr           = AndExpr ("or" AndExpr)*
AndExpr          = EqualityExpr ("and" EqualityExpr)*
EqualityExpr     = CompareExpr (("=" | "!=") CompareExpr)*
CompareExpr      = AddExpr (("<" | ">" | "<=" | ">=") AddExpr)*
AddExpr          = MulExpr (("+" | "-" | "++") MulExpr)*
MulExpr          = UnaryExpr (("*" | "/" | "%") UnaryExpr)*
UnaryExpr        = ("-" | "not")? PostfixExpr
PostfixExpr      = PrimaryExpr ("." Identifier)*
PrimaryExpr      = Literal | Identifier | CallExpr | "(" Expr ")"
CallExpr         = Identifier "(" (Expr ("," Expr)*)? ")"
```

---

*End of Part I: Foundations*
