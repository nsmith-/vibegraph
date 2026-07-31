#!/usr/bin/env bash
# Usage: profile.sh <test-name> [test-filter] [-- samply-args...]
# Builds the given integration test with the release-debug profile and records it with samply.
# The test name must match a file in tests/ (e.g. validate_madgraph_diagrams).
# An optional test-filter narrows which test cases run (passed to the test binary).
# Arguments after -- are forwarded to samply record.
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <test-name> [test-filter] [-- samply-args...]" >&2
    exit 1
fi

TEST_NAME="$1"
shift

TEST_FILTER=""
if [[ $# -gt 0 && "$1" != "--" ]]; then
    TEST_FILTER="$1"
    shift
fi

if [[ $# -gt 0 && "$1" == "--" ]]; then
    shift
fi

BUILD_OUTPUT=$(cargo test --profile release-debug --test "$TEST_NAME" --features extended-validation --no-run 2>&1)
echo "$BUILD_OUTPUT"

EXECUTABLE=$(echo "$BUILD_OUTPUT" | tail -1 | sed -n 's/.*Executable[^(]*(//;s/).*//p')

if [[ -z "$EXECUTABLE" ]]; then
    echo "error: could not parse executable path from cargo output" >&2
    exit 1
fi

if [[ -n "$TEST_FILTER" ]]; then
    exec samply record "$@" "$EXECUTABLE" "$TEST_FILTER"
else
    exec samply record "$@" "$EXECUTABLE"
fi
