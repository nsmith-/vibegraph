#!/usr/bin/env bash
# MadEvent: with exactly one beam's factorisation scale fixed, `scalefact` must
# still multiply the other beam's dynamical scale. `reweight.f`'s
# `if(.not. fixed_fac_scale1.or. fixed_fac_scale2)` skips it for
# fixed_fac_scale1 = T, fixed_fac_scale2 = F.
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC ./repro.sh [workdir]
#
#   MG5      command that runs an MG5 script file (default: mg5_aMC on PATH)
#   workdir  scratch directory (default: ./work)
#
# Process `p p > z` (2 -> 1, so the clustering scale is the Z mass), with
# `dynamical_scale_choice = -1`, `scalefact = 2` and the fixed beam's scale at
# 10 GeV, so SCALUP = sqrt(max(q2fact(1), q2fact(2))) is the dynamical beam's
# scale. Three runs:
#   tf  fixed_fac_scale1 = T, fixed_fac_scale2 = F   (the defect)
#   ft  fixed_fac_scale1 = F, fixed_fac_scale2 = T   (control)
#   ff  both dynamical                               (control)
#   tf_fixed  tf on a copy of the process directory whose reweight.f carries
#             the one-line fix (.not.fixed_fac_scale1.or..not.fixed_fac_scale2)
# For each run, prints the range of SCALUP / sqrt(s_hat) over every event
# (expected 2 = scalefact) and the cross section. tf and ft are not expected to
# agree with each other: in a mirrored subprocess q2fact(1) is the scale of
# process leg 1, not of beam 1, so fixing it fixes the quark's scale in both
# orientations. tf_fixed is the cross section tf should have.
set -euo pipefail

MG5="${MG5:-mg5_aMC}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"
PROC="$WORK/ppz"

run_card_lines() {  # <fixed1> <fixed2>
  printf 'set nevents 1000\nset iseed 1\nset use_syst False\n'
  printf 'set dynamical_scale_choice -1\nset scalefact 2\n'
  printf 'set fixed_fac_scale1 %s\nset fixed_fac_scale2 %s\n' "$1" "$2"
  printf 'set dsqrt_q2fact1 10\nset dsqrt_q2fact2 10\n'
}

mg5() {  # <script>
  (cd "$WORK" && $MG5 "$1") >> "$WORK/mg5.log" 2>&1 || {
    tail -30 "$WORK/mg5.log"; exit 1; }
}
opts() {
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
}

rm -rf "$PROC" "$PROC.fixed" "$WORK/mg5.log"
{ opts; printf 'import model sm\ngenerate p p > z\noutput %s -nojpeg\n' "$PROC"; } > "$WORK/gen.mg5"
mg5 "$WORK/gen.mg5"

# The control: the same directory with the one-line fix in reweight.f.
cp -R "$PROC" "$PROC.fixed"
f="$PROC.fixed/SubProcesses/reweight.f"
grep -q 'if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then' "$f"
sed -i.orig 's/if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then/if(.not.fixed_fac_scale1.or..not.fixed_fac_scale2) then/' "$f"
if cmp -s "$f" "$f.orig"; then echo "fix not applied to $f" >&2; exit 1; fi

{
  opts
  for v in "$PROC tf True False" "$PROC ft False True" "$PROC ff False False" \
           "$PROC.fixed tf_fixed True False"; do
    set -- $v
    printf 'launch %s -n %s\n' "$1" "$2"
    run_card_lines "$3" "$4"
    printf 'done\n'
  done
} > "$WORK/run.mg5"
mg5 "$WORK/run.mg5"

printf '%-9s %-19s %8s %22s %22s\n' run "fixed_fac_scale1,2" events "SCALUP/sqrt(s_hat)" "sigma [pb]"
for v in "$PROC tf T,F" "$PROC ft F,T" "$PROC ff F,F" "$PROC.fixed tf_fixed T,F"; do
  set -- $v
  dir=$1; shift
  lhe="$dir/Events/$1/unweighted_events.lhe.gz"
  sigma=$(gzip -dc "$lhe" | awk '/<init>/{getline; getline; s = $1 "+-" $2} END {print s}')
  # Per event: SCALUP from the event line, sqrt(s_hat) from the two incoming
  # partons (status -1). Prints the min and max ratio over the sample.
  ratio=$(gzip -dc "$lhe" | awk '
    /<event>/ { getline; scalup = $4; e = 0; pz = 0; inev = 1; next }
    inev && $2 == -1 { e += $10; pz += $9 }
    /<\/event>/ { r = scalup / sqrt(e*e - pz*pz); n++
                  if (n == 1 || r < lo) lo = r; if (n == 1 || r > hi) hi = r; inev = 0 }
    END { printf "%d %.6f-%.6f", n, lo, hi }')
  set -- $1 $2 $ratio
  printf '%-9s %-19s %8s %22s %22s\n' "$1" "$2" "$3" "$4" "$sigma"
done
echo "expected: SCALUP/sqrt(s_hat) = 2 (scalefact) in every run, and sigma(tf) = sigma(tf_fixed)"
