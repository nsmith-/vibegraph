#!/usr/bin/env bash
# MadEvent's per-channel cross sections, for the configuration-share oracle.
#
# MadEvent integrates every configuration of a `P` directory in its own `G<n>`
# channel, the matrix element multiplied by the single-diagram-enhancement
# factor `AMP2_c / Σ AMP2` (`sde_strategy = 1`, `tmin_for_channel = -1` on every
# card here), so the σ in `G<n>/results.dat` is that configuration's share of
# the directory's cross section. vibegraph banks the same partition per
# configuration of each diagram set (`IntegrateArtifact::configurations`).
#
# Rows, all on `import model sm` (restrict_default), each over SEEDS in one
# freshly generated directory per seed (later seeds of a shared directory
# inherit their predecessors' grids and are not independent draws):
#
#   ee_mumu   e+ e- > mu+ mu-   s-channel γ and Z: two configurations
#   ee_ee     e+ e- > e+ e-     s-channel γ, s-channel Z, and the t-channel
#                               γ and Z, which IdentifyConfigTag makes one
#   ee_uuxg   e+ e- > u u~ g    γ and Z, the gluon off either quark: four
#   pp_eej    p p > e+ e- j     NNPDF23 (lhaid 247000), μR = μF = 91.188 GeV;
#                               two P directories, each grouping two flavour
#                               groups and both beam orderings
#
# The run cards are committed (config_share_<row>_run_card.dat) and installed
# verbatim, iseed apart, so MadEvent and the oracle test read the same cuts,
# beams, PDF and scales.
#
# Per seed it banks the directory's total from SubProcesses/results.dat and
# every channel's from SubProcesses/P*/G*/results.dat, and per directory its
# subprocesses (processes.dat), its channels' propagators (configs.inc) and
# symfact.dat, so the test maps a channel onto a vibegraph configuration by
# propagator content. Everything lands in config_share_reference.json
# (committed); the process directories are deleted once read, unless KEEP=1.
#
# Usage: pixi run -e madgraph generate-config-share
#        ROWS="ee_mumu" SEEDS="1 2" ... to run a subset; rows not run keep their
#        committed entries. WORK names the scratch area (default
#        validation/madgraph/work/config_share).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
. "$HERE/madevent_seeds.sh"
OUT="${WORK:-$HERE/work/config_share}"
RESULT_JSON="${RESULT_JSON:-$HERE/config_share_reference.json}"
SEEDS="${SEEDS:-$(seq -f '%.0f' -s " " 20261110 20261114)}"
NB_CORE="${NB_CORE:-2}"

LHAPDF_DATA_PATH="$HERE/../pdf${LHAPDF_DATA_PATH:+:$LHAPDF_DATA_PATH}"
export LHAPDF_DATA_PATH

mkdir -p "$OUT"

# name|proc lines (';'-separated)
ALL_ROWS=(
  "ee_mumu|generate e+ e- > mu+ mu-"
  "ee_ee|generate e+ e- > e+ e-"
  "ee_uuxg|generate e+ e- > u u~ g"
  "pp_eej|define p = g u c d s u~ c~ d~ s~;define j = p;generate p p > e+ e- j"
)
ROWS="${ROWS:-$(printf '%s\n' "${ALL_ROWS[@]}" | cut -d'|' -f1 | tr '\n' ' ')}"

# The channels of one finished run, one `P G value err` line each.
channel_results() {
  local procdir="$1"
  for f in "$procdir"/SubProcesses/P*/G*/results.dat; do
    [ -s "$f" ] || continue
    local g p
    g="$(basename "$(dirname "$f")")"
    p="$(basename "$(dirname "$(dirname "$f")")")"
    awk -v p="$p" -v g="$g" 'NR==1{print p, g, $1, $2}' "$f"
  done
}

RECORDS="$OUT/records_$$.txt"
: > "$RECORDS"
for row in "${ALL_ROWS[@]}"; do
  IFS='|' read -r name proc <<< "$row"
  case " $ROWS " in *" $name "*) ;; *) continue ;; esac
  card="$HERE/config_share_${name}_run_card.dat"
  for seed in $SEEDS; do
    procdir="$OUT/${name}_$seed"
    banked="$OUT/${name}_$seed.channels"
    if [ "${VG_FORCE:-0}" = 1 ] || [ ! -s "$banked" ]; then
      # A parallel build of a fresh directory now and then loses the race for
      # libgammaUPC.a ("archive has no index") and the run fails before it
      # integrates anything; such a directory is regenerated and run again.
      result=""
      for attempt in 1 2 3; do
        rm -rf "$procdir"
        mes_generate_dir "$procdir" "$proc" "$NB_CORE"
        mes_install_card "$card" "$procdir/Cards/run_card.dat" "" "$seed"
        if result="$(mes_run_seed "$procdir" "${name}_$seed" "$OUT/${name}_$seed.log")"; then
          break
        fi
        echo "!!! [$name $seed] attempt $attempt failed" >&2
        result=""
      done
      [ -n "$result" ] || exit 1
      python3 "$HERE/config_share_dir.py" "$procdir" "$result" > "$banked.tmp"
      channel_results "$procdir" >> "$banked.tmp"
      mv "$banked.tmp" "$banked"
      [ "${KEEP:-0}" = 1 ] || rm -rf "$procdir"
    else
      echo ">>> [$name $seed] banked: $banked" >&2
    fi
    printf '%s|%s|%s|%s\n' "$name" "$proc" "$seed" "$banked" >> "$RECORDS"
  done
done

python3 "$HERE/config_share_dir.py" --write "$RECORDS" "$RESULT_JSON" \
  "$ROOT/research/refs/mg5amcnlo/VERSION"
rm -f "$RECORDS"
echo ">>> wrote $RESULT_JSON" >&2
