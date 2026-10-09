---
type: Backlog Item
title: The Pythia gate proves our LHE files readable, not read as meant
description: Nothing checks Pythia's reading of SCALUP, AQCDUP or the <init> cross section, and only the Buffer (IDWTUP = -4) strategy is ever fed to Pythia.
area: validation
state: open
priority: medium
closes_when: The Pythia gate asserts that Pythia's scale, αs and init cross section match what was written, and it feeds a StochasticRounding (IDWTUP = +3) sample through the same checks.
blocked_by: []
opened: 2026-08-06
tags: [pythia, lhef, idwtup, scalup]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L496-L505", title: "TODO.md entry T037"}
---
`validation/pythia/consume.py` shows that Pythia accepts every event of the
emitted samples. It does not show that Pythia interprets them as intended.

**Header and scale semantics.** Nothing compares what Pythia read for the
following fields against what we wrote:
- `SCALUP`: the event scale, which bounds the shower's starting scale;
- `AQCDUP`;
- the `<init>` cross section and error.

**Weighting strategies.** Only the `Buffer` strategy (`IDWTUP = -4`) is fed to
Pythia. `StochasticRounding` (`IDWTUP = +3`) writes a different `<init>` header
and is unexercised. A second sample unweighted under it, run through the same
checks, closes that gap.

The momentum and colour blind spots of the same gate are in
[pythia-gate-momenta-unchecked](pythia-gate-momenta-unchecked.md).
