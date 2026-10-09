---
type: Backlog Item
title: No MLM row separates the two readings of the resonance code
description: On every MLM row the clustered configuration and the integration channel give the Z the same code, so no gate tells MadEvent's two readings apart.
area: validation
state: open
priority: medium
closes_when: A banked row exists where the clustered configuration's and the integration channel's resonance codes differ (or where isbw's stale flag is reachable), and the status-2 record gate passes on it.
blocked_by: []
opened: 2026-09-29
tags: [mlm, lhef, resonance, coverage]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L262-L267", title: "TODO.md entry T009"}
---
`validate_mlm_dumps` checks the status-2 lines event by event: code and legs of
the clustered configuration's timelike propagators whose leg sets the
integration channel's `cut_bw` put on a Breit–Wigner (`checkbw`'s `isbw`).
On every MLM row the Z gets the same code from both readings. On
`pp_to_llj_mlm`, 9507 of 10000 events list the Z and on none does the
channel's forest give another code. The gate therefore cannot see which
reading MadEvent uses.

Two untested cases:
- A card where the clustered-configuration code and the channel code differ.
- A card where `isbw`'s stale flag is reachable. Entries are cleared only for
  the channel's own sets, so a clustered leg set that the channel lacks reads a
  stale flag. This is unreachable on these rows (the lepton pair).

Either would pin the reading.

Detail: [note 41 M4](../../history/notes/41-mlm-feature-sprint-plan.md).
