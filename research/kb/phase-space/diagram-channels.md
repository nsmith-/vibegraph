---
type: Design
title: Per-diagram channels from the propagator chain
description: "How a diagram's Prop chain becomes a 2-body decomposition tree, with non-prefix s-channel recovery and the beam-content rule that survives feyngraph's momentum routing."
status: draft
tags: [phase-space, multichannel, diagrams, feyngraph, decomposition]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n21-substrate, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L52-L81", title: "Note 21, the diagram substrate (Prop topology)"}
  - {id: n21-nonprefix, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L119-L167", title: "Note 21 addendum: non-prefix s-channel recovery"}
  - {id: n21-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L231-L299", title: "Note 21, resonance-sampling close-out"}
---

# Per-diagram channels from the propagator chain

`DiagramChannel` (`vibegraph-lib/src/phasespace/diagram_channel.rs`) is the
building block of multichannel phase space: one importance-sampling map read
off one tree diagram. Which diagrams actually become integration channels, and
how channels with identical maps are merged, is
[phase-space/channel-set](channel-set.md); how the maps are mixed and their
weights adapted is [phase-space/multichannel](multichannel.md).

## Raw material: the diagram's `Prop` chain

Each internal line of an enumerated `Diagram` (`diagrams/diagram.rs`;
enumeration is [process/diagram-enumeration](../process/diagram-enumeration.md))
is a `Prop { particle, endpoints, momentum: Vec<i8> }`: the particle gives the
mass and width through the model, and `momentum` is the signed combination of
external momenta the line carries, which says which invariant the pole sits
on. `Prop::is_spacelike(n_in)` separates t-channel (spacelike) from s-channel
(timelike) lines.[^n21-substrate] No other diagram plumbing is needed.

## The decomposition tree

A tree diagram organises its final state into nested subsystems. Each timelike
line bounds one subsystem, the outgoing legs on the beam-free side of the cut
it makes, and those subsystems form a laminar family, i.e. a tree.
`DiagramChannel` turns the tree into a chain of 2-body decays:

1. The total system `(√ŝ, 0, 0, 0)` splits into two daughters; each daughter is
   either a single outgoing particle (fixed mass) or a composite subsystem
   (sampled invariant mass), and each composite recurses (`build_root`,
   `build_node`).
2. A node's children are the maximal candidate sets strictly inside it, where
   candidates are the diagram's subsystems plus every singleton. For a tree
   diagram these partition the node.
3. A vertex with more than two children (a contact vertex) is folded by
   `binarize` into a right-leaning caterpillar. The interior branches carry an
   auxiliary invariant with no pole; the outermost carries the subsystem's own
   resonance.
4. Each composite node records its propagator's mass and width, so the
   invariant's draw can be resonance-aware without changing the tree
   ([phase-space/resonance-and-pole-maps](resonance-and-pole-maps.md)).

The weight is the exact product of the 2-body LIPS factors
`R₂ = π|p*|/√s` and each invariant's draw measure `ds/dx`. A flat average of
`weight · f` therefore estimates `∫ dR_n f` over the same invariant volume that
flat RAMBO integrates: a channel is a reparametrisation of the same phase space,
not a different one. Unit tests pin the volume (`flat_volume_matches_massless_v_n`,
`flat_volume_matches_flat_rambo`, `resonant_channel_volume_still_v_n`,
`t_channel_volume_still_v2`) for `2 → 2` up to `2 → 6`.

Spacelike lines are not subsystems: they are momentum transfers. A diagram
with spacelike lines is decomposed as a peripheral chain of rungs instead of an
all-timelike tree ([phase-space/t-channel-spine](t-channel-spine.md)), but only
when the final state has two legs or a positive fiducial scale is supplied
(`from_diagram_regulated`); otherwise it falls back to the all-timelike tree.
A `1 → n` decay has no spacelike line, so its channel is always the
all-timelike tree at `√ŝ = M`.

## Reading the subsystem: beam content, not the stored side

feyngraph routes momenta by giving each external a unit indicator and then
**eliminating the highest-indexed external** through global conservation. The
stored vector of an internal line is the signed combination for the cut side
*away from* that external, so its raw beam coefficients depend on that
convention and cannot be read as "this is the beam side".[^n21-nonprefix]

The convention-robust rule (`subsystem_mask`, `diagram_channel.rs:1298`)
classifies a line by the **beam content of its cut**. Only the nonzero pattern
matters; the coefficients carry flow signs.

| Beams with nonzero stored coefficient | Meaning | Subsystem |
|---|---|---|
| 0 | timelike, stored side is beam-free | the stored outgoing set |
| all `n_in` | timelike, stored side is the complement | the complementary outgoing set |
| some but not all | spacelike transfer | none (handled by the spine) |

A result is kept only when it spans between 2 and `n_out − 1` legs; the
s-channel core, whose beam-free side is the whole final state, is excluded.

The case that forced the rule: in `e+ e- > mu+ mu- ta+ ta-` (externals
`0=e+, 1=e-, 2=mu-, 3=mu+, 4=ta-, 5=ta+`; feyngraph eliminates `5`), the μμ
line is stored as `[0,0,1,1,0,0]` (no beams) but the τ⁺τ⁻ Z line as
`[1,1,−1,−1,0,0]` (both beams, τ slots zero). A rule that dropped any line with
a beam coefficient resonated on the μ pair only and never on the τ pair. The
difference between the two pairs is an artefact of the elimination, not of the
physics.

**Pinned from two sides.** `subsystem_classification_matches_graph_cut`
(`diagram_channel.rs:4448`) checks the stored-momentum classifier against an
independent graph-cut derivation (connected components after removing the
line) on a spread of processes, including genuine t-channels (Bhabha,
`u u~ > u u~`) and the non-prefix τ pair. The peripheral side has the same
cross-check (`the_rung_chain_agrees_with_an_independent_graph_cut`,
`tests/diagram_channel.rs:1713`). A future change to feyngraph's routing
convention trips one of these. Which FeynGraph commit the routing was read at
is recorded in [references/codebases/feyngraph](../references/codebases/feyngraph.md).

## Energy independence

The tree (masks, masses, resonances, spacelike poles) does not depend on the
collision energy. The fixed-energy `PhaseSpaceMap`/`Channel` impls draw at the
`sqrt_s` the channel was built with; `ScaledChannel` takes `√ŝ` per draw, and
the beams are rebuilt from the stored `beam_masses` at that energy. One channel
set therefore serves a hadronic run whose `ŝ` changes every event
([phase-space/hadronic-tau-y-sampling](hadronic-tau-y-sampling.md)).

## Map identity

`map_key()` and `map_identity()` encode everything a channel's draw and density
read (`n_out`, beam masses, `sqrt_s`, the whole topology with each node's floor,
resonance, window and angle map, and each rung's pole, bound and remainder
floor), every float by its exact bit pattern. Two channels with equal
identities are the same function of `(√ŝ, u)` and `(√ŝ, momenta)`, whatever
diagrams they came from. `map_identity` destructures each struct whole, so a
field added to the map without being added to the identity does not compile.
The recorded `t_channels` list is deliberately absent: nothing that samples or
prices reads it. The identity is finer than pointwise density equality, so it
can miss a coincidence but never invent one. This is the key the channel set
merges on ([phase-space/channel-set](channel-set.md)).

## What is validated, and how

Each map hazard has a test that fires if the map is wrong, and a passing σ is
never taken as confirmation:[^n21-closeout]

| Hazard | Firing test |
|---|---|
| Breit–Wigner denominator, `ds/dθ` | `bw_map_is_measure_preserving`, `bw_map_zero_variance_on_bw_integrand` |
| t-channel invariant ordering | `spine_transfer_pairs_emitted_with_beam0`, `spine_emitted_is_forward_biased` (a silent swap flips the bias) |
| Threshold `s → (m₁ + m₂)²` | `t_channel_threshold_window_collapses`, `t_bounds_include_initial_state_mass` |
| Overlapping resonances on one invariant | `overlapping_resonances_double_peak_resolved` |

The variance wins (resonance map, combiner, spine, α-adaptation) are each
pinned by a test against flat RAMBO at fixed N that asserts direction and
strict inequality, not a number: the synthetic integrands are close to the
zero-variance optimum, so their factor (about 1900× per-point variance for
α-adaptation on a two-peak toy) is a best case, not what a real `|M|²` with
continuum and interference gets.

[^n21-substrate]: Note 21: the `Prop` topology as channel raw material.
[^n21-nonprefix]: Note 21 addendum: the feyngraph routing convention and the τ-pair instance.
[^n21-closeout]: Note 21 close-out: the decomposition, resonance maps and hazard inventory.
