---
type: Design
title: "A pluggable importance sampler in front of the phase-space map (proposed)"
description: "Where integration, the w_max scan, unweighting and the artifact assume a VegasGrid; the trait that would separate draw, accumulate, refine, freeze and serialise; and what each candidate sampler needs of it."
status: draft
tags: [vegas, sampler, integration, unweighting, artifact, design, refactor]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: vegas-rs, resource: "vibegraph-lib/src/vegas.rs", title: "VegasGrid: draw_into, run_iter_batched, refine_grid, adapt_blocks_iteration, serde"}
  - {id: budget-rs, resource: "vibegraph-lib/src/budget.rs", title: "integrate_channels and draw_iteration"}
  - {id: unweight-rs, resource: "vibegraph-lib/src/unweight.rs", title: "ChannelIntegrand, scan_channel, Unweighter::trial"}
  - {id: artifact-rs, resource: "vibegraph-lib/src/artifact.rs", title: "ChannelGrid, FORMAT_VERSION, version_for"}
  - {id: generate-rs, resource: "vibegraph-cli/src/generate.rs", title: "Artifact replay: grid.ndim checks and Unweighter::scan_with"}
  - {id: pr48, resource: "https://github.com/nsmith-/vibegraph/pull/48", title: "vegas-low-acceptance-bins: GridRule, ChannelIntegrand::grid_rule, hill_tail_index"}
---

> **Amended by [the joint seam design](sampler-and-map-seams.md) (2026-10-22).**
> The trait below stands, plus coordinate roles, mergeable accumulators,
> product parts and a per-role bin-width bound; the sampler never sees a
> coordinate window. The artifact versioning in "serialise" follows PR #60's
> rules (every artifact at the current version), not `version_for`.

Every adaptive density in the crate is a `VegasGrid`. This concept records
where that is assumed, the seam that would let another importance sampler
replace it per run, and what each candidate needs from that seam. It is a
proposal: nothing here is built. Line numbers are as of `57c3d8c`.

## What stays put

The sampler sits **in front of** the phase-space map and is a density on the
unit hypercube: it hands the map `u ∈ [0,1]^d` and returns the Jacobian
`1/p(u)` of its own draw ([channel contract](channel-contract.md)). Channels,
maps, the combiner's `Σ αⱼ gⱼ`, the Kleiss–Pittau α survey (it draws flat from
the mixture before any grid exists, `ProtonIntegrand::adapt_alphas`) and the
budget's allocation and stopping rule all stay where they are: they consume an
iteration's `(integral, variance, accepted)` per channel, never its bins
([budget allocation](channel-budget-allocation.md)). One trained sampler per
channel stays the arrangement ([per-channel grids](per-channel-vegas-grids.md)).

## Where `VegasGrid` is assumed

| site | assumption |
|---|---|
| `budget::integrate_channels` (`budget.rs:647`) | builds `VegasGrid::new(ndim, VEGAS_NBINS, vegas_alpha)` per channel, reads `warmup()` off the first grid, calls `refine_grid(&out.hist, neval)` |
| `vegas::adapt_blocks_iteration` (`vegas.rs:829`) | seeks ChaCha8 to `first_point · ndim · WORDS_PER_DRAW`: **exactly `ndim` draws per point**; ships bin indices out of each chunk as `u16`; forms `BlockIteration::hist` (per-dimension, per-bin) in point order |
| `VegasGrid` fields `warmup`, `combination` (and `rule` under PR #48) | adaptation settings stored on the trained grid, `serde(skip)` so artifact bytes do not move |
| `ChannelIntegrand::grid_rule()` (PR #48) | a VEGAS refinement knob on the integrand trait, which otherwise knows nothing about the sampler |
| `hadronic::ChannelIntegration::grid`, `artifact::ChannelGrid::grid` (`artifact.rs:283`) | the trained state is a bincode `VegasGrid`; the artifact has no sampler-kind tag |
| `unweight::scan_channel`, `Unweighter::trial` (`unweight.rs:353`, `:597`) | `grid.draw(rng, u)` on the **caller's** generator, `ndim` draws per point, between the channel draw and the accept draw |
| `generate.rs:957`, `:1502` | replay validates `grid.ndim()` against the integrand |

The determinism contract ([RNG substreams](rng-substreams-and-parallel-determinism.md))
is the binding constraint: `-j` must not move a bit, which holds because a
point's draws are addressable by its index and the refinement statistics are
reduced in point order. Any sampler has to keep both.

## The proposed seam

A trait (working name `HypercubeSampler`; not `Sampler`, which is already
`hadronic::Sampler { Flat, Multi }` and `artifact::ChannelSampler`, both the
phase-space map) with:

1. **draw** — fill `u`, return `1/p(u)`, and write a per-point `Record` the
   refinement needs (VEGAS: the bin index per dimension); a fixed
   `draws_per_point()` so chunks can seek;
2. **accumulate** — fold `(f·w, Record)` into a per-iteration `Accumulator`,
   called in point order (VEGAS: `BinHistogram`);
3. **refine** — consume the accumulator (VEGAS: `refine_grid` under its
   `GridRule`);
4. **freeze** — the trained state, `Clone + Send + Sync`, drawn from with no
   record (the scan and the trials);
5. **serialise** — the trained state only; adaptation settings (damping,
   `nbins`) belong to a per-kind config, except the VEGAS `GridRule`, which the
   artifact records top-level for provenance (`grid_rule: Option<GridRule>`,
   format version 12; `None` for an artifact with no VEGAS grid), and warm-up and iteration
   combination to the driver in `budget.rs`.

Static dispatch inside the hot loop (`integrate_channels` and
`adapt_blocks_iteration` generic over the trait, as they are generic over the
integrand today) and an enum of kinds at the boundaries (artifact, CLI,
`Unweighter`) keep the VEGAS path's arithmetic and draw order exactly as now.
The artifact precedent is `IntegrateArtifact::version_for`: a writer records
the oldest version whose schema holds what it banks, so a VEGAS-only artifact
can stay byte-identical while another kind needs a new version that older
readers refuse by number.

## Generic over the scalar, batchable over lanes

The phase-space side is already generic: `PhaseSpaceMap<F: Real>::sample(&[F])`
(`phasespace/channel.rs:50`), with `Real` the crate's scalar trait
(`helas/repr/mod.rs:97`, `num_traits::Float + FloatConst + Send + Sync`) and
`LaneField<N>` (`helas/eval/lane_field.rs`) its SIMD implementor, each lane
bit-identical to scalar `f64`, though `hadronic.rs` builds the maps only at
`f64`. The sampler is one of several `f64`-only links, beside scales, αs, cuts
and the PDF lookup: `VegasGrid` stores `Vec<Vec<f64>>` and draws `f64`. The
seam's draw, weight and accumulate signatures are therefore generic over
`F: Real`, with a batch form beside the scalar one so a lane pack or, later, a
GPU kernel can draw many points per call, and `F = f64` reproduces today's bits.
The seam is built and tested at `f64` first; the lane implementation is a later
step. Three things make that more than a type change:

- **The uniform.** `draw_into` takes `rng.random::<f64>()` (`vegas.rs:583`), not
  the crate's documented `u64_to_uniform<F>` (`phasespace/rng.rs:51-55`), which is the
  conversion lane sampling is pinned to. Routing through it keeps `f64` bits only
  if the two agree on every draw, a claim a test has to pin.
- **The bin index.** `(pos as usize).min(nbins − 1)` (`vegas.rs:585`) is a
  per-point integer, then a gather of two edges (`:587-588`). On `LaneField`, a
  conversion to an integer goes through lane 0 (`ToPrimitive`), so a per-lane
  index and gather need an operation `Real` does not have.
- **Accumulation and refinement.** Only accumulation is per point: a
  scatter-add by bin index in point order (`vegas.rs:667`, `:910`), which is
  what the `-j` contract rests on, so a lane form must not reorder the sums.
  Refinement runs once per iteration over bins, not per point, and stays `f64`:
  Lepage's damping branches per bin and calls `ln` and `powf` (`:695-705`);
  smoothing is a three-bin stencil (`:707-713`); the edge rebuild is a
  data-dependent serial scan (`:715-742`); PR #48 adds a sort and a fixed-point
  loop (`bound_bin_width`) and a per-bin acceptance count (`BinHistogram::add`).
  Vectorising it is across bins or dimensions, not lanes of points. The per-lane
  bin index, gather and scatter-add, and how refine is vectorised, are designed
  together, in a dedicated design session with the user, because jet clustering
  needs the same extension:
  [real-has-no-lane-masks-or-bin-search](../backlog/performance/real-has-no-lane-masks-or-bin-search.md).
  The lane port is
  [sampler-draw-not-lane-generic](../backlog/performance/sampler-draw-not-lane-generic.md).

## What the candidates need of it

| candidate | draws per point | accumulator | trained state | notes |
|---|---|---|---|---|
| uniform (no adaptation) | `d` | none | none | the second implementor that proves the trait is not VEGAS-shaped |
| VEGAS+ ([paper](../references/papers/vegas-plus.md)) | `d` | bin histogram plus per-hypercube variance | grid plus stratum allocation | allocation is per point index, so it must be a function of the plan, not of thread order |
| FOAM ([paper](../references/papers/foam.md)) | `1 + d` (cell, then position) | per-cell exploration statistics | binary cell tree | splits are sequential; the exploration of one cell is parallel |
| boosted trees ([paper](../references/papers/bendavid-bdt-dnn-integration.md)) | `1 + d` (box, then position) | the points and values themselves | tree ensemble | the training set is every `(u, f)` the run drew, so memory grows with the budget |
| normalizing flow ([i-flow](../references/papers/i-flow.md), [MadNIS](../references/papers/madnis.md)) | `d` | the points, values and densities of a batch | network weights | needs an ML dependency; inference order must be fixed for `-j` identity |

The backlog chain that builds this starts at
[sampler-hardwired-to-vegas-grid](../backlog/feature/sampler-hardwired-to-vegas-grid.md).
