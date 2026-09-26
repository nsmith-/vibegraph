#!/usr/bin/env bash
# MadEvent cross sections for the forbidden on-shell s-channel (`$ A`) gate.
#
# `$ A` keeps every diagram and rejects a phase-space point in each integration
# configuration whose marked `A` line is inside |m - M| < bwcutoff * Gamma
# (myamp.f's cut_bw, gForceBW = 2); banner.py forces sde_strategy = 1 for such
# a process. Each row below is one process under one run card, run over SEEDS
# so the gate reads MadEvent's own seed spread and not only its quoted error.
#
# Rows, all on `import model sm` (restrict_default):
#
#   ee_z_pole       e+ e- > mu+ mu- $ z     sqrt(s) = M_Z: every point is inside
#                                           the Z window
#   ee_z_100        e+ e- > mu+ mu- $ z     sqrt(s) = 100: inside at bwcutoff 15
#   ee_z_100_bw3    e+ e- > mu+ mu- $ z     sqrt(s) = 100, bwcutoff 3: outside
#   ee_z_200        e+ e- > mu+ mu- $ z     sqrt(s) = 200: outside
#   ee_pole, ee_100 e+ e- > mu+ mu-         the unrestricted process, the
#                                           comparison a veto that never fired
#                                           would match
#   uu_tt           u u~ > w+ b w- b~ $ t t~ fixed sqrt(s) = 500, both top
#                                           windows vetoed pointwise
#   uu_full         u u~ > w+ b w- b~       the unrestricted process
#   t_bev_w         t > b e+ ve $ w+        the one-incoming case: the W window
#                                           removed from the partial width
#   pp_z            p p > e+ e- $ z         dy13_default_run_card.dat
#   pp_full         p p > e+ e-             the same card
#   pp_z_chain      p p > z, z > e+ e-      the same card with cut_decays = T, so
#                                           the decay leptons see the cuts the
#                                           other two rows apply
#
# The e+ e-, u u~ and top rows start from a committed copy of MadGraph's own
# generated run card for the process (onshell_*_run_card.dat, written from the
# generated default when absent) with only the beams (and bwcutoff) changed;
# every row gets nevents and iseed. The Rust gate applies the same overrides to
# the same files.
#
# Usage: pixi run -e madgraph bash validation/madgraph/gen_onshell_veto.sh
#        (ROWS="ee_z_pole ee_z_200" and SEEDS="1 2" narrow a run; rows already in
#        the JSON are kept)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="${ONSHELL_WORK:-$ROOT/target/mg_onshell_veto}"
RESULT_JSON="$HERE/onshell_veto_reference.json"
SEEDS="${SEEDS:-$(seq -s " " 20260925 20260929)}"
NB_CORE="${NB_CORE:-2}"

mkdir -p "$OUT"

# name|proc lines (';'-separated)|base card|overrides
ALL_ROWS=(
  "ee_z_pole|generate e+ e- > mu+ mu- \$ z|onshell_ee_run_card.dat|ebeam1=45.594;ebeam2=45.594;nevents=10000"
  "ee_z_100|generate e+ e- > mu+ mu- \$ z|onshell_ee_run_card.dat|ebeam1=50;ebeam2=50;nevents=10000"
  "ee_z_100_bw3|generate e+ e- > mu+ mu- \$ z|onshell_ee_run_card.dat|ebeam1=50;ebeam2=50;bwcutoff=3;nevents=10000"
  "ee_z_200|generate e+ e- > mu+ mu- \$ z|onshell_ee_run_card.dat|ebeam1=100;ebeam2=100;nevents=10000"
  "ee_pole|generate e+ e- > mu+ mu-|onshell_ee_run_card.dat|ebeam1=45.594;ebeam2=45.594;nevents=10000"
  "ee_100|generate e+ e- > mu+ mu-|onshell_ee_run_card.dat|ebeam1=50;ebeam2=50;nevents=10000"
  "uu_tt|generate u u~ > w+ b w- b~ \$ t t~|onshell_uu_run_card.dat|lpp1=0;lpp2=0;ebeam1=250;ebeam2=250;nevents=10000"
  "uu_full|generate u u~ > w+ b w- b~|onshell_uu_run_card.dat|lpp1=0;lpp2=0;ebeam1=250;ebeam2=250;nevents=10000"
  "t_bev_w|generate t > b e+ ve \$ w+|onshell_t_bev_run_card.dat|nevents=10000"
  "pp_z|generate p p > e+ e- \$ z|dy13_default_run_card.dat|nevents=20000"
  "pp_full|generate p p > e+ e-|dy13_default_run_card.dat|nevents=20000"
  "pp_z_chain|generate p p > z, z > e+ e-|dy13_default_run_card.dat|nevents=20000;cut_decays=True"
)
ROWS="${ROWS:-$(printf '%s\n' "${ALL_ROWS[@]}" | cut -d'|' -f1 | tr '\n' ' ')}"

# Generate one process directory (idempotent), silenced for a batch run.
generate_dir() {
  local procdir="$1" proc="$2"
  if [ ! -f "$procdir/bin/generate_events" ]; then
    echo ">>> generating '$proc' into $procdir ..." >&2
    local script
    script="$(mktemp -t gen_onshell_XXXX).mg5"
    { echo "import model sm"; echo "$proc" | tr ';' '\n'; echo "output $procdir -nojpeg"; } > "$script"
    bash "$HERE/mg5_pinned.sh" "$script" >&2
    rm -f "$script"
    cp "$procdir/Cards/run_card.dat" "$procdir/Cards/run_card_default.dat"
  fi
  local cfg="$procdir/Cards/me5_configuration.txt"
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center|run_mode|nb_core)\s*=' "$cfg" > "$cfg.tmp"
  printf 'automatic_html_opening = False\nnotification_center = False\nrun_mode = 2\nnb_core = %s\n' "$NB_CORE" >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
}

# Write `base` with `overrides` (k=v;...) and iseed applied to the process's run
# card; a key the base card does not carry is appended.
install_card() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import re, sys
src, dst, overrides, seed = sys.argv[1:5]
text = open(src).read()
pairs = [kv.split("=", 1) for kv in overrides.split(";") if kv] + [["iseed", seed]]
for key, value in pairs:
    pat = r"^\s*\S+\s*=\s*%s\b" % re.escape(key)
    text, n = re.subn(pat, "  %s = %s" % (value, key), text, flags=re.M | re.I)
    assert n <= 1, "%s found %d times" % (key, n)
    if n == 0:
        text += "  %s = %s\n" % (value, key)
open(dst, "w").write(text)
PY
}

RESULTS="$OUT/rows_$$.txt"
: > "$RESULTS"
for row in "${ALL_ROWS[@]}"; do
  IFS='|' read -r name proc base overrides <<< "$row"
  case " $ROWS " in *" $name "*) ;; *) continue ;; esac
  key="$(echo "$proc" | tr -c 'a-zA-Z0-9\n' '_' | tr -s '_')"
  procdir="$OUT/$key"
  generate_dir "$procdir" "$proc"
  base_path="$HERE/$base"
  [ -f "$base_path" ] || cp "$procdir/Cards/run_card_default.dat" "$base_path"
  sde="$(awk '/= *sde_strategy/{print $1}' "$procdir/Cards/run_card_default.dat")"
  for seed in $SEEDS; do
    install_card "$base_path" "$procdir/Cards/run_card.dat" "$overrides" "$seed"
    log="$OUT/${name}_$seed.log"
    echo ">>> [$name] madevent, iseed $seed ..." >&2
    rm -rf "$procdir/Events/run_${name}_$seed"
    "$procdir/bin/generate_events" -f "run_${name}_$seed" > "$log" 2>&1 || {
      echo "!!! [$name] generate_events failed; see $log" >&2; tail -40 "$log" >&2; exit 1; }
    read -r sigma err < <(awk 'NR==1{printf "%.10g %.10g\n", $1, $2}' "$procdir/SubProcesses/results.dat")
    printf '%s|%s|%s|%s|%s|%s|%s|%s\n' "$name" "$proc" "$base" "$overrides" "$sde" "$seed" "$sigma" "$err" >> "$RESULTS"
    echo "    $name $seed: $sigma +- $err" >&2
  done
done

python3 - "$RESULTS" "$RESULT_JSON" "$ROOT/research/refs/mg5amcnlo/VERSION" <<'PY'
import json, os, re, sys
rows_path, out_path, version_path = sys.argv[1:4]
old = json.load(open(out_path))["rows"] if os.path.exists(out_path) else {}
rows = {}
for line in open(rows_path):
    name, proc, base, overrides, sde, seed, sigma, err = line.rstrip("\n").split("|")
    row = rows.setdefault(name, {
        "process": proc.replace("generate ", "", 1),
        "run_card": base,
        "overrides": overrides,
        "generated_sde_strategy": int(sde) if sde else None,
        "runs": [],
    })
    row["runs"].append({"iseed": int(seed), "value": float(sigma), "err": float(err)})
old.update(rows)
version = re.search(r"version\s*=\s*(\S+)", open(version_path).read()).group(1)
out = {
    "_comment": "MadEvent results.dat values for the forbidden on-shell s-channel ($) gate, "
                "one run per iseed: cross sections in pb, the t > b e+ ve $ w+ row a "
                "partial width in GeV. Generated by validation/madgraph/gen_onshell_veto.sh.",
    "mg_version": version,
    "rows": dict(sorted(old.items())),
}
json.dump(out, open(out_path, "w"), indent=2)
print("wrote", out_path)
PY
