# Draft: MadGraph bug report

This draft is for the user to review, edit and file, either on Launchpad
(<https://bugs.launchpad.net/mg5amcnlo>) or as an issue on
<https://github.com/mg5amcnlo/mg5amcnlo>, which MadGraph's README names as the
places for bug reports.
- **Policy.** MadGraph publishes no policy on AI-assisted reports; checked on
  2026-10-16 (see the index,
  `research/kb/validation/madgraph-upstream-report-drafts.md`).
- **Before filing.** Re-run the claims from [`README.md`](README.md) and be
  ready to answer questions on them.
- **Disclosure.** Keep or reword the `Assisted-by` line at the end; it
  discloses the tool use.
- **Attach:** `repro.sh` from this folder.

---

**Title:** `setxqcuts`: the jet test on an s-channel vertex reads
`iforest(2,k,j)` twice and never `iforest(1,k,j)`, so the xqcut grid hint
depends on leg order

**Version:** 3.8.1 `Template/LO/SubProcesses/setcuts.f:948-951` (3.7.1
`:939-942`).

**Summary.** `xqcutij` sets the xqcut grid hint for a final-state pair joined
at an s-channel vertex with a jet daughter. That is the condition
`setclscales` uses for the `xqcut` rejection (`reweight.f:1066-1085`).

The condition is written as
`iabs(idup(iforest(2,…)))≤maxjetflavor .or. idup(iforest(2,…)).eq.21`, twice.
MadGraph stores the higher leg number first. So a (jet, non-jet) pair gets
the hint only when the jet has the lower leg number.

The hint feeds the s-channel minimum in `myamp.f`:
`xm(i) = max(xm(i), xqcutij(l1,l2))`.

**Reproduce.** Generate `u u~ > g b b~` and `u u~ > b b~ g`, with
`ickkw = 1`, `xqcut = 20` and the default `maxjetflavor = 4`. Compare the
`xqcutij` table printed in `SubProcesses/P*/G1/run1_app.log`.

**Expected:** both orderings give 20 for the two g–b pairs.

**Observed:**
- `u u~ > g b b~` gives 20 for both pairs.
- `u u~ > b b~ g` gives 0 for both.
- With the fix it gives 20.

Same on 3.7.1 and 3.8.1. σ agrees across the three runs within its 0.9 % error.
The effect is on the grids; a σ effect has not been looked for.

**Fix:**

```diff
                   if(iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
      $                 idup(iforest(2,k,j),1,1).eq.21.or.
-     $                 iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
-     $                 idup(iforest(2,k,j),1,1).eq.21)then
+     $                 iabs(idup(iforest(1,k,j),1,1)).le.maxjetflavor.or.
+     $                 idup(iforest(1,k,j),1,1).eq.21)then
```

Assisted-by: claude-code:claude-opus-5-5
