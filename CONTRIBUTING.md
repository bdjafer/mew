# Contributing to MEW

Thank you for your interest in contributing to MEW!

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [just](https://github.com/casey/just) (recommended)
- Git

### Setup

```bash
git clone https://github.com/bdjafer/mew.git
cd mew
just setup    # Install rustfmt, clippy, cargo-watch
just build    # Verify build works
just test     # Run tests
```

## Development Workflow

### TDD-First Approach

This project uses Test-Driven Development:

1. **Specs first** — Check `specs/` for behavior specifications
2. **Tests exist** — Tests in `mew/tests/` and `examples/` define expected behavior
3. **Tests may fail** — Failing tests indicate unimplemented features, not bugs
4. **Implement to pass** — Write code to make tests pass

### Making Changes

1. **Create a branch**
   ```bash
   git checkout -b feature/description
   # or
   git checkout -b fix/description
   ```

2. **Make changes**
   - Follow existing code style
   - Keep changes focused
   - Add tests for new functionality

3. **Run checks**
   ```bash
   just ci    # Format, lint, build, test
   ```

4. **Commit**
   ```bash
   git commit -m "component: short description"
   ```

5. **Open PR**
   - Fill out the PR template
   - Link related issues

## Code Standards

### Formatting

```bash
just fmt       # Format code
just fmt-check # Verify formatting
```

### Linting

```bash
just lint      # Run clippy
```

All clippy warnings are treated as errors (`-D warnings`).

### Testing

```bash
just test          # Quick tests (unit + integration)
just test-all      # All tests including testgen
just test-pkg PKG  # Test specific package
just watch         # Watch mode
```

### Documentation

- Public APIs should have doc comments
- Complex logic should have inline comments explaining "why"
- Update relevant docs when changing behavior

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/) for automated changelog generation.

Format: `type(scope): description`

### Types

| Type | Description | Changelog Section |
|------|-------------|-------------------|
| `feat` | New feature | Features |
| `fix` | Bug fix | Bug Fixes |
| `perf` | Performance improvement | Performance |
| `refactor` | Code refactoring | Refactoring |
| `docs` | Documentation | Documentation |
| `test` | Tests | Tests |
| `ci` | CI/CD changes | CI/CD |
| `deps` | Dependency updates | Dependencies |
| `chore` | Maintenance (hidden from changelog) | — |

### Scopes (optional)

- `core`, `graph`, `parser`, `analyzer`, `compiler`, `registry`
- `pattern`, `query`, `mutation`, `constraint`, `rule`
- `transaction`, `journal`, `session`, `repl`
- `tests`, `testgen`, `examples`

### Examples

```
feat(parser): add support for edge attributes
fix(constraint): correct cardinality validation
perf(query): optimize pattern matching
docs: update REPL usage examples
ci: add coverage reporting
deps: update thiserror to 1.0.50
```

### Breaking Changes

For breaking changes, add `!` after the type or add `BREAKING CHANGE:` in the footer:

```
feat(parser)!: change AST node structure

BREAKING CHANGE: NodeType enum variants renamed
```

## Pull Request Checklist

- [ ] `just ci` passes locally
- [ ] Tests added/updated for changes
- [ ] Documentation updated if needed
- [ ] Commit messages follow format
- [ ] PR description explains the "why"

## Architecture Notes

### Crate Dependencies

```
core ← graph ← pattern ← query
  ↑      ↑        ↑        ↑
  └──────┴────────┴────────┤
                           ↓
parser → analyzer → compiler → registry
                           ↓
                    constraint ← rule
                           ↓
                    mutation ← transaction
                           ↓
                       session
                           ↓
                         repl
```

### Key Principles

1. **Graph is truth** — All data lives in the hypergraph
2. **Specs are authority** — Code follows specs, not vice versa
3. **Explicit over implicit** — Structure is visible, not hidden
4. **Safe by default** — Invalid states are unrepresentable

## Getting Help

- Check existing issues and discussions
- Read the specs in `specs/`
- Look at examples in `examples/`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
