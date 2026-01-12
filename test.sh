#!/bin/bash
# MEW Unified Test Runner
#
# Single entry point for all test types:
#   - Unit tests (cargo test)
#   - Integration tests (mew-tests scenarios)
#   - Generated tests (testgen)
#
# Usage:
#   ./test.sh              # Run all tests
#   ./test.sh unit         # Run unit tests only
#   ./test.sh integration  # Run integration tests only
#   ./test.sh testgen      # Run generated tests only
#   ./test.sh quick        # Run unit + integration (skip testgen)
#   ./test.sh --help       # Show help

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output (disable if not a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

# Track results
UNIT_RESULT=""
INTEGRATION_RESULT=""
TESTGEN_RESULT=""

print_header() {
    echo ""
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${BLUE}  $1${NC}"
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_status() {
    if [ "$2" = "pass" ]; then
        echo -e "${GREEN}[PASS]${NC} $1"
    elif [ "$2" = "fail" ]; then
        echo -e "${RED}[FAIL]${NC} $1"
    elif [ "$2" = "skip" ]; then
        echo -e "${YELLOW}[SKIP]${NC} $1"
    else
        echo -e "${BLUE}[INFO]${NC} $1"
    fi
}

show_help() {
    cat << 'EOF'
MEW Unified Test Runner

USAGE:
    ./test.sh [COMMAND] [OPTIONS]

COMMANDS:
    all           Run all test types (default)
    unit          Run unit tests only (cargo test --workspace)
    integration   Run integration tests only (mew-tests scenarios)
    testgen       Run generated tests only (from examples/)
    quick         Run unit + integration (skip testgen)

OPTIONS:
    -v, --verbose     Show verbose output
    -p, --package     Run tests for specific package (unit tests only)
    --level N         Filter testgen by complexity level (1-5)
    --ontology NAME   Filter testgen by ontology name
    --no-execute      Skip testgen execution (generate only)
    -h, --help        Show this help message

EXAMPLES:
    ./test.sh                      # Run all tests (including testgen execution)
    ./test.sh unit                 # Unit tests only
    ./test.sh unit -p mew-graph    # Unit tests for mew-graph only
    ./test.sh integration          # Integration scenarios only
    ./test.sh testgen --level 1    # Generate and execute level-1 tests
    ./test.sh testgen --no-execute # Generate tests only (skip execution)
    ./test.sh quick                # Fast check (unit + integration)

TEST TYPES:
    Unit Tests:
        Standard Rust tests in each crate. Tests individual components
        in isolation. Run via 'cargo test --workspace'.

    Integration Tests:
        Scenario-based tests in mew-tests crate. Test full workflows
        using ontologies from examples/. Run via 'cargo test -p mew-tests'.

    Generated Tests (testgen):
        Auto-generated tests from ontology schemas. Uses mew-testgen
        to create test cases with oracle verification.

REPORTS:
    testgen reports are saved to: mew/tests/reports/
EOF
}

run_unit_tests() {
    local package=""
    local verbose=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--package)
                package="$2"
                shift 2
                ;;
            -v|--verbose)
                verbose="--nocapture"
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    print_header "Unit Tests"

    cd mew

    if [ -n "$package" ]; then
        echo "Running unit tests for package: $package"
        if cargo test -p "$package" -- $verbose 2>&1; then
            UNIT_RESULT="pass"
        else
            UNIT_RESULT="fail"
        fi
    else
        echo "Running all unit tests..."
        if cargo test --workspace -- $verbose 2>&1; then
            UNIT_RESULT="pass"
        else
            UNIT_RESULT="fail"
        fi
    fi

    cd "$SCRIPT_DIR"
}

run_integration_tests() {
    local verbose=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--verbose)
                verbose="--nocapture"
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    print_header "Integration Tests (Scenarios)"

    cd mew

    echo "Running scenario-based integration tests..."
    if cargo test -p mew-tests -- $verbose 2>&1; then
        INTEGRATION_RESULT="pass"
    else
        INTEGRATION_RESULT="fail"
    fi

    cd "$SCRIPT_DIR"
}

run_testgen() {
    local testgen_args=""
    local do_execute=true

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --level)
                testgen_args="$testgen_args --level $2"
                shift 2
                ;;
            --ontology)
                testgen_args="$testgen_args --ontology $2"
                shift 2
                ;;
            --no-execute)
                do_execute=false
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    # Execute by default
    if $do_execute; then
        testgen_args="$testgen_args --execute"
    fi

    print_header "Generated Tests (testgen)"

    # Build testgen if needed
    echo "Building testgen runner..."
    cd mew
    cargo build -p mew-testgen --bin testgen-runner --quiet 2>/dev/null || cargo build -p mew-testgen --bin testgen-runner
    cd "$SCRIPT_DIR"

    if $do_execute; then
        echo "Generating and executing tests..."
    else
        echo "Generating tests (execution skipped)..."
    fi

    if ./mew/target/debug/testgen-runner \
        --ontologies-dir examples \
        --output mew/tests/reports \
        $testgen_args 2>&1; then
        TESTGEN_RESULT="pass"
        echo ""
        echo "Reports saved to: mew/tests/reports/"
    else
        TESTGEN_RESULT="fail"
    fi
}

print_summary() {
    print_header "Test Summary"

    local all_pass=true

    if [ -n "$UNIT_RESULT" ]; then
        print_status "Unit Tests" "$UNIT_RESULT"
        [ "$UNIT_RESULT" != "pass" ] && all_pass=false
    else
        print_status "Unit Tests" "skip"
    fi

    if [ -n "$INTEGRATION_RESULT" ]; then
        print_status "Integration Tests" "$INTEGRATION_RESULT"
        [ "$INTEGRATION_RESULT" != "pass" ] && all_pass=false
    else
        print_status "Integration Tests" "skip"
    fi

    if [ -n "$TESTGEN_RESULT" ]; then
        print_status "Generated Tests" "$TESTGEN_RESULT"
        [ "$TESTGEN_RESULT" != "pass" ] && all_pass=false
    else
        print_status "Generated Tests" "skip"
    fi

    echo ""

    if $all_pass; then
        echo -e "${GREEN}${BOLD}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}${BOLD}Some tests failed.${NC}"
        return 1
    fi
}

# Main execution
main() {
    local command="${1:-all}"
    shift || true

    case $command in
        -h|--help|help)
            show_help
            exit 0
            ;;
        all)
            run_unit_tests "$@"
            run_integration_tests "$@"
            run_testgen "$@"
            print_summary
            ;;
        unit)
            run_unit_tests "$@"
            print_summary
            ;;
        integration)
            run_integration_tests "$@"
            print_summary
            ;;
        testgen)
            run_testgen "$@"
            print_summary
            ;;
        quick)
            run_unit_tests "$@"
            run_integration_tests "$@"
            print_summary
            ;;
        *)
            echo -e "${RED}Unknown command: $command${NC}"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

main "$@"
