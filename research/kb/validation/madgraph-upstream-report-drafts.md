---
type: Audit
title: MadGraph upstream report drafts
description: "Index of the MadGraph defects this project found, each checked against the latest release (3.8.1), with a folder per defect under research/upstream/ holding its MadGraph-only reproducer and, if still present, a report draft; filing is the user's step."
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

The MadGraph5_aMC@NLO defects this project found, each checked against the
latest release. Each defect has a folder under `research/upstream/` that needs
nothing from vibegraph. It holds:
- the MadGraph-only reproducer;
- a `README.md`: what it shows, the versions tested with exact commands and
  expected against actual output, file and line, root cause marked verified or
  inferred, and status;
- a `report-draft.md`, if the defect is still present upstream.

**Nothing here has been filed.** Filing is the user's step
([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md)).
A filed report's link goes in its row below. What each defect means for this
crate is in the register, [madgraph-defects](madgraph-defects.md).

## Index

Lines are 3.8.1's. "Ran" means the reproducer was run on 3.7.1 and 3.8.1, with
the output in the folder's README; "read" means source only.

| defect | folder | 3.8.1 status | file:line (3.8.1) | draft | filed |
|---|---|---|---|---|---|
| H1: grouped first `setclscales` on the unpermuted point | [`madgraph-grouped-first-call-unpermuted`](../../upstream/madgraph-grouped-first-call-unpermuted/README.md) | present (ran) | `super_auto_dsig_group_v4.inc:856` | yes | no |
| `scalefact` skipped with only `fixed_fac_scale1` set | [`madgraph-fixed-fac-scale-precedence`](../../upstream/madgraph-fixed-fac-scale-precedence/README.md) | present (ran) | `reweight.f:1138` | yes | no |
| `setxqcuts` tests `iforest(2)` twice | [`madgraph-setxqcuts-duplicate-test`](../../upstream/madgraph-setxqcuts-duplicate-test/README.md) | present (ran) | `setcuts.f:948-951` | yes | no |
| `NMASSLESS` `.and.`/`.or.` precedence (`ktdurham`, `ptlund`) | [`madgraph-nmassless-precedence`](../../upstream/madgraph-nmassless-precedence/README.md) | present (ran) | `cuts.f:643-644`, `:745-746` | yes | no |
| `addmothers` compares `igraphs(1)` to a stale loop index | [`madgraph-addmothers-stale-index`](../../upstream/madgraph-addmothers-stale-index/README.md) | present, unreachable at runtime (read) | `addmothers.f:115` | yes (code quality) | no |
| `RivetCard.setWeightName` `.str(…)` | [`madgraph-rivet-setweightname`](../../upstream/madgraph-rivet-setweightname/README.md) | present (ran, code level) | `banner.py:1831` | yes | no |
| `rewgt` keeps a non-jet leg's stale `ipdgcl` | [`madgraph-rewgt-stale-ipdgcl`](../../upstream/madgraph-rewgt-stale-ipdgcl/README.md) | present (ran) | `reweight.f:1537-1538` | yes | no |
| ALOHA `P1D`: `$` never vetoes `FFV2P1D_1` | [`madgraph-aloha-p1d-flipped-fermion`](../../upstream/madgraph-aloha-p1d-flipped-fermion/README.md) | **fixed in 3.8.1** (ran), PR #428 | `create_aloha.py:263-280` | no | n/a |
| `AQCDUP`/`AQEDUP` divide by `3.1415926` | none; note 07's appendix[^n07-older] | present (read) | `unwgt.f:768-769` | note 07 | no |
| `rambo.py` massive overflow warning tests `iwarn[4] > 5` | none; note 07's appendix[^n07-older] | present (read) | `rambo.py:218` | note 07 | no |

The `AQCDUP` and `rambo.py` drafts were checked only by reading the 3.8.1
lines. They have no folder or reproducer here.

## Versions

| | tag | commit | date |
|---|---|---|---|
| pinned | `v3.7.1` | `b7687064` | 2026-04-29 |
| latest | `v3.8.1` | `a5b17913` | 2026-10-09 |

- **Latest release.** `v3.8.1` is the highest tag on the GitHub repository and
  its default-branch head. Launchpad's newest tarball is
  `MG5_aMC_v3.8.1.tar.gz`. Both were checked on 2026-10-16.[^mg381]
- **How the trees were installed.** Both are shallow clones of their tags,
  run with the `madgraph` pixi environment's Python 3.11 and gfortran. The
  pinned submodule was not touched.

## MadGraph's policy on AI-assisted reports

MadGraph publishes none; checked on 2026-10-16. The places checked:

- the repository: its `README.md` (bugs go to
  <https://bugs.launchpad.net/mg5amcnlo> or a GitHub issue), with no
  CONTRIBUTING, policy or agent-rules file, and no organisation `.github`
  repository for `mg5amcnlo`;
- the Launchpad project, bug-tracker and answers pages
  (<https://launchpad.net/mg5amcnlo>, <https://bugs.launchpad.net/mg5amcnlo>,
  <https://answers.launchpad.net/mg5amcnlo>), which carry no reporting
  guidelines;
- a web search.

Upstream's own development uses AI assistance openly. 262 commits between
`v3.7.1` and `v3.8.1`, including the ALOHA fix and `e9576dbce`, carry a
`Co-Authored-By: Claude …` trailer.

The drafts are therefore written as reports for the user to review, edit and
file. Each ends in an `Assisted-by: claude-code:claude-opus-5-5` line that
the user can keep or reword. This differs from rustc's, whose LLM policy bans
LLM-originated issue text.

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
[^n07-older]: Note 07 appendix: the `AQCDUP`, `rambo.py` and ALOHA `P1D` findings and drafts.
