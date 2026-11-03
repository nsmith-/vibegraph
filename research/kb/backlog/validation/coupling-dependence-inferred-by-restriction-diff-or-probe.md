---
type: Backlog Item
title: "Tests may infer coupling dependence by diffs or probes that miss joint dependencies"
description: "Diffing two restrictions misses couplings that are products of two parameters zero at both; moving one parameter misses a dependence whose cofactor is zero at the probe point."
area: validation
state: open
priority: low
closes_when: "Every test that infers which couplings depend on a parameter is listed, and each either uses NaN poisoning (sm_ckm_no_b_mass.rs poisoned) or states the blind spot in its doc."
blocked_by: []
opened: 2026-10-26
tags: [tests, coupling, restriction, blind-spot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "../../sprints/ckm-no-b-mass/sessions/F1-report.md", title: "ckm-no-b-mass F1 report, Found 3"}
  - {id: v1, resource: "../../sprints/ckm-no-b-mass/sessions/V1-report.md", title: "ckm-no-b-mass V1 report, Found 3"}
---
Two sessions on the ckm_no_b_mass restriction hit the same blind spot in two
forms:

- **Restriction diffs.** F1's first version found the b sector as "couplings
  that differ between `default` and `no_b_mass`", which missed GC_13, GC_14,
  GC_28 and GC_29: each is `yb` times an off-diagonal CKM element, and both
  factors vanish at both restrictions.
- **One-parameter probes.** V1 found that moving one parameter misses six
  `yc·CKM` couplings, whose cofactor is zero at the probe point.

`poisoned` in `vibegraph-lib/tests/sm_ckm_no_b_mass.rs` (a NaN in the parameter,
traced to every coupling it reaches) has neither blind spot. Ten test files call
`sm_model(SMRestrict::…)`; none has been checked for the pattern
([F1](../../sprints/ckm-no-b-mass/sessions/F1-report.md),
[V1](../../sprints/ckm-no-b-mass/sessions/V1-report.md)).
