---
type: Design
title: "The clustering configuration is drawn ∝ AMP2 per flavour group and beam ordering"
description: "Each flavour group and beam ordering draws its kT-clustering configuration ∝ its own AMP2_c (or MadEvent's channel-cut weight) from a substream uniform, so σ is partition-free."
status: draft
tags: [scales, kt-clustering, hadronic, amp2, madevent]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n29-b0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4708-L4757", title: "Note 29 Chain B §B.0 (the scale read the sampler's channel; Fact 3)"}
  - {id: n29-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4758-L4825", title: "Note 29 Chain B §B.1 (MadEvent's rule is conditional on sde_strategy and tmin_for_channel)"}
  - {id: n29-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4826-L5167", title: "Note 29 Chain B §B.2–B.7 (movement census; where the draw lives, its randomness, pinned coupling, index composition, fallback; stages)"}
  - {id: n29-b11, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5262-L5352", title: "Note 29 Chain B §B.11–B.12 (risks; errors in the brief)"}
  - {id: n29-bres, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5523-L5756", title: "Note 29 Chain B results (why the draw reproduces MadEvent; acceptance tests)"}
  - {id: n40, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L16-L116", title: "Note 40 §1–4 (per group and per beam ordering; per-event oracles; byte identity)"}
  - {id: n41-11, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L49-L68", title: "Note 41 §1.1 (call flow; colour from igraphs(1) under matching)"}
  - {id: n41-34, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L276-L283", title: "Note 41 §3.4 (colour from the clustered graph)"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 §4 M1 (clustered_config; colour under ickkw = 1)"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 §4 M3 (the draw under matching, E_vg/W_MG)"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2701", title: "Note 41 §4 F-B (merged channels need no configuration rule)"}
  - {id: mg-sde, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/matrix_madevent_group_v4.inc#L213-L238", title: "MadGraph 3.7.1 matrix_madevent_group_v4.inc (single-diagram enhancement block)"}
  - {id: mg-chcut, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L1817-L1881", title: "MadGraph 3.7.1 genps.f get_channel_cut"}
---
# The clustering configuration is drawn ∝ AMP2 per flavour group and beam ordering

MadGraph's default scale is clustered inside an integration configuration
([cluster-scale-channel-dependence](cluster-scale-channel-dependence.md)).
vibegraph does not cluster in its sampler's channel: at every point, each
flavour group and each beam ordering draws its own configuration `c` with
probability `w_c / Σ w` from its own matrix element and clusters in it. That
reproduces MadEvent's per-event scale distribution while σ stays independent of
either side's channel partition.

## MadEvent's rule, and the condition it holds under

MadEvent does not draw anything: it clusters in the channel that sampled the
point (`genps.f:221,245` set `this_config = iconfig`). But under single-diagram
enhancement the integrand of channel `c` carries `AMP2_c / XTOT`
([`matrix_madevent_group_v4.inc:213-238`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/matrix_madevent_group_v4.inc#L213-L238)):

```fortran
	    if(sde_strat.eq.1) then
	         AMP2(J) = AMP2(J) * GET_CHANNEL_CUT(P, I)
	    else
	    	 AMP2(J) = GET_CHANNEL_CUT(P, I)
	    endif
        ...
		ANS=ANS*AMP2(channel)/XTOT
```

and a point sampled in `c` carries the map density `g_c`, which cancels against
the multichannel weight. The configuration of an event at momentum `p` is
therefore distributed as

```text
P(c | p) = w_c(p) / Σ_i w_i(p)
```

independent of `g_c` and of the partition. Drawing that conditional directly is
the same distribution. The weight `w_c` is `AMP2_c · CC_c`, where `CC_c` is
`get_channel_cut`, a product of inverse propagator denominators that
short-circuits to 1 only when
([`genps.f:1878-1881`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L1878-L1881))

```fortran
      if(sde_strat.eq.1.and.tmin_for_channel.eq.-1)then
         get_channel_cut = 1d0
```

So `w_c = AMP2_c` only under `sde_strategy = 1` **and** `tmin_for_channel = -1`
(both defaults); at `sde_strategy = 2`, `w_c = CC_c`.[^n29-b1][^n29-bres]

vibegraph follows both branches. `EventScaleSource::weights_configurations_by_amp2`
(`hadronic.rs`) reads both fields; where it is false,
`compile_configuration_weights` builds channel forests and the draw weights come
from `ChannelSet::channel_cuts` (`coupling/cluster/graph.rs`, a transcription of
`get_channel_cut` without the `tmin` factor). `draws_configuration()` is true for
every per-event clustered prescription whatever the card; only the weight
changes. A card that sets `tmin_for_channel` off its default is refused at parse
(`IgnoredPhysics`), so the `tmin` factor is never needed; `SDE_strategy` is a
`Consumed` field ([field-classification](../run-card/field-classification.md)).
`the_configuration_draw_needs_both_run_card_fields` (`hadronic.rs`) pins both
guards. A `1 → n` decay with `sde_strategy = 2` is refused
([decay-card-sde-strategy-2-refused](../backlog/feature/decay-card-sde-strategy-2-refused.md)).

## Per group and per beam ordering

MadEvent integrates each subprocess group separately, clustering with
`ipdgcl(·, igraphs(1), iproc)` of the subprocess whose matrix element the point
evaluates: a point's scale is never another group's. At `IMIRROR = 2` it flips
the momenta the matrix element reads and clusters the unflipped event in the
subprocess's own flavour order, i.e. the mirrored physical event as its matrix
element reads it, weighted by that matrix element's `AMP2`.

`ProtonIntegrand::per_group_sum` (`proton.rs`) does the same. For each group:

- **direct term:** draw `c ∝ w_c(q)` from the group's own matrix element at the
  partonic point `q`, cluster at the lab momenta, read the densities at its own
  `μF` and bind `αs(μR)`;
- **mirrored term** (groups with distinct beam partons): draw from `w_c(Rq)`,
  `R` the rotation by π about x, cluster at the lab momenta rotated the same way
  with the beams exchanged (`mirror_lab_into`), and swap the per-beam `μF` back
  to physical beam order.

All draws share the trailing uniform, which correlates them without changing
any term's distribution; σ is linear in each term. `ProtonEvent::group_scales`
holds `[direct, mirrored]` per group; the record's `SCALUP`/`AQCDUP` are the
selected term's ([record-scales](record-scales.md)). Constant scales and closed
forms keep one shared scale. See
[beam-mirror-identity](../hadronic/beam-mirror-identity.md) and
[proton-integrand](../hadronic/proton-integrand.md).[^n40]

`FixedBeamIntegrand` (one group, no mirror) draws the same way; every
fixed-beam 2 → n card with a dynamical scale compiles the prescription, even
when the matrix element carries no `αs`, so the record reports the clustered scale.

## Where the randomness comes from

An accepted event is rebuilt from `(channel, u)` alone (`AcceptedPoint` →
`event_in_channel`) after rejected trials ran in between, so the draw's
randomness must be a function of those arguments; a per-call counter would
record a scale other than the one the weight used.[^n29-b0] So:

- `scale_draw_ndim()` is 1 when the prescription draws; `u` is
  `channel_grid_ndim() + scale_draw_ndim()` long (asserted) and its trailing
  coordinate is the draw's uniform `v`.
- `v` comes from `SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE + j)`
  (`phasespace/rng.rs`) in the unweighter, the integration and the α survey,
  consuming **zero bits** of any pre-existing stream: point sequences, channel
  selections, acceptance draws and VEGAS grids are unchanged.
- Rejected: an extra VEGAS dimension (resamples every clustered row; a step
  function under grid refinement) and summing `Σ_c w_c f(p, μ_c)` (an event
  carries one `SCALUP`, so events and σ would disagree).[^n29-b3]

**`AMP2` at a pinned coupling.** The scale is unknown until after the draw, so
the weights are formed at the coupling the amplitudes were bound at
(`RunningCouplingReport::alpha_s_ref`), then `αs(μR)` is set: the drawn
configuration is a function of the momenta, not of evaluation history, even where
configurations carry different `NQCD` (`pp_to_bb*`'s `P1_qq_bbx`).

**Index composition.** `AMP2`'s evaluator order and the forests' order are
composed through the diagram index, never assumed equal
([kt-clustering-engine](kt-clustering-engine.md)).

**The draw lives in the integrand**, which owns the evaluator; it passes the
configuration down through `SampledChannel`/`ClusterInput`, and nothing in
`coupling/` knows about the draw.

**Fallback.** Where no weight carries probability (`select_index` → `None`) a
group keeps its own member of the sampling channel (or its first configuration),
a mirrored term its first configuration. Such points are counted
(`scale_draw_fallbacks()`, asserted zero on every gated integration; MadEvent's
`NB_FAIL` stops after ten). This is the only place the sampling channel reaches
a scale, so merged sampling channels need no configuration rule.[^n41-fb]

## Under MLM matching

With `ickkw > 0` MadEvent draws the colour flow and writes mothers from the
clustered graph `igraphs(1)`, not the integration channel
(`super_auto_dsig_group_v4.inc:1120-1142`, `addmothers.f:109-116`).
`EventScales::clustered_config` is `igraphs(1) − 1`, and
`select_config_and_flow` takes it as `clustered: Option<usize>`; at `ickkw = 0`
it is `None` and the colour configuration comes from the `AMP2` draw. The same
drawn configuration feeds the jet memo. On `pp_to_ll_0j2j_mlm`'s `@2`, the
draw's expected matched weight over MadEvent's at MadEvent's own channel is
`0.994 ± 0.005` (`0.988 ± 0.006` on `g q`).[^n41-m1][^n41-m3] See
[mlm-scales](mlm-scales.md).

## What pins it, and what each check cannot see

| check | what it shows | blind to |
|---|---|---|
| `madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2` (`tests/validate_hadronic.rs`) | on `pp_to_llj_dyn`'s 7197 `q g → ℓℓq` events with configurations at two scales, 1429 land on the higher against 1488.3 expected (pull −1.83); the unrotated-mirror control expects 1748.8 (pull −9.31) and is asserted rejected | `q q̄` groups (one scale per configuration); the integrand's own use of the rule |
| `our_own_events_replay_in_their_own_flavour_group` (`vibegraph-cli/tests/cli_decay_chain_events.rs`) | 2000/2000 generated events on `p p > t t~` (decayed, dynamical) and `p p > l+ l- j` replay `SCALUP` (to 1e-6) and `AQCDUP` in their own group | which configuration inside the group; mirror orientation on these cards |
| `probe_the_scale_draw_reads_the_point_and_not_the_sampler` (`tests/validate_sigma.rs`, ignored) | scale independent of the sampling channel; unchanged after 256 intervening evaluations | the frequency law |
| `the_amp2_configuration_order_matches_the_forest_order` (`hadronic.rs`) | the two index orders agree on `g g → g g` (vibegraph's 4 diagrams, the contact one carrying all three colour structures, against MadGraph's 6; 3 configurations) | anything about momenta |
| fixed-scale byte identity | constant-scale artifacts and LHE files byte-identical across the per-group change | the clustered path |

The `∝ w_c` frequency law of *this* integrand's draw is asserted only through
those pieces, and the zero-spread census of rows whose scale no configuration
moves was a one-off measurement
([drawn-scale-config-frequency-law-ungated](../backlog/validation/drawn-scale-config-frequency-law-ungated.md)).
`pp_to_jj`'s groups agree on the scale to seven digits, not to the last bit
([pp-jj-across-group-scale-spread](../backlog/validation/pp-jj-across-group-scale-spread.md)).
σ evidence: [configuration-draw-sigma-shifts](configuration-draw-sigma-shifts.md);
replay gate: [scale-replay-gate](../validation/scale-replay-gate.md).

**Cost.** One extra `eval_amp2` per group per point (two for a mirrored group)
plus a `set_alpha_s`; the draw is noisier at low budget (`pp_to_llj_dyn`'s
five-seed χ²/dof 6.38 at `neval = 75 000`, ≤ 0.82 from 150 000 up). Artifacts
from before the draw are refused on a clustering-scale card
([artifact-format-versioning](../pipeline/artifact-format-versioning.md)).

[^n29-b1]: Note 29 Chain B §B.1, re-read at the pinned template.
[^n29-bres]: Note 29 "Chain B results", "Why the draw reproduces MadEvent even though MadEvent does not draw".
[^n40]: Note 40 §1–§3.
[^n29-b0]: Note 29 Chain B §B.0, Fact 3.
[^n29-b3]: Note 29 Chain B §B.3, Decisions 1–7.
[^n41-fb]: Note 41 §4 F-B.
[^n41-m1]: Note 41 §4 M1 and §3.4.
[^n41-m3]: Note 41 §4 M3.
