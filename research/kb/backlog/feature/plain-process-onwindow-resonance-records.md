---
type: Backlog Item
title: Plain processes write no status-2 records for free on-window resonances
description: MadEvent writes status-2 records for free resonances inside bwcutoff in plain processes too; this generator writes them only on decay-chain cards and under matching.
area: feature
state: open
priority: medium
closes_when: "Plain-process events carry MadEvent's status-2 records for free on-window resonances, gated per event against a MadEvent run of one such row (e.g. the Z of e+ e- > mu+ mu- at the pole)."
blocked_by: []
opened: 2026-09-26
tags: [process-grammar, lhef, event-records, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L317-L332", title: "TODO.md entry T019"}
---
MadEvent writes a status-2 record for every s-channel line of `ICONFIG` that
`cut_bw` leaves `OnBW` (`addmothers.f:253`, `ickkw = 0`). That includes
free lines within `bwcutoff·Γ` with Γ/M < 0.1 in plain processes: the `Z` of
`e+ e- > mu+ mu-` at the pole, the `Z` of `p p > mu+ mu- / a`, the `W` of
`t > b e+ ve`. `SubprocessRecord::event_with_intermediates` writes these
records only on decay-chain cards and under matching (`generate.rs`).

Decided (user, 2026-10-09): match MadEvent. This changes every existing LHE
file for such processes, which is accepted; the record matters downstream,
since a shower keeps a resonance's mass when it reshuffles momenta only if the
resonance is in the record. The record rules are in the note: same-flavour daughter handling
(`myamp.f:146`), mothers, `elim_indices` colour, virtuality as mass. Sibling order may
stay by lowest outgoing leg, since MadGraph's order depends on the configuration
and carries no physics. Detail: [note 38 §4 E1](../../history/notes/38-process-grammar-sprint-plan.md).
