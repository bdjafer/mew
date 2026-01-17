---
name: specs-explorer
description: Navigate, search, filter, and query MEW specifications by frontmatter metadata. Use when exploring specs, finding related specs, or understanding spec dependencies.
---

# Specs Explorer

Navigate and query the MEW specification corpus using the `specs` CLI.

## Setup

Run once per session:

```bash
source .claude/skills/specs-explorer/init.sh
```

Then use `specs <command>` freely.

## Quick Reference

```bash
specs help              # Full help
specs list              # All specs
specs count             # Count by directory
specs overview          # Table of all specs with metadata

specs cat statement     # Filter by category
specs status draft      # Filter by status
specs pri essential     # Filter by priority

specs deps <file>       # Show dependencies
specs rdeps <name>      # Reverse dependencies

specs show <file>       # Pretty-print spec summary
specs meta <file>       # Show frontmatter only

specs todo              # Essential + draft (needs work)
specs filter <cat> <st> # Combined filter
specs grep <term>       # Search content
```

## Commands

### Listing

| Command | Description |
|---------|-------------|
| `specs list` | List all spec files |
| `specs count` | Count specs by directory |
| `specs tree` | Show directory tree |
| `specs overview` | Table with all specs + metadata |

### Filter by Frontmatter

| Command | Description |
|---------|-------------|
| `specs cat <category>` | Filter by category |
| `specs status <status>` | Filter by status |
| `specs pri <priority>` | Filter by priority |

**Categories:** `literal` `type` `expression` `pattern` `declaration` `statement` `modifier` `execution`

**Statuses:** `draft` `stable` `deprecated`

**Priorities:** `essential` `common` `convenience` `specialized`

### Dependencies

| Command | Description |
|---------|-------------|
| `specs deps <file>` | What does this spec require? |
| `specs rdeps <name>` | What specs depend on this? |

### Inspect

| Command | Description |
|---------|-------------|
| `specs show <file>` | Pretty-print spec summary |
| `specs meta <file>` | Show frontmatter only |

### Combined Queries

| Command | Description |
|---------|-------------|
| `specs todo` | Essential + draft specs (need implementation) |
| `specs filter <cat> <status>` | Filter by category AND status |
| `specs dist [field]` | Show distribution (default: category) |

### Search

| Command | Description |
|---------|-------------|
| `specs search <term>` | Find specs containing term |
| `specs grep <term>` | Search with line context |
| `specs find <pattern>` | Find by filename pattern |

## Examples

```bash
# What statements exist?
specs cat statement

# What's still draft?
specs status draft

# What's essential and needs implementation?
specs todo

# What does MATCH require?
specs deps statements/match.md

# What uses patterns?
specs rdeps pattern

# Draft statements only
specs filter statement draft

# Search for SPAWN
specs grep SPAWN

# Show spec details
specs show statements/match.md
```

## Spec Locations

```
specs/
├── architecture.md              # System architecture
├── meta-roadmap.md              # Implementation roadmap
├── spec_template.md             # Template for new specs
├── core/                        # 3 files - 1_LANGUAGE, 2_LAYER0, 3_DSL
├── declarations/                # 7 files - ontology, node, edge, rule, constraint...
├── types/                       # 5 files - any, optional, union, duration, edge_references
├── literals/                    # 2 files - duration, timestamp
├── expressions/                 # 9 files - aggregations, functions, null_handling...
├── patterns/                    # 4 files - node, edge, negative, transitive
├── modifiers/                   # 15 files - required, unique, indexed, cardinality...
└── statements/                  # 18 files - match, spawn, kill, link...
```

**Total: 66 spec files across 9 directories**

## Frontmatter Schema

Every spec uses this YAML frontmatter:

```yaml
---
spec: kebab-case-name           # Unique identifier
version: "1.0"                  # Semantic version
status: draft | stable | deprecated
category: literal | type | expression | pattern | declaration | statement | modifier | execution
capability: what-it-enables     # Short description
requires: [dep1, dep2]          # Prerequisites
priority: essential | common | convenience | specialized
---
```

## Document Hierarchy

When specs conflict, higher level wins:

```
1. specs/core/1_LANGUAGE.md        # Highest authority
2. specs/core/2_LAYER0.md
3. specs/core/3_DSL.md
4. specs/architecture.md
5. specs/declarations/*.md
6. specs/types/*.md
7. specs/expressions/*.md
8. specs/patterns/*.md
9. specs/modifiers/*.md
10. specs/statements/*.md          # Lowest
```

## Workflows

### Understand a Feature

```bash
specs find match                    # 1. Find the spec
specs show statements/match.md      # 2. See its metadata
specs deps statements/match.md      # 3. What does it need?
specs rdeps match                   # 4. What depends on it?
```

### Plan Implementation Order

```bash
specs todo                          # Essential specs still draft
specs deps <each-spec>              # Check dependencies
# Implement specs with no unimplemented deps first
```

### Explore by Domain

```bash
specs cat statement                 # All query/mutation commands
specs cat modifier                  # All constraint modifiers
specs cat expression                # All expression features
specs cat pattern                   # All pattern matching
```

## Tips

- **Start with core**: Read `1_LANGUAGE.md` first
- **Follow requires**: Understand dependencies before the spec
- **Check hierarchy**: Higher-level spec wins on conflict
- **Focus on essential**: Implement `essential` before `convenience`
- **Use `specs todo`**: Shows what needs work
