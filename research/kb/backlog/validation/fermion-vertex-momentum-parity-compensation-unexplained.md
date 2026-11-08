---
type: Backlog Item
title: "Why one-momentum fermion vertices need no momentum-parity sign is unexplained"
description: "The all-outgoing vs all-incoming argument behind boson_momentum_sign predicts −1 on odd-P fermion dipoles, yet the dipole rows match MadGraph without it; the compensating sign is not identified."
area: validation
state: open
priority: low
closes_when: "ε_slot of the bilinear slot reversal (vertex_root_signs' CΓᵀC⁻¹) is derived for the γ^μ and σ^{μν} structures, and (−1)^{n_P} · ε_slot = +1 is shown, by a committed test over every momentum-bearing fermion vertex of SMEFTsim and the toy dipole, or a counterexample is found and fixed."
blocked_by: []
opened: 2026-11-08
tags: [amplitudes, sign-convention, fermion, dipole, smeftsim]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: ha2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-A2-report.md", title: "H-A2 report, §6 and Found: the fermion-dipole open question"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log, 2026-11-07: A, the rule"}
---
#108 derived the all-boson rule: MadGraph's all-outgoing momenta against
vibegraph's all-incoming ones give `(−1)^{n_P} × ε_slot` per vertex. The same
argument, applied naively, puts −1 on a one-momentum fermion vertex: the
`σ^{μν}q_ν` dipole, or SMEFTsim's `P(-1,3)·γγ` FFV structures. Yet these rows
agree with MadGraph with no such factor:
- `ee_to_ttx_dipole`;
- `ll_to_qqx_toy_dipole`;
- the `ctZ`/`ctA` chains of `ee_to_ttx_smeft`.

The candidate reading (H-A2) is that the bilinear reversal in
`vertex_root_signs` supplies a compensating `ε_slot`, as the W-pair slot
exchange does for `γW⁺W⁻`. This is not established. Under the repository's
rule that convention claims are hypotheses, the agreement today rests on rows,
not on a reason. A fermion structure where the two do not cancel would be
silently wrong.

Recorded as an open question in
[convention-sign-inventory](../../amplitudes/convention-sign-inventory.md).
The W-pair analogue is
[w-pair-vvv-slot-exchange-oddness-unpinned](w-pair-vvv-slot-exchange-oddness-unpinned.md).
