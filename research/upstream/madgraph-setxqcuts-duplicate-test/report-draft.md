---
severity: low
rank: 6  # of 9; 1 = most severe
severity_reason: "An integration hint is set or not depending on the order the particles are written; it changes only how the integrator samples, not the result."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** `setxqcuts`: jet test checks `iforest(2,…)` twice, so the xqcut grid hint depends on leg order

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested.

### Summary

`setxqcuts` gives a final-state pair the xqcut grid hint (`xqcutij`) when one
of the two daughters is a jet. The test looks at the same daughter twice. As a
result, a (jet, non-jet) pair gets the hint only when the jet has the lower
leg number, so the same process written in a different order gets different
integration grids. This affects grids only; I saw no change in σ.

### How to reproduce

The default `maxjetflavor = 4` makes the `b` a non-jet. Run:

```
import model sm
generate u u~ > g b b~
output gbb
generate u u~ > b b~ g
output bbg
```

Launch each with `set ickkw 1` and `set xqcut 20`, then compare the
`xqcutij` table printed at the top of `SubProcesses/P1_*/G1/run1_app.log`.
The script below runs both orderings and a patched one.

<details>
<summary><code>repro.sh</code></summary>

```bash
#!/usr/bin/env bash
# MadEvent's `setxqcuts` (setcuts.f) must give a final-state pair (jet, non-jet)
# the same xqcut grid hint whichever order the legs are written in. Its jet test
# reads `idup(iforest(2,k,j),1,1)` twice and never `iforest(1,k,j)`, so the
# hint depends on which daughter MadGraph stores first.
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC ./repro.sh [workdir]
#
#   MG5      command that runs an MG5 script file (default: mg5_aMC on PATH)
#   workdir  scratch directory (default: ./work)
#
# Two orderings of one process, `u u~ > g b b~` and `u u~ > b b~ g`, at
# `ickkw = 1`, `xqcut = 20` and the default `maxjetflavor = 4`, so the b quarks
# are not jets and the gluon is. Each run's first channel log (`G*/run1_app.log`)
# prints the `xqcutij` table (cuts.f, first call to passcuts). The script prints that table
# and the g-b and g-b~ entries of both runs; expected: 20 in both orderings.
# A third run, `u u~ > b b~ g` with the one-line fix applied to its generated
# setcuts.f, is the control.
set -euo pipefail
declare -A procs

MG5="${MG5:-mg5_aMC}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"

mg5() {  # <script>
  (cd "$WORK" && $MG5 "$1") >> "$WORK/mg5.log" 2>&1 || {
    tail -30 "$WORK/mg5.log"; exit 1; }
}
opts() {
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
}

rm -rf "$WORK/gbb" "$WORK/bbg" "$WORK/bbg.fixed" "$WORK/mg5.log"
{
  opts
  printf 'import model sm\n'
  printf 'generate u u~ > g b b~\noutput %s -nojpeg\n' "$WORK/gbb"
  printf 'generate u u~ > b b~ g\noutput %s -nojpeg\n' "$WORK/bbg"
} > "$WORK/gen.mg5"
mg5 "$WORK/gen.mg5"

# The control: the second test of the pair reads iforest(1).
cp -R "$WORK/bbg" "$WORK/bbg.fixed"
f="$WORK/bbg.fixed/SubProcesses/setcuts.f"
cp "$f" "$f.orig"
awk '
  /if\(iabs\(idup\(iforest\(2,k,j\),1,1\)\)\.le\.maxjetflavor\.or\./ { n++ }
  n == 1 && /idup\(iforest\(2,k,j\),1,1\)\.eq\.21\.or\./ { sub(/iforest\(2,k,j\)/, "iforest(1,k,j)"); n++; print; next }
  n == 2 && /iabs\(idup\(iforest\(2,k,j\),1,1\)\)\.le\.maxjetflavor\.or\./ { sub(/iforest\(2,k,j\)/, "iforest(1,k,j)"); n++ }
  { print }' "$f.orig" > "$f"
[ "$(diff "$f.orig" "$f" | grep -c '^>')" -eq 2 ] || {
  echo "fix not applied to $f" >&2; diff "$f.orig" "$f" >&2; exit 1; }

{
  opts
  for d in gbb bbg bbg.fixed; do
    printf 'launch %s -n run\n' "$WORK/$d"
    printf 'set nevents 100\nset iseed 1\nset use_syst False\n'
    printf 'set ickkw 1\nset xqcut 20\n'
    printf 'done\n'
  done
} > "$WORK/run.mg5"
mg5 "$WORK/run.mg5"

procs=([gbb]="g b b~" [bbg]="b b~ g" [bbg.fixed]="b b~ g, fixed")
for d in gbb bbg bbg.fixed; do
  # The channel's survey log; MadEvent 3.x keeps it as run1_app.log.
  log=$(ls "$WORK/$d"/SubProcesses/P*/G*/run1_app.log | head -1)
  sigma=$(gzip -dc "$WORK/$d/Events/run/unweighted_events.lhe.gz" |
          awk '/<init>/ { getline; getline; s = $1 " +- " $2 } END { print s }')
  echo "== $d  (legs 3 4 5 = ${procs[$d]}; sigma $sigma pb)"
  grep -E '^ *xqcutij #' "$log"
  # Row j lists xqcutij(i,j) for i = 3..nexternal.
  awk '/xqcutij #/ { j = $3 + 0; sub(/.*>/, ""); for (i = 1; i <= NF; i++) x[i + 2, j] = $i }
       END { printf "  xqcutij(4,3) = %s  xqcutij(5,3) = %s  xqcutij(5,4) = %s\n", x[4,3], x[5,3], x[5,4] }' "$log"
done
cat <<'TXT'
expected: the gluon-quark pairs carry xqcut = 20 in every run
  gbb: legs g b b~ -> xqcutij(4,3) = xqcutij(5,3) = 20, xqcutij(5,4) = 0
  bbg: legs b b~ g -> xqcutij(5,3) = xqcutij(5,4) = 20, xqcutij(4,3) = 0
and sigma the same in all three runs (the table only shapes the grids)
TXT
```

</details>

### Expected vs observed

The g–b and g–b~ pairs should carry 20 in both orderings.

| process | g–b, g–b~ entries |
|---|---|
| `u u~ > g b b~` | 20, 20 |
| `u u~ > b b~ g` | **0, 0** |
| `u u~ > b b~ g`, with the fix | 20, 20 |

### Cause

`Template/LO/SubProcesses/setcuts.f` (3.8.1 lines 948–951; 3.5.17 lines
946–949):

```fortran
                  if(iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
     $                 idup(iforest(2,k,j),1,1).eq.21.or.
     $                 iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
     $                 idup(iforest(2,k,j),1,1).eq.21)then
```

The second pair of tests repeats `iforest(2,k,j)`. In `u u~ > b b~ g` the
generated `configs.inc` stores the gluon (leg 5) in `iforest(1,…)` for both g–b
vertices, so the gluon is never tested.

### Suggested fix

```diff
                   if(iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
      $                 idup(iforest(2,k,j),1,1).eq.21.or.
-     $                 iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
-     $                 idup(iforest(2,k,j),1,1).eq.21)then
+     $                 iabs(idup(iforest(1,k,j),1,1)).le.maxjetflavor.or.
+     $                 idup(iforest(1,k,j),1,1).eq.21)then
```

This makes the hint match `setclscales`, which applies `xqcut` to any
clustering vertex with a jet daughter.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
