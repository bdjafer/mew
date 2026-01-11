# HOHG Language Specification

## Part I: Foundations

**Version:** 1.0
**Status:** Draft
**Scope:** Shared constructs for Ontology DSL and HOHG Language

---

# 1. Introduction

## 1.1 Purpose

This specification defines the syntax and semantics of the HOHG language family:

- **Ontology DSL**: A declarative language for defining graph schemas (node types, edge types, constraints, rules). Ontology DSL files are compiled into Layer 0 structures.

- **HOHG Language**: A runtime language for operating on graphs (observation, transformation, administration, versioning). HOHG statements are interpreted against a running engine.

Both languages share common foundational constructs defined in this Part.

## 1.2 Conformance

An implementation conforms to this specification if it:

1. Accepts all valid programs as defined by the grammar
2. Rejects all invalid programs with appropriate errors
3. Executes valid programs with the semantics defined herein
4. Produces the AST structures defined for each construct

## 1.3 Notation

### 1.3.1 Grammar Notation

This specification uses a variant of Extended Backus-Naur Form (EBNF):

| Notation | Meaning |
|----------|---------|
| `rule = expr` | Rule definition |
| `"text"` | Literal text (case-sensitive) |
| `'text'` | Literal text (case-sensitive) |
| `A B` | Sequence: A followed by B |
| `A \| B` | Alternative: A or B |
| `A?` | Optional: zero or one A |
| `A*` | Repetition: zero or more A |
| `A+` | Repetition: one or more A |
| `(A B)` | Grouping |
| `[a-z]` | Character range |
| `~[x]` | Any character except x |
| `/* comment */` | Grammar comment |

### 1.3.2 Semantic Notation

- **MUST**: Absolute requirement
- **MUST NOT**: Absolute prohibition
- **SHOULD**: Recommended but not required
- **MAY**: Optional

### 1.3.3 Type Notation

| Notation | Meaning |
|----------|---------|
| `T` | A type named T |
| `T?` | Optional T (may be null) |
| `T[]` | Array of T |
| `T \| U` | Union: T or U |
| `{ f: T }` | Object with field f of type T |

---

# 2. Lexical Structure

## 2.1 Source Text

### 2.1.1 Character Set

Source text MUST be encoded as UTF-8.

```
SourceCharacter = /* any Unicode code point */
```

### 2.1.2 Line Terminators

```
LineTerminator = 
    "\n"              /* LF: Line Feed, U+000A */
  | "\r\n"            /* CRLF: Carriage Return + Line Feed */
  | "\r"              /* CR: Carriage Return, U+000D */
```

Line terminators are significant only within string literals and for line counting in error messages.

### 2.1.3 Input Elements

Source text is processed into a sequence of input elements:

```
InputElement = 
    Whitespace
  | Comment
  | Token
```

Whitespace and comments are discarded after lexing. Only tokens are passed to the parser.

---

## 2.2 Whitespace

```
Whitespace = WhitespaceChar+

WhitespaceChar = 
    " "               /* Space, U+0020 */
  | "\t"              /* Tab, U+0009 */
  | LineTerminator
```

Whitespace separates tokens but is otherwise ignored. Multiple whitespace characters are equivalent to one.

---

## 2.3 Comments

Two comment forms are supported:

### 2.3.1 Line Comments

```
LineComment = "--" ~[\n\r]* LineTerminator?
```

A line comment begins with `--` and extends to the end of the line.

```
-- This is a line comment
node Event {
  timestamp: Int  -- inline comment
}
```

### 2.3.2 Block Comments

```
BlockComment = "/*" BlockCommentContent "*/"

BlockCommentContent = (~[*] | "*" ~[/])*
```

A block comment begins with `/*` and ends with `*/`. Block comments do not nest.

```
/* This is a 
   block comment */

node Event {
  /* temporarily disabled
  priority: Int,
  */
  timestamp: Int
}
```

### 2.3.3 Documentation Comments

```
DocComment = "---" ~[\n\r]* LineTerminator?
```

Documentation comments begin with `---` (three dashes). They are preserved in the AST and may be attached to subsequent declarations.

```
--- Represents a point in time when something occurred.
--- Events can be connected by causal relationships.
node Event {
  --- Milliseconds since Unix epoch
  timestamp: Int
}
```

---

## 2.4 Tokens

```
Token = 
    Keyword
  | Identifier
  | Literal
  | Operator
  | Punctuation
```

---

## 2.5 Keywords

Keywords are reserved and cannot be used as identifiers.

### 2.5.0 Case Sensitivity Rules

**Keywords** are **case-insensitive**:
```
MATCH t: Task RETURN t    -- OK
match t: Task return t    -- OK (same as above)
Match T: Task Return T    -- OK (same as above)
```

**Identifiers** (type names, variable names, attribute names) are **case-sensitive**:
```
MATCH t: Task RETURN t    -- OK: 'Task' is a type
MATCH t: task RETURN t    -- ERROR: Type 'task' not found. Did you mean 'Task'?

MATCH t: Task RETURN t.Title  -- ERROR: Attribute 'Title' not found. Did you mean 'title'?
```

**Rationale:** Case-insensitive keywords improve usability (no need to remember casing). Case-sensitive identifiers prevent subtle bugs and match most programming languages.

### 2.5.1 Ontology DSL Keywords

```
OntologyKeyword =
    "ontology" | "node" | "edge" | "constraint" | "rule"
  | "abstract" | "sealed" | "required" | "unique"
  | "where" | "not" | "exists" | "and" | "or"
  | "true" | "false" | "null"
```

### 2.5.2 HOHG Language Keywords

```
HOHGKeyword =
    /* Observation */
    "match" | "walk" | "from" | "follow" | "return"
  | "as" | "collect" | "until" | "depth"
  | "inspect"
    
    /* Filtering & Ordering */
  | "where" | "order" | "by" | "asc" | "desc" | "limit" | "offset"
  | "distinct"
    
    /* Transformation */
  | "spawn" | "kill" | "link" | "unlink" | "set"
    
    /* Transaction */
  | "begin" | "commit" | "rollback"
    
    /* Schema */
  | "load" | "extend" | "show" | "types" | "edges" | "constraints" | "rules"
    
    /* Index */
  | "index" | "on" | "drop" | "indexes"
    
    /* Version */
  | "snapshot" | "checkout" | "diff" | "versions"
  | "branch" | "switch" | "merge"
    
    /* Debug */
  | "explain" | "profile"
    
    /* Context */
  | "in" | "app" | "scope" | "using"
    
    /* Logic */
  | "and" | "or" | "not" | "exists"
  | "true" | "false" | "null"
```

### 2.5.3 All Keywords (Combined)

```
Keyword = OntologyKeyword | HOHGKeyword
```

The complete list of reserved keywords:

```
abstract    and         app         as          asc
begin       branch      by          case        checkout
coalesce    collect     commit      constraint  depth
desc        diff        distinct    drop        edge
else        end         exists      explain     extend
false       follow      from        if          in
index       indexes     inspect     kill        limit
link        load        match       merge       node
not         null        offset      on          ontology
or          order       profile     required    return
rollback    rule        rules       scope       sealed
set         show        snapshot    spawn       switch
then        true        types       unique      unlink
until       using       versions    walk        when
where
```

Keywords are case-sensitive: `NODE` is not a keyword, `node` is.

---

## 2.6 Identifiers

```
Identifier = IdentifierStart IdentifierContinue*

IdentifierStart = 
    [a-z] | [A-Z] | "_"

IdentifierContinue = 
    IdentifierStart | [0-9]
```

### 2.6.1 Identifier Rules

1. Identifiers MUST begin with a letter or underscore
2. Identifiers MAY contain letters, digits, and underscores
3. Identifiers are case-sensitive: `Event` ≠ `event`
4. Identifiers MUST NOT be keywords
5. Identifiers starting with `_` are reserved for Layer 0 (see 2.6.2)

### 2.6.2 Reserved Identifier Prefixes

| Prefix | Reserved For |
|--------|--------------|
| `_` | Layer 0 types, edges, and attributes |
| `__` | Engine internal use |

User-defined identifiers MUST NOT begin with underscore.

### 2.6.3 Identifier Examples

```
/* Valid identifiers */
Event
myVariable
user_name
Person2
_NodeType        /* Layer 0 only */

/* Invalid identifiers */
2things          /* starts with digit */
my-name          /* contains hyphen */
node             /* keyword */
```

### 2.6.4 Qualified Identifiers

Qualified identifiers reference items in a namespace:

```
QualifiedIdentifier = Identifier ("::" Identifier)*
```

Examples:
```
Event                    /* simple */
Physics::Event           /* qualified with ontology */
TodoApp::Task            /* qualified with application */
```

---

## 2.7 Literals

```
Literal = 
    StringLiteral
  | IntLiteral
  | FloatLiteral
  | BoolLiteral
  | NullLiteral
  | TimestampLiteral
  | DurationLiteral
```

### 2.7.1 String Literals

```
StringLiteral = '"' StringCharacter* '"'

StringCharacter = 
    ~["\\\n\r]        /* any character except ", \, newline */
  | EscapeSequence

EscapeSequence =
    "\\" ["\\/bfnrt]  /* standard escapes */
  | "\\u" HexDigit HexDigit HexDigit HexDigit  /* Unicode escape */

HexDigit = [0-9a-fA-F]
```

#### Escape Sequences

| Sequence | Character | Unicode |
|----------|-----------|---------|
| `\"` | Double quote | U+0022 |
| `\\` | Backslash | U+005C |
| `\/` | Forward slash | U+002F |
| `\b` | Backspace | U+0008 |
| `\f` | Form feed | U+000C |
| `\n` | Line feed | U+000A |
| `\r` | Carriage return | U+000D |
| `\t` | Tab | U+0009 |
| `\uXXXX` | Unicode code point | U+XXXX |

#### String Examples

```
"hello"
"line1\nline2"
"quote: \"text\""
"unicode: \u00E9"      /* é */
""                     /* empty string */
```

#### Unicode in Strings

String literals support full Unicode:
1. **Direct inclusion:** `"Hello 😀"` (UTF-8 encoded)
2. **BMP escapes:** `\uXXXX` for U+0000 to U+FFFF

For characters above U+FFFF (emoji, rare scripts), include them directly in the string. The `\uXXXX` escape only handles the Basic Multilingual Plane.

### 2.7.2 Integer Literals

```
IntLiteral = "-"? DecimalDigits

DecimalDigits = [0-9]+
```

Integers are 64-bit signed values.

#### Range

| Bound | Value |
|-------|-------|
| Minimum | -9,223,372,036,854,775,808 |
| Maximum | 9,223,372,036,854,775,807 |

#### Integer Examples

```
0
42
-17
9223372036854775807
```

### 2.7.3 Float Literals

```
FloatLiteral = "-"? DecimalDigits "." DecimalDigits Exponent?
             | "-"? DecimalDigits Exponent

Exponent = [eE] [+-]? DecimalDigits
```

Floats are 64-bit IEEE 754 double-precision values.

#### Special Values

| Value | Representation |
|-------|----------------|
| Positive infinity | Not representable as literal; use expression |
| Negative infinity | Not representable as literal; use expression |
| NaN | Not representable as literal; use expression |

#### Float Examples

```
3.14
-0.5
1.0e10
2.5e-3
42.0
```

### 2.7.4 Boolean Literals

```
BoolLiteral = "true" | "false"
```

### 2.7.5 Null Literal

```
NullLiteral = "null"
```

Null represents the absence of a value for optional attributes or expressions.

---

## 2.8 Operators

```
Operator =
    /* Comparison */
    "=" | "!=" | "<" | ">" | "<=" | ">="
    
    /* Arithmetic */
  | "+" | "-" | "*" | "/" | "%"
    
    /* Logical */
  | "and" | "or" | "not"
    
    /* String */
  | "++"              /* concatenation */
    
    /* Access */
  | "."               /* attribute access */
    
    /* Type */
  | ":"               /* type annotation */
  | "|"               /* union type */
  | "?"               /* optional type */
    
    /* Edge */
  | "->"              /* edge direction (future) */
  | "<-"              /* edge direction (future) */
```

### 2.8.1 Operator Precedence

From highest to lowest precedence:

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (highest) | `.` | Left |
| 2 | unary `-`, `not` | Right |
| 3 | `*`, `/`, `%` | Left |
| 4 | `+`, `-`, `++` | Left |
| 5 | `<`, `>`, `<=`, `>=` | Left |
| 6 | `=`, `!=` | Left |
| 7 | `and` | Left |
| 8 (lowest) | `or` | Left |

Parentheses override precedence.

---

## 2.9 Punctuation

```
Punctuation =
    "("  | ")"        /* grouping, signatures */
  | "{"  | "}"        /* blocks, objects */
  | "["  | "]"        /* modifiers, lists */
  | ","               /* separator */
  | ";"               /* statement terminator (optional) */
  | ":"               /* type annotation */
  | "::"              /* namespace qualifier */
  | "=>"              /* implication, production */
  | "="               /* assignment, equality */
```

---

# 3. Scalar Types

## 3.1 Type Overview

HOHG has six scalar (primitive) types:

| Type | Description | Size |
|------|-------------|------|
| `String` | UTF-8 text | Unbounded |
| `Int` | Signed integer | 64 bits |
| `Float` | Floating point | 64 bits |
| `Bool` | Boolean | 1 bit |
| `Timestamp` | Unix epoch milliseconds | 64 bits |
| `ID` | Opaque identifier | Implementation-defined |

---

## 3.2 String

### 3.2.1 Definition

A `String` is a sequence of zero or more Unicode code points encoded as UTF-8.

### 3.2.2 Operations

| Operation | Syntax | Result Type | Description |
|-----------|--------|-------------|-------------|
| Equality | `a = b` | Bool | True if identical |
| Inequality | `a != b` | Bool | True if different |
| Less than | `a < b` | Bool | Lexicographic comparison |
| Greater than | `a > b` | Bool | Lexicographic comparison |
| Less or equal | `a <= b` | Bool | Lexicographic comparison |
| Greater or equal | `a >= b` | Bool | Lexicographic comparison |
| Concatenation | `a ++ b` | String | Concatenate strings |
| Length | `length(a)` | Int | Number of characters |
| Contains | `contains(a, b)` | Bool | True if a contains b |
| Starts with | `starts_with(a, b)` | Bool | True if a starts with b |
| Ends with | `ends_with(a, b)` | Bool | True if a ends with b |
| Substring | `substring(a, start, len)` | String | Extract substring |
| Lower case | `lower(a)` | String | Convert to lower case |
| Upper case | `upper(a)` | String | Convert to upper case |
| Trim | `trim(a)` | String | Remove leading/trailing whitespace |

### 3.2.3 Lexicographic Ordering

String comparison uses Unicode code point order:
- "a" < "b"
- "A" < "a" (uppercase before lowercase)
- "a" < "aa"
- "" < "a" (empty string is smallest)

---

## 3.3 Int

### 3.3.1 Definition

An `Int` is a 64-bit signed two's complement integer.

### 3.3.2 Range

```
Minimum: -9223372036854775808  (-2^63)
Maximum:  9223372036854775807  (2^63 - 1)
```

### 3.3.3 Operations

| Operation | Syntax | Result Type | Description |
|-----------|--------|-------------|-------------|
| Equality | `a = b` | Bool | True if equal |
| Inequality | `a != b` | Bool | True if not equal |
| Less than | `a < b` | Bool | True if a < b |
| Greater than | `a > b` | Bool | True if a > b |
| Less or equal | `a <= b` | Bool | True if a ≤ b |
| Greater or equal | `a >= b` | Bool | True if a ≥ b |
| Addition | `a + b` | Int | Sum |
| Subtraction | `a - b` | Int | Difference |
| Multiplication | `a * b` | Int | Product |
| Division | `a / b` | Int | Integer division (truncates toward zero) |
| Modulo | `a % b` | Int | Remainder |
| Negation | `-a` | Int | Arithmetic negation |
| Absolute | `abs(a)` | Int | Absolute value |
| Minimum | `min(a, b)` | Int | Smaller value |
| Maximum | `max(a, b)` | Int | Larger value |

### 3.3.4 Division Semantics

Integer division truncates toward zero:
- `7 / 3` = `2`
- `-7 / 3` = `-2`
- `7 / -3` = `-2`

Division by zero is a runtime error.

### 3.3.5 Overflow Behavior

Integer overflow wraps (two's complement behavior):
- `9223372036854775807 + 1` = `-9223372036854775808`

Implementations MAY provide overflow checking as an option.

---

## 3.4 Float

### 3.4.1 Definition

A `Float` is a 64-bit IEEE 754 double-precision floating-point number.

### 3.4.2 Special Values

| Value | Description |
|-------|-------------|
| `+0.0`, `-0.0` | Positive and negative zero (compare equal) |
| `+∞`, `-∞` | Positive and negative infinity |
| `NaN` | Not a Number |

### 3.4.3 Operations

| Operation | Syntax | Result Type | Description |
|-----------|--------|-------------|-------------|
| Equality | `a = b` | Bool | True if equal (NaN ≠ NaN) |
| Inequality | `a != b` | Bool | True if not equal |
| Less than | `a < b` | Bool | True if a < b |
| Greater than | `a > b` | Bool | True if a > b |
| Less or equal | `a <= b` | Bool | True if a ≤ b |
| Greater or equal | `a >= b` | Bool | True if a ≥ b |
| Addition | `a + b` | Float | Sum |
| Subtraction | `a - b` | Float | Difference |
| Multiplication | `a * b` | Float | Product |
| Division | `a / b` | Float | Quotient |
| Negation | `-a` | Float | Arithmetic negation |
| Absolute | `abs(a)` | Float | Absolute value |
| Floor | `floor(a)` | Int | Round toward negative infinity |
| Ceiling | `ceil(a)` | Int | Round toward positive infinity |
| Round | `round(a)` | Int | Round to nearest integer |
| Minimum | `min(a, b)` | Float | Smaller value |
| Maximum | `max(a, b)` | Float | Larger value |

### 3.4.4 NaN Behavior

NaN (Not a Number) has special comparison behavior:
- `NaN = NaN` → `false`
- `NaN != NaN` → `true`
- `NaN < x` → `false` for any x
- `NaN > x` → `false` for any x

To test for NaN, use `is_nan(x)`.

### 3.4.5 Type Coercion

Int may be implicitly converted to Float when required:
- In mixed arithmetic: `1 + 2.0` → `3.0`
- In Float-typed contexts

Float is NOT implicitly converted to Int. Use explicit conversion: `floor(x)`, `ceil(x)`, or `round(x)`.

### 3.4.6 Division by Zero

Float division by zero follows IEEE 754:
- `x / 0.0` where x > 0 → `+∞`
- `x / 0.0` where x < 0 → `-∞`
- `0.0 / 0.0` → `NaN`

Integer division by zero is a runtime error (see Section 3.3.4).

---

## 3.5 Bool

### 3.5.1 Definition

A `Bool` has exactly two values: `true` and `false`.

### 3.5.2 Operations

| Operation | Syntax | Result Type | Description |
|-----------|--------|-------------|-------------|
| Equality | `a = b` | Bool | True if same value |
| Inequality | `a != b` | Bool | True if different values |
| Logical AND | `a and b` | Bool | True if both true |
| Logical OR | `a or b` | Bool | True if either true |
| Logical NOT | `not a` | Bool | Logical negation |

### 3.5.3 Short-Circuit Evaluation

Logical operators use short-circuit evaluation:

- `false and x` → `false` (x not evaluated)
- `true or x` → `true` (x not evaluated)

This is significant when x has side effects or when x would error.

### 3.5.4 Truthiness

Boolean contexts (WHERE clauses, conditions) accept:
1. `Bool` expressions: must be `true` or `false`
2. `Bool?` expressions: `null` is treated as `false`

There is no implicit conversion from non-boolean types:

```
/* INVALID */
WHERE x.count        /* Int is not Bool */

/* VALID */
WHERE x.count > 0    /* comparison produces Bool */
WHERE x.active       /* Bool?, null treated as false */
WHERE x.active = true  /* explicit: only matches true, not null */
```

---

## 3.6 Timestamp

### 3.6.1 Definition

A `Timestamp` represents a point in time as milliseconds since the Unix epoch (1970-01-01T00:00:00Z).

Internally, `Timestamp` is stored as an `Int` (64-bit signed integer).

### 3.6.2 Range

| Bound | Value | Approximate Date |
|-------|-------|------------------|
| Minimum | -9223372036854775808 | ~292 billion years ago |
| Maximum | 9223372036854775807 | ~292 billion years in future |

### 3.6.3 Operations

| Operation | Syntax | Result Type | Description |
|-----------|--------|-------------|-------------|
| Equality | `a = b` | Bool | True if same instant |
| Inequality | `a != b` | Bool | True if different |
| Less than | `a < b` | Bool | a is before b |
| Greater than | `a > b` | Bool | a is after b |
| Less or equal | `a <= b` | Bool | a is at or before b |
| Greater or equal | `a >= b` | Bool | a is at or after b |
| Add duration | `a + n` | Timestamp | Add n milliseconds |
| Subtract duration | `a - n` | Timestamp | Subtract n milliseconds |
| Difference | `a - b` | Int | Milliseconds between |
| Current time | `now()` | Timestamp | Current system time |

### 3.6.4 Timestamp Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `now()` | `() → Timestamp` | Current system time |
| `year(t)` | `Timestamp → Int` | Year (e.g., 2024) |
| `month(t)` | `Timestamp → Int` | Month (1-12) |
| `day(t)` | `Timestamp → Int` | Day of month (1-31) |
| `hour(t)` | `Timestamp → Int` | Hour (0-23) |
| `minute(t)` | `Timestamp → Int` | Minute (0-59) |
| `second(t)` | `Timestamp → Int` | Second (0-59) |
| `millisecond(t)` | `Timestamp → Int` | Millisecond (0-999) |
| `day_of_week(t)` | `Timestamp → Int` | Day of week (0=Sunday, 6=Saturday) |
| `timestamp(s)` | `String → Timestamp` | Parse ISO 8601 string |

### 3.6.5 Timestamp Literals

Timestamps can be written directly using the `@` prefix with ISO 8601 format:

```
TimestampLiteral = "@" ISODate
ISODate = Date ("T" Time Timezone?)?
Date = Year "-" Month "-" Day
Time = Hour ":" Minute (":" Second ("." Milliseconds)?)?
Timezone = "Z" | ("+" | "-") Hour ":" Minute
```

**Examples:**
```
@2024-01-15                     -- Date only (midnight UTC)
@2024-01-15T10:30:00Z           -- Full timestamp with UTC
@2024-01-15T10:30:00            -- Local time (converted to UTC)
@2024-01-15T10:30:00+05:30      -- With timezone offset
@2024-01-15T10:30:00.500Z       -- With milliseconds
```

**Usage in observations:**
```
-- Before (string parsing):
WHERE t.created_at > timestamp("2024-01-15T00:00:00Z")

-- After (literal):
WHERE t.created_at > @2024-01-15

-- Date range:
WHERE t.created_at >= @2024-01-01 AND t.created_at < @2024-02-01

-- Combined with Duration:
WHERE t.created_at > @2024-01-15 + 12.hours
```

**Note:** When only a date is provided (no time), midnight UTC is assumed. When no timezone is provided, UTC is assumed.

### 3.6.6 Timestamp Arithmetic

```
t + 1000          -- 1 second later
t - 86400000      -- 1 day earlier
t2 - t1           -- milliseconds between t1 and t2

-- With Duration literals (preferred):
t + 1.day         -- 1 day later
t - 30.minutes    -- 30 minutes earlier
```

### 3.6.7 Timezone

All Timestamp operations use UTC (Coordinated Universal Time):

- `now()` returns current UTC time
- `year(t)`, `month(t)`, `day(t)`, etc. extract UTC components
- No implicit timezone conversion

Timestamp values themselves are timezone-agnostic (milliseconds since epoch). The UTC specification applies to extraction functions and display.

To work with local time, applications should handle timezone conversion externally.

---

## 3.7 Duration

### 3.7.1 Definition

A `Duration` represents a span of time in milliseconds. Unlike `Timestamp` (a point in time), `Duration` is a relative quantity.

### 3.7.2 Duration Literals

```
DurationLiteral = IntLiteral "." DurationUnit

DurationUnit =
    "millisecond" | "milliseconds" | "ms"
  | "second" | "seconds" | "s"
  | "minute" | "minutes" | "min"
  | "hour" | "hours" | "h"
  | "day" | "days" | "d"
  | "week" | "weeks" | "w"
```

**Examples:**
```
1.millisecond       -- 1 ms
500.ms              -- 500 ms
1.second            -- 1000 ms
30.seconds          -- 30000 ms
5.s                 -- 5000 ms
1.minute            -- 60000 ms
15.min              -- 900000 ms
1.hour              -- 3600000 ms
2.h                 -- 7200000 ms
1.day               -- 86400000 ms
7.days              -- 604800000 ms
1.week              -- 604800000 ms
```

### 3.7.3 Duration Arithmetic

```
-- Duration + Duration = Duration
1.hour + 30.minutes            -- 5400000 ms (1.5 hours)

-- Duration - Duration = Duration
1.day - 1.hour                 -- 82800000 ms (23 hours)

-- Duration * Int = Duration
30.minutes * 4                 -- 7200000 ms (2 hours)

-- Duration / Int = Duration
1.hour / 2                     -- 1800000 ms (30 minutes)

-- Timestamp + Duration = Timestamp
now() + 1.day                  -- tomorrow
now() - 30.days                -- 30 days ago

-- Timestamp - Timestamp = Duration
t2 - t1                        -- duration between
```

### 3.7.4 Duration in Queries

Duration literals make time-based queries more readable:

```
-- Before (magic numbers):
WHERE t.created_at < now() - 86400000

-- After (clear intent):
WHERE t.created_at < now() - 1.day

-- Complex duration:
WHERE t.created_at > now() - 2.weeks - 3.days

-- Timeout:
TIMEOUT 30.seconds
```

### 3.7.5 Duration Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `to_milliseconds(d)` | `Duration → Int` | Convert to raw ms |
| `to_seconds(d)` | `Duration → Float` | Convert to seconds |
| `to_minutes(d)` | `Duration → Float` | Convert to minutes |
| `to_hours(d)` | `Duration → Float` | Convert to hours |
| `to_days(d)` | `Duration → Float` | Convert to days |

### 3.7.6 Duration Constants

| Constant | Value |
|----------|-------|
| `1.millisecond` | 1 |
| `1.second` | 1,000 |
| `1.minute` | 60,000 |
| `1.hour` | 3,600,000 |
| `1.day` | 86,400,000 |
| `1.week` | 604,800,000 |

---

## 3.8 ID

### 3.7.1 Definition

An `ID` is an opaque identifier for nodes and edges.

### 3.7.2 Properties

1. IDs are **immutable**: Once assigned, an ID never changes
2. IDs are **unique**: No two nodes/edges share an ID
3. IDs are **opaque**: Internal structure is not guaranteed
4. IDs are **comparable**: Only for equality

### 3.7.3 Operations

| Operation | Syntax | Result Type | Description |
|-----------|--------|-------------|-------------|
| Equality | `a = b` | Bool | True if same ID |
| Inequality | `a != b` | Bool | True if different IDs |

IDs do NOT support ordering (`<`, `>`, etc.).

### 3.7.4 ID Access

Every node and edge has an implicit `id` attribute:

```
MATCH e: Event
RETURN e.id
```

---

## 3.9 Null Handling

### 3.9.1 Null Semantics

`null` represents the absence of a value. It is valid only for optional attributes (`T?`).

### 3.9.2 Null Propagation

Operations involving null generally propagate null:

| Expression | Result |
|------------|--------|
| `null + 1` | `null` |
| `null * 5` | `null` |
| `null ++ "text"` | `null` |
| `length(null)` | `null` |

### 3.9.3 Null Comparison

| Expression | Result |
|------------|--------|
| `null = null` | `true` |
| `null != null` | `false` |
| `x = null` (x not null) | `false` |
| `x != null` (x not null) | `true` |
| `null < x` | `false` |
| `null > x` | `false` |
| `null <= x` | `false` |
| `null >= x` | `false` |

### 3.9.4 Null in Logical Operations

When `null` appears in logical operations (typically via `Bool?` values):

| Expression | Result | Explanation |
|------------|--------|-------------|
| `null and x` | `false` | null is falsy, short-circuits |
| `x and null` | `false` | if x is true, null is falsy → false; if x is false, short-circuits |
| `null or x` | `x` | null is falsy, evaluates second operand |
| `x or null` | `x` if x true, else `null` | short-circuit or return second |
| `not null` | `true` | null is falsy |

### 3.9.5 Null Testing

```
x = null          -- true if x is null
x != null         -- true if x is not null
is_null(x)        -- explicit null test (same as x = null)
coalesce(x, y)    -- returns x if not null, else y
```

---

# 4. Type Expressions

## 4.1 Overview

Type expressions describe the types of attributes, variables, and edge targets.

```
TypeExpr = 
    NamedType
  | OptionalType
  | UnionType
  | EdgeRefType
  | AnyType
  | ScalarType
```

---

## 4.2 Named Types

```
NamedType = QualifiedIdentifier
```

A named type references a declared node type.

```
Event                   -- simple reference
Physics::Event          -- qualified reference
```

### 4.2.1 Resolution

Named types are resolved in order:
1. Current ontology's declared types
2. Inherited ontology types
3. Layer 0 types (if not user-restricted)

### 4.2.2 AST

```typescript
interface NamedTypeExpr {
  kind: "NamedType"
  name: string
  qualifier: string | null    // namespace qualifier
}
```

---

## 4.3 Optional Types

```
OptionalType = TypeExpr "?"
```

An optional type allows `null` in addition to values of the base type.

```
String?                 -- String or null
Event?                  -- Event or null
(Task | Project)?       -- Task, Project, or null
```

### 4.3.1 Semantics

- Required attribute (`T`): MUST have a value
- Optional attribute (`T?`): MAY be null

### 4.3.2 AST

```typescript
interface OptionalTypeExpr {
  kind: "OptionalType"
  inner: TypeExpr
}
```

---

## 4.4 Union Types

```
UnionType = TypeExpr "|" TypeExpr
```

A union type accepts values of either constituent type.

```
Task | Project          -- Task or Project
String | Int            -- String or Int (mixed scalar)
Event | State | Agent   -- multiple alternatives
```

### 4.4.1 Union Semantics

A value of type `A | B` can be:
- Any value of type A, or
- Any value of type B

### 4.4.2 Precedence

Union binds looser than optional:
```
Task | Project?        -- means: Task | (Project?)
(Task | Project)?      -- means: (Task | Project) or null
```

### 4.4.3 AST

```typescript
interface UnionTypeExpr {
  kind: "UnionType"
  left: TypeExpr
  right: TypeExpr
}
```

---

## 4.5 Edge Reference Types

```
EdgeRefType = "edge" "<" (QualifiedIdentifier | "any") ">"
```

An edge reference type indicates that a value is an edge (not a node).

```
edge<causes>            -- reference to a 'causes' edge
edge<any>               -- reference to any edge type
```

### 4.5.1 Usage

Edge reference types are used in higher-order edge signatures:

```
edge confidence(
  about: edge<causes>,   -- targets a 'causes' edge
  level: Float
) {}

edge provenance(
  about: edge<any>,      -- targets any edge
  source: String
) {}
```

### 4.5.2 AST

```typescript
interface EdgeRefTypeExpr {
  kind: "EdgeRefType"
  edgeType: string | null    // null means 'any'
}
```

---

## 4.6 Any Type

```
AnyType = "any"
```

The `any` type matches any node type. Used in polymorphic signatures.

```
edge tagged(
  entity: any,           -- any node
  tag: Tag
) {}
```

### 4.6.1 Constraints

- `any` can only appear in edge signatures
- `any` cannot be used as an attribute type
- `any` matches nodes only, not edges (use `edge<any>` for edges)

### 4.6.2 AST

```typescript
interface AnyTypeExpr {
  kind: "AnyType"
}
```

---

## 4.7 Scalar Types

```
ScalarType = "String" | "Int" | "Float" | "Bool" | "Timestamp" | "ID"
```

Scalar types are built-in primitive types used for attributes.

### 4.7.1 AST

```typescript
interface ScalarTypeExpr {
  kind: "ScalarType"
  scalarType: "String" | "Int" | "Float" | "Bool" | "Timestamp" | "ID"
}
```

---

## 4.8 Type Compatibility

### 4.8.1 Subtyping Rules

```
T <: T                                    -- reflexive
T <: U, U <: V  =>  T <: V               -- transitive
Child <: Parent  (if inheritance)         -- inheritance
T <: T | U                                -- union left
U <: T | U                                -- union right
T <: T?                                   -- optional
null <: T?                                -- null in optional
```

### 4.8.2 Type Checking

When assigning value of type S to location of type T:
- If S <: T, assignment is valid
- Otherwise, type error

### 4.8.3 Examples

```
/* Valid */
event: Event           -- Event <: Event
event: Event?          -- Event <: Event?
null: Event?           -- null <: Event?
task: Task | Project   -- Task <: Task | Project

/* Invalid */
event: Event = null    -- null not <: Event (not optional)
task: Task = project   -- Project not <: Task (no inheritance)
```

---

# 5. Expressions

## 5.1 Overview

Expressions compute values. They appear in:
- Constraint conditions
- Rule conditions and productions
- WHERE clauses
- RETURN clauses
- Attribute assignments

```
Expr =
    LiteralExpr
  | VarRefExpr
  | AttrAccessExpr
  | BinaryExpr
  | UnaryExpr
  | CallExpr
  | ExistsExpr
  | ParenExpr
```

---

## 5.2 Literal Expressions

```
LiteralExpr = Literal
```

Literal expressions produce constant values.

```
42                      -- Int
3.14                    -- Float
"hello"                 -- String
true                    -- Bool
null                    -- null
```

### 5.2.1 AST

```typescript
interface LiteralExpr {
  kind: "Literal"
  valueType: "String" | "Int" | "Float" | "Bool" | "Null"
  value: string | number | boolean | null
}
```

---

## 5.3 Variable Reference Expressions

```
VarRefExpr = Identifier
```

Variable references retrieve the value bound to a variable.

```
MATCH e: Event
WHERE e.timestamp > 1000    -- 'e' is a variable reference
RETURN e
```

### 5.3.1 Scope

Variables must be declared before use. Variables are in scope:
- In the pattern that declares them
- In WHERE clauses following the pattern
- In RETURN clauses following the pattern
- In production actions (for rule patterns)

### 5.3.2 AST

```typescript
interface VarRefExpr {
  kind: "VarRef"
  name: string
}
```

---

## 5.4 Attribute Access Expressions

```
AttrAccessExpr = Expr "." Identifier
```

Attribute access retrieves an attribute value from a node or edge.

```
e.timestamp             -- timestamp attribute of e
e.id                    -- built-in id attribute
x.y.z                   -- chained access
```

### 5.4.1 Built-in Attributes

All nodes and edges have implicit attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `id` | ID | Unique identifier |
| `_type` | String | Type name |

### 5.4.2 Type Checking

The attribute must exist on the type of the base expression:
- Look up attribute on declared type
- Include inherited attributes
- Error if attribute not found

### 5.4.3 AST

```typescript
interface AttrAccessExpr {
  kind: "AttrAccess"
  base: Expr
  attribute: string
}
```

### 5.4.4 Union Type Attribute Access

When accessing an attribute on a union type `A | B`:

**Common attribute:** If the attribute exists on ALL types in the union with compatible types, access is valid:

```
-- Both Task and Project have 'name: String'
x: Task | Project
x.name              -- Valid, type is String
```

**Disjoint attribute:** If the attribute exists on only SOME types, access is a compile-time error:

```
-- Task has 'priority', Project does not
x: Task | Project
x.priority          -- ERROR: 'priority' not found on Project
```

**Different types:** If the attribute exists on all types but with different types, the result is a union:

```
-- Task.meta: String, Project.meta: Int
x: Task | Project
x.meta              -- Type is String | Int
```

---

## 5.5 Binary Expressions

```
BinaryExpr = Expr BinaryOp Expr

BinaryOp =
    "=" | "!=" | "<" | ">" | "<=" | ">="    -- comparison
  | "+" | "-" | "*" | "/" | "%"              -- arithmetic
  | "++"                                      -- string concat
  | "and" | "or"                              -- logical
```

### 5.5.1 Comparison Operators

| Operator | Name | Valid For |
|----------|------|-----------|
| `=` | Equals | All types |
| `!=` | Not equals | All types |
| `<` | Less than | Int, Float, String, Timestamp |
| `>` | Greater than | Int, Float, String, Timestamp |
| `<=` | Less or equal | Int, Float, String, Timestamp |
| `>=` | Greater or equal | Int, Float, String, Timestamp |

### 5.5.2 Arithmetic Operators

| Operator | Name | Valid For | Result |
|----------|------|-----------|--------|
| `+` | Add | Int, Float, Timestamp+Int | Same |
| `-` | Subtract | Int, Float, Timestamp-Int, Timestamp-Timestamp | Same/Int |
| `*` | Multiply | Int, Float | Same |
| `/` | Divide | Int, Float | Same |
| `%` | Modulo | Int | Int |

### 5.5.3 String Operator

| Operator | Name | Valid For | Result |
|----------|------|-----------|--------|
| `++` | Concatenate | String | String |

### 5.5.4 Logical Operators

| Operator | Name | Valid For | Result |
|----------|------|-----------|--------|
| `and` | Logical AND | Bool | Bool |
| `or` | Logical OR | Bool | Bool |

### 5.5.5 AST

```typescript
interface BinaryExpr {
  kind: "Binary"
  operator: string
  left: Expr
  right: Expr
}
```

---

## 5.6 Unary Expressions

```
UnaryExpr = UnaryOp Expr

UnaryOp = "-" | "not"
```

| Operator | Name | Valid For | Result |
|----------|------|-----------|--------|
| `-` | Negate | Int, Float | Same |
| `not` | Logical NOT | Bool | Bool |

### 5.6.1 AST

```typescript
interface UnaryExpr {
  kind: "Unary"
  operator: "-" | "not"
  operand: Expr
}
```

---

## 5.7 Call Expressions

```
CallExpr = Identifier "(" (Expr ("," Expr)*)? ")"
```

Call expressions invoke built-in functions.

```
length(name)
coalesce(x, "default")
now()
min(a, b)
```

### 5.7.1 Built-in Functions

#### String Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `length(s)` | `String → Int` | Character count |
| `lower(s)` | `String → String` | Lower case |
| `upper(s)` | `String → String` | Upper case |
| `trim(s)` | `String → String` | Remove whitespace |
| `contains(s, sub)` | `String × String → Bool` | Substring test |
| `starts_with(s, pre)` | `String × String → Bool` | Prefix test |
| `ends_with(s, suf)` | `String × String → Bool` | Suffix test |
| `substring(s, start, len)` | `String × Int × Int → String` | Extract substring |
| `replace(s, old, new)` | `String × String × String → String` | Replace occurrences |
| `split(s, delim)` | `String × String → String[]` | Split into array |

#### Numeric Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs(n)` | `Int → Int` or `Float → Float` | Absolute value |
| `min(a, b)` | `Int × Int → Int` or `Float × Float → Float` | Minimum |
| `max(a, b)` | `Int × Int → Int` or `Float × Float → Float` | Maximum |
| `floor(f)` | `Float → Int` | Round down |
| `ceil(f)` | `Float → Int` | Round up |
| `round(f)` | `Float → Int` | Round to nearest |

#### Timestamp Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `now()` | `() → Timestamp` | Current time |
| `year(t)` | `Timestamp → Int` | Extract year |
| `month(t)` | `Timestamp → Int` | Extract month (1-12) |
| `day(t)` | `Timestamp → Int` | Extract day (1-31) |
| `hour(t)` | `Timestamp → Int` | Extract hour (0-23) |
| `minute(t)` | `Timestamp → Int` | Extract minute (0-59) |
| `second(t)` | `Timestamp → Int` | Extract second (0-59) |

#### General Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `coalesce(a, b)` | `T? × T → T` | First non-null value |
| `is_null(x)` | `T? → Bool` | Null test |
| `type_of(x)` | `any → String` | Runtime type name |

### 5.7.2 AST

```typescript
interface CallExpr {
  kind: "Call"
  function: string
  arguments: Expr[]
}
```

---

## 5.8 Exists Expressions

```
ExistsExpr = "exists" "(" Pattern ")"
           | "not" "exists" "(" Pattern ")"
```

Exists expressions test for the existence of a pattern.

```
-- True if task has any assignment
exists(p: Person, assigned_to(task, p))

-- True if task has no assignment
not exists(assigned_to(task, _))
```

### 5.8.1 Variable Scoping

Variables from enclosing scope are visible inside EXISTS:

```
MATCH t: Task
WHERE exists(p: Person, assigned_to(t, p))
--                                   ^ 't' from outer scope
RETURN t
```

Variables declared inside EXISTS are NOT visible outside:

```
MATCH t: Task
WHERE exists(p: Person, assigned_to(t, p))
RETURN p    -- ERROR: 'p' not in scope
```

**Shadowing is forbidden:** Variables inside EXISTS cannot reuse names from the enclosing scope:

```
MATCH t: Task
WHERE exists(t: Project, ...)  -- ERROR: 't' already declared
```

Variable names must be unique across the entire pattern including nested EXISTS.

### 5.8.2 AST

```typescript
interface ExistsExpr {
  kind: "Exists"
  negated: boolean
  pattern: Pattern
}
```

---

## 5.9 Parenthesized Expressions

```
ParenExpr = "(" Expr ")"
```

Parentheses override operator precedence.

```
(a + b) * c
not (x and y)
```

---

## 5.10 Conditional Expressions

### 5.10.1 IF Expression

The IF expression provides inline conditional logic:

```
IfExpr = "if" Expr "then" Expr "else" Expr

-- Syntax:
IF condition THEN value_if_true ELSE value_if_false
```

**Examples:**
```
-- Simple conditional
IF t.priority > 5 THEN "high" ELSE "normal"

-- Nested conditionals
IF t.priority >= 8 THEN "critical"
ELSE IF t.priority >= 5 THEN "high"
ELSE "normal"

-- In RETURN
MATCH t: Task
RETURN t.title, IF t.completed THEN "done" ELSE "pending" AS status

-- In SET
SET #task_123.label = IF #task_123.priority > 5 THEN "urgent" ELSE "normal"
```

**Type rules:**
- Condition must be Bool or Bool?
- Both branches must have compatible types
- Result type is the common type of both branches

```
-- Valid: both branches are String
IF x > 0 THEN "positive" ELSE "non-positive"

-- Valid: branches are Int and Int
IF x > 0 THEN x ELSE 0

-- Invalid: branches have incompatible types
IF x > 0 THEN "text" ELSE 42  -- ERROR: String vs Int
```

### 5.10.2 CASE Expression

The CASE expression provides multi-way conditional logic:

```
CaseExpr = 
    "case" Expr WhenClause+ ElseClause?
  | "case" WhenClause+ ElseClause?

WhenClause = "when" Expr "then" Expr
ElseClause = "else" Expr
```

**Simple CASE (value matching):**
```
CASE t.status
  WHEN "todo" THEN 0
  WHEN "in_progress" THEN 1
  WHEN "done" THEN 2
  ELSE -1
END
```

**Searched CASE (condition matching):**
```
CASE
  WHEN t.priority >= 8 THEN "critical"
  WHEN t.priority >= 5 THEN "high"
  WHEN t.priority >= 3 THEN "medium"
  ELSE "low"
END
```

**Examples:**
```
-- In RETURN
MATCH t: Task
RETURN t.title,
       CASE t.priority
         WHEN 10 THEN "🔴 Critical"
         WHEN 5..9 THEN "🟡 High"
         ELSE "🟢 Normal"
       END AS priority_label

-- In WHERE
MATCH t: Task
WHERE CASE t.status
        WHEN "done" THEN t.completed_at < @2024-01-01
        ELSE true
      END
RETURN t

-- In SET
SET #task_123.category = CASE
  WHEN #task_123.priority >= 8 THEN "urgent"
  WHEN #task_123.due_date < now() THEN "overdue"
  ELSE "normal"
END
```

**Type rules:**
- All THEN branches must have compatible types
- ELSE branch (if present) must have compatible type
- If no ELSE, result is nullable (T?)

```
-- Valid: all branches are String
CASE x WHEN 1 THEN "one" WHEN 2 THEN "two" ELSE "other" END

-- Valid but nullable: no ELSE
CASE x WHEN 1 THEN "one" WHEN 2 THEN "two" END  -- Type: String?

-- Invalid: incompatible types
CASE x WHEN 1 THEN "one" WHEN 2 THEN 2 END  -- ERROR: String vs Int
```

### 5.10.3 COALESCE Expression

Returns the first non-null value:

```
COALESCE(expr1, expr2, ...)

-- Examples:
COALESCE(t.nickname, t.name, "Anonymous")
COALESCE(t.description, "")
```

**Shorthand operator `??`:**
```
t.nickname ?? t.name ?? "Anonymous"
-- Equivalent to COALESCE(t.nickname, t.name, "Anonymous")
```

---

## 5.11 Expression Type Rules

### 5.11.1 Type Inference

Each expression has a static type determined by its form:

| Expression | Type |
|------------|------|
| `42` | Int |
| `3.14` | Float |
| `"text"` | String |
| `true`, `false` | Bool |
| `null` | Null (compatible with T?) |
| `x` (variable) | Type of x |
| `x.attr` | Type of attr on type of x |
| `a + b` | Common numeric type |
| `a = b` | Bool |
| `a and b` | Bool |
| `f(args)` | Return type of f |
| `exists(...)` | Bool |

### 5.11.2 Type Checking Errors

Type errors are reported at compile time:

```
"text" + 42              -- ERROR: cannot add String and Int
x.nonexistent            -- ERROR: attribute 'nonexistent' not found
not "text"               -- ERROR: 'not' requires Bool
```

---

# 6. Summary

## 6.1 Part I Contents

This part defined the foundational constructs shared by both HOHG languages:

| Section | Contents |
|---------|----------|
| 1. Introduction | Purpose, conformance, notation |
| 2. Lexical Structure | Characters, whitespace, comments, tokens |
| 3. Scalar Types | String, Int, Float, Bool, Timestamp, ID |
| 4. Type Expressions | Named, optional, union, edge ref, any |
| 5. Expressions | Literals, variables, operators, functions |

## 6.2 What's Next

**Part II: Shared Constructs** — Patterns, the core construct for matching graph structure (used in both languages)

**Part III: Ontology DSL** — Declarations for types, edges, constraints, rules

**Part IV: HOHG Language** — Observation, transformation, administration, versioning

---

## 6.3 Grammar Summary (Part I)

```ebnf
(* Lexical *)
Whitespace       = (" " | "\t" | "\n" | "\r")+
LineComment      = "--" (~[\n\r])* 
BlockComment     = "/*" (~[*] | "*" ~[/])* "*/"
DocComment       = "---" (~[\n\r])*

Identifier       = [a-zA-Z_][a-zA-Z0-9_]*
QualifiedIdent   = Identifier ("::" Identifier)*

StringLiteral    = '"' (StringChar | EscapeSeq)* '"'
IntLiteral       = "-"? [0-9]+
FloatLiteral     = "-"? [0-9]+ "." [0-9]+ ([eE][+-]?[0-9]+)?
BoolLiteral      = "true" | "false"
NullLiteral      = "null"

(* Type Expressions *)
TypeExpr         = UnionType
UnionType        = OptionalType ("|" OptionalType)*
OptionalType     = PrimaryType "?"?
PrimaryType      = NamedType | EdgeRefType | AnyType | ScalarType | "(" TypeExpr ")"
NamedType        = QualifiedIdent
EdgeRefType      = "edge" "<" (QualifiedIdent | "any") ">"
AnyType          = "any"
ScalarType       = "String" | "Int" | "Float" | "Bool" | "Timestamp" | "ID"

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
PrimaryExpr      = Literal | Identifier | CallExpr | ExistsExpr | "(" Expr ")"
CallExpr         = Identifier "(" (Expr ("," Expr)*)? ")"
ExistsExpr       = "not"? "exists" "(" Pattern ")"
Literal          = StringLiteral | IntLiteral | FloatLiteral | BoolLiteral | NullLiteral
```

---

*End of Part I: Foundations*