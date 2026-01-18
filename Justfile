# Justfile for MEW
# Install: cargo install just

set shell := ["bash", "-cu"]

default:
    @just --list

# ============================================================================
# Setup
# ============================================================================

# Install development dependencies
setup:
    @echo "Installing development tools..."
    rustup component add rustfmt clippy
    cargo install cargo-watch cargo-llvm-cov 2>/dev/null || true
    git config core.hooksPath .githooks
    @echo "✓ Setup complete"

# ============================================================================
# Build
# ============================================================================

# Build all crates
build:
    cd mew && cargo build --workspace

# Build in release mode
release:
    cd mew && cargo build --workspace --release

# ============================================================================
# Testing
# ============================================================================

# Run all tests (use test.sh for detailed output)
test:
    ./test.sh quick

# Run all tests including testgen
test-all:
    ./test.sh all

# Run unit tests only
test-unit:
    cd mew && cargo test --workspace --lib

# Run integration tests only
test-integration:
    cd mew && cargo test -p mew-tests --no-fail-fast

# Run tests for a specific package
test-pkg pkg:
    cd mew && cargo test -p {{pkg}}

# ============================================================================
# Code Quality
# ============================================================================

# Format all code
fmt:
    cd mew && cargo fmt --all

# Check formatting without modifying
fmt-check:
    cd mew && cargo fmt --all -- --check

# Run clippy linter
lint:
    cd mew && cargo clippy --workspace --all-targets -- -D warnings

# Run all CI checks locally
ci: fmt-check lint build test-unit

# ============================================================================
# Development
# ============================================================================

# Start the REPL
repl:
    cd mew && cargo run --bin mew

# Start REPL with a file (path relative to project root)
repl-file file:
    cd mew && cargo run --bin mew -- ../{{file}}

# Start REPL in verbose mode
repl-verbose:
    cd mew && cargo run --bin mew -- -v

# Watch for changes and run tests
watch:
    cd mew && cargo watch -x 'test --workspace --lib'

# Watch for changes and check
watch-check:
    cd mew && cargo watch -x check

# ============================================================================
# Documentation
# ============================================================================

# Generate and open documentation
docs:
    cd mew && cargo doc --workspace --no-deps --open

# Generate documentation without opening
docs-build:
    cd mew && cargo doc --workspace --no-deps

# ============================================================================
# Coverage
# ============================================================================

# Generate coverage report (requires cargo-llvm-cov)
coverage:
    cd mew && cargo llvm-cov --workspace --html --open

# Generate coverage report as lcov
coverage-lcov:
    cd mew && cargo llvm-cov --workspace --lcov --output-path lcov.info

# ============================================================================
# Maintenance
# ============================================================================

# Clean build artifacts
clean:
    cd mew && cargo clean

# Update dependencies
update:
    cd mew && cargo update

# Check for outdated dependencies
outdated:
    cd mew && cargo outdated -R

# ============================================================================
# Playground
# ============================================================================

# Build WASM module for playground
playground-wasm:
    cd mew/playground && wasm-pack build --target web --out-dir web/pkg

# Install playground dependencies
playground-setup:
    cd mew/playground/web && npm install

# Start playground dev server
playground-dev: playground-wasm
    cd mew/playground/web && npm run dev

# Build playground for production
playground-build: playground-wasm
    cd mew/playground/web && npm run build

# ============================================================================
# Shortcuts
# ============================================================================

# Alias: run the REPL
mew: repl

# Alias: format code
f: fmt

# Alias: run tests
t: test

# Alias: run clippy
c: lint

# Alias: start playground
pg: playground-dev
