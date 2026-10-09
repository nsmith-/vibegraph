---
type: Backlog Item
title: ee_to_mumua sits about 1% high in the radiative-return pt(γ) windows
description: ee_to_mumua σ carries a fixed +1.04% offset; a ~0.9–1.2% excess at pt(γ) in [39.4, 144) survives chain D's D1 verdict and is unattributed.
area: validation
state: open
priority: high
closes_when: The radiative-return excess is attributed to one side with a measurement; on attribution the row's σ pull and samples cell re-arm as enforced gates.
blocked_by: []
opened: 2026-08-03
tags: [ee-to-mumua, sigma, madgraph-coverage, chain-d, info-cell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L376-L384", title: "TODO.md entry T025"}
---
`ee_to_mumua`'s σ is a fixed **+1.04%** above the bank: five seeds give χ²/dof
0.50 and a pull of +4.11. The pull is reported, not asserted
(`PULL_REPORTED_NOT_ASSERTED`, `vibegraph-lib/tests/validate_sigma.rs:199`);
`rel_tol 0.03` stays enforced and the `samples` cell is `info`. No threshold moved.

Chain D's verdict D1 attributes the low-`pt(γ)` part to the reference: MadGraph's
banked sample under-covers `pt(γ) ∈ [10, 20)` by ≥15σ against its own windowed
cross sections. It does not explain a localised excess, this side high, in
`[39.4, 77)` (+1.22% ± 0.20%) and `[77, 144)` (+0.87% ± 0.16%).

- **Not window-blind sampling.** In the W3 refocus, the re-surveyed
  `MG-cut(W3)` lands with `MG-part(W3)` and closes only 22% of the gap (4.7σ on
  seed-spread errors).
- **Points at MadGraph's `pt(γ)`/`η(γ)` coverage.** Re-integrating its own phase
  space in `m(μμ)` slices, MadGraph recovers this side's total to +0.016%
  (0.16σ), and its two complete `dummy_cuts` partitions of one run disagree at
  16.7σ. Neither matrix element (gated to 1e-11) nor the Z propagator is
  implicated.

**Next probe:** the 2D `MG-part` cell `[39.4, 77) × [86, 96)` in
`(pt(γ), m(μμ))`, against the same cell here. It separates "our `η(γ)` at fixed
`m(μμ)` differs" from "MadGraph's `pt(γ)`-restricted runs under-recover".

Blind spot: `C_VG` on the `pt(γ)` axis is this side's only coverage audit.
`validation/madgraph/pta_window_reference.json` is the durable record of
MadGraph's windowed runs.

Detail: [note 29, chain D measurement, W3 refocus and `m(mumu)` axis](../../history/notes/29-v01-validation-sprint-plan.md).
