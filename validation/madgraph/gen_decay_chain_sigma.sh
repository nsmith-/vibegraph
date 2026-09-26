#!/usr/bin/env bash
# MadEvent cross sections for decay-chain processes (`p p > t t~, t > w+ b, ...`).
#
# A decay chain is a process whose decaying legs are forced on shell: MadEvent
# flags each decay's propagator `gForceBW = 1` and `cut_bw` (myamp.f) rejects a
# point whose resonance mass lies outside `bwcutoff` widths of its pole. The
# gated quantity is that decay-chain cross section itself, not sigma x BR.
#
# Rows, all on `import model sm` (restrict_default), each over its seeds (five
# by default; one MadEvent run is one draw, and the gate reads the seed spread as
# well as the quoted error):
#
#   zz_emu          e+ e- > z z, z > e+ e-, z > mu+ mu-              ee500
#   zz_emu_cut      the same with cut_decays = T                      ee500_cutdecays
#   ttx_wb          e+ e- > t t~, t > w+ b, t~ > w- b~                ee500
#   ttx_nested      e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~  ee500
#   pp_ttx_lep      p p > t t~, t > b e+ ve, t~ > b~ mu- vm~          pp13
#   zz_ee           e+ e- > z z, z > e+ e-                            ee500
#
# `zz_emu_cut` is banked over twenty seeds (20260926..20260945, its own list
# below): MadEvent's seeds on it scatter four times wider than they quote, with
# a one-sided low tail, and five of them read χ²/dof 90.
#
# `zz_ee` has identical particles across its two decays. MadGraph keeps one
# pairing of the four leptons and divides by `identical_decay_chain_factor`;
# vibegraph keeps both pairings and their interference, so that row is
# informational.
#
# The run cards are the committed decay_chain_*_run_card.dat, copied verbatim
# into each process directory, so both sides read the same beams, cuts and
# scales. Only a handful of scalars are committed (decay_chain_sigma_reference.json);
# the process directories and runs stay in DECAY_CHAIN_WORK (default
# validation/madgraph/work/decay_chain_sigma), cached as madevent_seeds.sh
# describes.
#
# Usage: pixi run -e madgraph generate-decay-chain-sigma
#        ROWS="zz_emu ttx_wb" SEEDS="1 2" ... to run a subset; rows not run keep
#        their committed entries, and a row that is run is replaced by the seeds
#        of this invocation.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
. "$HERE/madevent_seeds.sh"
OUT="${DECAY_CHAIN_WORK:-$HERE/work/decay_chain_sigma}"
RESULT_JSON="${RESULT_JSON:-$HERE/decay_chain_sigma_reference.json}"
# SEEDS, when set, replaces every row's list; otherwise a row runs its own list,
# or DEFAULT_SEEDS.
USER_SEEDS="${SEEDS:-}"
DEFAULT_SEEDS="$(seq -s " " 20260926 20260930)"

LHAPDF_DATA_PATH="$ROOT/validation/pdf"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

mkdir -p "$OUT"

# name|process|run card|seeds (empty: DEFAULT_SEEDS)
ALL_ROWS=(
  "zz_emu|e+ e- > z z, z > e+ e-, z > mu+ mu-|decay_chain_ee500_run_card.dat|"
  "zz_emu_cut|e+ e- > z z, z > e+ e-, z > mu+ mu-|decay_chain_ee500_cutdecays_run_card.dat|$(seq -s " " 20260926 20260945)"
  "ttx_wb|e+ e- > t t~, t > w+ b, t~ > w- b~|decay_chain_ee500_run_card.dat|"
  "ttx_nested|e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~|decay_chain_ee500_run_card.dat|"
  "pp_ttx_lep|p p > t t~, t > b e+ ve, t~ > b~ mu- vm~|decay_chain_pp13_run_card.dat|"
  "zz_ee|e+ e- > z z, z > e+ e-|decay_chain_ee500_run_card.dat|"
)
SELECTED="${ROWS:-zz_emu zz_emu_cut ttx_wb ttx_nested pp_ttx_lep zz_ee}"

RESULTS="$OUT/rows_$$.txt"
: > "$RESULTS"
for row in "${ALL_ROWS[@]}"; do
  IFS='|' read -r name process card row_seeds <<< "$row"
  case " $SELECTED " in *" $name "*) ;; *) continue ;; esac
  procdir="$OUT/chain_${name%_cut}"
  mes_generate_dir "$procdir" "generate $process"
  seeds="${USER_SEEDS:-${row_seeds:-$DEFAULT_SEEDS}}"
  for seed in $seeds; do
    mes_install_card "$HERE/$card" "$procdir/Cards/run_card.dat" "" "$seed"
    result="$(mes_run_seed "$procdir" "${name}_$seed" "$OUT/chain_${name}_$seed.log")"
    read -r sigma err wall <<< "$result"
    printf '%s|%s|%s|%s|%s|%s|%s\n' "$name" "$process" "$card" "$seed" "$sigma" "$err" "$wall" | tee -a "$RESULTS" >&2
  done
done

python3 - "$RESULTS" "$RESULT_JSON" "$ROOT/research/refs/mg5amcnlo/VERSION" <<'PY'
import json, os, re, sys
rows_path, out_path, version_path = sys.argv[1:4]
old = json.load(open(out_path))["rows"] if os.path.exists(out_path) else {}
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
    "rows": dict(old, **rows),
}
json.dump(out, open(out_path, "w"), indent=2)
print("wrote", out_path)
PY
rm -f "$RESULTS"
