#!/bin/bash
# MEW Generative Test Runner Helper
#
# Usage:
#   ./run_tests.sh              # Generate tests for all ontologies
#   ./run_tests.sh --level 1    # Generate tests for level-1 only
#   ./run_tests.sh --execute    # Execute tests against MEW
#   ./run_tests.sh --help       # Show all options

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Build the test runner if needed
echo "Building test runner..."
cd mew
cargo build -p mew-testgen --bin testgen-runner --quiet 2>/dev/null || cargo build -p mew-testgen --bin testgen-runner
cd ..

# Run with passed arguments
./mew/target/debug/testgen-runner \
    --ontologies-dir ontologies \
    --output mew/tests/reports \
    "$@"

echo ""
echo "Reports saved to: mew/tests/reports/"
