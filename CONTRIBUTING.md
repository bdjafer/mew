# Contributing

## Setup

```bash
git clone https://github.com/bryandjafer/mew.git
cd mew
cargo install just
just build && just test
```

## Workflow

1. Create branch: `git checkout -b feature/name` or `fix/name`
2. Make changes
3. Run checks: `just ci`
4. Commit: `component: description`
5. Open PR

## Standards

- `cargo fmt --all` before commit
- `cargo clippy --workspace -- -D warnings` must pass
- All public APIs return `Result<T, E>`
- Tests follow Given-When-Then pattern

## Commit Format

```
component: short description

Fixes #123
```

## PR Checklist

- [ ] `just ci` passes
- [ ] Tests added for new functionality
- [ ] Documentation updated if needed
