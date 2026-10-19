#!/usr/bin/env bash
# MadEvent's Durham-kT merging cut (cuts.f) applies only when the event has a
# massless parton subject to the merging cut (NMASSLESS > 0). The count reads
#   is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and. is_a_j(i).or.is_a_b(i)
# which Fortran groups as (... .and. is_a_j(i)) .or. is_a_b(i): any b quark
# counts, even one from a decay or outside pdgs_for_merging_cut, and switches
# the cut on for events that have no massless merging parton at all.
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC ./repro.sh [workdir]
#
#   MG5      command that runs an MG5 script file (default: mg5_aMC on PATH)
#   workdir  scratch directory (default: ./work)
#
# Process `e+ e- > t t~ z, z > b b~` at sqrt(s) = 1 TeV with the default
# maxjetflavor = 4 (so the b quarks are is_a_b) and cut_decays = F (so they are
# from_decay). The merging-cut particles are the t and t~, which are massive:
# NMASSLESS should be 0 and ktdurham should not cut. Three runs:
#   off         ktdurham = -1
#   on          ktdurham = 500
#   on_fixed    ktdurham = 500, with the count parenthesised in the
#               generated cuts.f: ... .and. (is_a_j(i).or.is_a_b(i))
# Expected: sigma(on) = sigma(on_fixed) = sigma(off).
set -euo pipefail

MG5="${MG5:-mg5_aMC}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"
PROC="$WORK/eettz"
KT=500

mg5() {  # <script>
  (cd "$WORK" && $MG5 "$1") >> "$WORK/mg5.log" 2>&1 || {
    tail -30 "$WORK/mg5.log"; exit 1; }
}
opts() {
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
}

rm -rf "$PROC" "$PROC.fixed" "$WORK/mg5.log"
{
  opts
  printf 'import model sm\ngenerate e+ e- > t t~ z, z > b b~\n'
  printf 'output %s -nojpeg\n' "$PROC"
} > "$WORK/gen.mg5"
mg5 "$WORK/gen.mg5"

# The control: both NMASSLESS counts (Durham kT and the ptlund cut that
# follows it) parenthesised.
cp -R "$PROC" "$PROC.fixed"
f="$PROC.fixed/SubProcesses/cuts.f"
cp "$f" "$f.orig"
sed 's/is_a_j(i)\.or\.is_a_b(i)) then/(is_a_j(i).or.is_a_b(i))) then/I' "$f.orig" > "$f"
[ "$(diff "$f.orig" "$f" | grep -c '^>')" -eq 2 ] || {
  echo "fix not applied to $f" >&2; diff "$f.orig" "$f" >&2; exit 1; }

{
  opts
  for v in "$PROC off -1" "$PROC on $KT" "$PROC.fixed on_fixed $KT"; do
    set -- $v
    printf 'launch %s -n %s\n' "$1" "$2"
    printf 'set lpp1 0\nset lpp2 0\nset ebeam1 500\nset ebeam2 500\n'
    printf 'set nevents 2000\nset iseed 1\nset use_syst False\n'
    printf 'set ktdurham %s\n' "$3"
    printf 'done\n'
  done
} > "$WORK/run.mg5"
mg5 "$WORK/run.mg5"

printf '%-9s %-9s %-24s\n' run ktdurham "sigma [pb]"
for v in "$PROC off -1" "$PROC on $KT" "$PROC.fixed on_fixed $KT"; do
  set -- $v
  sigma=$(gzip -dc "$1/Events/$2/unweighted_events.lhe.gz" |
          awk '/<init>/ { getline; getline; s = $1 " +- " $2 } END { print s }')
  printf '%-9s %-9s %-24s\n' "$2" "$3" "$sigma"
done
echo "expected: the three cross sections agree (no massless parton is subject to the merging cut)"
