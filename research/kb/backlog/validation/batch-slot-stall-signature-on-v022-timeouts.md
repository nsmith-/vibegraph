---
type: Backlog Item
title: "v022-1009 batch timeouts show a stall signature: high sys CPU, empty stdout, on a card that refuses in 16 s"
description: "Card 290 timed out on 5 batch seeds at 40–55% sys CPU with empty stdout; the same release binary on the same inputs refuses it in 16.6 s. Cause and reach unmeasured."
area: validation
state: open
priority: medium
closes_when: "A census of v022-1009 timeouts lists those with the signature (sys share > 25% plus empty stdout or no stage rows) by family; signature cards are re-classified where a re-run finishes or refuses within the cap; and the slot-side cause is identified from a re-run with stdout and a trace captured, or the item records why it cannot be."
blocked_by: []
opened: 2026-11-23
tags: [batch, harness, timeout, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Card 290 (VBFWto2Q, `loop_sm`) timed out on all five v022-1009 seeds.
- On the batch proxy: 1216–1274 s wall, 437–707 s of sys CPU against
  552–777 s user, about 690 MB RSS, and an empty stdout capture on every
  seed.
- Re-run on the validator host: the released musl-v3 v0.2.2 binary, on the
  same staged inputs, refuses it as a named `process` refusal (a massless
  spacelike propagator with the beam uncut) in 16.6 s at 123 MB.

This is an observed signature, not a diagnosis. Candidates:
- the batch job read `loop_sm` from the template's model store rather than
  the bundled directory;
- cvmfs PDF access;
- stage-in.

The records cannot separate them. Other v022 timeouts with a high sys share
and empty stdout may share the cause, so they would count as
environment-caused, not as cost.

Evidence: the "Card 290 and card 722" section of
[the phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).

**Census (2026-11-23).** The signature reduces to a sys share above 25%
(stdout and stage rows are empty for every killed job). 39 of 1105 timeout
attempts carry it, on 11 cards, and so do 63 refusals. 36 attempts are
reclassified as environmental, including 6 of the 178 never-retried cards.
The account is in the [sprint L log](../../sprints/v023-orders-majorana-timeouts/log.md)
(2026-11-23). Still open: local confirmation on 136 and 726, and the
slot-side cause.
