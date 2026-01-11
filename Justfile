# Justfile for MEW
# Install: cargo install just

default:
    @just --list

# Build
build:
    cd mew && cargo build --workspace

# Test
test:
    cd mew && cargo test --workspace

test-pkg pkg:
    cd mew && cargo test -p {{pkg}}

# Code quality
fmt:
    cd mew && cargo fmt --all

lint:
    cd mew && cargo clippy --workspace --all-targets -- -D warnings

# All CI checks
ci: 
    cd mew && cargo fmt --all -- --check
    cd mew && cargo clippy --workspace --all-targets -- -D warnings
    cd mew && cargo test --workspace

# Docs
docs:
    cd mew && cargo doc --workspace --no-deps --open

# Dev
repl:
    cd mew && cargo run --bin mew-repl

watch:
    cd mew && cargo watch -x 'test --workspace'

# Coverage (requires cargo-llvm-cov)
coverage:
    cd mew && cargo llvm-cov --workspace --html --open
