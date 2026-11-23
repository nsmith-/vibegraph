---
type: Backlog Item
title: "A batch slot stalled card 290 for 20 minutes when the binary refuses it in 16 s"
description: "v022-1009's card 290 timed out on all 5 batch seeds with 40–55% sys CPU, ~690 MB RSS and empty stdout, yet the same release binary on identical inputs refuses it in 16.6 s; the batch environment, not vibegraph, caused it."
area: validation
state: open
priority: medium
closes_when: "The stall's cause on the batch slot (model store vs bundled UFO, cvmfs PDF access, or stage-in) is identified from a re-run with stdout and a trace captured, and the v022-1009 timeouts sharing its signature (high sys share, empty stdout) are listed and re-classified."
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

Candidates:
- the batch job read `loop_sm` from the template's model store rather than
  the bundled directory;
- cvmfs PDF access;
- stage-in.

The records cannot separate them. Other v022 timeouts with a high sys share
and empty stdout may share the cause, so they would count as
environment-caused, not as cost.

Evidence: the "Card 290 and card 722" section of
[the phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
