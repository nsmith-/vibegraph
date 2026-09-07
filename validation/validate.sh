#!/usr/bin/env bash
# The banked layer end to end: clear the previous run's cells, run every gate,
# collate the report.
#
# Usage:
#   pixi run validate            # with the fetch tasks as dependencies
#   bash validation/validate.sh  # on a machine whose inputs are already there
#
# The per-category row files are deleted first, so the report is what *this*
# invocation measured: a row that stopped being written shows up as a missing
# cell instead of being served from the last run. The host block goes with them,
# so no run reads its durations against another machine's identity. `standalone/` is left alone —
# the gates with drivers of their own (Pythia) run under separate tasks, and the
# collator says so when their verdict predates this run's cells.
#
# The gates and the collator both run whatever the other does: a failing gate
# still gets a rendered report naming which cell failed, which is the artifact CI
# uploads. Clippy runs first for the same reason and is likewise not allowed to
# skip the gates. The exit status names the gates first, then the lints, then the
# collator.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${CARGO_TARGET_DIR:-$ROOT/target}/validation-report"

rm -rf "$REPORT_DIR/diagrams" "$REPORT_DIR/amplitudes" \
  "$REPORT_DIR/integrals" "$REPORT_DIR/samples"
# The machine block belongs to the run whose durations it labels, and the gates
# rewrite it as they write their first row.
rm -f "$REPORT_DIR/host.json"

# CI lints the workspace with no features (`ci.yml`), which leaves every
# `#[cfg(feature = "extended-validation")]` target — the gates below and their
# shared `common/` modules — unlinted. Running it here is what keeps a lint from
# being green in CI and red on the code the banked layer is made of. It is a
# build of its own, so it goes first: a lint failure is cheap to read and the
# gates take minutes.
cargo clippy --manifest-path "$ROOT/Cargo.toml" --workspace --all-targets \
  --features vibegraph/extended-validation,vibegraph-lib/extended-validation \
  -- -D warnings
lints=$?

cargo test --manifest-path "$ROOT/Cargo.toml" --workspace --profile release-debug \
  --features vibegraph/extended-validation,vibegraph-lib/extended-validation \
  -- --nocapture
gates=$?

cargo run --manifest-path "$ROOT/Cargo.toml" --profile release-debug \
  -p vibegraph-validation-report
collated=$?

if [ "$gates" -ne 0 ]; then
  printf '!!! the banked gates failed (exit %s); the report above is what they left behind\n' \
    "$gates" >&2
  exit "$gates"
fi
if [ "$lints" -ne 0 ]; then
  printf '!!! clippy failed on the extended-validation targets (exit %s)\n' "$lints" >&2
  exit "$lints"
fi
exit "$collated"
