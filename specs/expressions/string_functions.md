---
spec: string_functions
version: "1.0"
status: draft
category: expression
capability: string manipulation
requires: []
priority: common
---

# Spec: String Functions

## Overview

String functions provide operations for manipulating and inspecting string values. These functions are essential for data transformation, filtering, and text processing in queries and rules. They operate on `String` values and support common text operations like measuring length, changing case, searching for substrings, and extracting or replacing portions of text.

## Syntax

### Grammar

```ebnf
StringFunctionCall =
    "length" "(" Expr ")"
  | "lower" "(" Expr ")"
  | "upper" "(" Expr ")"
  | "trim" "(" Expr ")"
  | "contains" "(" Expr "," Expr ")"
  | "starts_with" "(" Expr "," Expr ")"
  | "ends_with" "(" Expr "," Expr ")"
  | "substring" "(" Expr "," Expr "," Expr ")"
  | "replace" "(" Expr "," Expr "," Expr ")"
  | "split" "(" Expr "," Expr ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `length` | Expression - returns character count |
| `lower` | Expression - case conversion |
| `upper` | Expression - case conversion |
| `trim` | Expression - whitespace removal |
| `contains` | Expression - substring test |
| `starts_with` | Expression - prefix test |
| `ends_with` | Expression - suffix test |
| `substring` | Expression - extraction |
| `replace` | Expression - substitution |
| `split` | Expression - tokenization |

### Examples

```
-- Get the length of a string
length(t.title)

-- Convert to lowercase for case-insensitive comparison
lower(p.email)

-- Check if title contains a keyword
contains(t.title, "urgent")

-- Extract first 10 characters
substring(t.description, 0, 10)
```

## Semantics

### Function Signatures and Behavior

| Function | Signature | Description |
|----------|-----------|-------------|
| `length(s)` | `String -> Int` | Returns the number of Unicode characters in the string |
| `lower(s)` | `String -> String` | Converts all characters to lowercase using Unicode rules |
| `upper(s)` | `String -> String` | Converts all characters to uppercase using Unicode rules |
| `trim(s)` | `String -> String` | Removes leading and trailing whitespace (spaces, tabs, newlines) |
| `contains(s, sub)` | `String x String -> Bool` | Returns true if `s` contains `sub` as a substring |
| `starts_with(s, prefix)` | `String x String -> Bool` | Returns true if `s` begins with `prefix` |
| `ends_with(s, suffix)` | `String x String -> Bool` | Returns true if `s` ends with `suffix` |
| `substring(s, start, len)` | `String x Int x Int -> String` | Extracts `len` characters starting at position `start` (0-indexed) |
| `replace(s, old, new)` | `String x String x String -> String` | Replaces all occurrences of `old` with `new` |
| `split(s, delim)` | `String x String -> String[]` | Splits `s` by `delim` and returns an array of strings |

### Type Rules

- All string functions require their string arguments to be of type `String` or `String?`
- If any argument is `null`, the function returns `null` (null propagation)
- The `length` function returns `Int` (or `Int?` if input is nullable)
- Boolean functions (`contains`, `starts_with`, `ends_with`) return `Bool` (or `Bool?` if input is nullable)
- The `split` function returns `String[]` (or `String[]?` if input is nullable)

### Edge Cases

**Empty strings:**
- `length("")` returns `0`
- `contains("", "")` returns `true`
- `contains("abc", "")` returns `true`
- `starts_with("", "")` returns `true`
- `ends_with("", "")` returns `true`
- `split("", ",")` returns `[""]`
- `trim("")` returns `""`

**substring bounds:**
- If `start` is negative, it is treated as `0`
- If `start` exceeds string length, returns empty string `""`
- If `start + len` exceeds string length, returns characters from `start` to end
- If `len` is negative, returns empty string `""`

**replace behavior:**
- Replaces all occurrences, not just the first
- If `old` is empty string, the behavior is undefined (implementation may error)
- If `old` is not found, returns the original string unchanged

**split behavior:**
- Empty delimiter (`""`) splits into individual characters
- If delimiter is not found, returns array with single element (original string)
- Consecutive delimiters produce empty strings in the result array

### Unicode Considerations

- `length` counts Unicode code points, not bytes
- `lower` and `upper` use Unicode case mapping rules
- `substring` indexes by code points, not bytes
- String comparison in `contains`, `starts_with`, `ends_with` is exact (byte-level)

## Layer 0

None.

## Examples

### Filtering by String Content

```
-- Find tasks with "urgent" in the title
MATCH t: Task
WHERE contains(lower(t.title), "urgent")
RETURN t

-- Find persons with email from a specific domain
MATCH p: Person
WHERE ends_with(lower(p.email), "@company.com")
RETURN p.name, p.email

-- Find tasks with long descriptions
MATCH t: Task
WHERE length(t.description) > 500
RETURN t.title, length(t.description) AS desc_length
```

### String Transformation

```
-- Normalize names to uppercase
MATCH p: Person
RETURN upper(p.name) AS normalized_name, p.email

-- Extract username from email
MATCH p: Person
RETURN p.name, substring(p.email, 0, length(p.email) - length("@company.com")) AS username

-- Clean up whitespace in titles
MATCH t: Task
SET t.title = trim(t.title)
```

### Search and Replace

```
-- Replace deprecated terminology
MATCH d: Document
WHERE contains(d.content, "old_term")
SET d.content = replace(d.content, "old_term", "new_term")

-- Check for file type by extension
MATCH f: File
WHERE ends_with(lower(f.name), ".pdf")
RETURN f.name, f.size
```

### Working with Delimited Data

```
-- Split comma-separated tags
MATCH t: Task
RETURN t.title, split(t.tags_csv, ",") AS tag_list

-- Find items with specific tag in CSV field
MATCH t: Task
WHERE contains(t.tags_csv, "priority")
RETURN t
```

## Errors

| Condition | Message |
|-----------|---------|
| Non-string argument to string function | Type error: `length` expects String, got Int |
| Non-integer start/length in substring | Type error: `substring` expects (String, Int, Int), got (String, String, Int) |
| Empty old string in replace | Runtime error: `replace` old string cannot be empty |
