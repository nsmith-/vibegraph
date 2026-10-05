#!/usr/bin/env bash
# The vibegraph side of the matched Pythia comparison (mlm_match.py): the
# pp_to_ll_0j2j_mlm card (p p > e+ e- @0, + j @1, + j j @2; ickkw = 1,
# xqcut = 20), integrated and unweighted once per seed through the shipped
# binary, so Pythia is handed what a user's `generate` run writes.
#
# Each seed is an independent integration (its own grids) and an independent
# sample, drawn with the same seed. Twenty by default, the size the comparison
# against MadEvent's twenty-one banked directories was measured at. The budget is the one the row's σ was
# measured at (--fixed-budget --allocate neyman --neval 200000 --niter 8), and
# the event count is MadEvent's samples-grade run's.
#
# Output: target/mlm-pythia-samples/vg-<seed>.lhe (and the artifact beside it).
#
# Usage:
#   bash validation/pythia/generate_mlm_samples.sh
#   SEEDS="20260928 20260929" NEVENTS=10000 BIN=path/to/vibegraph ... to override
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${OUT:-$ROOT/target/mlm-pythia-samples}"
RUN_CARD="$ROOT/validation/madgraph/pp_to_ll_0j2j_mlm_run_card.dat"
SEEDS="${SEEDS:-$(seq -f '%.0f' -s " " 20260928 20260947)}"
NEVENTS="${NEVENTS:-10000}"
BIN="${BIN:-}"

mkdir -p "$OUT"
PROC="$OUT/pp_to_ll_0j2j_mlm.proc"
# The proc lines of validation/madgraph/scripts/pp_to_ll_0j2j_mlm.mg5.
cat > "$PROC" <<'EOF'
import model sm
generate p p > e+ e- @0
add process p p > e+ e- j @1
add process p p > e+ e- j j @2
EOF

if [ -z "$BIN" ]; then
  echo ">>> building the release-debug binary ..."
  cargo build --manifest-path "$ROOT/Cargo.toml" --profile release-debug --bin vibegraph
  BIN="${CARGO_TARGET_DIR:-$ROOT/target}/release-debug/vibegraph"
fi
echo ">>> binary $BIN sha256 $(sha256sum "$BIN" | cut -d' ' -f1)"

for seed in $SEEDS; do
  art="$OUT/vg-$seed"
  echo ">>> [$seed] integrating ..."
  "$BIN" integrate "$PROC" --run-card "$RUN_CARD" --pdf-dir "$ROOT/validation/pdf" \
    --out "$art" --force --fixed-budget --allocate neyman --neval 200000 --niter 8 --seed "$seed"
  echo ">>> [$seed] generating $NEVENTS events ..."
  "$BIN" generate "$art/grid.bin.zst" "$PROC" --run-card "$RUN_CARD" --pdf-dir "$ROOT/validation/pdf" \
    --nevents "$NEVENTS" --seed "$seed" --force -o "$OUT/vg-$seed.lhe"
done
echo ">>> done"
