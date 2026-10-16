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

**Title:** `setclscales`: `if(.not. fixed_fac_scale1.or. fixed_fac_scale2)`
drops `scalefact` (and `q2bck`) for beam 2 when only `fixed_fac_scale1` is
set

**Version:** 3.8.1 and 3.7.1, `Template/LO/SubProcesses/reweight.f:1138` (the
same line in both).

**Summary.** The block that applies `scalefact²` to the dynamical
factorisation scales, and copies them to `q2bck`, is guarded by
`.not. fixed_fac_scale1.or. fixed_fac_scale2`. Fortran reads that as
`(.not.fixed_fac_scale1) .or. fixed_fac_scale2`.

With `fixed_fac_scale1 = T, fixed_fac_scale2 = F`, the guard is false. The
block is then skipped:
- `q2fact(2)` keeps the unscaled clustering scale;
- `q2bck(2)` is not set.

The block is reached with `dynamical_scale_choice = -1`, matched or not.

**Reproduce.** Run `generate p p > z` with:
- `dynamical_scale_choice = -1` and `scalefact = 2`;
- `fixed_fac_scale1 = T`, `fixed_fac_scale2 = F`;
- `dsqrt_q2fact1 = 10`.

SCALUP = `sqrt(max(q2fact))` is then beam 2's scale, and for 2 → 1 the
clustering scale is √ŝ.

**Expected:** SCALUP = 2 √ŝ on every event, as with `(F,T)`, `(F,F)` or the
fix.

**Observed:** SCALUP = √ŝ on every one of 1000 events, on 3.7.1 and on 3.8.1.
With the fix, σ rises by 10.4 % (3.8.1: 38 500 → 42 503 pb, one seed each).

**Fix:**

```diff
-      if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then
+      if(.not.fixed_fac_scale1.or..not.fixed_fac_scale2) then
```

This is the form the same file's vectorised path writes at `:1903`.

Assisted-by: claude-code:claude-opus-5-5
