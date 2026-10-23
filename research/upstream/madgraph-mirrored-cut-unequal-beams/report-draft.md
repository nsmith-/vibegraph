---
severity: medium
rank: unranked  # placed by the user against the nine drafted on 2026-10-16
severity_reason: "On the LTS line, any grouped proton run with unequal beam energies and a rapidity-type cut gets a silently wrong cross section and events outside its own cuts (5 % on Drell-Yan at 6500 x 4000); fixed in 3.8.1, so the ask is a backport."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md). Not yet run on any version: run `repro.sh` on 3.5.17
and 3.8.1 and replace the predicted rows below with the output before posting.

---

**Title:** LTS: with `ebeam1 ≠ ebeam2` the mirrored beam ordering is cut in the wrong frame (fixed in 3.8.1, not in 3.5.17)

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 no. Both read from source;
the reproducer below has not been run yet.

### Summary

In grouped output with unequal beam energies, the mirrored beam ordering
(`IMIRROR = 2`) is cut on a frame displaced by `ln(ebeam1/ebeam2)` in
rapidity. On `p p > e+ e-` at 6500 × 4000 GeV with the default `etal = 2.5`,
mirrored events are accepted with leptons at `−2.014 < η < 2.986`. The sample
therefore contains events outside its own cuts, and the cross section changes
when the two beam energies are exchanged, although that is the same collision.
3.8.1 cuts each orientation in its own frame (`CUTS_ORIENT` in `cuts.f`), so
this is a request to bring that fix to the LTS line.

### How to reproduce

```
import model sm
generate p p > e+ e-
output ppee
launch ppee
set ebeam1 6500
set ebeam2 4000
set nevents 10000
done
```

Then the same with `ebeam1 4000` and `ebeam2 6500`. A script that runs both and
counts the events with a lepton outside `|η| ≤ 2.5`, split by which parton is
on the `+z` beam, is below.

<details>
<summary><code>repro.sh</code></summary>

```bash
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
```

</details>

### Expected vs observed

| | events with `|η(e)| > 2.5` | σ(6500 × 4000) / σ(4000 × 6500) |
|---|---|---|
| expected | 0 | 1 |
| 3.5.17, predicted | some, all with the antiquark on the `+z` beam, at `2.5 < η ≤ 2.986` | about 0.9 |
| 3.8.1, predicted | 0 | 1 |

On 3.7.1, with `NNPDF23_lo_as_0130_qed` and both scales fixed at `m_Z`,
MadEvent gives 717.25 ± 0.15 pb at 6500 × 4000. An independent LO
generator gives 754.96 ± 0.35 pb, and 717.58 ± 0.31 pb when it cuts the
mirrored ordering as described below.

### Cause

3.5.17, `madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`, in
`DSIGPROC`:

```fortran
      IF(IMIRROR.EQ.2)THEN
C       Flip momenta (rotate around x axis)
        DO I=1,NEXTERNAL
          P1(2,I)=-P1(2,I)
          P1(3,I)=-P1(3,I)
        ENDDO
...
C       Flip CM_RAP (to get rapidity right)
        CM_RAP=-CM_RAP
      ENDIF

      DSIGPROC=0D0

      IF (PASSCUTS(P1)) THEN
```

`CM_RAP` of the sampled point is `½ ln(x₁·ebeam(1) / (x₂·ebeam(2)))`. The
mirrored event has the parton at `x₂` on beam 1 and the one at `x₁` on beam 2,
so its own rapidity is `ln(ebeam(1)/ebeam(2)) − CM_RAP`, not `−CM_RAP`. The
two agree only at equal beam energies. In 3.7.1 the same window arises
differently: `passcuts` runs once per point in `dsample.f` before
`select_grouping`, and `CM_RAP` is flipped only afterwards.

### Suggested fix

3.8.1's `PASSCUTS` (`Template/LO/SubProcesses/cuts.f:24-98`), which evaluates
the mirrored orientation at `CM_RAP = DLOG(EBEAM(1)/EBEAM(2)) - CM_RAP` with
`XBK` swapped, and the `CUTS_ORIENT` checks that go with it in `dsample.f` and
`super_auto_dsig_group_v4.inc`. For the 3.5.17 code path the one-line form
would be:

```diff
-        CM_RAP=-CM_RAP
+        CM_RAP=DLOG(EBEAM(1)/EBEAM(2))-CM_RAP
```

with the flip back after the mirrored call changed to match. Not tested.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph.

Assisted-by: claude-code:claude-opus-5-5
