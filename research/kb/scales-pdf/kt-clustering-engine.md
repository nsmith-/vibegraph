---
type: Design
title: "vibegraph's kT clustering engine and how the scale is wired"
description: "coupling/cluster/{graph,kt,setclscales,configs}.rs: channel forests derived from our diagrams, the per-event inputs the engine consumes, ScaleChoice::cluster_scales and ClusterInput, one channel set per group."
status: draft
tags: [kt-clustering, scales, coupling, hadronic, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n22-collapse, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/22-dynamical-scales-plan.md#L90-L126", title: "Note 22 §1.3 (what -1 collapses to on 2 → 2)"}
  - {id: n28-k3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2013-L2020", title: "Note 28 §K3 (the engine's modules)"}
  - {id: n28-k32, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2065-L2095", title: "Note 28 §K3.2 (declared consumed state)"}
  - {id: n28-k38, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2245-L2270", title: "Note 28 §K3.8 (what production wiring inherits)"}
  - {id: n28-k4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2404-L2478", title: "Note 28 §K4, §K4.1–K4.2 (ConfigForest from our diagrams; closed forms deleted)"}
  - {id: n28-k6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3283-L3343", title: "Note 28 §K6.1–K6.3 (SampledChannel, the diagram→config map, per-group sets)"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L2193-L2197", title: "MadGraph 3.7.1 export_v4.py (minimal-arity filter)"}
---
# vibegraph's kT clustering engine and how the scale is wired

The engine is a transcription of MadGraph's `cluster.f` and `setclscales`
([kt-clustering-algorithm](kt-clustering-algorithm.md),
[setclscales](setclscales.md)) into `vibegraph-lib/src/coupling/cluster/`. It is
the only implementation of `dynamical_scale_choice = -1`: the closed forms that
once stood in for degenerate 2 → 2 cases, together with `coupling::topology`,
`ClusterTopology` and `BeamConnections`, are deleted, not kept as a second
path.[^n28-k4]

## Modules

| file | role | sees |
|---|---|---|
| `configs.rs` | `derive_channels`: channel forests (`ConfigForest`) from our enumerated diagrams | diagrams, model |
| `graph.rs` | the merge table: which leg sets may combine, their PDGs, resonance flags, the coupling-order filter | forests, no momenta |
| `kt.rs` | `cluster.f`: measures, tie-break, merge order, boosts | momenta, no colour |
| `setclscales.rs` | `reweight.f`'s walk and the two scale formulas, the `μF` floor | cluster sequence, colour table |
| `rewgt.rs` | MLM's α_s and PDF reweighting along the clustering ([mlm-rewgt](mlm-rewgt.md)) | cluster sequence |

`ScaleChoice::cluster_scales(&ScaleEvent, &ClusterInput)`
(`coupling/scales.rs`) is the run card's side of the call. `ClusterInput`
carries the process's `ChannelSet`, a `ColorTable`, `this_config` (the channel,
from 1), `iproc` (the subprocess, from 1) and optionally prebuilt merge tables
per coupling order. `cluster_history` makes both `setclscales` calls under
matching ([mlm-scales](mlm-scales.md)). `hadronic::compile_scale_source` builds
one `Channels` (derived forests, colour table, merge tables) per subprocess a
sampling channel can come from: one for a fixed-beam run, one per flavour group
on a hadronic run. No integrand carries a table keyed by process name.

## Channel forests from our own diagrams

`configs::derive_channels` re-roots each diagram toward beam 2, writes
s-channel lines (subtree carrying neither beam) ahead of the spacelike chain
(subtree carrying beam 1) ordered from beam 1 inward, writes the closing vertex
only for a channel that reaches the beams through a spacelike line
(`export_v4.py:2229`, `if len(tchannels) > 1`), and drops every diagram whose
largest vertex exceeds the set's minimum arity
([`export_v4.py:2193-2197`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L2193-L2197)).
On a timelike line `configs.inc` writes the particle that *decays into* the
subtree, not the one leaving it.

The derivation is gated as a bijection over channels against MadGraph's own
`IFOR` dump records by `derived_channel_forests_match_the_generated_ones`
(`tests/validate_kt_cluster.rs`): every line's leg set, both daughters,
`tprid`, `sprop`, mass and width, 6570 lines over six single-subprocess runs
(615, 579, 25, 8, 2 and 2 channels), all equal. Channel *numbering* is not
compared and cannot be: MadGraph numbers configurations by its own diagram
order, and the merge table reads a channel's identity only through its QCD
order. The four-point filter drops no diagram in any of those six, so it is
pinned hermetically: `g g → g g` has 4 diagrams and 3 channels.

## Diagram index versus configuration index

The sampling channels and the channel forests are both one per configuration of
MadGraph's channel mapping, built from the same diagram slice, so
`Channels::config_of_channel` is the channel index plus the Fortran origin.
Diagrams and configurations are the two numberings that differ: the filter
drops contact diagrams and the mapping can merge several diagrams into one
configuration. `DerivedChannels::config_of_diagram` is `None` for a dropped
diagram. `the_channel_to_config_map_is_not_the_identity` (`configs.rs`)
identifies the dropped `g g → g g` diagram by its four-gluon vertex and asserts
`[None, Some(1), Some(2), Some(3)]`;
`a_sampled_channel_names_the_integration_channel_of_its_own_diagram`
(`hadronic.rs`) pins the wiring (3 channels, `config_of_channel = [1, 2, 3]`,
the integrand's channel count equal to the set's). The configuration draw
composes the evaluator's `AMP2` order with the forest order through the diagram
index, never by assuming the two agree;
`the_amp2_configuration_order_matches_the_forest_order` asserts they do, so a
reorder is a named failure.[^n28-k6]

## What a scale evaluation consumes

The cluster scale is not a function of momenta and process alone
([cluster-scale-channel-dependence](cluster-scale-channel-dependence.md)). Per
event the engine takes:

1. **the integration configuration** (`this_config`) and **the subprocess**
   (`iproc`);
2. **the momenta** as the clustering receives them;
3. **the channel forests** and **the run-card constants**;
4. for a replay only, **carried-over on-shell flags**
   (`cluster(…, carried_on_shell)` in `kt.rs`): MadGraph's `isbw` is a common
   block that keeps flags from earlier events. Production passes none, because
   the generator owns its own state and the pure-function reading is the right
   one.

Everything downstream is derived: leg sets and complements, line PDGs, the
resonance map, the coupling-order filter, BW tagging, every measure, the
tie-break, the merge order, frame changes, the beam walk and both formulas. The
jet memo starts empty on every event and stores the event's own
channel-restricted jet count; that is MadEvent's value on every event, not only a
channel's first, as measured in
[cluster-scale-channel-dependence](cluster-scale-channel-dependence.md).

## Where `this_config` comes from

In production each point's configuration is **drawn** per flavour group and per
beam ordering, `∝ AMP2_c` of that group's own matrix element (or MadEvent's
channel-cut weight where the card asks for it), from a dedicated trailing
uniform; see [clustering-configuration-draw](clustering-configuration-draw.md).
The sampled phase-space channel (`SampledChannel { group, channel }`, threaded
through `FixedBeamIntegrand` and `ProtonIntegrand` and through
`MultiChannel::adapt_alphas`'s survey integrand) reaches the scale only as the
draw's fallback when a group's weights carry no probability.

Every pooled sampling channel must name a forest in its own group's set, and the
proton integrand asserts it at setup. A hadronic run needs one set per
group because `g u → ℓ⁺ℓ⁻ u` and `u ū → ℓ⁺ℓ⁻ g` do not share a merge graph; the
banked replay has always keyed forests on the event's own external flavours.

## How the general path relates to the 2 → 2 closed forms

The general path, run once against the deleted closed forms on every event of
all 14 closed-form runs under every channel, agreed to `0.0` or `1.1e-16` on 8
runs. On `pp_to_bb*` it differs by `1e-9` relative, up to `1.5e-6` on some
events: there the first merge is initial-state and the leftover leg's measure is
taken in the boosted frame, which the closed form ignored. The general path is
the one pinned bit-for-bit against MadGraph's intermediates on
`pp_to_bb_qcd2`, so the closed form was the approximation. The comparison was a
scaffold and is not kept. The standing nets are the enforced replay
([scale-replay-gate](../validation/scale-replay-gate.md)) and
`the_general_path_keeps_the_beam_crossing_population`
(`tests/validate_scales.rs`), which requires `u ū → u ū`'s tie-break population
to be exactly 16 events at `250.000125`. The collapse table itself is MadGraph
behaviour, kept as a consistency check in
[madgraph-scale-choice](madgraph-scale-choice.md).[^n22-collapse]

## Oracles and their blind spots

- [kt-cluster-dump-oracle](../validation/kt-cluster-dump-oracle.md): MadGraph
  instrumented per event (merge tables, every candidate, merges, frames, both
  scale formulas). The derived tables were compared whole on the seven
  single-directory dumped runs (120 tables, all equal).
- [scale-replay-gate](../validation/scale-replay-gate.md): every banked run's
  printed `SCALUP`, `<rscale>`, `<pdfrwt>` and `AQCDUP`.
- The four `2 → 3` partonic `llj` rows (`uux_to_epemg`, `ddx_to_epemg`,
  `gu_to_epemu`, `gux_to_epemux`) have **no clustering dump**. Their only
  reference is the printed scale, which is blind to a wrong tie-break or line
  PDG that does not move the number. They are `lpp = 0` 2 → 3, so no boost
  fires and the merge graph is small.[^n28-k38]

The per-point cost of re-clustering every member under matching is tracked in
[matched-mixture-reclustering-per-point-cost](../backlog/performance/matched-mixture-reclustering-per-point-cost.md).

[^n28-k4]: Note 28 §K4–K4.2.
[^n28-k6]: Note 28 §K6.1–K6.3.
[^n22-collapse]: Note 22 §1.3, reconciled in note 28 §K1.8.
[^n28-k38]: Note 28 §K3.8.
