---
type: Design
title: "Phase-space seam: maps, channels, combiners and the density contract"
description: "PhaseSpaceMap/Channel/Combiner traits; sample()'s walk weight vs the combiner's Σα·g from density(), positive at foreign points; cut-first pricing; the subtree memo."
status: draft
tags: [phase-space, multichannel, density, traits, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n21-substrate, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L52-L81", title: "Note 21, substrate the seam was built on"}
  - {id: n21-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L231-L299", title: "Note 21 close-out (the seam, the combiner)"}
  - {id: n24-p0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L525-L618", title: "Note 24 P0 (three-body spine probe; vacuous reciprocity)"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L879-L937", title: "Note 24 P2 (walk weight; plan correction 3)"}
  - {id: n28-s24, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1623-L1669", title: "Note 28 §S2.4 (foreign-configuration density contract for rung chains)"}
  - {id: n34-draw, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L29-L55", title: "Note 34 §1.1 (cut-first density)"}
  - {id: n34-s4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L251-L325", title: "Note 34 wave 2 (S4 subtree memo close-out)"}
measured:
  - {commit: 470eb8f, landed_in: 443f6bc, command: "probe_2to6_eval_cost"}
  - {commit: f85718d, landed_in: df109b9}
---

# Phase-space seam: maps, channels, combiners and the density contract

## The traits

All in `vibegraph-lib/src/phasespace/channel.rs`:

| Item | Line | Role |
|---|---|---|
| `PhaseSpacePoint` | :36 | `n` on-shell momenta in the CM frame plus a weight `J = 1/g`; the `(2π)` measure factors live in the cross-section prefactor, not here |
| `PhaseSpaceMap` | :46 | `ndim()` and `sample(u) → PhaseSpacePoint`, `u ∈ [0,1]^ndim`. All an integrator sees, so a VEGAS grid composes in front of any map |
| `Channel` | :70 | a map on a fixed `√ŝ` and mass set that also reports `density(momenta)` at **any** on-shell configuration, plus `density_memo`. `Send + Sync`: holds no evaluation state |
| `SubsystemMemo` | :134 | per-point cache of subtree momenta and invariants, shared across channels |
| `ScaledChannel` | :230 | the same with `√ŝ` per draw (`sample_at`, `density_at`), for hadronic runs ([phase-space/hadronic-tau-y-sampling](hadronic-tau-y-sampling.md)) |
| `Combiner` | :308 | a set of channels presented as one `PhaseSpaceMap` |
| `MultiChannel` / `ScaledMultiChannel` | :365 / :749 | the combiners built here |

Concrete channels: `DiagramChannel` ([phase-space/diagram-channels](diagram-channels.md)),
flat `RamboChannel` and the massless 2-body `Lips2Channel`.[^n21-substrate][^n21-closeout] The mixture,
selection weights and α-adaptation are
[phase-space/multichannel](multichannel.md); this concept is the contract a
channel owes the combiner.

## Two weights: the walk and the density

A channel produces a weight two independent ways:

- **The walk weight.** `sample` accumulates the product of 2-body LIPS factors
  and draw measures *as it walks*, from the invariants it drew.
- **The density.** `density` rebuilds every invariant from the realised momenta
  and recomputes the product.

At a point the channel itself generated the two must agree,
`density = 1/weight`, to rounding. They multiply the same factors in different
orders from different inputs, so the agreement is a real check on the map.
The bound is `WALK_DENSITY_TOL = 1e-7` (`vibegraph-lib/tests/diagram_channel.rs:103`):
the worst measured gap is 7.1e-9 over every diagram-derived channel and 1.2e-8
on a floored llj spine (near-threshold points, where Källén functions cancel
hardest), and an unfloored three-body spine reaches 4e4.[^n24-p2]

**Blind spot, and why it matters.** A reciprocity check is only as good as the
independence of its two sides. If `sample` takes its weight as
`1/density(momenta)`, reciprocity holds by construction and the check cannot see
a sampling-versus-weighting mismatch, which is exactly the error class it is
meant to catch. This happened: with the weight defined through `density`, an
unregulated three-body spine overstated `V₃` by 3.09–3.48×, read at the time as
bias in the map. With the walk weight, the same unregulated spine reproduces
flat RAMBO's `V₃` to 1.003; the "bias" was the weight definition.[^n24-p0] Any
new map's reciprocity test must compare two genuinely separate computations; a
test that recomputes the density through `sample`'s own path proves nothing
about this. Integrated checks (`V_n` against flat RAMBO, seed sweeps) are the
fallback, and are what caught it.

## The combiner reads `density`, so the density must be right everywhere

`MultiChannel` discards each channel's own walk weight. A point drawn from
channel `i` gets weight

```
w(p) = 1 / g(p),    g(p) = Σ_j α_j g_j(p)        (mixture)
w_j(p) = α_j / g(p)                              (channel-split term j)
```

with every `g_j` from `Channel::density` at the **same** configuration. So the
combiner is exposed to every flaw in `density`, including ones the walk weight
avoids. The contract, stated for rung chains in note 28 §S2.4 and holding for
every channel:[^n28-s24]

1. **Reciprocity.** `density = 1/walk weight` at the channel's own points, to
   `WALK_DENSITY_TOL`.
2. **Totality at foreign points.** The density is defined, finite and
   **strictly positive** at every on-shell, momentum-conserving configuration
   at the channel's `√ŝ`, including points that look nothing like its own
   topology (another channel drew them). A rung order is a property of the
   map, not a constraint on the configuration; a channel returning zero or
   `NaN` at a foreign point biases every other channel through the shared sum.
3. **Frame independence.** Everything the density reads is an invariant built
   from the configuration and the stored beams (`t_i = (p_a − Σ_{S_i} p)²`, blob
   and remainder invariants).
4. **Window totality.** Per-rung `[t_min, t_max]` and per-invariant `[lo, hi]`
   windows are the decomposition's own kinematic limits, so no physical
   configuration falls outside them unless the map is restricted on purpose.
5. **Support honesty.** A channel that deliberately narrows its support (the
   fiducial transfer bound, cut-implied floors) reports density **exactly zero**
   outside it, and the channel *set* must still cover everywhere the integrand
   is non-zero.
6. **Degenerate configurations return zero, not `NaN`** (lightlike transfer,
   blob at threshold, vanishing `k`).

`positive_density` relies on (2): `g ≥ α_i g_i > 0` at any point channel `i`
generated, and a `debug_assert!(g > 0)` guards the division. Chains are tested
against (2) and (3) directly: `the_chain_density_contract_holds_at_foreign_configurations`
and `the_chain_density_reads_only_invariants` (`tests/diagram_channel.rs`).

### Why the spacelike floor exists

An unfloored massless spacelike pole puts the transfer's upper edge on a
cancelling difference, and `density`'s recomputed `t` then carries rounding the
drawn `t` does not. Over the six llj cuts at `√ŝ = 500`, measured with note
24 P2's 5 GeV pole-mass floor:[^n24-p2]

| Spacelike pole | Worst walk-vs-density gap | Non-positive self-densities |
|---|---|---|
| unregulated (`m = 0`) | 3.99e4 | 145 of 800 000 |
| floored at 5 GeV | 1.2e-8 | 0 |

A non-positive density at a point the channel itself drew is a zero in the
combiner's denominator. So the floor is there for the **combiner's density
positivity**, not for the channel's own unbiasedness: the correct statement is
"the unfloored spine breaks the density contract a combiner rests on", pinned by
`an_unregulated_three_body_spine_breaks_the_density_a_combiner_weights_by` and
`an_unregulated_spine_breaks_the_positive_density_contract`. The regulator
production uses today is not that 5 GeV floor but the process's fiducial
scale: each rung's transfer is bounded at `t ≤ −scale` (support narrowed,
density exactly zero above it) and the pole is floored at
`10⁻³ · scale` (`POLE_FRACTION_OF_FIDUCIAL_SCALE`). A spine is built for more
than two outgoing legs only when that scale is positive
(`DiagramChannel::from_diagram_with`); otherwise the channel is the
all-timelike tree. The floor's construction is [phase-space/spacelike-floor](spacelike-floor.md)
and the spine itself [phase-space/t-channel-spine](t-channel-spine.md). The
timelike counterpart, [phase-space/cut-implied-timelike-floors](cut-implied-timelike-floors.md),
moves where a map puts its density without narrowing its support: a
configuration below a timelike floor keeps a positive density, which is
consistent because the same bound says it fails the cuts and carries integrand
zero.

## Cut-first pricing: draw, then cut, then weight

The density sum costs a `density` call in every channel, so the samplers split a
draw from its weight:[^n34-draw]

- `draw_in_channel` / `draw_from` (`ScaledMultiChannel::draw_in_channel_at`)
  return the point with the drawing channel's own weight only;
- `channel_weight` / `mixture_weight` (`channel_weight_at`) price `α_j/g` or
  `1/g` afterwards.

Every consumer draws, applies the cut, evaluates `|M|²`, and prices the mixture
density only if both are non-zero (`hadronic.rs` `value_in_channel`,
`proton.rs:2555`). A rejected point never evaluates the loop. No consumer reads
a rejected point's weight, including unweighting's trial accounting, and the
change was order-preserving (byte-identical `IntegrateArtifact` SHA-256 at fixed
seed on 4-, 24- and 579-channel rows).

Measured at the introducing commit: `probe_2to6_eval_cost` went from
62.9/68.5 to 4.6/6.8 µs per point (13.7×/10.1×) on the 579/615-channel `2 → 6`
rows. The gain is large because real fresh-grid multichannel acceptance is
about 3%.

## Shared-subtree memo

Across the channels priced at one point, different diagrams route the momenta
through overlapping subsystems; with hundreds of channels on a six-body final
state that is thousands of subtree sums where at most `2⁶` leg sets exist.
`SubsystemMemo` caches each subtree's four-momentum and invariant for the
current point; `density_memo` / `density_at_memo` thread it through, and the
combiners keep one per thread.[^n34-s4]

- **The key is the leg mask and a bracketing fingerprint**, not the mask alone.
  Four-momentum sums are not associative in floating point, so two bracketings
  of one leg set can differ in the last bits; sharing on the mask alone would be
  a silent reassociation. With the fingerprint, what comes back is bit for bit
  what the plain recursion computes, pinned by a both-directions bit-identity
  test and a two-bracketings separation test.
- `MEMO_MAX_LEGS = 10`: past ten outgoing legs no table is built and every
  lookup misses (the plain recursion).
- `SubsystemMemo::disabled()` is the ablation switch, so the sharing can be
  measured against its own absence.

Measured on the `2 → 6` rows: density is 37.1% / 27.5% of an accepted point's
cost; the memo cut the density arm by 38% on both, about 10–14% end to end.

[^n21-substrate]: Note 21: what the seam was built on.
[^n21-closeout]: Note 21 close-out: the seam and the combiner as delivered.
[^n24-p0]: Note 24 P0: the 3.09–3.48× reading and the vacuous reciprocity check.
[^n24-p2]: Note 24 P2 and plan correction 3: walk weight, `WALK_DENSITY_TOL`, the positivity table.
[^n28-s24]: Note 28 §S2.4: the foreign-configuration density contract.
[^n34-draw]: Note 34 §1.1: cut-first pricing and its measurement.
[^n34-s4]: Note 34 S4 close-out: the subtree memo and the bracketing key.
