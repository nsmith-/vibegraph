---
type: Backlog Item
title: A stored LHE file cannot be reweighted
description: Reweighting runs only at generation time; reweighting an existing .lhe needs each event's subprocess recovered from its record.
area: feature
state: open
priority: medium
closes_when: A command reweights an existing vibegraph .lhe with a reweight_card.dat and writes the same <rwgt> weights generate --reweight-card writes for those events.
blocked_by: []
opened: 2026-10-05
tags: [reweight, lhef, cli]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L668-L730", title: "TODO.md entry T059"}
---
`generate --reweight-card` computes per-event `|M|²_new/|M|²_old` of the
event's own concrete subprocess at generation time. A stored `.lhe` cannot be
reweighted afterwards: the event's subprocess (flavours, beam ordering, MLM
part) has to be recovered from its record. `vibegraph-cli/tests/reweight_mg_oracle.rs`
already does this test-side for its banked events, which is the starting point.
`generate`'s per-event audit (card-point `|M|²` and flavours against the
drawing integrand) is the check a stored-file path would replay.
