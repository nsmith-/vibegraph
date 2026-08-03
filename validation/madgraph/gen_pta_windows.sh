#!/usr/bin/env bash
# MadGraph reference runs for `e+ e- > mu+ mu- a` at 250 + 250 GeV, partitioned
# into windows of the photon transverse momentum.
#
# The banked run of this process disagrees with the vibegraph integral by +0.80%
# (2.8 sigma on the two quoted errors), and `pt(a)` is the observable whose
# shape comparison sits closest to the sample gate's p-value floor. A single
# cross section over the whole phase space cannot say which side mis-covers a
# region: a partition can, because the windows must add up to the unwindowed
# control on each side separately, and a run whose partition does not close on
# its own quoted errors has a coverage miss regardless of how well its total
# agrees with anything.
#
# Two independent windowed estimators are produced per side, because they fail
# differently:
#
#   * `--stage partition-*` imposes the window through `dummy_cuts` in
#     SubProcesses/dummy_fct.f, which passcuts() applies after every other cut
#     and which the phase-space generator never sees. A windowed run therefore
#     integrates the *same* integrand MadGraph already integrated, restricted to
#     the window, and the sum over windows is a genuine audit of the unwindowed
#     run's coverage.
#   * `--stage refocus` imposes it as run-card cuts `pta`/`ptamax` instead.
#     setcuts.f feeds `ptamax` into etmax(i), which the generator reads, so the
#     channel maps re-optimise for the window. That makes it the better estimate
#     of the true windowed cross section and disqualifies it from the closure
#     test -- it is a cross-check on a window, never a term in a sum.
#
# The photon is external leg 5 (`leshouche.inc` gives IDUP = -11,11,-13,13,22).
# The script asserts that rather than assuming it: a silent mismatch would
# window the wrong leg and produce a confidently wrong table.
#
# Every run uses the banked run's own Cards/run_card.dat and Cards/param_card.dat
# verbatim, with only `nevents`, `iseed` and -- for `refocus` alone -- `pta` and
# `ptamax` changed. Cross sections come from SubProcesses/results.dat (survey +
# refine), never from the event count: MadEvent may fail to fill `nevents` in a
# narrow window, and a short event file is not a failed measurement.
#
# Both MadGraph lines on this machine are used, and which one is which matters:
# the reference bank is generated from the pinned submodule at 3.7.1
# (mg5_pinned.sh), while `pixi run -e madgraph` puts the packaged 3.5.7 on PATH.
# The previous reference bank came from 3.5.7, so running both is what turns
# "the reference moved between versions" into a measurement rather than an
# assertion.
#
# Usage:
#   pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage control-371
#   pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage control-357
#   pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage partition-371
#   pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage refocus
#
# Environment overrides: NEVENTS, SEEDS, WINDOWS (1-based indices into the edge
# list), TAG_SUFFIX. Each stage appends to output/pta_windows_<stage>.tsv and
# rebuilds pta_window_reference.json from every TSV present, so stages are
# independent and individually re-runnable.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/output"
BANKED="$OUT/ee_to_mumua"
# Committed: a few dozen scalars, expensive to produce and stable, so they
# travel in git like sigma_reference.json rather than in the fetched bundle.
RESULT_JSON="$HERE/pta_window_reference.json"

# Frozen before any measurement (see the design note): the outer edges are the
# `pta` cut and sqrt(s)/2, the 20 GeV edge is twice the cut, and 39.4 GeV is
# p_RR / cosh(etaa) -- the pt(a) below which no on-shell-Z radiative-return
# event survives the rapidity cut. The two interior edges are the equal-
# population tertiles of the banked sample above that threshold.
EDGES=(10 20 39.4 77 144 250)

# The secondary axis, frozen in the same design section and carrying no verdict
# of its own: below-Z continuum, low shoulder, the Z peak, high shoulder, and the
# m(mumu) -> sqrt(s) non-radiative region. `pt(a)` is a smeared image of the
# structure that carries this cross section -- an on-Z event lands anywhere in
# pt(a) in [39.4, 241.7] depending on eta(a) -- whereas m(mumu) resolves the
# Breit-Wigner directly, so this axis is what says whether a disagreement
# localised in pt(a) sits on the Z peak or in the continuum.
MLL_EDGES=(0 60 86 96 200 500)

STAGE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --stage) STAGE="$2"; shift 2 ;;
    *) echo "!!! unknown argument: $1" >&2; exit 2 ;;
  esac
done
[ -n "$STAGE" ] || {
  echo "usage: $0 --stage <control-371|control-357|partition-371|partition-357|refocus|mll-371>" >&2
  exit 2
}

NEVENTS="${NEVENTS:-100000}"
SEEDS="${SEEDS:-20260803 20260804 20260805 20260806 20260807}"
TAG_SUFFIX="${TAG_SUFFIX:-}"

if [ ! -f "$BANKED/Cards/run_card.dat" ]; then
  echo "!!! the banked run card is missing at $BANKED/Cards/run_card.dat" >&2
  echo "    run 'pixi run fetch-refdata' first" >&2
  exit 1
fi

TSV="$OUT/pta_windows_$STAGE.tsv"
mkdir -p "$OUT"

# A generated process directory carries its own copy of MadGraph's settings and
# `bin/generate_events` reads that copy, so a batch of runs would otherwise open
# a browser page and post a desktop notification per run.
silence_madgraph_ui() {
  local cfg="$1"
  [ -f "$cfg" ] || return 0
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center)\s*=' "$cfg" > "$cfg.tmp"
  {
    printf 'automatic_html_opening = False\n'
    printf 'notification_center = False\n'
  } >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
}

# Generate one template process directory per MadGraph line, and assert that it
# is the process the banked run integrates.
#
# The proc card is deliberately minimal rather than a replay of the banked
# Cards/proc_card_mg5.dat: its `set` lines are version-specific and an unknown
# key aborts generation, so equivalence is established by comparing the
# *generated* diagram topology against the bank instead of by trusting that the
# same switches mean the same thing in both versions.
generate_template() {
  local dir="$1" version="$2"
  if [ -f "$dir/bin/generate_events" ]; then
    echo ">>> template $dir already generated" >&2
  else
    echo ">>> generating $dir with MadGraph $version ..." >&2
    local tmp_mg5
    tmp_mg5="$(mktemp -t gen_ptawin_XXXX).mg5"
    cat > "$tmp_mg5" <<EOF
set gauge unitary
generate e+ e- > mu+ mu- a
output $dir -nojpeg
EOF
    case "$version" in
      3.7.1) bash "$HERE/mg5_pinned.sh" "$tmp_mg5" >&2 ;;
      3.5.7) LDFLAGS="${LDFLAGS:-} -lc++" mg5_aMC "$tmp_mg5" >&2 ;;
      *) echo "!!! unknown MadGraph version $version" >&2; exit 1 ;;
    esac
    rm -f "$tmp_mg5"
  fi

  local got
  got="$(cat "$dir/MGMEVersion.txt")"
  if [ "$got" != "$version" ]; then
    echo "!!! $dir reports MadGraph $got, expected $version" >&2
    exit 1
  fi

  # The window is cut on leg 5. Assert the leg order rather than assume it.
  local les
  les="$(find "$dir/SubProcesses" -name leshouche.inc | head -1)"
  if ! grep -q 'IDUP(I,1,1),I=1,5)/ *-11, *11, *-13, *13, *22/' "$les"; then
    echo "!!! leg order is not (-11,11,-13,13,22) in $les" >&2
    grep 'IDUP' "$les" >&2
    exit 1
  fi
  echo ">>> [$version] leg order asserted: $(grep 'IDUP(I,1,1)' "$les")" >&2

  # `dummy_cuts` must be the stock one-line body the window patch replaces.
  local n
  n="$(grep -c '^      dummy_cuts=\.true\.$' "$dir/SubProcesses/dummy_fct.f" || true)"
  if [ "$n" != "1" ]; then
    echo "!!! dummy_cuts body appears $n times in $dir/SubProcesses/dummy_fct.f" >&2
    exit 1
  fi

  # The channel decomposition the partition audits: same configs as the bank.
  local banked_cfg gen_cfg
  banked_cfg="$(find "$BANKED/SubProcesses" -name configs.inc | head -1)"
  gen_cfg="$(find "$dir/SubProcesses" -name configs.inc | head -1)"
  if diff -q "$banked_cfg" "$gen_cfg" >/dev/null; then
    echo ">>> [$version] configs.inc is identical to the banked run's" >&2
  else
    echo "!!! [$version] configs.inc differs from the banked run's -- the channel" >&2
    echo "    decomposition is not the one the reference integrates" >&2
    diff "$banked_cfg" "$gen_cfg" >&2 || true
    # The bank is a 3.7.1 run, so 3.7.1 must reproduce it exactly; the 3.5.7
    # line is *expected* to be allowed to differ and the difference is the
    # measurement, so it is recorded and the run proceeds.
    if [ "$version" = "3.7.1" ]; then exit 1; fi
  fi

  # Run 0's fourth fact: what the auto-selection rule picks for this process in
  # this MadGraph line, read off the directory MadGraph itself just wrote.
  echo ">>> [$version] auto-selected sde_strategy: $(grep 'sde_strategy' "$dir/Cards/run_card.dat" || true)" >&2
  echo ">>> [$version] proc_characteristics: $(grep -E 'single_color|gauge' "$dir/SubProcesses/proc_characteristics" | tr '\n' ' ')" >&2

  silence_madgraph_ui "$dir/Cards/me5_configuration.txt"
}

# Clone a compiled template into a fresh run directory. The Fortran is cloned
# with it (APFS copy-on-write, so this is free), but every G* survey directory
# and the combined result are removed: a run must re-survey with its own seed,
# not inherit the previous run's adapted grids.
clone_run_dir() {
  local template="$1" dest="$2"
  rm -rf "$dest"
  cp -Rc "$template" "$dest" 2>/dev/null || cp -R "$template" "$dest"
  rm -rf "$dest"/SubProcesses/P*/G* "$dest"/SubProcesses/results.dat "$dest"/Events/*
  silence_madgraph_ui "$dest/Cards/me5_configuration.txt"
}

# Install the banked cards verbatim, changing only nevents, iseed and -- when a
# refocused window is asked for -- the photon pt cuts.
install_cards() {
  local dir="$1" seed="$2" ptlo="${3:--}" pthi="${4:--}"
  cp "$BANKED/Cards/run_card.dat" "$dir/Cards/run_card.dat"
  cp "$BANKED/Cards/param_card.dat" "$dir/Cards/param_card.dat"
  python3 - "$dir/Cards/run_card.dat" "$NEVENTS" "$seed" "$ptlo" "$pthi" <<'PY'
import re, sys
path, nev, seed, ptlo, pthi = sys.argv[1:6]
text = open(path).read()
text = re.sub(r"^\s*\S+\s*=\s*nevents\b", "  %s = nevents" % nev, text, flags=re.M)
text = re.sub(r"^\s*\S+\s*=\s*iseed\b", "  %s = iseed" % seed, text, flags=re.M)
if ptlo != "-":
    n = len(re.findall(r"^\s*\S+\s*=\s*pta\b", text, flags=re.M))
    assert n == 1, "expected exactly one pta line, found %d" % n
    text = re.sub(r"^\s*\S+\s*=\s*pta\b", "  %s = pta" % ptlo, text, flags=re.M)
    n = len(re.findall(r"^\s*\S+\s*=\s*ptamax\b", text, flags=re.M))
    assert n == 1, "expected exactly one ptamax line, found %d" % n
    text = re.sub(r"^\s*\S+\s*=\s*ptamax\b", "  %s = ptamax" % pthi, text, flags=re.M)
open(path, "w").write(text)
PY
}

# Replace dummy_cuts' body with a window on pt(a). The momenta reaching
# dummy_cuts are in the partonic rest frame, which for these fixed equal-energy
# beams is the lab frame, so the transverse components need no boost -- and pt
# is invariant under the boost along the beam axis in any case.
install_window() {
  local dir="$1" lo="$2" hi="$3"
  python3 - "$dir/SubProcesses/dummy_fct.f" "$lo" "$hi" <<'PY'
import sys
path, lo, hi = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
marker = "      dummy_cuts=.true.\n"
assert text.count(marker) == 1, "dummy_cuts body not found exactly once"
body = """      double precision pta2, ptlo, pthi
      parameter (ptlo = %sd0, pthi = %sd0)
      dummy_cuts=.true.
c     window on pt(gamma): external 5 (idup 22)
      pta2 = p(1,5)**2 + p(2,5)**2
      if (pta2.lt.ptlo*ptlo .or. pta2.ge.pthi*pthi) dummy_cuts=.false.
""" % (lo, hi)
open(path, "w").write(text.replace(marker, body))
PY
  grep -n "pta2\|parameter (ptlo" "$dir/SubProcesses/dummy_fct.f" >&2
}

# The same, for a window on the muon-pair invariant mass. The muons are externals
# 3 and 4 -- asserted against leshouche.inc by `generate_template`, which requires
# IDUP = (-11, 11, -13, 13, 22) -- and an invariant mass needs no frame argument.
install_mll_window() {
  local dir="$1" lo="$2" hi="$3"
  python3 - "$dir/SubProcesses/dummy_fct.f" "$lo" "$hi" <<'PY'
import sys
path, lo, hi = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
marker = "      dummy_cuts=.true.\n"
assert text.count(marker) == 1, "dummy_cuts body not found exactly once"
body = """      double precision mll2, mlo, mhi
      parameter (mlo = %sd0, mhi = %sd0)
      dummy_cuts=.true.
c     window on m(mu+ mu-): externals 3 and 4 (idup -13, 13)
      mll2 = (p(0,3)+p(0,4))**2 - (p(1,3)+p(1,4))**2
     &     - (p(2,3)+p(2,4))**2 - (p(3,3)+p(3,4))**2
      if (mll2.lt.mlo*mlo .or. mll2.ge.mhi*mhi) dummy_cuts=.false.
""" % (lo, hi)
open(path, "w").write(text.replace(marker, body))
PY
  grep -n "mll2\|parameter (mlo" "$dir/SubProcesses/dummy_fct.f" >&2
}

# Run madevent and echo "<sigma> <err>" (pb) from SubProcesses/results.dat.
run_one() {
  local dir="$1" tag="$2"
  local log="$OUT/pta_window_$tag.log"
  echo ">>> [$tag] running madevent in $dir ..." >&2
  # The conda activation exports its own LDFLAGS, which suppresses MadGraph's
  # make_opts STDLIB=-lc++, so the link needs it appended explicitly.
  LDFLAGS="${LDFLAGS:-} -lc++" "$dir/bin/generate_events" -f "run_$tag" >"$log" 2>&1 || {
    echo "!!! [$tag] generate_events returned non-zero; see $log" >&2
    tail -30 "$log" >&2
  }
  local results="$dir/SubProcesses/results.dat"
  if [ ! -s "$results" ]; then
    echo "!!! [$tag] no results.dat at $results" >&2
    tail -30 "$log" >&2
    exit 1
  fi
  # The integration strategy actually used, read off the generated include
  # rather than off the card: this is the quantity the design's premise rests on.
  echo ">>> [$tag] $(grep -E 'TMIN_FOR_CHANNEL|SDE_STRAT' "$dir/Source/run_card.inc" | tr -d ' ' | tr '\n' ' ')" >&2
  awk 'NR==1{printf "%.10g %.10g\n", $1, $2}' "$results"
}

# One measurement: clone, configure, run, append a row to the stage's TSV.
measure() {
  local template="$1" version="$2" estimator="$3" widx="$4" seed="$5" axis="${6:-pt_a}"
  local lo hi tag dir s e
  if [ "$widx" = "0" ]; then
    lo="-"; hi="-"
    tag="${STAGE}${TAG_SUFFIX}_n${NEVENTS}_s${seed}"
  elif [ "$axis" = "m_mumu" ]; then
    lo="${MLL_EDGES[$((widx - 1))]}"; hi="${MLL_EDGES[$widx]}"
    tag="${STAGE}${TAG_SUFFIX}_w${widx}_n${NEVENTS}_s${seed}"
  else
    lo="${EDGES[$((widx - 1))]}"; hi="${EDGES[$widx]}"
    tag="${STAGE}${TAG_SUFFIX}_w${widx}_n${NEVENTS}_s${seed}"
  fi
  dir="$OUT/pta_$tag"
  clone_run_dir "$template" "$dir"
  case "$estimator:$axis" in
    control:*)     install_cards "$dir" "$seed" ;;
    part:pt_a)     install_cards "$dir" "$seed"; install_window "$dir" "$lo" "$hi" ;;
    part:m_mumu)   install_cards "$dir" "$seed"; install_mll_window "$dir" "$lo" "$hi" ;;
    cut:pt_a)      install_cards "$dir" "$seed" "$lo" "$hi" ;;
    *) echo "!!! unknown estimator/axis $estimator/$axis" >&2; exit 1 ;;
  esac
  read -r s e < <(run_one "$dir" "$tag")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$STAGE" "$version" "$estimator" "$widx" "$lo" "$hi" "$seed" "$NEVENTS" "$s" "$e" "$axis" >> "$TSV"
  echo ">>> [$tag] sigma = $s +- $e pb" >&2
}

case "$STAGE" in
  control-371)
    T="$OUT/pta_template_371"
    generate_template "$T" 3.7.1
    for seed in $SEEDS; do measure "$T" 3.7.1 control 0 "$seed"; done
    ;;
  control-357)
    T="$OUT/pta_template_357"
    generate_template "$T" 3.5.7
    for seed in $SEEDS; do measure "$T" 3.5.7 control 0 "$seed"; done
    ;;
  partition-371)
    T="$OUT/pta_template_371"
    generate_template "$T" 3.7.1
    for w in ${WINDOWS:-1 2 3 4 5}; do
      for seed in $SEEDS; do measure "$T" 3.7.1 part "$w" "$seed"; done
    done
    ;;
  partition-357)
    T="$OUT/pta_template_357"
    generate_template "$T" 3.5.7
    for w in ${WINDOWS:-1 2 3 4 5}; do
      for seed in $SEEDS; do measure "$T" 3.5.7 part "$w" "$seed"; done
    done
    ;;
  refocus)
    T="$OUT/pta_template_371"
    generate_template "$T" 3.7.1
    for w in ${WINDOWS:-1 5}; do
      for seed in $SEEDS; do measure "$T" 3.7.1 cut "$w" "$seed"; done
    done
    ;;
  mll-371)
    T="$OUT/pta_template_371"
    generate_template "$T" 3.7.1
    for w in ${WINDOWS:-1 2 3 4 5}; do
      for seed in $SEEDS; do measure "$T" 3.7.1 part "$w" "$seed" m_mumu; done
    done
    ;;
  *)
    echo "!!! unknown stage: $STAGE" >&2
    exit 2
    ;;
esac

# Rebuild the committed reference from every stage TSV on disk, so a stage that
# is re-run or added later does not require the others to be re-run.
python3 - "$RESULT_JSON" "$OUT" "${#EDGES[@]}" "${EDGES[@]}" "${MLL_EDGES[@]}" <<'PY'
import glob, json, os, sys
out, outdir, npt = sys.argv[1], sys.argv[2], int(sys.argv[3])
edges = [float(x) for x in sys.argv[4:4 + npt]]
mll_edges = [float(x) for x in sys.argv[4 + npt:]]
rows = []
for path in sorted(glob.glob(os.path.join(outdir, "pta_windows_*.tsv"))):
    for line in open(path):
        f = line.rstrip("\n").split("\t")
        # A row written before the secondary axis existed has no axis column and
        # is pt(a) by construction.
        if len(f) == 10:
            f = f + ["pt_a"]
        elif len(f) != 11:
            continue
        rows.append({
            "stage": f[0], "mg_version": f[1], "estimator": f[2],
            "axis": f[10],
            "window": int(f[3]),
            "pt_lo": None if f[4] == "-" else float(f[4]),
            "pt_hi": None if f[5] == "-" else float(f[5]),
            "iseed": int(f[6]), "nevents": int(f[7]),
            "sigma_pb": float(f[8]), "sigma_err_pb": float(f[9]),
        })
doc = {
    "_comment": "MadGraph5 LO sigma(e+ e- > mu+ mu- a) at 250+250 GeV with the banked "
                "ee_to_mumua run card, measured unwindowed and restricted to windows in "
                "pt(gamma), over independent seeds and both MadGraph lines on this "
                "machine. estimator 'control' is unwindowed; 'part' imposes the window "
                "through dummy_cuts, leaving the phase-space generator untouched, so the "
                "windows sum to the control; 'cut' imposes it as run-card pta/ptamax, "
                "which lets the generator re-optimise and so does not belong in that sum. "
                "Rows carry an 'axis': 'pt_a' windows the photon's transverse momentum "
                "(external leg 5), 'm_mumu' the muon-pair invariant mass (externals 3 and "
                "4) on the secondary axis, whose window index refers to m_mumu_edges_gev. "
                "Generated by validation/madgraph/gen_pta_windows.sh.",
    "pt_edges_gev": edges,
    "m_mumu_edges_gev": mll_edges,
    "runs": rows,
}
with open(out, "w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print("wrote %s (%d runs)" % (out, len(rows)))
PY
