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
merging-cut partons are `t` and `t~`. `repro.sh` (attached) runs both and a
patched version.

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
