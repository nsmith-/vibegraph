---
type: Design
title: The hadronic cross section and ProtonIntegrand
description: "σ = ∫dτ dy dΦ Σ_g avg_g[L_d|M(q)|² + L_m|M(Rq)|²]Θ/(2ŝ): PDF convolution and averaging, (τ,y) outer map, pooled inner map, |M|² in the partonic CM with lab-frame cuts."
status: draft
tags: [hadronic, proton, integrand, pdf, cross-section]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-goal, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L11-L37", title: "Note 18 (hadronic cross section: goal formula)"}
  - {id: n18-assembly, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L283-L311", title: "Note 18 §2.5 (hadronic assembly)"}
  - {id: n18-h7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 decision records (H7 averaging, cut frame, x-map)"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L943-L1010", title: "Note 24 P2 (design decisions for the general integrand)"}
  - {id: n24-p2-cost, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1032-L1041", title: "Note 24 P2 (what P3 must know)"}
  - {id: n24-p2c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1349-L1386", title: "Note 24 P2c (what the ProtonIntegrand session must know)"}
  - {id: n24-p2d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1394-L1423", title: "Note 24 P2d (ProtonIntegrand as built)"}
  - {id: n24-p2d-corr, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1502-L1524", title: "Note 24 P2d (plan corrections)"}
  - {id: n24-p2d-p3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1548-L1578", title: "Note 24 P2d (what P3 must know)"}
  - {id: n40, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L16-L65", title: "Note 40 §1–2 (per-group, per-ordering scales)"}
  - {id: proton-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L1233-L1285", title: "ProtonIntegrand type documentation"}
---

# The hadronic cross section and `ProtonIntegrand`

## The formula

A proton–proton run (`lpp1 = lpp2 = 1`) computes

```text
σ = ∫ dτ dy dΦ_n  Σ_g avg_g · [ L_g^direct(x₁,x₂,μF) |M_g(q)|²
                              + L_g^mirror(x₁,x₂,μF) |M_g(Rq)|² ] · Θ_cuts(q) / (2ŝ)
```

summed over the process's [flavour groups](flavour-groups.md).[^proton-rs] It
generalises the first hadronic observable, `p p → ℓ⁺ℓ⁻`, which convolved one coupling
class per `Z/γ` charge with both luminosity orderings.[^n18-goal][^n18-assembly] Each
symbol, and where it lives:

| symbol | meaning | code |
|---|---|---|
| `g` | a flavour group: subprocesses sharing one compiled `|M|²`, mass list, cut filter and colour basis | `FlavorGroup`, `proton.rs` |
| `avg_g` | `1/Π_a(n_spin·n_colour)` over the incoming legs; `1/96` for `g q`, `1/36` for `q q̄` | `FlavorGroup::spin_color_average`, from `hadronic::initial_spin_color_average` |
| `L_g^direct`, `L_g^mirror` | the group's two beam orderings' `x·f` products, summed over members, each member times its own identical-particle factor | `FlavorGroup::symmetry_weighted_luminosity` |
| `R` | rotation by π about x, applied to the outgoing legs only | `FlavorGroup::mirror_into`; see [the mirror identity](beam-mirror-identity.md) |
| `Θ_cuts(q)` | **one** cut indicator, on the unreflected final state, in the lab frame | the groups' shared `Cuts` |
| `1/(2ŝ)` | flux | — |

`|M_g|²` is colour- and helicity-**summed** (MadGraph's `MATRIX1` convention), which
is why the averages sit in the prefactor rather than in the matrix element.[^n18-h7]

**The identical-particle factor is a per-member scalar, never a map weight.** A
group is a statement about the matrix element, not about the outgoing multiset, and the
grouping rule does not hold the multiset fixed: `p p → j j` sums `g g → g g` and
`q q̄ → g g` (factor ½) with `q g → q g` and `g g → q q̄` (factor 1) under one mass list
(`pp_to_jj` is the banked row that exercises the mix). Each member therefore carries its own
`Subprocess::symmetry_factor()` (`phasespace::identical_particle_factor`) into the
luminosity sum. See [phase-space/identical-particle-factor](../phase-space/identical-particle-factor.md).

The mirror term is mandatory. The enumerator emits one ordering per unordered
initial state ([process/subprocess-enumeration](../process/subprocess-enumeration.md)),
and the exchanged ordering is a different physical initial state whose densities are
read at swapped momentum fractions. Drell–Yan could once sum both luminosity orderings
against one `|M|²` because its map was symmetric; a process with a jet cannot.[^n24-p2]
A member whose two beams carry the same parton contributes to `direct` only.

## The outer map: `(τ, y)`

The two outer coordinates are `τ = ŝ/s = x₁x₂` and `y = ½ ln(x₁/x₂)`, with
`τ_min = Cuts::shat_min()/s`. Since the grids return `x·f` and `x₁x₂ = τ`, the `1/x₁x₂`
of `f = (x·f)/x` cancels against `dτ`, and the luminosity is built straight from `x·f`
products. The direct map `x_i = x_min^(1−u_i)` came out **6% low (5.8σ)** on a dilepton
mass window, because the window is a thin diagonal band in `(x₁, x₂)` that VEGAS cannot
resolve; in `(τ, y)` it is a one-dimensional bound on `τ`.[^n18-h7] The map, its
Jacobian under each `TauMap`, and the `ŝ_min` derivation are
[phase-space/hadronic-tau-y-sampling](../phase-space/hadronic-tau-y-sampling.md). A `τ`
draw below the true kinematic threshold passes no cut and returns exactly `0.0`; how
loose `ŝ_min` is for a given card is [phase-space/cut-implied-timelike-floors](../phase-space/cut-implied-timelike-floors.md).

## The inner map: every group's channels, pooled

The remaining `3n − 4` coordinates go to one `ScaledMultiChannel` over **the per-diagram
channels of every group**, sampled at the event's own `√ŝ = √(τs)`.[^n24-p2d] Pooling is
what covers a peak one group's own diagrams do not: the `g q` group's mirrored peak, at
small `(p_b1 − p_jet)²`, equals `(p_b0 − p_ℓℓ)²` by momentum conservation and is covered
by the `q q̄` groups' spines.[^n24-p2] Diagrams whose maps are the same function
(`DiagramChannel::map_identity`), within a group or across groups, share one channel at
their summed selection weight (`ProtonIntegrand::new`); `new_unmerged_with_maps` builds
the unmerged mixture so the merge can be measured against it. Channel sets and their
merging are [phase-space/channel-set](../phase-space/channel-set.md).

- `channel_grid_ndim = 2 + (3n − 4)`; the undivided mixture form (`value`) adds one
  channel-selection coordinate, and one more uniform for the configuration draw where
  the per-term scale path is live (`vegas_ndim = channel_grid_ndim + 1 +
  scale_draw_ndim`). The scale-draw uniform is not a grid coordinate.
- A channel is named by a `ChannelId { group, channel }`: the first (group, per-diagram
  channel) pair whose map it is; `channel_members` lists every pair merged into it. An
  artifact must carry the dimension and the ids; a fixed-beam artifact cannot be read as
  a hadronic one by shape alone.[^n24-p2d-p3]
- Peripheral channels are floored at `Cuts::spacelike_floor()`, the scale the process's
  own transverse-momentum cuts imply. A final state of more than two legs has no
  peripheral spine without it; the floor also reshapes a `2 → 2` spine, which leaves the
  estimator unbiased but changes the map a banked run was taken with. See
  [phase-space/spacelike-floor](../phase-space/spacelike-floor.md).
- The per-iteration budget floor is `MIN_CHANNEL_NEVAL` per channel
  (`budget.rs`), whatever `neval` says, so a process with many channels has a large
  minimum iteration ([phase-space/channel-budget-allocation](../phase-space/channel-budget-allocation.md)).
- Re-installing banked channel weights must go through `set_channel_alphas`;
  re-running `adapt_alphas` reproduces them only by accident.[^n24-p2d-p3]

## Frames

`|M|²` is evaluated in the **partonic CM** with the beams on ±z: the frame the
helicity-pruned `BoundAmplitude::eval_m2` requires and the frame the channels generate
in. The cut filter and the scale prescription read **lab-frame** momenta, the outgoing
legs boosted along z by `y`, because `cuts.f`'s rapidity, pT and ΔR observables are not
boost-invariant.[^n18-h7] One consequence, worth knowing before trusting a cut to be
live: at LO a back-to-back lepton pair has Δφ = π in every frame, so `drll` never fires
on `p p → ℓℓ`.

## Guards the construction enforces

| refusal | condition | why |
|---|---|---|
| `GroupCutsDiffer` | two **groups** compile to different cut filters | the formula has one `Θ`; a process cut differently per group needs one indicator per group |
| `AmplitudeCount` / `AmplitudeMismatch` | the bound amplitudes are not one per group, in group order, each from that group's evaluator (`std::ptr::eq`) | crossing the pairing weights one group's `|M|²` with another's luminosity — a smooth σ shift with no other symptom, pinned by a test that swaps two amplitudes[^n24-p2d-corr] |
| from `derive_flavor_groups` | unequal outgoing masses, unequal cuts or colour basis within a group, degenerate groups | see [flavour groups](flavour-groups.md) |

## Scales: shared path and per-term path

`use_run_card_scales` compiles the run card's prescription against the set's `αs`
tabulation, so the integrand needs the `PdfSet`, not just a member.[^n24-p2d-p3] Where
the `αs` comes from is [scales-pdf/alpha-s-sources](../scales-pdf/alpha-s-sources.md).
`ProtonIntegrand::shape` then takes one of two paths:[^n40]

- **Shared**: a constant scale, or a closed form that reads no configuration. One scale,
  one pair of density rows, every group evaluated at it.
- **Per term**: the kT clustering (`dynamical_scale_choice = -1`) and anything under
  matching. Every group draws its own clustering configuration `∝ AMP2_c` of its own
  matrix element from one shared trailing uniform, clusters at the lab momenta, reads
  its densities at its own `μF` and binds `αs(μR)`. A group with a mirrored ordering
  draws a second configuration from `AMP2` at `Rq` and clusters the mirrored physical
  event (`mirror_lab_into`). MadEvent integrates each subprocess group apart, so a
  point is never clustered in another group's merge graph. The draw is
  [scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md).

So "every group at one scale per point" holds only on the shared path. Under the
clustering each group and each ordering takes its own scale, and the event record carries
the drawn term's scales ([events/generate](../events/generate.md)).

Under MLM matching (`ickkw = 1`) each member's luminosity is further multiplied, per
ordering, by its own `rewgt` factor ([scales-pdf/mlm-rewgt](../scales-pdf/mlm-rewgt.md)).
Forbidden on-shell s-channels (`$`) zero the vetoed amplitudes per group
([process/s-channel-restrictions](../process/s-channel-restrictions.md)). A card with
several final-state multiplicities builds one `ProtonIntegrand` per multiplicity inside
a `MultiplicitySum` ([phase-space/mixed-multiplicity-integrand](../phase-space/mixed-multiplicity-integrand.md)).

## Threads

The integrand is immutable while an integration runs. Each thread that evaluates a point
forks its own `ProtonScratch` (amplitudes, buffers, the `(μR, αs)` memo) through
`ThreadLocal`, and nothing in it carries a value from one point to the next: the
amplitudes rescale from the card's own `αs`, not from the previous point's. A point's
value is therefore the same whichever thread takes it, and the point loops run as rayon
regions ([phase-space/rng-substreams-and-parallel-determinism](../phase-space/rng-substreams-and-parallel-determinism.md)).

## Cost per point

One `|M|²` per group for the direct term, plus one per group with a mirror. A group
whose members all have identical beams (`g g`) has no mirror term and costs one
evaluation.[^n24-p2-cost][^n24-p2c][^n24-p2d-corr] Under the per-term scale path each group also pays
one `eval_amp2` per ordering for its configuration draw.[^n40] Absolute counts and
per-iteration floors depend on the process and on channel merging, so they are not
quoted here; measure them on the commit in question.

## Validation

Drell–Yan runs through this general path; there is no dedicated Drell–Yan integrand.
The cross-section rows are [validation/sigma-gate](../validation/sigma-gate.md), and the
oracles below σ (fixed-ŝ slices, the pointwise PDF × flux × `|M|²` × cut integrand oracle)
are [validation/integrand-and-sampler-oracles](../validation/integrand-and-sampler-oracles.md).
Single-seed pulls up to 2.7 were seen on quantities that agree to a few per mille, so a
hadronic σ is gated over a seed sweep, never on one seed ([AGENTS.md](../../../AGENTS.md),
"Samplers gate statistically").[^n24-p2d-p3]

[^n18-goal]: Note 18, the first hadronic observable and its formula.
[^n18-assembly]: Note 18 §2.5, the original assembly (direct x-map, coupling classes).
[^n18-h7]: Note 18 §5, H7: averaging and flux, the lab-frame cut, the `(τ, y)` remap.
[^n24-p2]: Note 24 P2, design decisions for the general integrand (mirror term, channel coverage).
[^n24-p2-cost]: Note 24 P2, cost notes for P3.
[^n24-p2c]: Note 24 P2c, the API and cost of the decomposition.
[^n24-p2d]: Note 24 P2d, `ProtonIntegrand` as built.
[^n24-p2d-corr]: Note 24 P2d, plan corrections (cut filter across groups, amplitude pairing, `has_mirror`).
[^n24-p2d-p3]: Note 24 P2d, what the next session must know (API, artifact keys, replay, seeds, `PdfSet`).
[^n40]: Note 40 §1–2, per-group and per-ordering scales.
[^proton-rs]: `ProtonIntegrand`'s type documentation, `vibegraph-lib/src/proton.rs`.
