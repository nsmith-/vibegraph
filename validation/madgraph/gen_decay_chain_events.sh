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
# Usage: PATH=<madgraph env>/bin:$PATH bash validation/madgraph/gen_decay_chain_events.sh
#        ROWS="pp_ttx_lep_dyn" SEEDS="1 2" ... to run a subset.
#        DECAY_CHAIN_EVENTS_WORK=<dir> keeps the process directories and the
#        event files there (default: a fresh temporary directory).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="${DECAY_CHAIN_EVENTS_WORK:-$(mktemp -d "${TMPDIR:-/tmp}/vg-chain-events-XXXXXX")}"
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

# Generate one process directory (idempotent); `;` separates the card's lines.
generate_dir() {
  local procdir="$1" lines="$2"
  if [ ! -f "$procdir/bin/generate_events" ]; then
    echo ">>> generating $lines into $procdir ..." >&2
    local script
    script="$(mktemp -t gen_events_XXXX).mg5"
    {
      printf 'import model sm\n'
      printf '%s\n' "$lines" | tr ';' '\n'
      printf 'output %s -nojpeg\n' "$procdir"
    } > "$script"
    bash "$HERE/mg5_pinned.sh" "$script" >&2
    rm -f "$script"
  fi
  local cfg="$procdir/Cards/me5_configuration.txt"
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center|run_mode|nb_core)\s*=' "$cfg" > "$cfg.tmp"
  printf 'automatic_html_opening = False\nnotification_center = False\nrun_mode = 2\nnb_core = 2\n' >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
}

# Install a run card with its iseed and a 10k-event budget, run madevent, and
# echo "<sigma> <err> <wall>" (pb) from results.dat.
run_one() {
  local procdir="$1" card="$2" seed="$3" tag="$4"
  python3 - "$card" "$procdir/Cards/run_card.dat" "$seed" <<'PY'
import re, sys
src, dst, seed = sys.argv[1:4]
text = open(src).read()
text, n = re.subn(r"^\s*\S+\s*=\s*iseed\b", "  %s = iseed" % seed, text, flags=re.M)
assert n == 1, "iseed not found exactly once"
text, n = re.subn(r"^\s*\S+\s*=\s*nevents\b", "  10000 = nevents", text, flags=re.M)
assert n == 1, "nevents not found exactly once"
open(dst, "w").write(text)
PY
  local log="$OUT/events_$tag.log"
  echo ">>> [$tag] madevent, iseed $seed ..." >&2
  rm -rf "$procdir/Events/run_$tag"
  local started
  started="$(date +%s)"
  "$procdir/bin/generate_events" -f "run_$tag" > "$log" 2>&1 || {
    echo "!!! [$tag] generate_events failed; see $log" >&2
    tail -40 "$log" >&2
    exit 1
  }
  local wall=$(( $(date +%s) - started ))
  awk -v wall="$wall" 'NR==1{printf "%.10g %.10g %d\n", $1, $2, wall}' "$procdir/SubProcesses/results.dat"
}

RESULTS="$OUT/rows.txt"
: > "$RESULTS"
for row in "${ALL_ROWS[@]}"; do
  IFS='|' read -r name lines card <<< "$row"
  case " $SELECTED " in *" $name "*) ;; *) continue ;; esac
  procdir="$OUT/$name"
  generate_dir "$procdir" "$lines"
  for seed in $SEEDS; do
    read -r sigma err wall < <(run_one "$procdir" "$HERE/$card" "$seed" "${name}_$seed")
    lhe="$procdir/Events/run_${name}_$seed/unweighted_events.lhe.gz"
    printf '%s|%s|%s|%s|%s|%s|%s|%s\n' "$name" "$lines" "$card" "$seed" "$sigma" "$err" "$wall" "$lhe" \
      | tee -a "$RESULTS" >&2
  done
done

python3 "$HERE/summarise_decay_chain_events.py" "$RESULTS" "$RESULT_JSON" \
  "$ROOT/research/refs/mg5amcnlo/VERSION"
