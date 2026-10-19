---
severity: medium
rank: 3  # of 9; 1 = most severe
severity_reason: "For merged (CKKW-L) runs, the ktdurham cut can wrongly throw away events whose only b quarks come from a decay, removing up to a third of the cross section."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** `ktdurham`/`ptlund` cut is applied when the only "massless" parton is a decayed `b` (`.and.`/`.or.` precedence in `NMASSLESS`)

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested.

### Summary

The Durham-kT merging cut (`ktdurham`) is meant to be skipped when no massless
parton is subject to the merging cut. The counter `NMASSLESS` counts every `b`
quark, including one from a decay, so the cut is applied to events whose only
merging partons are massive. In the example below, `ktdurham = 500` removes 34 %
of σ when it should remove nothing.

### How to reproduce

```
import model sm
generate e+ e- > t t~ z, z > b b~
output eettz
launch eettz
set lpp1 0
set lpp2 0
set ebeam1 500
set ebeam2 500
set ktdurham 500
done
```

Repeat with `set ktdurham -1` and compare σ. The defaults (`maxjetflavor = 4`,
`cut_decays = F`) make the `b` quarks `is_a_b` and `from_decay`, so the only
merging-cut partons are `t` and `t~`. The script below runs both and a
patched version.

<details>
<summary><code>repro.sh</code></summary>

```bash
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
```

</details>

### Expected vs observed

σ should be the same with and without `ktdurham`.

| | 3.5.17 | 3.8.1 |
|---|---|---|
| `ktdurham = -1` | 6.820e-4 pb | 6.839e-4 pb |
| `ktdurham = 500` | **4.509e-4 pb (−34 %)** | **4.469e-4 pb (−35 %)** |
| `ktdurham = 500`, with the fix | 6.820e-4 pb (identical to the cut off) | 6.839e-4 pb (identical) |

### Cause

`Template/LO/SubProcesses/cuts.f`:

```fortran
          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
     &                        is_a_j(i).or.is_a_b(i)) then
            NMASSLESS = NMASSLESS + 1
```

This reads as `(… .and. is_a_j(i)) .or. is_a_b(i)`, so every `b` counts
whatever its origin. The same expression is used twice:

| version | Durham kT | ptlund |
|---|---|---|
| 3.8.1 | lines 643–644 | lines 745–746 |
| 3.5.17 | lines 551–552 | lines 653–654 |

### Suggested fix (both places)

```diff
           if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
-     &                        is_a_j(i).or.is_a_b(i)) then
+     &                        (is_a_j(i).or.is_a_b(i))) then
```

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
