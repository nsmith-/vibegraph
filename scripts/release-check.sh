#!/usr/bin/env bash
# Validates a release tag and finds its notes.
#
# Usage: bash scripts/release-check.sh TAG [ROOT]
#
# TAG is `vX.Y.Z` or a prerelease `vX.Y.Z-<suffix>` (`v0.2.0-rc1`). ROOT is the
# repository root holding `docs/release-notes/` (default: the repository this
# script lives in). On success prints `prerelease=true|false` and `notes=<path>`
# on stdout (for `$GITHUB_OUTPUT`). A tag with a hyphen is a prerelease. The
# notes file is named after the tag with the prerelease suffix removed, so
# `v0.2.0-rc1` and `v0.2.0` both read `docs/release-notes/v0.2.0.md`.
#
# Fails, naming the reason on stderr, when the tag is not a version tag, the
# notes file is missing or empty, or any line of it contains `TODO`.
set -uo pipefail

fail() {
  printf 'release-check: %s\n' "$1" >&2
  exit 1
}

tag="${1:-}"
root="${2:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]] ||
  fail "'$tag' is not a version tag (vX.Y.Z or vX.Y.Z-suffix)"

prerelease=false
[[ "$tag" == *-* ]] && prerelease=true
notes="docs/release-notes/${tag%%-*}.md"

[ -f "$root/$notes" ] || fail "$notes does not exist"
[ -s "$root/$notes" ] || fail "$notes is empty"
if grep -n 'TODO' "$root/$notes" >&2; then
  fail "$notes still contains TODO"
fi

printf 'prerelease=%s\n' "$prerelease"
printf 'notes=%s\n' "$notes"
