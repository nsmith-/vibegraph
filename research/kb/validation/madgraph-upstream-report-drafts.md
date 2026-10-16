---
type: Audit
title: MadGraph upstream report drafts
description: "Upstream bug-report drafts for the MadGraph defects this project found, each checked against the latest release (3.8.1) and paired with a MadGraph-only reproducer; filing them is the user's step."
status: draft
tags: [madgraph, defects, upstream-report, mlm, h1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: n41-15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L191-L205", title: "Note 41 §1.5, MadGraph defects met on the way"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L489-L522", title: "Note 41 M0, the ipdgcl and vec_igraph censuses"}
  - {id: n41-r1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1375-L1414", title: "Note 41 R1, the MadGraph-only reproducer for H1"}
  - {id: n07-h1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L539-L727", title: "Note 07 appendix, the H1 finding and its first draft"}
  - {id: n07-older, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L367-L538", title: "Note 07 appendix, the AQCDUP, rambo.py and ALOHA P1D drafts"}
  - {id: mg381, resource: "https://github.com/mg5amcnlo/mg5amcnlo/tree/a5b17913ef0b6e1c872016f94d0fb610207dab47", title: "MadGraph5_aMC@NLO v3.8.1 (tag commit a5b17913, 2026-10-09)"}
  - {id: mg381-dsig, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc#L819-L857", title: "3.8.1 DSIGPROC: P1, the mirror and the first update_scale_coupling call"}
  - {id: mg381-rw1138, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/Template/LO/SubProcesses/reweight.f#L1138-L1147", title: "3.8.1 reweight.f, the scalefact block"}
  - {id: mg381-setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/Template/LO/SubProcesses/setcuts.f#L944-L958", title: "3.8.1 setcuts.f, setxqcuts"}
  - {id: mg381-cuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/Template/LO/SubProcesses/cuts.f#L639-L650", title: "3.8.1 cuts.f, the Durham-kT NMASSLESS count"}
  - {id: mg381-addm, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/madgraph/iolibs/template_files/addmothers.f#L100-L116", title: "3.8.1 addmothers.f, lconfig"}
  - {id: mg381-banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/madgraph/various/banner.py#L1821-L1831", title: "3.8.1 banner.py, RivetCard.setWeightName"}
  - {id: mg381-rewgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/a5b17913ef0b6e1c872016f94d0fb610207dab47/Template/LO/SubProcesses/reweight.f#L1534-L1538", title: "3.8.1 reweight.f, rewgt's jet labels"}
  - {id: mg-pr428, resource: "https://github.com/mg5amcnlo/mg5amcnlo/pull/428", title: "mg5amcnlo PR #428, fix the dead $ veto on outgoing-fermion propagators"}
  - {id: mg-ebbdc97, resource: "https://github.com/mg5amcnlo/mg5amcnlo/commit/ebbdc974c7f697071cf3a90f0981a3047c4597b4", title: "mg5amcnlo commit ebbdc974, move back to refine_grid=False"}
---
# MadGraph upstream report drafts

Report drafts for the MadGraph5_aMC@NLO defects this project found. Each is
checked against the latest release and paired with a reproducer that needs only
a stock MadGraph. **Nothing here has been filed.** Filing is the user's step
([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md)),
and a filed report's link goes in its section below. The register of what each
defect means for this crate is [madgraph-defects](madgraph-defects.md).

## Versions

| | tag | commit | date |
|---|---|---|---|
| pinned | `v3.7.1` | `b7687064` | 2026-04-29 |
| latest | `v3.8.1` | `a5b17913` | 2026-10-09 |

- **Latest release.** `v3.8.1` is the highest tag on the GitHub repository and
  its default-branch head. Launchpad's newest tarball is
  `MG5_aMC_v3.8.1.tar.gz`. Both were checked on 2026-10-16.[^mg381]
- **How the trees were installed.** Both are shallow clones of their tags into
  a scratch directory, run with the `madgraph` pixi environment's Python 3.11
  and Fortran toolchain. The pinned submodule was not touched.
- **Which line numbers.** Lines are 3.8.1's unless marked; the 3.7.1 line
  follows where it differs.
- **How each result was obtained.** Each status below is marked *ran* (a
  reproducer run on both versions, output in its README) or *read* (source
  only).

## Status

| defect | in 3.8.1 | where in 3.8.1 | reproducer |
|---|---|---|---|
| H1: grouped first `setclscales` on the unpermuted point | present (ran) | `super_auto_dsig_group_v4.inc:856` | runs: [`permuted_first_call`](../../../validation/madgraph/repro/permuted_first_call/README.md) |
| `fixed_fac_scale1.or.fixed_fac_scale2` precedence | present (ran) | `reweight.f:1138` | runs: [`fixed_fac_scale_precedence`](../../../validation/madgraph/repro/fixed_fac_scale_precedence/README.md) |
| `setxqcuts` tests `iforest(2)` twice | present (ran) | `setcuts.f:948-951` (3.7.1 `:939-942`) | runs: [`setxqcuts_duplicate_test`](../../../validation/madgraph/repro/setxqcuts_duplicate_test/README.md) |
| `NMASSLESS` `.and.`/`.or.` precedence | present (ran) | `cuts.f:643-644` and `:745-746` (3.7.1 `:565-566`, `:667-668`) | runs: [`ktdurham_nmassless_precedence`](../../../validation/madgraph/repro/ktdurham_nmassless_precedence/README.md) |
| `addmothers` compares `igraphs(1)` to a stale loop index | present (read) | `addmothers.f:115` | none: unreachable, below |
| `RivetCard.setWeightName` `.str(…)` | present (ran) | `banner.py:1831` (3.7.1 `:1706`) | code level: [`rivet_setweightname`](../../../validation/madgraph/repro/rivet_setweightname/README.md) |
| `rewgt` keeps a non-jet leg's stale `ipdgcl` | present (ran) | `reweight.f:1537-1538` | runs: [`rewgt_stale_ipdgcl`](../../../validation/madgraph/repro/rewgt_stale_ipdgcl/README.md) |
| `AQCDUP`/`AQEDUP` divide by `3.1415926` | present (read) | `unwgt.f:768-769` | draft in note 07; not rerun |
| ALOHA `P1D`: `$` never vetoes `FFV2P1D_1` | **fixed** (ran) | `create_aloha.py:263-280` | fixed by PR #428; do not file |
| `rambo.py` massive overflow warning tests `iwarn[4] > 5` | present (read) | `rambo.py:218` | none needed |

**The ALOHA `P1D` defect is fixed in 3.8.1.**
- **The fix.** Upstream commits `a8561cf80`/`b496afe81` ("ALOHA: fix the dead $
  veto on outgoing-fermion propagators", 2026-09-24) leave an even power of a
  flipped momentum unsigned. Follow-ups `822525b2b`, `4e9883b33` and
  `6ffb13e07` extend this to `(P)**(2)` and `P**2e0`. All came through PR #428,
  merged on 2026-09-29.[^mg-pr428]
- **What the generated routines show.** Generated by 3.8.1, `generate u u~ > w+ b w- b~ $ t t~` gives
  `FFV2P1D_1.f` with
  `THETA_FUNCTIONR((TMP2*(-1D0)*(-TMP2+FCT1+FCT0)+FCT1*FCT0)`, the same form as
  `FFV2P1D_2.f`. 3.7.1 gives `(TMP2*(TMP2+FCT1+FCT0)+FCT1*FCT0)`.
- **Consequence.** Note 07's draft is obsolete for the latest release. The
  register's reference still uses the patched 3.7.1 ALOHA.

**The `AQCDUP` and `rambo.py` drafts** (note 07's appendix)[^n07-older] stand as
written, apart from line numbers: `unwgt.f:768-769` and `rambo.py:218` in
3.8.1. Both were checked by reading only; their reproducers were not rerun.

---

## H1: grouped MadEvent sets the scales on the unpermuted point

*Reproducer:* `validation/madgraph/repro/permuted_first_call/` (from note 41
R1[^n41-r1]; the first draft is in note 07[^n07-h1]). *Fix:*
`validation/madgraph/patches/first-call-unpermuted-momenta.patch`.

> **Title:** Grouped MadEvent sets the dynamical scales (and applies the MLM
> `xqcut` rejection) on the unpermuted phase-space point: σ is 3–4 % low when a
> subprocess group carries symmetry permutations
>
> **Version:** MadGraph5_aMC@NLO 3.8.1 (`a5b17913`) and 3.7.1,
> `madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`.
>
> **Summary.** In grouped output, `DSIGPROC` builds
> `P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))` (`:819`) and mirrors it
> (`:829`). The matrix element and `REWGT`'s second `setclscales` call read
> `P1`. But `update_scale_coupling(pp, wgt)` (`:856`) hands the **unpermuted**
> `PP` to the first `setclscales` call.
>
> That call does three things that depend on the momenta:
> - it sets μR;
> - it sets the PDF scales `DSIG` evaluates the densities at;
> - it sets `q2bck`, and it can reject the point through the `xqcut` check.
>
> On a channel whose symmetry permutation is not the identity, it clusters an
> event whose momenta do not match the flavours and diagram that are evaluated.
> - With the default `dynamical_scale_choice = -1`, this changes μR and μF.
> - With `ickkw = 1`, it also rejects points that `P1` would keep.
>
> Non-grouped output never permutes and is unaffected.
>
> **Reproduce** (`define q = u d; generate u q > z u q`, 13 TeV, default cards
> with `nevents = 5000`, five `iseed` values, each in a fresh copy of the
> generated directory). The one-line fix below is applied to the generated
> `auto_dsig.f`. Errors are the spread over seeds.
>
> | version, card | grouped (pb) | grouped + fix (pb) | shift |
> |---|---|---|---|
> | 3.8.1, default | 106.64 ± 0.20 | 110.55 ± 0.19 | +3.7 %, 14.6σ |
> | 3.8.1, `ickkw = 1`, `xqcut = 40` | 55.61 ± 0.09 | 57.53 ± 0.09 | +3.4 %, 15.4σ |
> | 3.7.1, default | 104.28 ± 0.34 | 108.62 ± 0.35 | +4.2 %, 8.9σ |
> | 3.7.1, `ickkw = 1`, `xqcut = 40` | 54.48 ± 0.14 | 56.33 ± 0.13 | +3.4 %, 9.6σ |
>
> **Expected against observed.** In 3.7.1 the same test with
> `set group_subprocesses False` gives 108.60 ± 0.45 pb (default card) and
> 56.14 ± 0.10 pb (matched). The fixed grouped output agrees with those values;
> the stock grouped output is 4.0 % and 3.0 % low.
>
> In 3.8.1, non-grouped and grouped differ for a second reason as well; see
> "Note on 3.8.1" below.
>
> `P1_qq_zqq/symperms.inc` shows the `(1,2,3,5,4)` permutations on configs 2,
> 5, 6 and 8. Per channel, the deficit sits in `G4` and `G7` (and `G1` with
> matching).
>
> **Fix** (scalar path):
>
> ```diff
>        IF (VECSIZE_MEMMAX.LE.1.and.imode.ne.5) THEN ! no-vector (NB not VECSIZE_USED!)
> -            call update_scale_coupling(pp, wgt)
> +            call update_scale_coupling(p1, wgt)
>        endif
> ```
>
> The vectorised path has the same shape and needs a larger change.
> `DSIG_VEC` calls `update_scale_coupling_vec(all_p, …)` (`:317`) before
> `DSIGPROC_VEC` builds and mirrors `ALL_P1`. The call has to move after the
> mirror loop, with the momenta copied into the `(4*maxdim/3+14, VECSIZE)`
> layout that routine reads. That change was found by reading and has not been
> measured.
>
> **Note on 3.8.1.** In 3.8.1, `ebbdc974` ("move back to refine_grid=False",
> `gen_ximprove.py`)[^mg-ebbdc97] changed grouped σ independently of this
> defect. Even the unpermuted channel `G3` rises by about 2 %, so 3.8.1's
> grouped output no longer matches its non-grouped output. On 3.8.1:
> - the fixed grouped output reads +1.8 % (4.0σ) above non-grouped;
> - the stock grouped output reads −1.8 %.
>
> Restoring `keep_grid_for_refine = True` in 3.8.1 alone restores 3.7.1's
> picture, with five seeds at the default card:
> - grouped: 104.38 ± 0.29 pb;
> - grouped + fix: 108.65 ± 0.32 pb;
> - non-grouped: 108.60 ± 0.45 pb.
>
> That is −3.9 % (−7.9σ) for grouped, and +0.05 % (0.1σ) for grouped + fix. So compare grouped against
> grouped + fix at the same seeds, as in the table, to see this defect on
> 3.8.1.

---

## `setclscales` skips `scalefact` on beam 2 when only beam 1's μF is fixed

*Reproducer:* `validation/madgraph/repro/fixed_fac_scale_precedence/` (ran on
both versions).

> **Title:** `setclscales`: `if(.not. fixed_fac_scale1.or. fixed_fac_scale2)`
> drops `scalefact` (and `q2bck`) for beam 2 when only `fixed_fac_scale1` is
> set
>
> **Version:** 3.8.1 and 3.7.1, `Template/LO/SubProcesses/reweight.f:1138` (the
> same line in both).
>
> **Summary.** The block that applies `scalefact²` to the dynamical
> factorisation scales, and copies them to `q2bck`, is guarded by
> `.not. fixed_fac_scale1.or. fixed_fac_scale2`. Fortran reads that as
> `(.not.fixed_fac_scale1) .or. fixed_fac_scale2`.
>
> With `fixed_fac_scale1 = T, fixed_fac_scale2 = F`, the guard is false. The
> block is then skipped:
> - `q2fact(2)` keeps the unscaled clustering scale;
> - `q2bck(2)` is not set.
>
> The block is reached with `dynamical_scale_choice = -1`, matched or not.
>
> **Reproduce.** Run `generate p p > z` with:
> - `dynamical_scale_choice = -1` and `scalefact = 2`;
> - `fixed_fac_scale1 = T`, `fixed_fac_scale2 = F`;
> - `dsqrt_q2fact1 = 10`.
>
> SCALUP = `sqrt(max(q2fact))` is then beam 2's scale, and for 2 → 1 the
> clustering scale is √ŝ.
>
> **Expected:** SCALUP = 2 √ŝ on every event, as with `(F,T)`, `(F,F)` or the
> fix.
>
> **Observed:** SCALUP = √ŝ on every one of 1000 events, on 3.7.1 and on 3.8.1.
> With the fix, σ rises by 10.4 % (3.8.1: 38 500 → 42 503 pb, one seed each).
>
> **Fix:**
>
> ```diff
> -      if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then
> +      if(.not.fixed_fac_scale1.or..not.fixed_fac_scale2) then
> ```
>
> This is the form the same file's vectorised path writes at `:1903`.

---

## `setxqcuts` tests the same daughter twice

*Reproducer:* `validation/madgraph/repro/setxqcuts_duplicate_test/` (ran on both
versions).

> **Title:** `setxqcuts`: the jet test on an s-channel vertex reads
> `iforest(2,k,j)` twice and never `iforest(1,k,j)`, so the xqcut grid hint
> depends on leg order
>
> **Version:** 3.8.1 `Template/LO/SubProcesses/setcuts.f:948-951` (3.7.1
> `:939-942`).
>
> **Summary.** `xqcutij` sets the xqcut grid hint for a final-state pair joined
> at an s-channel vertex with a jet daughter. That is the condition
> `setclscales` uses for the `xqcut` rejection (`reweight.f:1066-1085`).
>
> The condition is written as
> `iabs(idup(iforest(2,…)))≤maxjetflavor .or. idup(iforest(2,…)).eq.21`, twice.
> MadGraph stores the higher leg number first. So a (jet, non-jet) pair gets
> the hint only when the jet has the lower leg number.
>
> The hint feeds the s-channel minimum in `myamp.f`:
> `xm(i) = max(xm(i), xqcutij(l1,l2))`.
>
> **Reproduce.** Generate `u u~ > g b b~` and `u u~ > b b~ g`, with
> `ickkw = 1`, `xqcut = 20` and the default `maxjetflavor = 4`. Compare the
> `xqcutij` table printed in `SubProcesses/P*/G1/run1_app.log`.
>
> **Expected:** both orderings give 20 for the two g–b pairs.
>
> **Observed:**
> - `u u~ > g b b~` gives 20 for both pairs.
> - `u u~ > b b~ g` gives 0 for both.
> - With the fix it gives 20.
>
> Same on 3.7.1 and 3.8.1. σ agrees across the three runs within its 0.9 % error.
> The effect is on the grids; a σ effect has not been looked for.
>
> **Fix:**
>
> ```diff
>                    if(iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
>       $                 idup(iforest(2,k,j),1,1).eq.21.or.
> -     $                 iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
> -     $                 idup(iforest(2,k,j),1,1).eq.21)then
> +     $                 iabs(idup(iforest(1,k,j),1,1)).le.maxjetflavor.or.
> +     $                 idup(iforest(1,k,j),1,1).eq.21)then
> ```

---

## A decayed `b` switches the `ktdurham` (and `ptlund`) cut on

*Reproducer:* `validation/madgraph/repro/ktdurham_nmassless_precedence/` (ran on
both versions).

> **Title:** `cuts.f`: `NMASSLESS` counts every `b`, including decay products and
> non-merging partons (`.and.`/`.or.` precedence), so the Durham-kT cut is
> applied to events with only massive merging partons
>
> **Version:** 3.8.1 `Template/LO/SubProcesses/cuts.f:643-644` (Durham kT) and
> `:745-746` (ptlund). In 3.7.1 these are `:565-566` and `:667-668`.
>
> **Summary.** The cut is skipped when no massless merging parton is present.
> The source comment says this is "SINCE WE DO NOT WANT TO APPLY A CUT FOR JUST
> A SINGLE MASSIVE PARTICLE". The count is
> `is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and. is_a_j(i).or.is_a_b(i)`,
> which Fortran reads as `(… .and. is_a_j(i)) .or. is_a_b(i)`.
>
> Any `is_a_b` parton therefore counts. At `maxjetflavor = 4` that includes a
> `b` from a decay (with `cut_decays = F`) or one outside
> `pdgs_for_merging_cut`. The cut is then applied to the massive merging
> partons alone.
>
> **Reproduce.** Run `e+ e- > t t~ z, z > b b~` at √s = 1 TeV with default
> cards, at `ktdurham = -1` and at `ktdurham = 500`.
>
> **Expected:** the same σ, since the only merging partons are `t` and `t~`.
>
> **Observed:**
>
> | version | `ktdurham = -1` (pb) | `ktdurham = 500` (pb) | change |
> |---|---|---|---|
> | 3.7.1 | 6.818e-4 | 4.513e-4 | −34 % |
> | 3.8.1 | 6.839e-4 | 4.469e-4 | −35 % |
>
> With the fix, the `ktdurham = 500` run is bit-identical to the
> `ktdurham = -1` run.
>
> **Fix** (both places):
>
> ```diff
> -          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
> -     &                        is_a_j(i).or.is_a_b(i)) then
> +          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
> +     &                        (is_a_j(i).or.is_a_b(i))) then
> ```

---

## `addmothers` compares `igraphs(1)` to a stale loop index

*No reproducer: unreachable.*

> **Title:** `addmothers.f:115` compares `igraphs(1)` with the loop index `i`
> after its loop has ended
>
> **Version:** 3.8.1 and 3.7.1, `madgraph/iolibs/template_files/addmothers.f:115`
> (byte-identical file).
>
> **Summary.** The line reads
> `if (lconfig.eq.0.and.igraphs(1).ne.i.and.ivec.eq.1) lconfig = igraphs(1)`.
> Here `i` is the index of the `do i=-nexternal+2,nexternal` loop at `:100`, so
> it always holds `nexternal+1`.
>
> The test is meant to fall back to `igraphs(1)` when the event carries no
> matched graph. As written, it compares a graph number with a leg count.
> The likely intent is `igraphs(1).ne.0`.
>
> **Reachability.** The branch needs `vec_igraph(ivec) = 0` on a written event
> with `ickkw > 0`. By reading, this never happens:
> - `rewgt` (`reweight.f:1419-1471`) zeroes `vec_igraph(ivec)`.
> - It then either returns weight 0, so the event is not written, or sets it to
>   `igraphs(1)` after a successful `setclscales`.
> - `cluster` returns success only after storing at least one graph
>   (`cluster.f:663-665`, `:810-830`).
>
> Measured: `vec_igraph` was never 0 on the written events of five MLM samples
> in 3.7.1 (this project's M0 census).[^n41-m0]
>
> **Suggested change:** replace `igraphs(1).ne.i` with `igraphs(1).ne.0`, or
> drop the dead fallback. This is a code-quality report, with no observable
> effect today.

---

## `RivetCard.setWeightName` raises on every matched run

*Reproducer:* `validation/madgraph/repro/rivet_setweightname/` (code level,
ran on both versions).

> **Title:** `RivetCard.setWeightName`: `"Weight_MERGING={0}".str(…)` raises
> `AttributeError` for every `ickkw ≠ 0` run, so Rivet cannot run on an
> MLM-matched sample with the default `weight_name`
>
> **Version:** 3.8.1 `madgraph/various/banner.py:1831` (3.7.1 `:1706`).
>
> **Summary.** For `ickkw ≠ 0` with `weight_name = "default"`, the method builds
> the merged-weight name with `.str(round(py8card['JetMatching:qCut'],3))`, and
> `str` has no attribute `str`. `do_rivet`
> (`common_run_interface.py:3178`) calls it for every Rivet run that leaves
> `weight_name` at its default.
>
> **Reproduce** (no Rivet needed):
> `python check_setweightname.py <MG5 dir>`. It calls the method with a
> default `RunCardLO` at `ickkw = 1`, a `PY8Card` with
> `JetMatching:qCut = 30`, and a default `RivetCard`.
>
> **Expected:** `weight_name = 'Weight_MERGING=30.0'`.
>
> **Observed:** `AttributeError: 'str' object has no attribute 'str'`, on 3.7.1
> and on 3.8.1. An end-to-end Rivet run was not made, since Rivet is not
> installed here.
>
> **Fix:**
>
> ```diff
> -                self['weight_name'] = "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'],3))
> +                self['weight_name'] = "Weight_MERGING={0}".format(round(py8card['JetMatching:qCut'],3))
> ```

---

## MLM `rewgt` evaluates a non-jet parton's line with a stale flavour

*Reproducer:* `validation/madgraph/repro/rewgt_stale_ipdgcl/` (ran on both
versions).

> **Title:** MLM `rewgt`: a final-state parton that is not a jet keeps the PDG
> code an earlier event left in `ipdgcl`, so its PDF ratio is taken for the
> wrong flavour and depends on event history
>
> **Version:** 3.8.1 and 3.7.1, `Template/LO/SubProcesses/reweight.f:1537-1538`.
>
> **Summary.** `rewgt` labels the clustering tree with the drawn combination's
> codes for jet legs only
> (`if(isjet(idup(i,ipsel,iproc))) ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)`).
>
> `ipdgcl` is kept per (leg, graph, IPROC), not per combination.
> `initcluster` fills it from the IPROC's first combination (`cluster.f:373`).
> So a non-jet parton leg carries the code of:
> - the first combination; or
> - the last event that labelled that leg a jet.
>
> That code drives the flavour transmitted along the clustering, the density
> in the PDF-ratio chain and `goodjet`. The next event's first `setclscales`
> reads it too.
>
> **Reproduce.** Run `p p > z j` with:
> - `sm-no_b_mass`, with b in `p` and `j`;
> - `ickkw = 1`, `xqcut = 20`, `maxjetflavor = 4`;
> - `use_syst = T`, `systematics_program = none`.
>
> One IPROC of `P1_gq_zq` then holds `g d > z d`, `g s > z s` and
> `g b > z b`, and the b is not a jet. Read `<pdfrwt>` and `<asrwt>` on the
> `g b > z b` events.
>
> **Expected:** the same as when `g b > z b` is generated alone with the same
> cards, where the b is the first combination and its label is right. That is
> one α_s ratio and no PDF ratio on every event.
>
> **Observed:** on 3.7.1, 93 of 97 `g b` events take the PDF ratio of a `d` or
> `s` line (`<pdfrwt>` codes `21,±1` or `21,±3`), whichever an earlier event
> left. On 3.8.1, 87 of 97 do. Every `g u/c/d/s` event reads its own flavour.
>
> **Fix.** No one-line fix exists. Labelling every leg with the drawn code
> makes the b a non-jet in `setclscales` as well, and the per-config jet memo
> (`njetstore`/`iqjetstore`, `reweight.f:985-1018`) then aborts with
> `Error: Failed despite same graph` (tried on 3.7.1).
>
> A fix has to make two things consistent: the memo, or the grouping, with
> combinations of different jet-ness within one IPROC. The simplest option is
> not to group combinations whose final-state jet-ness differs at the run's
> `maxjetflavor`. Another is to key the memo by combination.

---

## Found while checking

These are new findings from this check. Neither has a report draft.

- **3.8.1's refine change moves grouped σ.** `ebbdc974` (`keep_grid_for_refine`
  `True → False` in `gen_ximprove_v4`, merged through 3.5.16 into 3.8.0)
  raises 3.8.1's grouped σ on `u q > z u q` by about 2 % in every channel,
  including the unpermuted `G3`. Non-grouped σ is unchanged: bit-identical per
  seed to 3.7.1 at the default card.
  - The step was located by bisecting the first-parent merges between the two
    tags at fixed seed 11: 109.63 → 110.78 pb at merge `31c57a65e`.
  - Reverting `e9576dbce` (the mirror-cuts fix) leaves 3.8.1 bit-identical.
  - Restoring `keep_grid_for_refine = True` on 3.8.1 brings the fixed grouped
    output back onto non-grouped: 108.65 ± 0.32 against 108.60 ± 0.45 pb over
    five seeds.
  - Which of the two settings integrates correctly is not established. With
    `keep_grid_for_refine = True` the two outputs agree. This bears on the
    default-survey under-integration candidate already filed under
    [madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md).
- **`fixed_fac_scale1/2` and `dsqrt_q2fact1/2` index process legs, not beams,
  in a mirrored subprocess.** In 3.8.1's generated `auto_dsig1.f`, beam 2's
  parton in the mirror (`IB(1) = 2`) is evaluated at `DSQRT(Q2FACT(1))`. The
  run card's comment says "fixed fac scale for beam 1". With one scale fixed
  this is not beam-symmetric: σ(`T,F`) ≠ σ(`F,T`) for `p p > z`, even with the
  `reweight.f:1138` fix.

[^mg381]: MadGraph5_aMC@NLO v3.8.1, tag commit `a5b17913`.
[^n41-r1]: Note 41 R1, the MadGraph-only reproducer for H1 and its results at 3.7.1.
[^n07-h1]: Note 07 appendix, "`super_auto_dsig_group_v4.inc` — Direct Bug Found", the first draft of H1's report.
[^n07-older]: Note 07 appendix: the `AQCDUP`, `rambo.py` and ALOHA `P1D` findings and drafts.
[^mg-pr428]: mg5amcnlo PR #428, merged 2026-09-29 into the 3.8.1 line.
[^mg-ebbdc97]: Upstream commit `ebbdc974`, 2026-04-16, "move back to refine_grid=False".
[^n41-m0]: Note 41 M0 "What the dump settles": `vec_igraph` is never 0 on a written event of the five MLM rows.
