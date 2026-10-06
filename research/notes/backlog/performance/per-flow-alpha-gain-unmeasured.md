---
type: Backlog Item
title: Per-flow channel-mixture α tuning has no gain measurement
description: Tuning a separate α per leading-colour stratum is unmeasured; the offline Kleiss–Pittau variance reduction against the evaluation overhead comes first.
area: performance
state: open
priority: low
closes_when: An offline measurement on recorded g_j(x), f(x), s_i(x) reports the achievable variance reduction against the ×(strata) evaluation overhead on uux_to_uux and gg_to_gg, and a sampler is built or rejected on it.
blocked_by: []
opened: 2026-08-01
tags: [multichannel, alpha, colour-flow, stratification, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1105-L1118", title: "TODO.md entry T102"}
---
The user proposed this on 2026-08-01. Stratify the integrand by leading-colour
share `s_i = |JAMP_i|²CF_ii / Σ_k |JAMP_k|²CF_kk`, which is positive and a
partition of unity, with interference apportioned pro rata. Then tune a separate
channel-mixture α per stratum.

**Measure first; do not build a sampler yet.** The Kleiss–Pittau optimal α and
its variance are computable offline from recorded `g_j(x)`, `f(x)` and `s_i(x)`
on existing samples. Report the achievable variance reduction against the
×(strata) evaluation overhead before building anything. `uux_to_uux` and
`gg_to_gg` are now valid test rows: their channel maps are not bit-identical
and their α is not uniform (note 28 §S4 B2). Flows overlap heavily, though, so
the gain lives in the inter-stratum covariance term and is expected to be
modest.

**Guardrail: split the tuning, never the coverage.** Every stratum keeps every
channel, with an α floor. Otherwise the `sde_strategy`-class fragility seen on
MadGraph's side comes back here.

Detail: [note 28 §B2](../../28-kt-spine-feature-sprint-plan.md),
[note 27 §B1](../../27-v3-backlog-plan.md).
