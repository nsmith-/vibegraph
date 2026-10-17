---
type: Backlog Item
title: Integration, the w_max scan and unweighting are hard-wired to VegasGrid
description: "No seam separates draw, accumulate, refine and freeze from the VEGAS grid, and the grid is f64-only; every other sampler, and lane-batched drawing, needs one, bit-for-bit inert on VEGAS."
area: feature
state: blocked
priority: medium
closes_when: "A sampler trait generic over F: Real (scalar and batch draw, point-order accumulate, refine, freeze) has VegasGrid and a no-adaptation uniform sampler as implementors, drives integrate_channels, adapt_blocks_iteration, scan_channel and Unweighter, and the default path's grid.bin.zst, LHE and validation report are byte-identical before and after at F = f64."
blocked_by: [real-has-no-lane-masks-or-bin-search]
opened: 2026-10-17
tags: [vegas, sampler, refactor, integration, unweighting, simd, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: seam, resource: "../../phase-space/importance-sampler-seam.md", title: "The proposed importance-sampler seam"}
---
The user asked (2026-10-17) for pluggable sampling so FOAM, VEGAS+, tree and
flow samplers can be tried per run. `VegasGrid` is assumed from
`integrate_channels` through `Unweighter::trial`; the census and the proposed
seam are in [importance-sampler-seam](../../phase-space/importance-sampler-seam.md).

**What to build.** A trait (not named `Sampler`, which is the phase-space map's
name twice already) with draw → `1/p(u)` plus a per-point record and a fixed
draws-per-point, point-order accumulate, refine, freeze. Its draw, weight and
accumulate signatures are generic over `F: Real` (`helas/repr/mod.rs:97`) with a
batch form beside the scalar one, so `LaneField<N>` and later GPU code can draw
many points per call. Move `GridRule`, damping and `nbins` into a VEGAS config,
warm-up and combination into `budget.rs`, and drop `grid_rule()` from
`ChannelIntegrand`. A uniform, non-adapting implementor proves the trait is not
VEGAS-shaped. The artifact keeps `VegasGrid`; tagging it is
[artifact-has-no-sampler-kind](artifact-has-no-sampler-kind.md).

**The refine loop goes lane-friendly here, on the designed extension.**
Lane-wise comparisons, selects and bin search for `F: Real` are designed first:
[real-has-no-lane-masks-or-bin-search](../performance/real-has-no-lane-masks-or-bin-search.md).
How accumulation and the edge rebuild are batched on top of it is the user's
design, made when this item starts; the loop's obstacles, with file:line, are in
the [seam concept](../../phase-space/importance-sampler-seam.md). The draw uses
`rng.random::<f64>()` (`vegas.rs:583`), not the lane-exact `u64_to_uniform`;
switching must be pinned equal by a test.

**Inertness.** A pure refactor: same draws in the same order, so it closes on
byte identity, not statistics ([no-change claims](../../validation/no-change-claims.md)):
`grid.bin.zst` on the PR #48 P1 rows at `-j 1` and `-j 4`, a fixed-seed
`generate` LHE, and an empty `pixi run --skip-deps validate` report diff. That
leaves the σ calibrations PR #45 re-records untouched.

**Sequencing.** After PR #48 (`GridRule`), PR #51 (map defaults) and the
`vegas-constant-integrand` branch (`ChannelIntegrand::is_constant`,
`StopReason::Constant` in `integrate_channels`) merge. `vegas-absolute-grid-coords`
([vegas-grid-coords-not-absolute](../performance/vegas-grid-coords-not-absolute.md))
inverts the grid↔map contract; whichever lands second is rebased onto the other.
