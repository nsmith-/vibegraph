---
type: Backlog Item
title: "A non-default small_width_treatment is refused"
description: "small_width_treatment is IgnoredPhysics: the clustering floors widths at a hardcoded 1e-6 while MadGraph floors the propagators too; idm's card (1e-12) is refused as shipped."
area: feature
state: open
priority: low
closes_when: "small_width_treatment is honoured on the propagators, the phase-space maps and the clustering together (MadGraph's fk_W), with a test at a non-default value, and the idm card runs as shipped; or the refusal is kept and the reason recorded in field-classification."
blocked_by: []
opened: 2026-11-08
tags: [run-card, refusal, widths, cms, idm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: fields, resource: "../../run-card/field-classification.md", title: "Run-card field classification: small_width_treatment"}
  - {id: swt, resource: "../../process/madevent-small-width-treatment.md", title: "MadEvent's small-width treatment"}
  - {id: hg2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G2-report.md", title: "H-G2 report: idm runs with --set small_width_treatment=0.000001"}
---
`small_width_treatment` is one of the 21 IgnoredPhysics run-card fields,
refused when set off its default (`runcard/classes.rs`, `P_SMALL_WIDTH`). The
recorded reason: the clustering floors widths at a hardcoded 1e-6, while
MadGraph floors the propagators at generation time. So reading the card would
move the clustering without moving the matrix element.

idm's gridpack sets `1e-12`. Attempt 1 is refused in the closing rerun.
gpdb's `runpatch` then resets the field to 1e-6 (inferred from `vgrun.py`'s
fallback, not read from the run's record). H-G2 ran it at 1e-6 by hand.
For a width that is zero or above `M · 1e-6`, the floor never applies, so the
patch is probably harmless on idm (inferred, not checked against its widths).

`onshell.rs`'s `BwWindow` already reads the field, so part of the machinery
exists. Whether gpdb should compare σ after such a patch is
[gpdb-compares-sigma-after-physics-changing-patches](../validation/gpdb-compares-sigma-after-physics-changing-patches.md).
