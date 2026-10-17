#!/usr/bin/env bash
# Whether a pull request changes anything the banked CI layer reads.
#
# Usage: bash scripts/banked-inputs-changed.sh BASE [HEAD]
#
# Prints one line on stdout, `run=true` or `run=false`, for `$GITHUB_OUTPUT`;
# the changed files and the reason go to stderr. It answers `run=false` only
# when it has a non-empty diff and every path in it is on the skip list below;
# any other outcome (no base, a failed diff, an empty diff, one unlisted path)
# is `run=true`.
#
# The skip list names paths nothing in `pixi run validate` reads. Everything
# else counts as an input, including files outside the crates that the build or
# the tests open: the licence files `vibegraph-cli` compiles in, the run cards
# under `validation/`, the submodule pointer, `pixi.toml` and the workflow.
set -uo pipefail

decide() {
  printf 'banked: %s\n' "$2" >&2
  printf 'run=%s\n' "$1"
  exit 0
}

[ -n "${1:-}" ] || decide true "run: no base revision given"
# `--no-renames` lists both sides of a rename, so a file moved out of an input
# directory still counts as a change to it.
files=$(git diff --no-renames --name-only "$1...${2:-HEAD}") ||
  decide true "run: git diff against $1 failed"
[ -n "$files" ] || decide true "run: the diff is empty"

printf 'changed files:\n%s\n' "$files" | sed '2,$s/^/  /' >&2
while IFS= read -r f; do
  case "$f" in
    # `vibegraph-cli/tests/cli_reference_docs.rs` compares this chapter with
    # the CLI's `--help` output.
    docs/src/cli/*) ;;
    # Crates and validation data, Markdown included.
    vibegraph-*/* | validation-report/* | validation/*) ;;
    # Read by `scripts/kb.py`, which runs in the `kb` job.
    research/kb/*) continue ;;
    # The mdBook, agent prompts and skills, README media, the rustdoc header.
    docs/* | .agents/* | .claude/* | assets/* | doc-include/*) continue ;;
    # Prose anywhere else: no build script, test or validation script opens it.
    *.md) continue ;;
  esac
  decide true "run: $f is a banked-layer input"
done <<<"$files"
decide false "skip: no banked-layer inputs changed"
