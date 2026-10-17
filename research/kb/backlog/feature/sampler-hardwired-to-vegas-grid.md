---
type: Backlog Item
title: Integration, the w_max scan and unweighting are hard-wired to VegasGrid
description: "No seam separates draw, accumulate, refine and freeze from the VEGAS grid; every other sampler, and later lane-batched drawing, needs one, bit-for-bit inert on VEGAS."
area: feature
state: open
priority: medium
closes_when: "A sampler trait whose draw, weight and accumulate signatures are generic over F: Real (scalar and batch draw, point-order accumulate, refine, freeze), implemented and tested at F = f64 only, has VegasGrid and a no-adaptation uniform sampler as implementors, drives integrate_channels, adapt_blocks_iteration, scan_channel and Unweighter, and the default path's grid.bin.zst, LHE and validation report are byte-identical before and after."
blocked_by: []
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
many points per call; this item implements and tests them at `F = f64` only.
Move `GridRule`, damping and `nbins` into a VEGAS config, warm-up and
combination into `budget.rs`, and drop `grid_rule()` from `ChannelIntegrand`. A
uniform, non-adapting implementor proves the trait is not VEGAS-shaped. The
artifact keeps `VegasGrid`; tagging it is
[artifact-has-no-sampler-kind](artifact-has-no-sampler-kind.md).

**The uniform.** The draw uses `rng.random::<f64>()` (`vegas.rs:583`), not the
lane-exact `u64_to_uniform` (`phasespace/rng.rs:51-55`); both compute
`(bits >> 11) / 2^53`. Switching to `u64_to_uniform` is pinned equal on every
draw by a test here, so the lane-generic draw inherits it.

**Lanes come next.** The lane-generic draw and accumulate, and refine's
vectorisation over bins, are
[sampler-draw-not-lane-generic](../performance/sampler-draw-not-lane-generic.md),
which waits on the `Real` lane extension; this item does not.

**Inertness.** A pure refactor: same draws in the same order, so it closes on
byte identity, not statistics ([no-change claims](../../validation/no-change-claims.md)):
`grid.bin.zst` on the rows PR #48's grid-rule measurements used, at `-j 1` and
`-j 4`, a fixed-seed `generate` LHE, and an empty `pixi run --skip-deps validate`
report diff. That leaves the σ calibrations PR #45 re-records untouched.

**Sequencing.** After PR #48 (`GridRule`), PR #51 (map defaults) and PR #50
(the constant-integrand stop rule: `ChannelIntegrand::is_constant`,
`StopReason::Constant` in `integrate_channels`) merge. Absolute grid coordinates
([vegas-grid-coords-not-absolute](../performance/vegas-grid-coords-not-absolute.md))
invert the grid↔map contract; whichever lands second is rebased onto the other.
