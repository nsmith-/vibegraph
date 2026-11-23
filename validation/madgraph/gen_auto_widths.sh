#!/usr/bin/env bash
# MadGraph's own `compute_widths`, the reference the `DECAY ... Auto` gate
# (`vibegraph-lib/tests/auto_widths.rs`) compares the computed widths with,
# channel by channel.
#
# Cases:
#
#   sm            `import model sm` (restrict_default), the top, Z, W and Higgs
#                 widths at the restriction's own parameters
#   g2hdm_m1000   the g2HDM UFO (CMS model store, g2HDM_UFO.tar.gz; not in this
#                 repository, so G2HDM_UFO must name its directory), the heavy
#                 neutral and charged Higgs bosons at M = 1000 GeV, at
#                 auto_widths_g2hdm_m1000_param_card.dat (MadGraph's own card for
#                 the model with mh1 = 125, mh2 = mh3 = mhc = 1000, every Yukawa
#                 matrix zero but yukawagur(3,2) = 0.4 and (3,3) = 0.6, and
#                 DECAY 25 4.07e-3)
#
# Each case runs twice: `--body_decay=2`, the two-body widths, and MadGraph's
# default, which adds modes beyond two bodies (a MadEvent integration; seconds
# to minutes). The cards MadGraph writes stay in AUTO_WIDTHS_WORK (default
# validation/madgraph/work/auto_widths); the table is auto_widths_reference.json.
#
# Usage: G2HDM_UFO=/path/to/g2HDM_UFO pixi run -e madgraph bash validation/madgraph/gen_auto_widths.sh
#        VG_MG5_ROOT=<copy of the pinned tree> runs MadGraph from there
#        (mg5_pinned.sh), which keeps its model caches out of the checkout.
#        HEAVY_LOCK=<file> holds that lock (flock) around each MadGraph run, on a
#        host whose heavy jobs take turns.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
WORK="${AUTO_WIDTHS_WORK:-$HERE/work/auto_widths}"
: "${G2HDM_UFO:?G2HDM_UFO must name the g2HDM_UFO model directory}"
G2HDM_UFO="$(cd "$G2HDM_UFO" && pwd)"
CARD="validation/madgraph/auto_widths_g2hdm_m1000_param_card.dat"

mkdir -p "$WORK"

run() { # name, import line, compute_widths arguments
  local name=$1 import=$2 args=$3
  for mode in two_body default; do
    local extra=""
    [ "$mode" = two_body ] && extra="--body_decay=2"
    rm -f "$WORK/${name}_$mode.dat"
    printf '%s\ncompute_widths %s %s --output=%s\n' "$import" "$args" "$extra" \
      "$WORK/${name}_$mode.dat" > "$WORK/${name}_$mode.mg5"
    ${HEAVY_LOCK:+flock "$HEAVY_LOCK"} bash "$HERE/mg5_pinned.sh" "$WORK/${name}_$mode.mg5" \
      > "$WORK/${name}_$mode.log" 2>&1
    # compute_widths returns 0 after a traceback too: the card is the evidence.
    grep -q '^DECAY' "$WORK/${name}_$mode.dat" || {
      echo "!!! $name $mode: no card written (see $WORK/${name}_$mode.log)" >&2
      exit 1
    }
  done
}

run sm "import model sm" "6 23 24 25"
run g2hdm_m1000 "import model $G2HDM_UFO" "35 36 37 --path=$ROOT/$CARD"

python "$HERE/parse_auto_widths.py" "$HERE/auto_widths_reference.json" \
  sm - 6,23,24,25 "$WORK/sm_two_body.dat" "$WORK/sm_default.dat" \
  g2hdm_m1000 "$CARD" 35,36,37 "$WORK/g2hdm_m1000_two_body.dat" "$WORK/g2hdm_m1000_default.dat"
echo "wrote $HERE/auto_widths_reference.json"
