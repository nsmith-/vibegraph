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

**Title:** `cuts.f`: `NMASSLESS` counts every `b`, including decay products and
non-merging partons (`.and.`/`.or.` precedence), so the Durham-kT cut is
applied to events with only massive merging partons

**Version:** 3.8.1 `Template/LO/SubProcesses/cuts.f:643-644` (Durham kT) and
`:745-746` (ptlund). In 3.7.1 these are `:565-566` and `:667-668`.

**Summary.** The cut is skipped when no massless merging parton is present.
The source comment says this is "SINCE WE DO NOT WANT TO APPLY A CUT FOR JUST
A SINGLE MASSIVE PARTICLE". The count is
`is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and. is_a_j(i).or.is_a_b(i)`,
which Fortran reads as `(… .and. is_a_j(i)) .or. is_a_b(i)`.

Any `is_a_b` parton therefore counts. At `maxjetflavor = 4` that includes a
`b` from a decay (with `cut_decays = F`) or one outside
`pdgs_for_merging_cut`. The cut is then applied to the massive merging
partons alone.

**Reproduce.** Run `e+ e- > t t~ z, z > b b~` at √s = 1 TeV with default
cards, at `ktdurham = -1` and at `ktdurham = 500`.

**Expected:** the same σ, since the only merging partons are `t` and `t~`.

**Observed:**

| version | `ktdurham = -1` (pb) | `ktdurham = 500` (pb) | change |
|---|---|---|---|
| 3.7.1 | 6.818e-4 | 4.513e-4 | −34 % |
| 3.8.1 | 6.839e-4 | 4.469e-4 | −35 % |

With the fix, the `ktdurham = 500` run is bit-identical to the
`ktdurham = -1` run.

**Fix** (both places):

```diff
-          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
-     &                        is_a_j(i).or.is_a_b(i)) then
+          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
+     &                        (is_a_j(i).or.is_a_b(i))) then
```

Assisted-by: claude-code:claude-opus-5-5
