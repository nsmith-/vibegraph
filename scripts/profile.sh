#!/usr/bin/env bash
# Usage: profile.sh <test-name> [test-filter] [-- harness-args...] [--samply samply-args...]
# Builds the given integration test with the release-debug profile and records it with samply.
# The test name must match a file in tests/ (e.g. validate_madgraph_diagrams).
# An optional test-filter narrows which test cases run (passed to the test binary).
# Arguments after -- go to the test binary (e.g. --ignored --test-threads=1) until a
# --samply marker; arguments after --samply go to samply record.
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <test-name> [test-filter] [-- harness-args...] [--samply samply-args...]" >&2
    exit 1
fi

TEST_NAME="$1"
shift

HARNESS_ARGS=()
SAMPLY_ARGS=()

if [[ $# -gt 0 && "$1" != "--" && "$1" != "--samply" ]]; then
    HARNESS_ARGS+=("$1")
    shift
fi

if [[ $# -gt 0 && "$1" == "--" ]]; then
    shift
    while [[ $# -gt 0 && "$1" != "--samply" ]]; do
        HARNESS_ARGS+=("$1")
        shift
    done
fi

if [[ $# -gt 0 && "$1" == "--samply" ]]; then
    shift
    SAMPLY_ARGS=("$@")
fi

BUILD_OUTPUT=$(cargo test --profile release-debug --test "$TEST_NAME" --features extended-validation --no-run 2>&1)
echo "$BUILD_OUTPUT"

EXECUTABLE=$(echo "$BUILD_OUTPUT" | tail -1 | sed -n 's/.*Executable[^(]*(//;s/).*//p')

if [[ -z "$EXECUTABLE" ]]; then
    echo "error: could not parse executable path from cargo output" >&2
    exit 1
fi

exec samply record ${SAMPLY_ARGS[@]+"${SAMPLY_ARGS[@]}"} "$EXECUTABLE" ${HARNESS_ARGS[@]+"${HARNESS_ARGS[@]}"}
