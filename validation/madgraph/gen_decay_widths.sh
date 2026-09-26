#!/usr/bin/env bash
# MadEvent partial widths for the 1 -> n decay gate, and the binned event
# distributions its sample row compares against.
#
# A decay process (`generate t > b e+ ve`) is an ordinary MadGraph process with
# one initial particle: MadEvent integrates it in the particle's rest frame to a
# partial width in GeV, which is what results.dat, XSECUP and XWGTUP then carry.
# setcuts.f's `nincoming = 1` branch decides how the run card is read -- no
# parton densities, the renormalisation scale fixed at the mother's mass, the
# factorisation scales fixed at the card's constants -- and banner.py writes a
# decay's default card with every cut removed (`remove_all_cut`) and
# `sde_strategy = 1`.
#
# Rows, all on `import model sm` (restrict_default):
#
#   t_wb, t_bev, h_4l, z_ee   the process's own generated run card, untouched but
#                             for iseed -- MadGraph's decay default, no cuts
#   h_4l_cuts, t_bev_cuts     the committed decay_*_cuts_run_card.dat, the same
#                             default with lepton / b cuts set, applied by cuts.f
#                             in the rest frame
#
# Each row runs over SEEDS (ten by default): one MadEvent run is one draw, and
# the gate reads MadEvent's own seed spread rather than its quoted error. They
# differ: on h > e+ e- mu+ mu- twelve seeds scatter by 0.29%, twice the 0.14%
# each run quotes. The first seed's t_bev events
# are binned into the distributions the sample row compares against.
#
# Committed: a handful of scalars and two histograms, cheap to regenerate (a
# decay run takes seconds) but written by MadGraph, so they travel in git like
# higgs_window_reference.json rather than in the fetched bundle.
#
# The process directories and their runs stay in DECAY_WORK (default
# validation/madgraph/work/decay_widths), cached as madevent_seeds.sh describes:
# a rerun reads every finished seed back. Everything the gate reads is in the
# JSON; the directories stay out of output/, whose runs the banked gates
# inventory.
#
# Usage: pixi run -e madgraph generate-decay-widths
#        ROWS="t_wb z_ee" SEEDS="1 2" ... to run a subset; rows not run keep
#        their committed entries.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
. "$HERE/madevent_seeds.sh"
OUT="${DECAY_WORK:-$HERE/work/decay_widths}"
RESULT_JSON="${RESULT_JSON:-$HERE/decay_width_reference.json}"
SEEDS="${SEEDS:-$(seq -s " " 20260925 20260934)}"

mkdir -p "$OUT"

ROWS_ALL=(
  "t_wb|t > w+ b|"
  "t_bev|t > b e+ ve|"
  "h_4l|h > e+ e- mu+ mu-|"
  "z_ee|z > e+ e-|"
  "h_4l_cuts|h > e+ e- mu+ mu-|decay_h4l_cuts_run_card.dat"
  "t_bev_cuts|t > b e+ ve|decay_tbev_cuts_run_card.dat"
)
SELECTED="${ROWS:-t_wb t_bev h_4l z_ee h_4l_cuts t_bev_cuts}"

RESULTS="$OUT/rows_$$.txt"
: > "$RESULTS"
for row in "${ROWS_ALL[@]}"; do
  IFS='|' read -r name process card <<< "$row"
  case " $SELECTED " in *" $name "*) ;; *) continue ;; esac
  procdir="$OUT/decay_${name%_cuts}"
  mes_generate_dir "$procdir" "generate $process"
  if [ -n "$card" ]; then
    card_path="$HERE/$card"
  else
    card_path="$procdir/Cards/run_card_default.dat"
  fi
  for seed in $SEEDS; do
    mes_install_card "$card_path" "$procdir/Cards/run_card.dat" "" "$seed"
    result="$(mes_run_seed "$procdir" "${name}_$seed" "$OUT/decay_${name}_$seed.log")"
    read -r width err _ <<< "$result"
    printf '%s|%s|%s|%s|%s|%s\n' "$name" "$process" "$card" "$seed" "$width" "$err" >> "$RESULTS"
  done
done

first_seed="${SEEDS%% *}"
EVENTS="$OUT/decay_t_bev/Events/run_t_bev_$first_seed/unweighted_events.lhe.gz"
case " $SELECTED " in *" t_bev "*) ;; *) EVENTS="" ;; esac

python3 - "$RESULTS" "$EVENTS" "$RESULT_JSON" "$ROOT/research/refs/mg5amcnlo/VERSION" <<'PY'
import gzip, json, math, os, re, sys
rows_path, events_path, out_path, version_path = sys.argv[1:5]
old = json.load(open(out_path)) if os.path.exists(out_path) else {}

rows = {}
for line in open(rows_path):
    name, process, card, seed, width, err = line.strip().split("|")
    row = rows.setdefault(name, {"process": process, "run_card": card or None, "runs": []})
    row["runs"].append({"iseed": int(seed), "width_gev": float(width), "err_gev": float(err)})

# Bin m(e+ ve) and m(b e+) over MadEvent's own unweighted t > b e+ ve sample.
# Final-state legs are read by PDG code, so the intermediate W record MadEvent
# writes (status 2) is skipped.
# m(e+ ve) is resolved around the W pole, where the Breit-Wigner mapping decides
# the shape; m(b e+) spans its whole kinematic range, up to M_t.
EDGES = {
    "m_eve": [0.0, 20.0, 40.0, 60.0, 70.0, 75.0, 78.0, 79.0, 80.0, 80.5, 81.0, 82.0,
              85.0, 90.0, 110.0, 130.0, 168.3],
    "m_be": [10.0 * i for i in range(18)] + [173.0],
}
counts = {k: [0] * (len(v) - 1) for k, v in EDGES.items()}
init = None
n = 0
def mass(*ps):
    e, x, y, z = (sum(p[i] for p in ps) for i in range(4))
    return math.sqrt(max(e * e - x * x - y * y - z * z, 0.0))
text = ""
if events_path:
    with gzip.open(events_path, "rt") as fh:
        text = fh.read()
    init = re.search(r"<init>\s*\n(.*?)\n(.*?)\n", text).groups()
for block in re.findall(r"<event>\s*\n(.*?)</event>", text, flags=re.S):
    lines = [l.split() for l in block.strip().splitlines() if l.strip() and not l.startswith("#") and not l.startswith("<")]
    parts = lines[1:int(lines[0][0]) + 1]
    mom = {}
    for p in parts:
        if int(p[1]) == 1:
            mom[int(p[0])] = [float(p[9]), float(p[6]), float(p[7]), float(p[8])]
    b, ep, ve = mom[5], mom[-11], mom[12]
    for key, value in (("m_eve", mass(ep, ve)), ("m_be", mass(b, ep))):
        edges = EDGES[key]
        for i in range(len(edges) - 1):
            if edges[i] <= value < edges[i + 1]:
                counts[key][i] += 1
                break
    n += 1

version = re.search(r"version\s*=\s*(\S+)", open(version_path).read()).group(1)
out = {
    "_comment": "MadEvent partial widths (GeV) for 1 -> n decays on `import model sm`, "
                "one run per iseed, and m(e+ ve) / m(b e+) histograms of the first seed's "
                "t > b e+ ve sample. Generated by validation/madgraph/gen_decay_widths.sh.",
    "mg_version": version,
    "rows": dict(old.get("rows", {}), **rows),
    "t_bev_sample": {
        "events": n,
        "init": list(init),
        "edges": EDGES,
        "counts": counts,
    } if events_path else old["t_bev_sample"],
}
json.dump(out, open(out_path, "w"), indent=2)
print("wrote", out_path)
PY
rm -f "$RESULTS"
