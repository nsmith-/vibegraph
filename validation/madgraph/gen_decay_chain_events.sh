#!/usr/bin/env bash
# MadEvent event samples for the decay-chain event records and the `@N`
# process split, summarised into decay_chain_events_reference.json.
#
# Each row is one proc card run through MadEvent over SEEDS (one run per
# iseed, 10k unweighted events each). The event files stay in the scratch
# work area; what is committed is what `summarise_decay_chain_events.py`
# reads out of them: the `<init>` block, binned distributions, and the
# counts of every categorical field the vibegraph side is compared on.
#
#   pp_ttx_lep_dyn  p p > t t~, t > b e+ ve, t~ > b~ mu- vm~   pp13_dyn
#                   MadGraph's default dynamical scale, so SCALUP and AQCDUP
#                   carry the clustered core scale; the status-2 t, t~ and the
#                   free W lines inside their Breit-Wigner windows.
#   ttx_nested      e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~  ee500
#                   a nested forced resonance and an undecayed W.
#   zz_ee           e+ e- > z z, z > e+ e-                   ee500
#                   identical particles across the decays: which pairing the
#                   status-2 records carry.
#   dy_two_procs    p p > e+ e- / z @1 ; add process p p > mu+ mu- / a @2
#                   dy13_mmll; one <init> line per process, IDPRUP per event.
#   veto_chain      e+ e- > mu+ mu- z $ z, z > e+ e-          ee500
#                   a `$` on the core of a decay chain (cross section only).
#
# Usage: pixi run -e madgraph generate-decay-chain-events
#        ROWS="pp_ttx_lep_dyn" SEEDS="1 2" ... to run a subset; rows not run
#        keep their committed entries.
#        DECAY_CHAIN_EVENTS_WORK=<dir> holds the process directories and the
#        event files (default validation/madgraph/work/decay_chain_events),
#        cached as madevent_seeds.sh describes.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
. "$HERE/madevent_seeds.sh"
OUT="${DECAY_CHAIN_EVENTS_WORK:-$HERE/work/decay_chain_events}"
RESULT_JSON="${RESULT_JSON:-$HERE/decay_chain_events_reference.json}"
SEEDS="${SEEDS:-$(seq -s " " 20260926 20260930)}"

LHAPDF_DATA_PATH="$ROOT/validation/pdf"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

mkdir -p "$OUT"

ALL_ROWS=(
  "pp_ttx_lep_dyn|generate p p > t t~, t > b e+ ve, t~ > b~ mu- vm~|decay_chain_pp13_dyn_run_card.dat"
  "ttx_nested|generate e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~|decay_chain_ee500_run_card.dat"
  "zz_ee|generate e+ e- > z z, z > e+ e-|decay_chain_ee500_run_card.dat"
  "dy_two_procs|generate p p > e+ e- / z @1;add process p p > mu+ mu- / a @2|dy13_mmll_run_card.dat"
  "veto_chain|generate e+ e- > mu+ mu- z \$ z, z > e+ e-|decay_chain_ee500_run_card.dat"
)
SELECTED="${ROWS:-pp_ttx_lep_dyn ttx_nested zz_ee dy_two_procs veto_chain}"

RESULTS="$OUT/rows_$$.txt"
: > "$RESULTS"
for row in "${ALL_ROWS[@]}"; do
  IFS='|' read -r name lines card <<< "$row"
  case " $SELECTED " in *" $name "*) ;; *) continue ;; esac
  procdir="$OUT/$name"
  mes_generate_dir "$procdir" "$lines"
  for seed in $SEEDS; do
    mes_install_card "$HERE/$card" "$procdir/Cards/run_card.dat" "nevents=10000" "$seed"
    result="$(mes_run_seed "$procdir" "${name}_$seed" "$OUT/events_${name}_$seed.log")"
    read -r sigma err wall <<< "$result"
    lhe="$procdir/Events/run_${name}_$seed/unweighted_events.lhe.gz"
    printf '%s|%s|%s|%s|%s|%s|%s|%s\n' "$name" "$lines" "$card" "$seed" "$sigma" "$err" "$wall" "$lhe" \
      | tee -a "$RESULTS" >&2
  done
done

python3 "$HERE/summarise_decay_chain_events.py" "$RESULTS" "$RESULT_JSON" \
  "$ROOT/research/refs/mg5amcnlo/VERSION"
rm -f "$RESULTS"
