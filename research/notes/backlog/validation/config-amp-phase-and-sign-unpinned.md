---
type: Backlog Item
title: Configuration-amplitude phases and per-process sign patterns are not pinned
description: amplitude_oracle asserts only |k| = 1 per configuration although k/G is measured exactly ±1, and the measured per-process route-sign patterns are not banked.
area: validation
state: open
priority: medium
closes_when: amplitude_oracle asserts |Im(k/G)| < LINEAR_REL_TOL per configuration, and checks run_config_amps' signs against a banked per-process sign-pattern table; both fail when deliberately broken.
blocked_by: []
opened: 2026-08-03
tags: [amplitudes, phase-conventions, chain-f, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L454-L460", title: "TODO.md entry T031"}
---
`vibegraph-lib/tests/amplitude_oracle.rs:1405-1432` fits one constant `k` per
configuration amplitude and asserts only `|k| = 1`. `G` is already pinned to ±i.

**Phase.** Measured over all 113 configurations of the banked set, `k/G ∈ {±1}`
exactly; the suite-wide worst residual is 1.19e-13 (`ee_to_mumu_tata_qcd0`).
Asserting `|Im(k/G)| < LINEAR_REL_TOL` (1e-12) turns a free per-configuration
phase into a pinned bit. It catches any rotation of a configuration amplitude,
a class `AMP2` is blind to. It needs no reference data and no tolerance move.

**Sign patterns.** `run_config_amps()[i]` and the single-diagram compile differ
by exactly ±1 (spread 0, production evaluators). The pattern is non-uniform on
three processes and uniform on the other 11:

| process | sign pattern |
|---|---|
| `ee_to_tatah` | `+ + + + −` |
| `ee_to_mumua` | `− − − − + + + +` |
| `ee_to_mumu_tata_qcd0` | 17:8 |

This is inert today: `run_config_amps` has no production consumer, and
`eval_amp2` is sign-blind. Asserting agreement would fail, and asserting
uniformity would be false. Pin the 14 per-process sign vectors as banked data
instead.

The two checks are independent.

Detail: [note 29 §F.14](../../29-v01-validation-sprint-plan.md).
