---
type: Backlog Item
title: Evaluator doc comments contradict the code
description: "Doc comments in helas/eval and helas/repr still describe per-diagram configurations, covariant vector slots, an unpinned ε sign, deleted tests and arena-order emission; the code has moved on."
area: hygiene
state: open
priority: medium
closes_when: "Every site listed in the body describes the current code, and cargo doc reports no broken intra-doc link in helas/eval."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, helas-eval, hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: compile, resource: "../../../../vibegraph-lib/src/helas/eval/compile.rs", title: "helas/eval/compile.rs; AmplitudeEvaluator::compile iterates config_groups at ~:232"}
---
Each site was checked against the code on 2026-10-09 (paths under `vibegraph-lib/`):

- `src/helas/eval/compile.rs:910-918` (`config_groups`): "**This is not the partition `AmplitudeEvaluator` integrates over** … one per entry of `config_carrying_diagrams`". `compile` builds its configurations from `config_groups` (~:232), and `eval_amp2` sums each group coherently.
- `compile.rs:446-458` (`select_color_flow`): "the configurations here are finer than its … these configurations stay one per diagram". They are MadGraph's merged ones. `GET_CHANNEL_CUT` is implemented and used at `sde_strategy = 2`.
- `tests/amplitude_oracle.rs:1089`, an error text: "this crate's configurations are one per diagram".
- `src/helas/color/flow_tags.rs:300-304` (`select_flow_reached_by`): says the configuration is drawn `∝ AMP2`. At `sde_strategy = 2` the weight is `ChannelSet::channel_cuts`.
- `src/helas/eval/root_diagram.rs:1422`: intra-doc link to `spine_sign_from_flow_matches_heuristic`, which no longer exists.
- `root_diagram.rs:1450-1452`: credits `e+ e- > ta+ ta- H`'s build sign to "ProjM/ProjP scalar-sink + the crossed-τ standalone projector". Only the standalone-projector arm fires there (note 29 F.13 item 3).
- `src/helas/eval/op.rs:46-49` (`Op::Propagate`): "dispatches on the input current's variance … a covariant `MetricVout` current … is raised back". Every vector slot is contravariant, and `propagate_core` has one vector arm.
- `src/helas/wavefn.rs:329-332` (`VectorWf`): says index-lowering kernels produce covariant `ε_μ`. None does.
- `wavefn.rs:486` (`ScalarWf.momentum`): "particle → +p, antiparticle → −p". The sign is the flow flag (outgoing +, incoming −), not the charge.
- `src/helas/repr/lorentz.rs:1094` (`epsilon4`): calls `ε^{0123} = −1` "a hypothesis about MadGraph until an MG comparison exercises it". `gg_to_h_cpodd` and `gg_to_gg_cg` gate it.
- `src/helas/eval/egraph.rs:9-12`: "every op has fixed arity except `PMomOut` and `Flows`". `Hels`, `Configs` and `AddScaled` are `(Vec Node)` constructors too (~:87-92).
- `src/helas/eval/kernel.rs:700-705`: "≲1e-15 … certified by the `fused_*` tests". The tests are `ffv_vout_matches_generic_chiral_pair`, `ffv_fermion_out_matches_generic_chiral_pair` and `outer_projector_equals_flipped_inner_bit_exactly`, at `FUSED_TOL = 1e-14`.
- `src/helas/eval/layout.rs:11-13`: arena elements are "the `wavefn.rs` currents (momentum still embedded)". Arenas hold bare `ComplexVector`/`Bispinor`, and momenta live in the per-point pool.
- `vibegraph-lib/Cargo.toml:16-19` (`eval-schedule-study`): "without it … the evaluator emits arena order". Production emits op-blocked order (`layout.rs` ~:1121-1145).

Found by drafters D1, D4, D6, D8, D9 and D12 and verifiers V1, V8, V9 and V12; several reported the same site.
