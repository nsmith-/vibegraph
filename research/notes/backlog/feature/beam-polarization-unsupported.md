---
type: Backlog Item
title: Beam polarization is refused
description: Nonzero polbeam1/polbeam2 is a hard error; there are no polarized matrix-element sums or per-event SPINUP for polarized beams.
area: feature
state: open
priority: low
closes_when: A run card with nonzero polbeam1/polbeam2 generates events whose sigma and SPINUP match MadGraph on a polarized lepton-collider row.
blocked_by: []
opened: 2026-08-02
tags: [descoped-v1, beams, polarization, runcard]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L582-L583", title: "TODO.md entry T048"}
---
Descoped from the v1 release goal (user, 2026-08-02). Today a nonzero
`polbeam1` or `polbeam2` is refused as ignored physics
(`vibegraph-lib/src/runcard/classes.rs:237`; pinned by
`vibegraph-cli/tests/cli_hard_errors.rs` and the `polbeam*_nonzero_is_rejected`
tests in `vibegraph-lib/src/runcard.rs`).

The feature needs polarized matrix-element sums (a weighted helicity sum over
the incoming legs in place of the unpolarized average) and the per-event
`SPINUP` consequences in the LHEF record. The helicity-restricted loop that
polarized external legs (`{L}`/`{R}`) already use is the likely attachment
point ([note 38 P1](../../38-process-grammar-sprint-plan.md)), but a beam
polarization is a fractional weighting, not a restriction.
