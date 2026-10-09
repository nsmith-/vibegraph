---
type: Design Decision
title: Policy for MadGraph defects
description: "A defect that changes a weight on a supported card is reproduced bug-for-bug or refused, never silently fixed; a documented, registered deviation is the user-approved third outcome."
status: draft
tags: [madgraph, defects, parity, policy, decision]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L191-L205", title: "Note 41 §1.5 — MadGraph defects met, and the two-outcome policy"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1 — the reweight.f:1138 refusal and the permuted first call found"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3 — D2 diagnosis and the user's decisions of 2026-09-29"}
  - {id: n41-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3490-L3512", title: "Note 41 §5 — decisions settled 2026-09-28"}
  - {id: code-scales, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/coupling/scales.rs#L180-L195", title: "coupling/scales.rs — the reweight.f:1138 refusal"}
  - {id: code-manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml#L1272", title: "validation/manifest.toml — pp_to_ll_0j2j_mlm integrals note naming the deviation"}
---

# Policy for MadGraph defects

This crate's target is MadGraph LO parity
([release scope](../decisions/release-scope-lo-mlm.md),
[release scope in brief](../pipeline/release-scope.md)), so a defect in
MadGraph's own code is a question about what "parity" means on the cards it
reaches. The decision:

**A MadGraph defect that changes a weight on a card this crate supports has three
permitted outcomes, and "silently fixed" is not one of them.**

1. **Reproduce it bug-for-bug**, with a comment naming the defect at the site
   that reproduces it.
2. **Refuse the card** (or the card combination that reaches it) with an error
   naming the defect, consistent with
   [unsupported surfaces being hard errors](../pipeline/release-scope.md).
3. **Document a registered deviation** — this crate keeps the corrected
   behaviour — when reproducing is impractical and refusing would refuse a card
   users need. The deviation is measured, its size stated, and it is registered
   where every reader of an affected result sees it: the manifest notes of the
   affected rows, the per-event gates (which report affected events as `info`
   with the reason rather than failing or hiding them), and the defect report
   draft. This outcome was added by the user on 2026-09-29; the policy as first
   written had only the first two[^n41-15][^n41-m3].

Every defect found goes into the catalogue of
[MadGraph defects](madgraph-defects.md) with an upstream report draft; filing
them upstream is the user's
([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md),
[mlm-madgraph-defects-undrafted](../backlog/validation/mlm-madgraph-defects-undrafted.md)).
A defect that changes no weight (grids only, Python systematics, a record-only
field) is catalogued and needs no outcome.

## The cases on record

| defect | changes a weight? | outcome |
|---|---|---|
| `reweight.f:1138`: `.not.fixed_fac_scale1.or.fixed_fac_scale2` precedence | yes, only with exactly one fixed μF | **refused** under `ickkw = 1` (`coupling/scales.rs`, the message names `reweight.f:1138`)[^n41-dec] |
| `super_auto_dsig_group_v4.inc:842`: grouped MadEvent's first `setclscales` call reads the unpermuted `PP` while the matrix element reads the permuted `P1` | yes: +1.8 % of `P2_qq_llqq`, ≈ +0.25 % of `pp_to_ll_0j2j_mlm`'s `@2`; on an unmatched default-scale `u q > z u q` grouped vs non-grouped differs by −4.0 % | **registered deviation** "permuted P1": this crate clusters `P1`. Reproducing needs MadEvent's per-channel symmetry permutation, which this crate's channels (its own diagrams) do not carry; refusing would refuse the canonical MLM card. See [the permuted first call](madgraph-permuted-first-call.md) |
| `rewgt` reads the final-state `ipdgcl` left by the previous event | only if jet-ness differs between flavour combinations of one `IPROC` | not reached: a census found no such `IPROC` on any banked MLM row |
| `setcuts.f:939-942` duplicate `iforest(2)` test; `banner.py:1706` `setWeightName` | no | catalogued |
| `cuts.f:565` `ktdurham` precedence | CKKW-L only | out of scope (CKKW-L is refused) |
| `addmothers.f:115` compares `igraphs(1)` to a stale loop index | record only, and unreachable on the banked MLM rows: `vec_igraph` is never 0 on a written event (M0 census) | catalogued |

The permuted-first-call case also shows the outcome's discipline: before the
decision the per-event harness reported the affected events as
`info, permuted P1` without gating on them, and the σ gate carries no allowance
for the deviation. Together with the generic 2 → 4 offset (about +0.33 pb and
+0.6 pb, together about 0.7 % of `@2`) it sits inside 3σ only while this side's
`@2` error stays near the 0.66 pb measured, which the row's manifest note
says[^n41-m1].
A reproducer inside MadGraph alone (`validation/madgraph/repro/permuted_first_call/`)
and the one-line fix (`validation/madgraph/patches/first-call-unpermuted-momenta.patch`)
back the report. A consequence for references: grouped MadEvent references at
`dynamical_scale_choice = −1` whose directories carry non-identity permutations
are biased whether or not they are matched, and no banked `ickkw = 0` row has
been checked for such permutations
([h1-ickkw0-grouped-rows-unchecked](../backlog/validation/h1-ickkw0-grouped-rows-unchecked.md)).

## What this is not

- It is not a licence to diverge. A deviation needs a measurement and a user
  decision; the default is reproduce or refuse.
- Reference-side artefacts that this crate does not compute at all — MadGraph's
  Fortran rounding a UFO literal to seven digits, its `π` truncated to eight
  digits in `AQCDUP` — are not "fixed" by this crate either; they are recorded,
  and the affected comparisons stay informational or compare at the precision the
  reference wrote ([the coupling oracle](coupling-oracle.md)).

[^n41-15]: Note 41 §1.5.
[^n41-m1]: Note 41 M1, the permuted first call found and held as `info` pending the decision.
[^n41-m3]: Note 41 M3, "D2 decisions (user, 2026-09-29)" and R1.
[^n41-dec]: Note 41 §5 (c).
