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
# uploads. The exit status is the worse of the two.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${CARGO_TARGET_DIR:-$ROOT/target}/validation-report"

rm -rf "$REPORT_DIR/diagrams" "$REPORT_DIR/amplitudes" \
  "$REPORT_DIR/integrals" "$REPORT_DIR/samples"
# The machine block belongs to the run whose durations it labels, and the gates
# rewrite it as they write their first row.
rm -f "$REPORT_DIR/host.json"

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
exit "$collated"
