---
type: Backlog Item
title: validate_scales.rs's module doc describes an older banked set
description: "The module doc cites SCALUP at unwgt.f:686, <mgrwt> on 6 of 20 runs, one fixed-scale run and scalefact as unpinned; the gate now covers 49 clustered runs, three fixed and a scalefact run."
area: hygiene
state: open
priority: low
closes_when: "The module doc of vibegraph-lib/tests/validate_scales.rs describes the run lists, scalefact coverage and MadGraph lines (at the pin) the file actually has."
blocked_by: []
opened: 2026-10-06
tags: [comments, scales, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: phase1, resource: "../../history/notes/42-okf-knowledge-bundle-plan.md", title: "Found during note 42 Phase 1 (chunk classification of note 28)"}
  - {id: phase3, resource: "../../../../vibegraph-lib/tests/validate_scales.rs", title: "Phase 3 drafters D3, D10, D11; re-checked against the file 2026-10-09"}
---
Stale lines in the module doc of `vibegraph-lib/tests/validate_scales.rs`
(widened 2026-10-09 from the scalefact lines alone):

- `:9`: `SCALUP` is filled at `unwgt.f:686`. At the pinned 3.7.1 (b7687064) it
  is `unwgt.f:752`. The other files' copies of this citation are in
  [madgraph-line-citations-predate-pin](madgraph-line-citations-predate-pin.md).
- `:17-18`: `<mgrwt>` appears "with `use_syst`, which is 6 of the 20 banked
  runs. The other 14…". `CLUSTERED_RUNS` alone now lists 49 runs (~:133).
- `:46-48`: every banked run has `scalefact = 1`, so where MadGraph applies it,
  "and the one place it applies it twice", is pinned only by unit tests. Both
  halves are stale. `pp_to_ll_scalefact2` pins the placement (`SCALEFACT_RUNS`,
  ~:322), and the double application was read off MadGraph 3.5.7: the pinned
  3.7.1 applies it once per beam
  ([note 22](../../history/notes/22-dynamical-scales-plan.md) against
  [note 28 §K1](../../history/notes/28-kt-spine-feature-sprint-plan.md)).
- `:49-50`: "One banked run (`pp_to_llj_fixed`) pins all three scales".
  `FIXED_SCALE_RUNS` has three (`pp_to_bb_fixed`, `pp_to_llj_fixed`,
  `ud_to_epemud_qcd0`, ~:267).

Not checked: whether the neighbouring "geometric-mean structure … is
unpinned" bullet (`:40-45`) still holds (drafter D11 raised it).
