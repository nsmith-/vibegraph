#!/usr/bin/env bash
# MadEvent: with ebeam1 != ebeam2, a grouped subprocess's mirrored beam
# ordering (IMIRROR = 2) must be cut on the event it writes. Grouped MadEvent
# evaluates passcuts once per phase-space point, at the sampled point's
# cm_rap, before select_grouping picks the ordering, so the mirrored ordering's
# lepton rapidity window is displaced by ln(ebeam1/ebeam2).
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC ./repro.sh [workdir]
#
#   MG5      command that runs an MG5 script file (default: mg5_aMC on PATH)
#   workdir  scratch directory (default: ./work)
#
# Process `p p > e+ e-` on the default run card (ptl = 10, etal = 2.5,
# drll = 0.4) apart from the beam energies. Two runs:
#   ab  ebeam1 = 6500, ebeam2 = 4000
#   ba  ebeam1 = 4000, ebeam2 = 6500   (the same collision reflected in z)
# For each run, prints sigma and, split by which parton is on the +z beam (the
# quark: the direct ordering; the antiquark: the mirrored one), the events
# carrying an electron or positron at |eta| > etal and the largest |eta|.
# Expected: no event outside the cut, and sigma(ab) = sigma(ba), since p p is
# symmetric and every cut on the card is symmetric under z -> -z.
set -euo pipefail

MG5="${MG5:-mg5_aMC}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"
PROC="$WORK/ppee"

mg5() {  # <script>
  (cd "$WORK" && $MG5 "$1") >> "$WORK/mg5.log" 2>&1 || {
    tail -30 "$WORK/mg5.log"; exit 1; }
}
opts() {
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
}

rm -rf "$PROC" "$WORK/mg5.log"
{ opts; printf 'import model sm\ngenerate p p > e+ e-\noutput %s -nojpeg\n' "$PROC"; } > "$WORK/gen.mg5"
mg5 "$WORK/gen.mg5"

{
  opts
  for v in "ab 6500 4000" "ba 4000 6500"; do
    set -- $v
    printf 'launch %s -n %s\n' "$PROC" "$1"
    printf 'set nevents 10000\nset iseed 1\nset use_syst False\n'
    printf 'set ebeam1 %s\nset ebeam2 %s\n' "$2" "$3"
    printf 'done\n'
  done
} > "$WORK/run.mg5"
mg5 "$WORK/run.mg5"

printf '%-4s %-11s %22s %-10s %8s %16s %12s\n' run ebeam1,2 "sigma [pb]" "+z parton" events "|eta(e)| > 2.5" "max |eta(e)|"
for v in "ab 6500,4000" "ba 4000,6500"; do
  set -- $v
  lhe="$PROC/Events/$1/unweighted_events.lhe.gz"
  sigma=$(gzip -dc "$lhe" | awk '/<init>/{getline; getline; s = $1 "+-" $2} END {print s}')
  # Per event: the parton with status -1 and pz > 0 is on the +z beam; an
  # electron or positron (|id| = 11, status 1) outside |eta| <= 2.5 is counted.
  gzip -dc "$lhe" | awk -v run="$1" -v beams="$2" -v sigma="$sigma" '
    /<event>/ { getline; inev = 1; plus = 0; out = 0; next }
    inev && NF >= 13 && $2 == -1 && $9 > 0 { plus = $1 }
    inev && NF >= 13 && $2 == 1 && ($1 == 11 || $1 == -11) {
      p = sqrt($7*$7 + $8*$8 + $9*$9); eta = 0.5 * log((p + $9) / (p - $9))
      a = eta < 0 ? -eta : eta
      side = plus > 0 ? "quark" : "antiquark"
      if (a > hi[side]) hi[side] = a
      if (a > 2.5) out = 1
    }
    /<\/event>/ { side = plus > 0 ? "quark" : "antiquark"; n[side]++; o[side] += out; inev = 0 }
    END {
      for (k = 0; k < 2; k++) {
        side = k == 0 ? "quark" : "antiquark"
        printf "%-4s %-11s %22s %-10s %8d %16d %12.4f\n", run, beams, sigma, side, n[side], o[side], hi[side]
      }
    }'
done
echo "expected: no event with |eta(e)| > 2.5, and sigma(ab) = sigma(ba) within errors"
