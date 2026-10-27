---
type: Backlog Item
title: "The mixed-mass known-wrong control is a borrowed value, not an integration"
description: "The control reads the e part's σ against MadGraph's τ directory, backed by a structural assert; no integration of p p > ta+ ta- with MTA = 0 pins the claim numerically."
area: validation
state: open
priority: low
closes_when: "A test integrates p p > ta+ ta- with the tau mass set to zero and asserts it agrees with the e part within the gate's tolerance."
blocked_by: []
opened: 2026-10-26
tags: [known-wrong-control, flavour-groups, sigma]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v2, resource: "../../sprints/flavour-mixed-masses/sessions/V2-report.md", title: "flavour-mixed-masses V2 report, Found 2"}
---
`sigma_mixed_outgoing_masses_vs_mg` prints a control: the e part's five-seed σ
read against MadGraph's τ directory, which must miss both bounds (pull +16.7,
rel +1.21 % on the M3's numbers). It stands for "the τ part integrated on
massless channels". A structural assert backs the substitution (same groups,
members, diagrams and incoming legs), but the two evaluators prune different
helicity configurations, since the tau is massive
([V2](../../sprints/flavour-mixed-masses/sessions/V2-report.md)).

A direct integration at `MTA = 0` would turn the substitution into a measurement.
