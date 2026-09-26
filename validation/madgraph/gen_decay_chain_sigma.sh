#!/usr/bin/env bash
# MadEvent cross sections for decay-chain processes (`p p > t t~, t > w+ b, ...`).
#
# A decay chain is a process whose decaying legs are forced on shell: MadEvent
# flags each decay's propagator `gForceBW = 1` and `cut_bw` (myamp.f) rejects a
# point whose resonance mass lies outside `bwcutoff` widths of its pole. The
# gated quantity is that decay-chain cross section itself, not sigma x BR.
#
# Rows, all on `import model sm` (restrict_default), each over SEEDS (five by
# default; one MadEvent run is one draw, and the gate reads the seed spread as
# well as the quoted error):
#
#   zz_emu          e+ e- > z z, z > e+ e-, z > mu+ mu-              ee500
#   zz_emu_cut      the same with cut_decays = T                      ee500_cutdecays
#   ttx_wb          e+ e- > t t~, t > w+ b, t~ > w- b~                ee500
#   ttx_nested      e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~  ee500
#   pp_ttx_lep      p p > t t~, t > b e+ ve, t~ > b~ mu- vm~          pp13
#   zz_ee           e+ e- > z z, z > e+ e-                            ee500
#
# `zz_emu_cut` is banked over twenty seeds (the default five, then
# SEEDS="$(seq -s ' ' 20260931 20260945)" ROWS=zz_emu_cut, merged): MadEvent's
# seeds on it scatter four times wider than they quote, with a one-sided low
# tail, and five of them read χ²/dof 90.
#
# `zz_ee` has identical particles across its two decays. MadGraph keeps one
# pairing of the four leptons and divides by `identical_decay_chain_factor`;
# vibegraph keeps both pairings and their interference, so that row is
# informational.
#
# The run cards are the committed decay_chain_*_run_card.dat, copied verbatim
# into each process directory, so both sides read the same beams, cuts and
# scales. Only a handful of scalars are committed (decay_chain_sigma_reference.json);
# the process directories are scratch.
#
# Usage: pixi run -e madgraph bash validation/madgraph/gen_decay_chain_sigma.sh
#        ROWS="zz_emu ttx_wb" SEEDS="1 2" ... to run a subset.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="${DECAY_CHAIN_WORK:-$(mktemp -d "${TMPDIR:-/tmp}/vg-decay-chain-XXXXXX")}"
RESULT_JSON="${RESULT_JSON:-$HERE/decay_chain_sigma_reference.json}"
SEEDS="${SEEDS:-$(seq -s " " 20260926 20260930)}"

LHAPDF_DATA_PATH="$ROOT/validation/pdf"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

mkdir -p "$OUT"

ALL_ROWS=(
  "zz_emu|e+ e- > z z, z > e+ e-, z > mu+ mu-|decay_chain_ee500_run_card.dat"
  "zz_emu_cut|e+ e- > z z, z > e+ e-, z > mu+ mu-|decay_chain_ee500_cutdecays_run_card.dat"
  "ttx_wb|e+ e- > t t~, t > w+ b, t~ > w- b~|decay_chain_ee500_run_card.dat"
  "ttx_nested|e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~|decay_chain_ee500_run_card.dat"
  "pp_ttx_lep|p p > t t~, t > b e+ ve, t~ > b~ mu- vm~|decay_chain_pp13_run_card.dat"
  "zz_ee|e+ e- > z z, z > e+ e-|decay_chain_ee500_run_card.dat"
)
SELECTED="${ROWS:-zz_emu zz_emu_cut ttx_wb ttx_nested pp_ttx_lep zz_ee}"

# Generate one process directory (idempotent), silenced for a batch run.
generate_dir() {
  local procdir="$1" process="$2"
  if [ ! -f "$procdir/bin/generate_events" ]; then
    echo ">>> generating $process into $procdir ..." >&2
    local script
    script="$(mktemp -t gen_chain_XXXX).mg5"
    printf 'import model sm\ngenerate %s\noutput %s -nojpeg\n' "$process" "$procdir" > "$script"
    bash "$HERE/mg5_pinned.sh" "$script" >&2
    rm -f "$script"
  fi
  local cfg="$procdir/Cards/me5_configuration.txt"
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center|run_mode|nb_core)\s*=' "$cfg" > "$cfg.tmp"
  printf 'automatic_html_opening = False\nnotification_center = False\nrun_mode = 2\nnb_core = 2\n' >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
}

# Install a run card with its iseed set, run madevent, and echo
# "<sigma> <err>" (pb) from results.dat.
run_one() {
  local procdir="$1" card="$2" seed="$3" tag="$4"
  python3 - "$card" "$procdir/Cards/run_card.dat" "$seed" <<'PY'
import re, sys
src, dst, seed = sys.argv[1:4]
text = open(src).read()
text, n = re.subn(r"^\s*\S+\s*=\s*iseed\b", "  %s = iseed" % seed, text, flags=re.M)
assert n == 1, "iseed not found exactly once"
open(dst, "w").write(text)
PY
  local log="$OUT/chain_$tag.log"
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
  IFS='|' read -r name process card <<< "$row"
  case " $SELECTED " in *" $name "*) ;; *) continue ;; esac
  procdir="$OUT/chain_${name%_cut}"
  generate_dir "$procdir" "$process"
  for seed in $SEEDS; do
    read -r sigma err wall < <(run_one "$procdir" "$HERE/$card" "$seed" "${name}_$seed")
    printf '%s|%s|%s|%s|%s|%s|%s\n' "$name" "$process" "$card" "$seed" "$sigma" "$err" "$wall" | tee -a "$RESULTS" >&2
  done
done

python3 - "$RESULTS" "$RESULT_JSON" "$ROOT/research/refs/mg5amcnlo/VERSION" <<'PY'
import json, re, sys
rows_path, out_path, version_path = sys.argv[1:4]
rows = {}
for line in open(rows_path):
    name, process, card, seed, sigma, err, wall = line.strip().split("|")
    row = rows.setdefault(name, {"process": process, "run_card": card, "runs": []})
    row["runs"].append({"iseed": int(seed), "sigma_pb": float(sigma), "err_pb": float(err),
                        "wall_s": int(wall)})
version = re.search(r"version\s*=\s*(\S+)", open(version_path).read()).group(1)
out = {
    "_comment": "MadEvent decay-chain cross sections (pb) on `import model sm`, one run per "
                "iseed, each of the run card's 10k unweighted events. wall_s is the whole "
                "generate_events call on two cores, survey and refine included. Generated by "
                "validation/madgraph/gen_decay_chain_sigma.sh.",
    "mg_version": version,
    "rows": rows,
}
json.dump(out, open(out_path, "w"), indent=2)
print("wrote", out_path)
PY
