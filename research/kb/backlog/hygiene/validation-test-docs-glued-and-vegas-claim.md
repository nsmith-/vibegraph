---
type: Backlog Item
title: "Two validation test docs are wrong: a glued doc and a stale VEGAS claim"
description: "sigma_llj_fixed_scale_vs_mg's doc runs into probe_fiducial_bound_on_llj_fixed's, and validate_sigma.rs's module doc says VEGAS combines iterations by 1/σ²."
area: hygiene
state: open
priority: low
closes_when: "Each doc in validate_hadronic.rs near sigma_llj_fixed_scale_vs_mg sits on its own item, and validate_sigma.rs's module doc describes vegas.rs's unweighted post-warm-up iteration combination."
blocked_by: []
opened: 2026-10-14
tags: [comments, validation-tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-14}
sources:
  - {id: seed-policy, resource: "../../validation/madevent-reference-seed-policy.md", title: "MadEvent reference seed policy"}
---
Found while moving the gates to one seed rule (2026-10-14).

- `vibegraph-lib/tests/validate_hadronic.rs`, near line 731: the doc of
  `sigma_llj_fixed_scale_vs_mg` continues straight into the doc of
  `probe_fiducial_bound_on_llj_fixed`.
- `vibegraph-lib/tests/validate_sigma.rs` module doc: says VEGAS combines its
  iterations by `1/σ²`; `IterationCombination::Unweighted` is the `#[default]`
  in `vegas.rs`.
