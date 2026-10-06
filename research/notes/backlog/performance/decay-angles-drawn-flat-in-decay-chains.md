---
type: Backlog Item
title: Decay angles in decay chains are drawn flat
description: Forced decay invariants are Breit–Wigner-mapped but decay angles are flat, so ε_unw falls 2.6–16× on decayed rows against their cores.
area: performance
state: open
priority: low
closes_when: Shaped decay-angle maps (or a MadSpin-style decay step) are built and the decayed rows' ε_unw is remeasured against their cores' and the current figures.
blocked_by: []
opened: 2026-09-26
tags: [decay-chains, phase-space-map, unweighting, madspin, process-grammar]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L975-L983", title: "TODO.md entry T090"}
---
In decay-chain channels the forced invariants are Breit–Wigner-mapped exactly,
but the decay angles are drawn flat. Their V−A and spin-correlation shapes reach
max/mean of a few per decay, and a factorised VEGAS grid learns them only
partly. Against the undecayed core, `|M|²` costs 0.7–1.2× as much, while ε_unw
falls:
- 2.6× on `e+ e- > z z` decayed;
- 5.7× on `p p > t t~` fully decayed;
- 16× on `p p > t t~ h h` fully decayed, at 40.5 ms/event against the core's
  1.9, with 11× the core's integration points.

Neither fix is needed for correctness or convergence. Two levers are named:
shaped decay-angle maps in the channels, which work inside the sampler, and a
MadSpin-style decay step, roughly an order of magnitude per event at eight legs.

Detail: [note 38 §4 D3](../../38-process-grammar-sprint-plan.md) (the sampler ladder).
