---
spec: <name>
version: "1.0"
status: draft | stable | deprecated
category: literal | type | expression | pattern | declaration | statement | modifier | execution
capability: <capability>
requires: [<dependencies>]
priority: essential | common | convenience | specialized
---

# Spec: <Name>

## Overview

<One paragraph. What this enables. Why it's needed.>

## Syntax

### Grammar

\`\`\`ebnf
<Productions added or modified>
\`\`\`

### Keywords

| Keyword | Context |
|---------|---------|
| `x` | Statement / Expression / Modifier |

### Examples

\`\`\`
<2-4 short examples showing the syntax>
\`\`\`

## Semantics

<How it works. Type rules. Evaluation rules. Edge cases.>

## Layer 0

### Nodes

\`\`\`
<New node types, if any>
\`\`\`

### Edges

\`\`\`
<New edge types, if any>
\`\`\`

### Constraints

\`\`\`
<New constraints, if any>
\`\`\`

<Or simply: "None." if no Layer 0 additions>

## Examples

<Longer, realistic examples showing the feature in context>

## Errors

| Condition | Message |
| ----------|---------|
| <when>    |<message>|

<Or omit section if no feature-specific errors>