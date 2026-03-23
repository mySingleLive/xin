#!/bin/bash
# Xin E2E Test Runner
# Usage: ./run_tests.sh [directory] [-v|--verbose] [-h|--help]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RESET='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
XIN_COMPILER="cargo run --"
TEST_DIRS=("basic" "strings" "operators" "templates" "control_flow" "functions" "arrays" "nullable" "maps" "floats")
VERBOSE=false
TARGET_DIR=""

# Verbose output helper
verbose_log() {
    if [ "$VERBOSE" = true ]; then
        printf "${CYAN}[VERBOSE] %s${RESET}\n" "$1"
    fi
}

# Help message
show_help() {
    echo "Xin E2E Test Runner"
    echo ""
    echo "Usage: ./run_tests.sh [options] [directory]"
    echo ""
    echo "Options:"
    echo "  -v, --verbose  Show detailed output"
    echo "  -h, --help     Show this help message"
    echo ""
    echo "Test directories: ${TEST_DIRS[*]}"
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            TARGET_DIR=$1
            shift
            ;;
    esac
done

# Determine which directories to test
if [ -n "$TARGET_DIR" ]; then
    DIRS=("$TARGET_DIR")
else
    DIRS=("${TEST_DIRS[@]}")
fi

# Test counters
PASSED=0
FAILED=0

# Print header
printf "${CYAN}[Xin E2E Tests]${RESET}\n"
echo ""

# Run tests
for dir in "${DIRS[@]}"; do
    test_dir="$SCRIPT_DIR/$dir"

    if [ ! -d "$test_dir" ]; then
        printf "${RED}Directory not found: $test_dir${RESET}\n"
        continue
    fi

    # Find all .xin files in the directory
    for xin_file in "$test_dir"/*.xin; do
        if [ ! -f "$xin_file" ]; then
            continue
        fi

        test_name="${xin_file%.xin}"
        test_name=$(basename "$test_name")
        expected_file="${xin_file%.xin}.expected"

        # Check if expected file exists
        if [ ! -f "$expected_file" ]; then
            printf "Running $dir/$test_name... ${RED}✗ MISSING EXPECTED FILE${RESET}\n"
            ((FAILED++))
            continue
        fi

        verbose_log "Compiling $xin_file"

        # Compile the xin file
        output_binary="/tmp/xin_test_${test_name}"
        compile_output=$(cd "$PROJECT_ROOT" && cargo run -- compile "$xin_file" -o "$output_binary" 2>&1) || {
            printf "Running $dir/$test_name... ${RED}✗ COMPILE FAILED${RESET}\n"
            verbose_log "$compile_output"
            ((FAILED++))
            continue
        }

        verbose_log "Running $output_binary"

        # Run the binary and capture output
        actual_output=$("$output_binary" 2>&1) || {
            printf "Running $dir/$test_name... ${RED}✗ RUNTIME ERROR${RESET}\n"
            verbose_log "$actual_output"
            ((FAILED++))
            continue
        }

        # Read expected output
        expected_output=$(cat "$expected_file")

        # Compare outputs (strip trailing whitespace per line)
        actual_stripped=$(echo "$actual_output" | sed 's/[[:space:]]*$//')
        expected_stripped=$(echo "$expected_output" | sed 's/[[:space:]]*$//')

        if [ "$actual_stripped" = "$expected_stripped" ]; then
            printf "Running $dir/$test_name... ${GREEN}✓${RESET}\n"
            ((PASSED++))
        else
            printf "Running $dir/$test_name... ${RED}✗ FAILED${RESET}\n"
            echo ""
            echo "--- Expected ---"
            echo "$expected_output"
            echo ""
            echo "--- Actual ---"
            echo "$actual_output"
            echo ""
            echo "Test failed: $dir/$test_name"
            echo "Stopped at first failure."
            echo ""
            echo "Summary: $PASSED passed, 1 failed"
            exit 1
        fi

        # Clean up
        rm -f "$output_binary"
    done
done

# Print summary
echo ""
if [ $FAILED -eq 0 ]; then
    printf "${GREEN}All tests passed! ($PASSED/$PASSED)${RESET}\n"
else
    echo "Summary: $PASSED passed, $FAILED failed"
fi

exit $FAILED