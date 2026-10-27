---
type: Backlog Item
title: "The mixed-mass gate classifies MadGraph's P directories by their label"
description: "The seed record holds no per-directory process line, so lepton_part_of reads the final token of P<N>_<initial>_<final>; a renamed label panics rather than misclassifies."
area: validation
state: open
priority: low
closes_when: "bank_seeded_sigma.sh records each P directory's process line or PDG list in the seed record, and lepton_part_of classifies on PDG codes and masses."
blocked_by: []
opened: 2026-10-26
tags: [seed-record, madevent-reference, flavour-groups]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v2, resource: "../../sprints/flavour-mixed-masses/sessions/V2-report.md", title: "flavour-mixed-masses V2 report, Found 1"}
---
`lepton_part_of` (`vibegraph-lib/tests/validate_hadronic.rs`) maps
`ll`/`epem`/`emep` to the massless part and `taptam`/`tamtap` to the tau part,
and panics on anything else. MadGraph 3.7.1 named the massless group `P1_qq_ll`,
which F1 had not expected; the label carries the first process written into the
group, not its masses. The record (`SeedRun`, `ChannelValues`) holds channel keys
and file names only
([V2](../../sprints/flavour-mixed-masses/sessions/V2-report.md)).

The failure mode is loud, so this is robustness, not a wrong number.
