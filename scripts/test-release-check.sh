#!/usr/bin/env bash
# Exercises scripts/release-check.sh against fixture trees.
#
# Usage: bash scripts/test-release-check.sh
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
check="$here/release-check.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
failures=0

expect_ok() { # tag root expected-prerelease expected-notes
  local out
  if ! out=$(bash "$check" "$1" "$2" 2>/dev/null); then
    echo "FAIL: $1 should be accepted"; failures=$((failures + 1)); return
  fi
  if [ "$out" != "prerelease=$3"$'\n'"notes=$4" ]; then
    echo "FAIL: $1 gave: $out"; failures=$((failures + 1)); return
  fi
  echo "ok: $1 accepted ($3, $4)"
}

expect_refused() { # tag root why
  if bash "$check" "$1" "$2" >/dev/null 2>&1; then
    echo "FAIL: $1 should be refused ($3)"; failures=$((failures + 1))
  else
    echo "ok: $1 refused ($3)"
  fi
}

good="$work/good"; mkdir -p "$good/docs/release-notes"
printf '# v0.2.0\n\n## Highlights\n\nSomething.\n' > "$good/docs/release-notes/v0.2.0.md"

todo="$work/todo"; mkdir -p "$todo/docs/release-notes"
printf '# v0.2.0\n\n## Highlights\n\nTODO: write\n' > "$todo/docs/release-notes/v0.2.0.md"

empty="$work/empty"; mkdir -p "$empty/docs/release-notes"
: > "$empty/docs/release-notes/v0.2.0.md"

none="$work/none"; mkdir -p "$none/docs/release-notes"

expect_ok v0.2.0 "$good" false docs/release-notes/v0.2.0.md
expect_ok v0.2.0-rc1 "$good" true docs/release-notes/v0.2.0.md
expect_refused v0.2.0 "$none" "missing notes"
expect_refused v0.2.0-rc1 "$none" "missing notes"
expect_refused v0.2.0 "$todo" "TODO stub"
expect_refused v0.2.0-rc1 "$todo" "TODO stub"
expect_refused v0.2.0 "$empty" "empty notes"
expect_refused v0.3.0 "$good" "notes for another version"
expect_refused refdata-9 "$good" "not a version tag"
expect_refused main "$good" "not a version tag"

[ "$failures" -eq 0 ] || { echo "$failures failure(s)"; exit 1; }
echo "all release-check cases passed"
