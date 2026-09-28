#!/usr/bin/env bash
# MadEvent references for MLM-matched leading-order generation (ickkw = 1 and
# the xqcut cut), one run per seed.
#
# Rows (proc card: scripts/<row>.mg5, run card: <row>_run_card.dat):
#
#   pp_to_llj_mlm         p p > e+ e- j, ickkw = 1, xqcut = 20
#   pp_to_llj_xqcut_only  the same with ickkw = 0: xqcut as a pure cut
#   pp_to_llj_mlm_alps2   pp_to_llj_mlm with alpsfact = 2 and use_syst = F
#   pp_to_ll_0j2j_mlm     p p > e+ e- @0, + j @1, + j j @2, ickkw = 1, xqcut = 20
#   pp_to_ttx_0j1j_mlm    p p > t t~ @0, + j @1, ickkw = 1, xqcut = 30
#
# Each row runs over its seeds (ten by default: the gates reading these are
# expected to be tighter than 0.3%, and the seed policy in
# validation/manifest.toml asks ten there).
#
#   * The first seed is the samples-grade run: it goes to output/<row>, the
#     banked layout (Events/run_01), so the reference bundle picks up its event
#     file and gen_kt_cluster_dumps.sh can replay it instrumented. A replay
#     reproduces a run only if that run was the first its process directory
#     made, so run_01 is only ever made in a freshly generated directory.
#   * The other seeds are cross-section draws in work/mlm/<row>, which the
#     bundle does not carry. Every run makes the run card's full event count.
#
# The run card is copied verbatim from the committed <row>_run_card.dat (with
# iseed set per seed). Before any run, every `set` line of the .mg5 script's
# launch block is checked against that card, so the proc card documents the run
# that was made. After each run the banner is checked for vector_size = 1: on
# the vector path SCALUP reads the lowered matrix-element PDF scale instead of
# q2bck (auto_dsig_v4.inc), so a reference taken there records a different
# event header.
#
# Only the per-seed scalars are committed (mlm_sigma_reference.json), the same
# shape as decay_chain_sigma_reference.json with each row's proc lines added.
# Runs are cached as madevent_seeds.sh describes; VG_FORCE=1 re-runs them.
#
# Three stages, all by default (MLM_STAGE=runs|dumps|census picks one):
#
#   runs    the per-seed MadEvent runs above
#   dumps   the instrumented replay of each row's samples-grade run
#           (gen_kt_cluster_dumps.sh; it must reproduce the banked event file
#           byte for byte), pinned in mlm_dump_manifest.json. The raw Fortran
#           shards are written through gzip and dropped once each dump is
#           extracted.
#   census  mlm_census.json (dump_mlm_census.py): jet-ness mixed within an IPROC,
#           and the jet memo's re-cluster branches from the dumps
#
# Usage: pixi run -e madgraph generate-mlm-references
#        ROWS="pp_to_llj_mlm" SEEDS="1 2" ... to run a subset; rows not run keep
#        their committed entries, and a row that is run is replaced by the seeds
#        of this invocation.
#        NB_CORE (default 2) is the number of cores each madevent run uses.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
. "$HERE/madevent_seeds.sh"
OUT="$HERE/output"
WORK="${MLM_WORK:-$HERE/work/mlm}"
RESULT_JSON="${RESULT_JSON:-$HERE/mlm_sigma_reference.json}"
NB_CORE="${NB_CORE:-2}"
USER_SEEDS="${SEEDS:-}"
DEFAULT_SEEDS="$(seq -f '%.0f' -s " " 20260928 20260937)"

LHAPDF_DATA_PATH="$ROOT/validation/pdf"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

ALL_ROWS=(pp_to_llj_mlm pp_to_llj_xqcut_only pp_to_llj_mlm_alps2 pp_to_ll_0j2j_mlm pp_to_ttx_0j1j_mlm)
SELECTED="${ROWS:-${ALL_ROWS[*]}}"

die() { printf '!!! %s\n' "$*" >&2; exit 1; }

# launch_sets ROW — the `set key value` lines of the script's launch block.
launch_sets() {
  sed -n '/^[[:space:]]*launch[[:space:]]*$/,/^[[:space:]]*done[[:space:]]*$/p' "$HERE/scripts/$1.mg5" |
    awk '$1 == "set" { print $2, $3 }'
}

# check_card ROW — every launch-block setting is what the committed card says.
check_card() {
  local row="$1" card="$HERE/$1_run_card.dat"
  [ -f "$card" ] || die "$row: no committed run card $card"
  launch_sets "$row" | python3 -c '
import re, sys
card = open(sys.argv[1]).read()
bad = []
for line in sys.stdin:
    key, want = line.split()
    m = re.search(r"^\s*(\S+)\s*=\s*%s\b" % re.escape(key), card, re.M | re.I)
    got = m.group(1) if m else None
    try:
        same = got is not None and float(got) == float(want)
    except ValueError:
        same = got is not None and got.lower() == want.lower()
    if not same:
        bad.append("%s: script %s, card %s" % (key, want, got))
if bad:
    sys.exit("run card disagrees with the script launch block: " + "; ".join(bad))
' "$card" || die "$row: $HERE/scripts/$row.mg5 and $card disagree"
}

# generate_dir ROW DIR — the script's proc lines, with `output` redirected.
generate_dir() {
  local row="$1" procdir="$2"
  echo ">>> [$row] generating $procdir" >&2
  local driver
  driver="$(mktemp "${TMPDIR:-/tmp}/vg_mlm_XXXXXX").mg5"
  sed -E "s|^([[:space:]]*output[[:space:]]+)[^[:space:]]+|\1$procdir|" "$HERE/scripts/$row.mg5" |
    sed -E '/^[[:space:]]*launch[[:space:]]*$/,$d' > "$driver"
  grep -qE "^[[:space:]]*output[[:space:]]+$procdir( |$)" "$driver" ||
    die "$row: .mg5 script has no 'output' line to redirect"
  bash "$HERE/mg5_pinned.sh" "$driver" >&2
  rm -f "$driver"
  mes_configure_dir "$procdir" "$NB_CORE"
}

# check_banner ROW RUNDIR — the run was made on the scalar path with the card's
# matching settings.
check_banner() {
  local row="$1" rundir="$2"
  local banner
  banner="$(ls "$rundir"/*_banner.txt 2>/dev/null | head -1)"
  [ -n "$banner" ] || die "$row: no banner in $rundir"
  python3 - "$banner" "$HERE/${row}_run_card.dat" <<'PY' || die "$row: banner check failed for $rundir"
import re, sys
banner, card = open(sys.argv[1]).read(), open(sys.argv[2]).read()
def val(text, key):
    m = re.search(r"^\s*(\S+)\s*=\s*%s\b" % re.escape(key), text, re.M | re.I)
    return m.group(1) if m else None
vs = val(banner, "vector_size")
assert vs is not None and int(vs) == 1, "vector_size = %r, the scalar path needs 1" % vs
for key in ("ickkw", "xqcut", "alpsfact", "use_syst", "maxjetflavor"):
    want = val(card, key)
    if want is None:
        continue
    got = val(banner, key)
    try:
        ok = float(got) == float(want)
    except (TypeError, ValueError):
        ok = str(got).lower().startswith(str(want).lower()[:1])
    assert ok, "%s: card %s, banner %s" % (key, want, got)
PY
}

MLM_STAGE="${MLM_STAGE:-all}"
SELECTED_ROWS=()
for row in "${ALL_ROWS[@]}"; do
  case " $SELECTED " in *" $row "*) SELECTED_ROWS+=("$row") ;; esac
done

if [ "$MLM_STAGE" = dumps ] || [ "$MLM_STAGE" = census ]; then
  RUNS=0
else
  RUNS=1
fi

RESULTS="$WORK/rows_$$.txt"
mkdir -p "$WORK"
: > "$RESULTS"
for row in "${ALL_ROWS[@]}"; do
  [ "$RUNS" = 1 ] || break
  case " $SELECTED " in *" $row "*) ;; *) continue ;; esac
  check_card "$row"
  card="$HERE/${row}_run_card.dat"
  seeds="${USER_SEEDS:-$DEFAULT_SEEDS}"
  first="${seeds%% *}"
  rest="${seeds#"$first"}"

  # The samples-grade seed, as the first run of a fresh output/<row>.
  procdir="$OUT/$row"
  record="$procdir/Events/run_01/vg_seed_result.txt"
  cached=0
  if [ -f "$procdir/bin/generate_events" ] && [ -s "$record" ] && [ "${VG_FORCE:-0}" != 1 ]; then
    tmpcard="$(mktemp)"
    mes_install_card "$card" "$tmpcard" "" "$first"
    read -r _ _ _ sha < "$record"
    [ "$sha" = "$(mes_sha256 "$tmpcard")" ] && cached=1
    rm -f "$tmpcard"
  fi
  if [ "$cached" != 1 ]; then
    rm -rf "$procdir"
    generate_dir "$row" "$procdir"
  fi
  mes_install_card "$card" "$procdir/Cards/run_card.dat" "" "$first"
  result="$(mes_run_seed "$procdir" 01 "$WORK/${row}_run_01.log")"
  check_banner "$row" "$procdir/Events/run_01"
  read -r sigma err wall <<< "$result"
  printf '%s|%s|%s|%s|%s|samples|%s\n' "$row" "$first" "$sigma" "$err" "$wall" "$procdir/Events/run_01" | tee -a "$RESULTS" >&2

  # The cross-section seeds.
  sdir="$WORK/$row"
  [ -f "$sdir/bin/generate_events" ] || generate_dir "$row" "$sdir"
  mes_configure_dir "$sdir" "$NB_CORE"
  for seed in $rest; do
    mes_install_card "$card" "$sdir/Cards/run_card.dat" "" "$seed"
    result="$(mes_run_seed "$sdir" "s$seed" "$WORK/${row}_$seed.log")"
    check_banner "$row" "$sdir/Events/run_s$seed"
    read -r sigma err wall <<< "$result"
    printf '%s|%s|%s|%s|%s|sigma|%s\n' "$row" "$seed" "$sigma" "$err" "$wall" "$sdir/Events/run_s$seed" | tee -a "$RESULTS" >&2
  done
done

[ "$RUNS" = 1 ] && python3 - "$RESULTS" "$RESULT_JSON" "$ROOT/research/refs/mg5amcnlo/VERSION" "$HERE" "$NB_CORE" <<'PY'
import gzip, json, os, re, sys
rows_path, out_path, version_path, here, nb_core = sys.argv[1:6]

def init_processes(rundir):
    # The <init> block's per-process lines: XSECUP XERRUP XMAXUP LPRUP. On a
    # card with `@N` process tags LPRUP is N, so this is sigma per multiplicity.
    with gzip.open(os.path.join(rundir, "unweighted_events.lhe.gz"), "rt") as f:
        text = f.read(200000)
    block = text[text.index("<init>") + 6 : text.index("</init>")].strip().splitlines()
    nprup = int(block[0].split()[-1])
    out = {}
    for line in block[1 : 1 + nprup]:
        xsec, xerr, _xmax, lprup = line.split()[:4]
        out[lprup] = {"sigma_pb": float(xsec), "err_pb": float(xerr)}
    return out

old = json.load(open(out_path))["rows"] if os.path.exists(out_path) else {}
rows = {}
for line in open(rows_path):
    name, seed, sigma, err, wall, kind, rundir = line.strip().split("|")
    if name not in rows:
        script = open(os.path.join(here, "scripts", name + ".mg5")).read()
        proc = [ln.strip() for ln in script.splitlines()
                if re.match(r"\s*(generate|add process)\b", ln)]
        card = open(os.path.join(here, name + "_run_card.dat")).read()
        nev = re.search(r"^\s*(\d+)\s*=\s*nevents\b", card, re.M).group(1)
        rows[name] = {"process": proc, "script": "scripts/%s.mg5" % name,
                      "run_card": "%s_run_card.dat" % name, "nevents": int(nev),
                      "runs": []}
    rows[name]["runs"].append({"iseed": int(seed), "sigma_pb": float(sigma),
                               "err_pb": float(err), "wall_s": int(wall),
                               "samples": kind == "samples",
                               "by_lprup": init_processes(rundir)})
version = re.search(r"version\s*=\s*(\S+)", open(version_path).read()).group(1)
out = {
    "_comment": "MadEvent cross sections (pb) for MLM-matched LO generation (ickkw = 1 / "
                "xqcut), one run per iseed, each the run card's full unweighted event count. "
                "The run marked samples is output/<row>/Events/run_01, the one the reference "
                "bundle carries and the instrumented replay reproduces; the others ran in "
                "work/mlm/<row>. wall_s is the whole generate_events call on %s cores, survey, "
                "refine and compilation included. by_lprup is the run's own <init> "
                "block per process (LPRUP = the @N tag where the card has them). Generated by "
                "validation/madgraph/gen_mlm_references.sh." % nb_core,
    "mg_version": version,
    "rows": dict(old, **rows),
}
json.dump(out, open(out_path, "w"), indent=2)
print("wrote", out_path)
PY
rm -f "$RESULTS"

if [ "$MLM_STAGE" = all ] || [ "$MLM_STAGE" = dumps ]; then
  VG_KT_MANIFEST=mlm_dump_manifest.json VG_NB_CORE="$NB_CORE" VG_KT_DROP_RAW=1 VG_KTDUMP_GZIP=1 \
    bash "$HERE/gen_kt_cluster_dumps.sh" "${SELECTED_ROWS[@]}"
fi
if [ "$MLM_STAGE" = all ] || [ "$MLM_STAGE" = census ]; then
  python3 "$HERE/dump_mlm_census.py" "${SELECTED_ROWS[@]}"
fi
