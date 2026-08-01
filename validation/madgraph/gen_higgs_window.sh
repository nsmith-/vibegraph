#!/usr/bin/env bash
# MadGraph reference runs for `e+ e- > mu+ mu- ta+ ta- QCD=0` split at the Higgs
# pole in m(tau+ tau-): the window, its complement, and the unwindowed control.
#
# The banked run of this process disagrees with the vibegraph integral by +2.2%,
# and binning both sides' events puts the whole offset in one 200 MeV bin at
# m(ta+,ta-) = 125 GeV. A cross section over the full phase space cannot say
# which side mis-covers that resonance; three cross sections can, because the
# window and its complement must add up to the unwindowed one.
#
# The window is imposed through `dummy_cuts` in SubProcesses/dummy_fct.f, not
# through a run-card cut, for a reason worth stating: the only run-card cut that
# constrains a lepton-pair invariant mass on this final state is the `mmll`
# family, and setcuts.f applies it to *every* same-flavour opposite-sign pair --
#
#   if((is_a_l(i).and.is_a_l(j)).and.
#  &   (abs(idup(i,1,iproc)).eq.abs(idup(j,1,iproc))).and.
#  &   (idup(i,1,iproc)*idup(j,1,iproc).lt.0))
#  &   s_min(j,i)=mmll*dabs(mmll)   !only on l+l- pairs (same flavour)
#
# -- so it would bite the mu+ mu- pair as well and measure a different thing.
# The per-PDG `mxx_min_pdg` cut cannot stand in for it either: banner.py refuses
# a PDG-specific cut for any lepton ("Can not use PDG related cut for light
# quark/b quark/lepton/gluon/photon", pdg 15 among them). `dummy_cuts` is
# applied in passcuts() after every other cut and touches nothing else, so a
# windowed run integrates the same integrand MadGraph already integrated,
# restricted to the window.
#
# Every run uses the banked run's own Cards/run_card.dat verbatim (ptl 10,
# etal 2.5, drll 0.4, bwcutoff 15, sde_strategy 2, e+e- at 250+250 GeV) with
# only nevents and iseed changed, so the control is comparable to the banked
# cross section and the three runs are comparable to each other.
#
# Usage: pixi run -e madgraph bash validation/madgraph/gen_higgs_window.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/output"
BANKED="$OUT/ee_to_mumu_tata_qcd0"
# Committed: a handful of scalars, expensive to produce and stable, so they
# travel in git like sigma_reference.json rather than in the fetched bundle.
RESULT_JSON="$HERE/higgs_window_reference.json"

MLO="${MLO:-124.9}"
MHI="${MHI:-125.1}"
NEVENTS="${NEVENTS:-10000}"
# Independent seeds for the unwindowed control: one MadEvent run is a single
# draw, and the question is whether its quoted error covers its own spread.
CONTROL_SEEDS="${CONTROL_SEEDS:-20260801 20260802 20260803}"

if [ ! -f "$BANKED/Cards/run_card.dat" ]; then
  echo "!!! the banked run card is missing at $BANKED/Cards/run_card.dat" >&2
  echo "    run 'pixi run fetch-refdata' first" >&2
  exit 1
fi

# Generate one process directory (idempotent) and install the banked cards.
generate_dir() {
  local procdir="$1" seed="$2"
  if [ ! -f "$procdir/bin/generate_events" ]; then
    echo ">>> generating $procdir ..." >&2
    local tmp_mg5
    tmp_mg5="$(mktemp -t gen_hwin_XXXX).mg5"
    cat > "$tmp_mg5" <<EOF
generate e+ e- > mu+ mu- ta+ ta- QCD=0
output $procdir -nojpeg
EOF
    LDFLAGS="${LDFLAGS:-} -lc++" mg5_aMC "$tmp_mg5" >&2
    rm -f "$tmp_mg5"
  fi
  cp "$BANKED/Cards/run_card.dat" "$procdir/Cards/run_card.dat"
  cp "$BANKED/Cards/param_card.dat" "$procdir/Cards/param_card.dat"
  # nevents sets the refine target; the cross section comes from results.dat.
  python3 - "$procdir/Cards/run_card.dat" "$NEVENTS" "$seed" <<'PY'
import re, sys
path, nev, seed = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
text = re.sub(r"^\s*\S+\s*=\s*nevents\b", "  %s = nevents" % nev, text, flags=re.M)
text = re.sub(r"^\s*\S+\s*=\s*iseed\b", "  %s = iseed" % seed, text, flags=re.M)
open(path, "w").write(text)
PY
}

# Replace dummy_cuts' body with an m(tau+ tau-) window ("in") or its complement
# ("out"). Legs 5 and 6 are the tau pair -- asserted against the generated
# leshouche.inc rather than assumed.
install_window() {
  local procdir="$1" side="$2" lo="$3" hi="$4"
  local les
  les="$(find "$procdir/SubProcesses" -name leshouche.inc | head -1)"
  if ! grep -q 'IDUP(I,1,1),I=1,6)/ *-11, *11, *-13, *13, *-15, *15/' "$les"; then
    echo "!!! leg order is not (-11,11,-13,13,-15,15) in $les" >&2
    grep 'IDUP' "$les" >&2
    exit 1
  fi
  python3 - "$procdir/SubProcesses/dummy_fct.f" "$side" "$lo" "$hi" <<'PY'
import sys
path, side, lo, hi = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
text = open(path).read()
marker = "      dummy_cuts=.true.\n"
assert text.count(marker) == 1, "dummy_cuts body not found exactly once"
test = {
    "in":  "if (mtt2.lt.mlo*mlo .or. mtt2.gt.mhi*mhi) dummy_cuts=.false.",
    "out": "if (mtt2.ge.mlo*mlo .and. mtt2.le.mhi*mhi) dummy_cuts=.false.",
}[side]
body = """      double precision mtt2, mlo, mhi
      parameter (mlo = %sd0, mhi = %sd0)
      dummy_cuts=.true.
c     window on m(tau+ tau-): externals 5 and 6 (idup -15, 15)
      mtt2 = (p(0,5)+p(0,6))**2 - (p(1,5)+p(1,6))**2
     &     - (p(2,5)+p(2,6))**2 - (p(3,5)+p(3,6))**2
      %s
""" % (lo, hi, test)
open(path, "w").write(text.replace(marker, body))
PY
  grep -n "mtt2\|parameter (mlo" "$procdir/SubProcesses/dummy_fct.f" >&2
}

# Run madevent and echo "<sigma> <err>" (pb) from SubProcesses/results.dat.
run_one() {
  local procdir="$1" tag="$2"
  echo ">>> [$tag] running madevent in $procdir ..." >&2
  local log="$OUT/higgs_window_$tag.log"
  LDFLAGS="${LDFLAGS:-} -lc++" "$procdir/bin/generate_events" -f "run_$tag" >"$log" 2>&1 || {
    echo "!!! [$tag] generate_events failed; see $log" >&2
    tail -40 "$log" >&2
  }
  local results="$procdir/SubProcesses/results.dat"
  if [ ! -s "$results" ]; then
    echo "!!! [$tag] no results.dat at $results" >&2
    exit 1
  fi
  awk 'NR==1{printf "%.10g %.10g\n", $1, $2}' "$results"
}

WIN="$OUT/ee_to_mumu_tata_qcd0_hwindow"
ANTI="$OUT/ee_to_mumu_tata_qcd0_hanti"

generate_dir "$WIN" 20260801
install_window "$WIN" in "$MLO" "$MHI"
read -r SW EW < <(run_one "$WIN" hwindow)
echo ">>> windowed     sigma = $SW +- $EW pb  (m(ta+,ta-) in [$MLO, $MHI])" >&2

generate_dir "$ANTI" 20260801
install_window "$ANTI" out "$MLO" "$MHI"
read -r SA EA < <(run_one "$ANTI" hanti)
echo ">>> anti-window  sigma = $SA +- $EA pb  (m(ta+,ta-) outside)" >&2

CONTROLS=""
for seed in $CONTROL_SEEDS; do
  dir="$OUT/ee_to_mumu_tata_qcd0_control_$seed"
  generate_dir "$dir" "$seed"
  read -r SC EC < <(run_one "$dir" "control_$seed")
  echo ">>> control $seed sigma = $SC +- $EC pb" >&2
  CONTROLS="$CONTROLS $seed:$SC:$EC"
done

python3 - "$RESULT_JSON" "$SW" "$EW" "$SA" "$EA" "$MLO" "$MHI" $CONTROLS <<'PY'
import json, sys
out, sw, ew, sa, ea, lo, hi = sys.argv[1:8]
controls = []
for spec in sys.argv[8:]:
    seed, s, e = spec.split(":")
    controls.append({"iseed": int(seed), "sigma_pb": float(s), "sigma_err_pb": float(e)})
doc = {
    "_comment": "MadGraph5 3.5.7 LO sigma(e+ e- > mu+ mu- ta+ ta- QCD=0) at 250+250 GeV with "
                "the banked run card: restricted to a window in m(tau+ tau-) around the "
                "Higgs pole via dummy_cuts, restricted to its complement, and unwindowed "
                "over independent seeds. Generated by validation/madgraph/gen_higgs_window.sh.",
    "mg_version": "3.5.7",
    "window": {"m_tautau_lo": float(lo), "m_tautau_hi": float(hi)},
    "hwindow": {"sigma_pb": float(sw), "sigma_err_pb": float(ew)},
    "hanti": {"sigma_pb": float(sa), "sigma_err_pb": float(ea)},
    "control": controls,
}
doc["sum_in_plus_out"] = {
    "sigma_pb": doc["hwindow"]["sigma_pb"] + doc["hanti"]["sigma_pb"],
    "sigma_err_pb": (doc["hwindow"]["sigma_err_pb"] ** 2
                     + doc["hanti"]["sigma_err_pb"] ** 2) ** 0.5,
}
with open(out, "w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print("wrote", out)
print(json.dumps(doc, indent=2))
PY
