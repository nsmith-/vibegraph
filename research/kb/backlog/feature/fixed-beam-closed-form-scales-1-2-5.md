---
type: Backlog Item
title: "Fixed beams refuse closed-form scale choices 1, 2 and 5"
description: "dynamical_scale_choice 1, 2 and 5 are transcribed and value-tested but refused at fixed beams, because no banked run selects one; supporting them needs a reference row each."
area: feature
state: open
priority: low
closes_when: "Each of dynamical_scale_choice 1, 2 and 5 is honoured at fixed beams, each with a banked fixed-beam row that selects it and a gated per-event scale replay in validate_scales."
blocked_by: []
opened: 2026-10-14
tags: [scales, refusal, reference-runs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-14}
sources:
  - {id: scale-choice, resource: "../../scales-pdf/madgraph-scale-choice.md", title: "MadGraph's scale choices, and which vibegraph honours"}
  - {id: r-e-report, resource: "../../sprints/hygiene/sessions/R-E-report.md", title: "R-E report (hygiene sprint)"}
---
`ScaleChoice::from_run_card_for` returns `ScaleError::UnhonouredScaleChoice` for
choices 1 (`Σ E_T`), 2 (`Σ m_T`) and 5 (the first incoming leg's mass) at fixed
beams as well as hadron beams. Their transcriptions are pinned by value tests in
`coupling/scales.rs` (`choice_1_is_the_summed_transverse_energy` and its
neighbours), but no MadGraph run checks a cross section or an event's scale
under any of them. Choice 3 is honoured on `gg_to_gg_cg`'s banked run, and
choice 4 because it is the collider energy at fixed beams.

The work per choice: a banked fixed-beam reference whose card selects it, a
`Coverage::ClosedForm` replay row in `validate_scales`, and lifting the choice
in `DynamicalChoice::has_fixed_beam_oracle`. Choice 5 is MadGraph's choice for
decay processes, whose scales `RunCard::for_decay` fixes before a prescription
is compiled, and on massless incoming beams it is degenerate; finding a row that
reaches it, or deciding it stays refused, is part of the item.
