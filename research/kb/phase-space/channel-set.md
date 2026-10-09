---
type: Design
title: "Integration channels: MadGraph's merged configurations, merged again by map identity"
description: "One channel per config_groups group (coherent |ΣAMP|², channel cuts per configuration), then one hadronic sampling channel per distinct map; MadEvent's config_subproc_map compared."
status: draft
tags: [phase-space, multichannel, channels, madevent, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n36-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L567-L650", title: "Note 36 B3 (MadGraph's channel set)"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2298", title: "Note 41 F-B (channel merging)"}
  - {id: n41-m6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2702-L2915", title: "Note 41 M6 (36 distinct maps among 336)"}
  - {id: mg-tag, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/group_subprocs.py#L56-L103", title: "MadGraph group_subprocs.py IdentifyConfigTag"}
  - {id: mg-confsub, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/group_subprocs.py#L387-L404", title: "MadGraph group_subprocs.py get_subproc_diagrams_for_config"}
---

# Integration channels

Which channels a run integrates over is settled in two layers:

1. **Per subprocess, MadGraph's configurations.** Diagrams are grouped into
   configurations exactly as MadGraph's `IdentifyConfigTag` groups them, and
   each configuration is one channel, built from its lowest diagram.
2. **Per hadronic run, merged by map identity.** Across all flavour groups (and
   multiplicities of a sum), `(group, configuration)` pairs whose phase-space
   maps are the same function share one sampling channel.

The channel's map is the per-diagram decomposition of
[phase-space/diagram-channels](diagram-channels.md); the mixture over channels
is [phase-space/multichannel](multichannel.md).

## Layer 1: configurations

`helas::eval::compile::config_groups` (`helas/eval/compile.rs:919`) ports
MadGraph's `IdentifyConfigTag` (`group_subprocs.py:56-103`):[^mg-tag] two
config-carrying diagrams are one configuration when their topologies agree with
each external leg's number, spin, mass, width and colour, and each propagator's
colour, mass and width. Consequences:

- Colour is part of the tag, so a gluon exchange and a photon exchange are
  **two** configurations even when their maps coincide.
- A t-channel Z or H takes the photon's mass and width in the tag, so it joins
  the photon's configuration.
- Contact diagrams (no propagator to enhance) carry no configuration
  (`config_carrying_diagrams`), though they still enter `|M|²`.

What follows from the grouping:

- `AmplitudeEvaluator`'s configurations are these groups. `eval_amp2`
  (`helas/eval/run.rs:477`) sums the member amplitudes coherently and squares,
  `(Σ AMP)·conj(Σ AMP)`, as MadGraph's `get_amp2_lines` writes it; squaring each
  member separately would be a different number and a different decomposition.
  `config_amp_counts` carries the spans.
- `hadronic::channel_diagrams` (`hadronic.rs:1814`) takes each group's lowest
  diagram as the channel's representative. Both integrands build their channels
  from it.
- `ChannelSet::channel_cuts` (`coupling/cluster/graph.rs:221`) is
  `get_channel_cut` per configuration; the card condition deciding between it
  and `AMP2` is [phase-space/madevent-single-diagram-enhancement](madevent-single-diagram-enhancement.md).
- The `ICOLAMP` mask of a configuration is the union of the flows its members
  reach, pinned against MadGraph's representative-column convention.
- `amplitude_oracle` asserts the partition against `matrix1.f`'s `AMP2`
  grouping, and a gate checks the channel count and `NCOLOR` against
  `coloramps.inc`'s declared dimensions on every single-subprocess run. The
  `cli_ufo_model.rs` capstone pins 36 diagrams over **2** configurations.

When configurations merge, a row's channel set changes, so σ moves within its
Monte Carlo error while every row whose configurations hold one diagram each
stays bit-identical.[^n36-b3] Gating the change needed a measured may-move set:
the pre-registered one (nine rows) was wrong twice. Three of its rows were
`2 → 1` with no channels, and it missed six gated σ rows that merge
(`ee_to_wpwm_cw`, `ee_to_zh_smeft`, three `ll_to_qqx_toy`, `pp_to_jj`) and three
that lose a *contact* channel (`gg_to_gg`, `tata_to_ttx_tensor4f`,
`uux_to_ttx_4f`). List merges from `config_groups` before predicting what moves
([validation/pre-registered-verdicts](../validation/pre-registered-verdicts.md)).

## Layer 2: one sampling channel per distinct map

A hadronic run pools the channels of every flavour group into one mixture.
Many `(group, configuration)` pairs build the same map: on
`pp_to_ll_0j2j_mlm` the 364 pairs are 43 maps (`@0` 4 → 1, `@1` 24 → 6,
`@2` 336 → 36); `p p > l+ l- j` is 24 → 6, `p p > j j` 15 → 3, and even
`p p > e+ e-` has four groups on one map.[^n41-fb][^n41-m6]
`ProtonIntegrand::build` (`proton.rs:1509`) keys each pair's built channel by
`DiagramChannel::map_identity` and appends a repeat to the first channel's
`channel_members`:

- the first-built pair names the channel (`channel_ids`);
- the merged channel's initial selection weight is `pairs_j / N`, so the
  starting mixture density is the unmerged uniform one, and a Kleiss–Pittau
  step on it is the step on the pairs summed;
- the channel pays one coverage floor and costs one density term instead of
  one per pair ([phase-space/channel-budget-allocation](channel-budget-allocation.md)
  has what that saved);
- when nothing merges, no line of the path changes.

`new_unmerged_with_maps` builds the unmerged mixture for measurement only.
The fixed-beam integrand does not merge (one subprocess, one channel per
configuration).

### The key is identity of the function, not of the diagram

`map_identity` covers every field the draw and the density read, each float by
its exact bit pattern (`integer_decode`, which tells −0 from +0), and
destructures each struct whole so a field left out does not compile. Keying on
the *diagram* (MadGraph's tag) would be wrong here: it separates gluon and
photon exchange, which have the same map, and joins a t-channel Z with the
photon, which does not have the same map. Pinned by:

- `pairs_with_equal_map_identities_draw_and_weigh_every_point_alike`
  (`proton.rs:4157`): on every pair of channels of `p p > l+ l- j` and the
  matched `p p > e+ e- j j`, equal identities draw bit-identical momenta and
  weights and give bit-identical densities at points from every channel;
  different identities differ in density somewhere; the integrand's channels
  are the identity classes in first-appearance order with `α_j = pairs_j/N`.
- `a_merged_channel_is_the_sum_of_the_unmerged_terms_it_replaces`
  (`proton.rs:4256`): a merged channel's term equals the sum of the unmerged
  terms at the same uniforms, worst relative difference 8.3e-16 over 1214
  terms (the two density sums group terms differently).

### Why a merged channel needs no configuration rule

In MadEvent a channel *is* a configuration and hands each subprocess its
clustering, jet memo and colour. vibegraph's sampling channels do not carry
that role. The estimator is the one-sample mixture `f/g`, and at every point
each flavour group draws its own clustering configuration `∝ AMP2` from its own
matrix element ([scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md)),
independent of the sampling channel. That configuration feeds the jet memo,
the colour flow and the record. A merged channel therefore hands every group
exactly what an unmerged one did.

The sampling channel reaches a term only through the draw's **fallback**, when
a group's `AMP2` carries no probability at the point: the group then keeps its
own diagram of the merged channel, or its first configuration if the channel
has none of its diagrams (`per_group_sum`, `proton.rs:2150`). Without a merge
that is the unmerged rule exactly. `scale_draw_fallbacks()` counts these
points; it is expected to be zero on any run that produces events.

## MadEvent's rule, for comparison

A `P<n>` directory's channels are its `configs.inc` configurations: one per
`IdentifyConfigTag` class across every subprocess of the directory.
`config_subproc_map.inc` holds `CONFSUB(IPROC, iconfig)`, the diagram of
subprocess `IPROC` on that configuration or 0 (`group_subprocs.py:387-404`,
written by `export_v4.py`).[^mg-confsub] At a point of channel `iconfig`:

| What | MadEvent |
|---|---|
| Which subprocesses contribute | only those with `CONFSUB ≠ 0` (`super_auto_dsig_group_v4.inc`) |
| A subprocess's channel weight | `AMP2(CONFSUB(IPROC, channel))` over its own mapped `AMP2` sum |
| Its clustering | restricted to `iconfig` (`cluster.f` `chcluster`) |
| Its jet memo | `njetstore(iconfig)`, shared by the directory's subprocesses (`reweight.f`) |
| Its colour | `select_color(…, iconfig, IPROC, …)` |

MadEvent has a second sharing layer, the symmetric configurations (`SYMCONF`,
`PERMS`), which integrates permutation-related configurations once and
permutes the point. vibegraph has no counterpart
([backlog](../backlog/performance/no-symmetric-configuration-sharing.md)).

## Artifacts

A channel of several pairs is banked as
`ChannelKey::MergedChannel { final_state, group, channel, pairs }`
(`artifact.rs:172`); every other channel keeps its key. The writer records the
oldest format version that holds its keys, so an artifact with no merged
channel is byte-for-byte what the previous version wrote.
`IntegrateArtifact::refuse_unmerged_grids` (`artifact.rs:709`) refuses to replay
an artifact from before merging on a process whose channels merge, naming both
versions and the pair count; `generate` calls it before its key check. The
version history is in `artifact.rs`'s `FORMAT_VERSION` doc comment and
[pipeline/artifact-format-versioning](../pipeline/artifact-format-versioning.md);
the artifact itself is [pipeline/integrate-artifact](../pipeline/integrate-artifact.md).

[^mg-tag]: MadGraph's `IdentifyConfigTag` at `b7687064`.
[^n36-b3]: Note 36 B3: coherent configurations, the integrator on `config_groups`, the measured may-move set.
[^n41-fb]: Note 41 F-B: merging by map identity, MadEvent's rule, the tests.
[^n41-m6]: Note 41 M6: 36 distinct maps among the two-jet part's 336 pairs.
[^mg-confsub]: MadGraph's `CONFSUB` map at `b7687064`.
