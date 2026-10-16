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
- **Attach:** `repro.sh`, `proc_grouped.mg5`, `proc_nongrouped.mg5`, `run_card.dat` and `run_card_ickkw0.dat` from this folder.

---

**Title:** Grouped MadEvent sets the dynamical scales (and applies the MLM
`xqcut` rejection) on the unpermuted phase-space point: σ is 3–4 % low when a
subprocess group carries symmetry permutations

**Version:** MadGraph5_aMC@NLO 3.8.1 (`a5b17913`) and 3.7.1,
`madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`.

**Summary.** In grouped output, `DSIGPROC` builds
`P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))` (`:819`) and mirrors it
(`:829`). The matrix element and `REWGT`'s second `setclscales` call read
`P1`. But `update_scale_coupling(pp, wgt)` (`:856`) hands the **unpermuted**
`PP` to the first `setclscales` call.

That call does three things that depend on the momenta:
- it sets μR;
- it sets the PDF scales `DSIG` evaluates the densities at;
- it sets `q2bck`, and it can reject the point through the `xqcut` check.

On a channel whose symmetry permutation is not the identity, it clusters an
event whose momenta do not match the flavours and diagram that are evaluated.
- With the default `dynamical_scale_choice = -1`, this changes μR and μF.
- With `ickkw = 1`, it also rejects points that `P1` would keep.

Non-grouped output never permutes and is unaffected.

**Reproduce** (`define q = u d; generate u q > z u q`, 13 TeV, default cards
with `nevents = 5000`, five `iseed` values, each in a fresh copy of the
generated directory). The one-line fix below is applied to the generated
`auto_dsig.f`. Errors are the spread over seeds.

| version, card | grouped (pb) | grouped + fix (pb) | shift |
|---|---|---|---|
| 3.8.1, default | 106.64 ± 0.20 | 110.55 ± 0.19 | +3.7 %, 14.6σ |
| 3.8.1, `ickkw = 1`, `xqcut = 40` | 55.61 ± 0.09 | 57.53 ± 0.09 | +3.4 %, 15.4σ |
| 3.7.1, default | 104.28 ± 0.34 | 108.62 ± 0.35 | +4.2 %, 8.9σ |
| 3.7.1, `ickkw = 1`, `xqcut = 40` | 54.48 ± 0.14 | 56.33 ± 0.13 | +3.4 %, 9.6σ |

**Expected against observed.** In 3.7.1 the same test with
`set group_subprocesses False` gives 108.60 ± 0.45 pb (default card) and
56.14 ± 0.10 pb (matched). The fixed grouped output agrees with those values;
the stock grouped output is 4.0 % and 3.0 % low.

In 3.8.1, non-grouped and grouped differ for a second reason as well; see
"Note on 3.8.1" below.

`P1_qq_zqq/symperms.inc` shows the `(1,2,3,5,4)` permutations on configs 2,
5, 6 and 8. Per channel, the deficit sits in `G4` and `G7` (and `G1` with
matching).

**Fix** (scalar path):

```diff
       IF (VECSIZE_MEMMAX.LE.1.and.imode.ne.5) THEN ! no-vector (NB not VECSIZE_USED!)
-            call update_scale_coupling(pp, wgt)
+            call update_scale_coupling(p1, wgt)
       endif
```

The vectorised path has the same shape and needs a larger change.
`DSIG_VEC` calls `update_scale_coupling_vec(all_p, …)` (`:317`) before
`DSIGPROC_VEC` builds and mirrors `ALL_P1`. The call has to move after the
mirror loop, with the momenta copied into the `(4*maxdim/3+14, VECSIZE)`
layout that routine reads. That change was found by reading and has not been
measured.

**Note on 3.8.1.** In 3.8.1, `ebbdc974` ("move back to refine_grid=False",
`gen_ximprove.py`) changed grouped σ independently of this
defect. Even the unpermuted channel `G3` rises by about 2 %, so 3.8.1's
grouped output no longer matches its non-grouped output. On 3.8.1:
- the fixed grouped output reads +1.8 % (4.0σ) above non-grouped;
- the stock grouped output reads −1.8 %.

Restoring `keep_grid_for_refine = True` in 3.8.1 alone restores 3.7.1's
picture, with five seeds at the default card:
- grouped: 104.38 ± 0.29 pb;
- grouped + fix: 108.65 ± 0.32 pb;
- non-grouped: 108.60 ± 0.45 pb.

That is −3.9 % (−7.9σ) for grouped, and +0.05 % (0.1σ) for grouped + fix. So compare grouped against
grouped + fix at the same seeds, as in the table, to see this defect on
3.8.1.

Assisted-by: claude-code:claude-opus-5-5
