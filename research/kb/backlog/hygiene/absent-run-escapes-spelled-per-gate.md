---
type: Backlog Item
title: "Each banked gate spells its own escape for a planned, unbundled or awaiting row"
description: "dyn_run_present, run_present, check_dy_run, not_yet_banked, seed_record_banked, awaiting_rebank and the unbundled_rows() readers each decide what an absent run means; no row uses them now."
area: hygiene
state: open
priority: medium
closes_when: "One tests/common/manifest.rs helper returns a row's declared state (bundled, unbundled, planned, awaiting rebank), every banked gate routes through it, and a test of the helper covers each state."
blocked_by: []
opened: 2026-10-26
tags: [tests, manifest, duplication, banked]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: ht-hy1, resource: "../../sprints/ht-cuts/sessions/HY1-report.md", title: "ht-cuts HY1 report, Found 3"}
  - {id: ht-v1, resource: "../../sprints/ht-cuts/sessions/V1-report.md", title: "ht-cuts V1 report, Found 1"}
  - {id: ub-v2, resource: "../../sprints/unequal-beams/sessions/V2-report.md", title: "unequal-beams V2 report, Found 1"}
  - {id: fm-v1, resource: "../../sprints/flavour-mixed-masses/sessions/V1-report.md", title: "flavour-mixed-masses V1 report, Found 4"}
  - {id: gg-hy1, resource: "../../sprints/gridpack-generate/sessions/HY1-report.md", title: "gridpack-generate HY1 report, Found 4"}
---
Five sessions found the same thing. Three states let a banked gate pass over an
absent run, and each gate spells the check itself:

- `bundled = false` (`AWAITING BUNDLE`): the `unbundled_rows()` readers in
  `validate_sigma.rs`, `validate_alphas.rs`, `validate_scales.rs`,
  `validate_hadronic.rs` and `vibegraph-cli/tests/validate_samples_proton.rs`;
- `status = "planned"` (`NOT YET BANKED`): `planned_rows()` in
  `validate_hadronic.rs` and `validate_samples_proton.rs`, plus the collator's
  planned branch;
- `AWAITING_REBANK_RUNS` / `awaiting_rebank` (`tests/common/mod.rs`), now empty.

A census on `main` at `cfd9f7a` (`grep` for the helper names and the two
messages) counts 34 matching lines in six files. With refdata-11 pinned, no manifest row
is planned or unbundled, so none of these branches runs on any committed row.
That is when they rot: two PRs of the sprint went red because a planned-row
test read a row's state that a later commit changed
([ht-cuts HY1](../../sprints/ht-cuts/sessions/HY1-report.md) H1–H2,
[flavour-mixed-masses HY1](../../sprints/flavour-mixed-masses/sessions/HY1-report.md) H1–H2).
