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
`repro.sh` (attached) runs both orderings and a patched one.

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
