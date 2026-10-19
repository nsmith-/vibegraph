---
severity: medium
rank: 2  # of 9; 1 = most severe
severity_reason: "Fixing only one beam's factorisation scale and varying scalefact silently gives the other beam the wrong scale, moving the cross section by about 10 %."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** `scalefact` is not applied to beam 2's factorisation scale when only `fixed_fac_scale1` is set

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested.

### Summary

With `fixed_fac_scale1 = True`, `fixed_fac_scale2 = False` and a dynamical
scale, the dynamical beam's factorisation scale ignores `scalefact`. With
`scalefact = 2`, beam 2's μF stays at the unscaled clustering scale on every
event. The cross section changes by about 10 % compared to the intended
behaviour.

### How to reproduce

```
import model sm
generate p p > z
output ppz
launch ppz
set dynamical_scale_choice -1
set scalefact 2
set fixed_fac_scale1 True
set fixed_fac_scale2 False
set dsqrt_q2fact1 10
set nevents 1000
done
```

For 2 → 1 the clustering scale is √ŝ = the Z mass. The fixed beam sits at
10 GeV, so `SCALUP = sqrt(max(q2fact(1), q2fact(2)))` is beam 2's scale. A
script that runs this, two controls and the patched version is below.

<details>
<summary><code>repro.sh</code></summary>

```bash
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
```

</details>

### Expected vs observed

| | SCALUP / √ŝ on every event | σ (3.8.1, one seed) |
|---|---|---|
| expected (`scalefact` applied) | 2 | 42 503 ± 106 pb (with the fix below) |
| observed | **1** | 38 500 ± 92 pb |

The controls behave as expected. Fixing beam 2 instead (`F, T`) gives SCALUP/√ŝ
= 2, and so does leaving both dynamical (`F, F`). 3.5.17 gives the same
numbers within errors.

### Cause

`Template/LO/SubProcesses/reweight.f`, in `setclscales` (3.8.1 line 1138;
3.5.17 line 1117):

```fortran
      if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then
         if(.not.fixed_fac_scale1) q2fact(1)=scalefact**2*q2fact(1)
         if(.not.fixed_fac_scale2) q2fact(2)=scalefact**2*q2fact(2)
```

`.not.` binds tighter than `.or.`, so the condition is false for exactly
`(T, F)`. The block is then skipped: `q2fact(2)` is not scaled, and `q2bck(2)`
is not set either.

### Suggested fix

```diff
-      if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then
+      if(.not.fixed_fac_scale1.or..not.fixed_fac_scale2) then
```

This is the form the vectorised path of the same file already uses
(`reweight.f:1903` in 3.8.1). With it, SCALUP/√ŝ = 2 on every event.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
