#!/usr/bin/env bash
# Produce the unweighted Les Houches samples the Pythia consumption gate reads.
#
# Both samples come from card sets that are already validated elsewhere — the
# banked `pp_to_llj_fixed` run card and the committed `dy13` cards — and both go
# through the shipped `vibegraph` binary rather than a library harness, so what
# Pythia is handed is exactly what a user's `generate` run writes.
#
# Everything is fixed: the cards, the integration budgets, the seeds and the
# event counts. Same inputs, same bytes, so the gate is rerunnable.
#
# Usage:
#   pixi run -e pythia generate-pythia-samples
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/target/pythia-samples"
PDF_DIR="$ROOT/validation/pdf"
BIN="$ROOT/target/release-debug/vibegraph"

# Shared across both samples: one seed for the integration and the generation,
# and an event count sized so the whole gate — build excluded — stays under a
# minute while still drawing every flavour group the llj process can emit.
SEED=20260801
NEVENTS=2000

mkdir -p "$OUT"

echo ">>> building the release-debug binary ..."
cargo build --manifest-path "$ROOT/Cargo.toml" --profile release-debug --bin vibegraph

# integrate + unweight one card set into "$OUT/<name>.lhe".
emit() {
  local name="$1" proc_card="$2" run_card="$3"
  shift 3

  echo ">>> [$name] integrating ..."
  # `--fixed-budget` because this gate wants a bounded, rerunnable sample rather
  # than an accuracy: the shipped default converges to a relative uncertainty and
  # so spends however many iterations that takes.
  "$BIN" integrate "$proc_card" \
    --run-card "$run_card" \
    --pdf-dir "$PDF_DIR" \
    --out "$OUT/$name" \
    --force \
    --fixed-budget \
    --seed "$SEED" \
    "$@"

  echo ">>> [$name] unweighting $NEVENTS events ..."
  "$BIN" generate "$OUT/$name/grid.bin.zst" "$proc_card" \
    --run-card "$run_card" \
    --pdf-dir "$PDF_DIR" \
    --nevents "$NEVENTS" \
    --seed "$SEED" \
    --out "$OUT/$name.lhe" \
    --force
}

# `p p > l+ l- j`: a coloured initial *and* final state over 24 flavour groups —
# the sample whose colour lines Pythia has never been shown before. The
# integration budget is the one the event-generation gate settled on.
LLJ_PROC="$OUT/llj_fixed_proc_card.dat"
cat >"$LLJ_PROC" <<'EOF'
import model sm
generate p p > l+ l- j QCD=2 QED=2
EOF
emit llj_fixed "$LLJ_PROC" \
  "$ROOT/validation/madgraph/output/pp_to_llj_fixed/Cards/run_card.dat" \
  --neval 300000 --niter 8

# `p p > e+ e-`: a colour-singlet final state off a q q~ initial state, so the
# only colour line in the record is the one joining the two beams.
emit dy13_default \
  "$ROOT/validation/madgraph/dy13_proc_card.dat" \
  "$ROOT/validation/madgraph/dy13_default_run_card.dat" \
  --neval 120000 --niter 12

python3 - "$OUT/samples.json" "$SEED" "$NEVENTS" <<'PY'
import json, sys

out, seed, nevents = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
doc = {
    "seed": seed,
    "nevents": nevents,
    "samples": [
        {
            "key": "llj_fixed",
            "process": "p p > l+ l- j QCD=2 QED=2",
            "lhe": "llj_fixed.lhe",
            "run_card": "validation/madgraph/output/pp_to_llj_fixed/Cards/run_card.dat",
        },
        {
            "key": "dy13_default",
            "process": "p p > e+ e-",
            "lhe": "dy13_default.lhe",
            "run_card": "validation/madgraph/dy13_default_run_card.dat",
        },
    ],
}
with open(out, "w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print("wrote", out)
PY
