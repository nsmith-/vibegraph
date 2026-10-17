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
script that runs this, two controls and the patched version is attached
(`repro.sh`).

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
