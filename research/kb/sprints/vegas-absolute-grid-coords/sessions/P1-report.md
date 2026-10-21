---
type: Session Report
title: "P1 report: contract inversion and fixed-transform maps (paused, work in progress)"
description: "Paused at the user's request before any gate ran: the channel walk now requests each coordinate from a source, VEGAS grids map coordinates on request, and invariant maps are fixed transforms with an inverse; bit-identity is argued, not yet measured."
status: draft
agent: performance-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## State at the pause

The user paused the session. Nothing below has been gated.

- **Code commit:** `3b23d87` on `vegas-absolute-grid-coords`. Subject: "wip: drive
  VEGAS grids from the channel walk; invariant maps as fixed transforms". Tree
  `e792ed11252272e439ddc555ad8ef34aa033988d`.
- **What ran on that tree:** `cargo fmt --all`, then
  `cargo check --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation`.
  The check finished with no errors and no warnings. Nothing else ran on it:
  no `cargo test`, no clippy, no seeded goldens, no banked layer. **The claim
  that every golden is bit-identical is not tested.** The argument for it is
  below.
- **Baseline banked run:** this runs `pixi run --skip-deps validate` in a
  separate detached checkout, `/home/user/wt/gridcoords-P1-baseline` at
  `a8b1427`. Its log is
  `/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/gridcoords-P1-baseline-validate.log`.
  When it finishes, it copies its `target/validation-report` to
  `.../scratchpad/gridcoords-P1-baseline-report`. At the pause it was still
  building.
- **A first baseline was discarded.** I started the first attempt inside the
  session worktree, then edited sources while cargo was still compiling the
  test targets, so those targets were built from mixed sources. I killed that
  run by PID and started the baseline again in its own checkout. **Rule for
  P2:** never take a baseline in the tree you are editing.

## What was implemented (commit `3b23d87`)

1. **Coordinate source** (`phasespace/coords.rs`). The trait is
   `Coordinates<F>`, with `within(lo, hi)` and `unit()`. A map asks for each
   coordinate when its walk reaches it, and names the window `[lo, hi] ⊆ [0, 1]`
   of that coordinate's axis.
   - `SliceCoords` serves a pre-drawn point. It returns `lo + (hi − lo)·u[i]`.
     Over `[0, 1]` that is `u[i]` exactly.
   - `PhaseSpaceMap::sample_driven` and `ScaledChannel::sample_at_driven` have
     default implementations that collect `ndim` unit coordinates and then call
     the slice form. `DiagramChannel` overrides both. Its slice `sample`/`sample_at`
     now go through `SliceCoords`. A counting wrapper keeps the debug assertion
     that the walk consumes exactly `ndim` coordinates.
2. **Grid side** (`vegas.rs`, `GridPoint`).
   - It draws a point's `ndim` uniforms up front, one 64-bit draw per dimension
     in order. This keeps `adapt_blocks_iteration`'s draw addressing
     (`point · ndim` words) valid whatever the integrand reads.
   - It maps a dimension through its bins only when that dimension is
     requested. `finish()` maps any dimensions that were never requested, in
     order. The Jacobian is accumulated in dimension order, which is the same
     product as before.
   - `adapt_blocks_iteration_driven` is the core loop. The slice-taking
     `adapt_blocks_iteration` survives as a test-only wrapper, so tests that
     call it are unchanged.
   - `draw`, `draw_point` and the eager `run_iter_batched` are now written on
     top of `GridPoint`.
3. **Production drivers.**
   - `ChannelIntegrand::value_in_channel_driven(channel, coords, trailing)`
     receives the scale-draw uniforms separately from the grid coordinates.
   - Its default collects the coordinates and calls `value_in_channel`.
     `FixedBeamIntegrand`, `ProtonIntegrand` (which asks for τ and y first, then
     the channel's coordinates) and `MultiplicitySum` override it.
   - The budget driver, the unweighting scan and the unweighting trials now go
     through the driven form. Accepted points still store the grid-mapped
     coordinates, so replays use the slice path.
4. **Fixed transforms** (`diagram_channel.rs`).
   - The `FixedTransform` trait has `support`, `value`, `interior_coordinate`
     (the inverse) and `measure`. Its `coordinate` method returns exactly 0 at
     the support's lower edge and exactly 1 at its upper edge.
   - `InvariantMap` covers flat, Breit–Wigner and the two-piece log map.
     `TransferMap` covers the t-channel `1/(m² − t)` map.
   - `draw_through(map, lo, hi, coords)` inverts the window through the map,
     asks the source for `y` in `[T⁻¹(lo), T⁻¹(hi)]`, and pays
     `T′(y)·(T⁻¹(hi) − T⁻¹(lo))`. The densities use the same
     `window_measure`, so a map's density depends on the window and the
     transform alone.
   - Every call site builds the transform **on the point's own window**. That
     reproduces today's window-relative draw.
   - The old `draw_invariant`/`invariant_measure`/`draw_t`/`t_measure` survive
     as test-only wrappers, so the existing map tests are unchanged.

### Why it should be bit-identical (argued, not measured)

- **Draws.** The uniforms are drawn the same way, in the same order, from the
  same words. Bin mapping and the Jacobian product use the same arithmetic in
  the same dimension order.
- **Request order.** The channel's requests follow the old cursor order: left
  invariant, right invariant, cos θ, φ, then the left and right recursions;
  for a spine, blob, rest, t, φ.
- **Window edges.** With the transform built on the window, `coordinate(lo)`
  is exactly 0 and `coordinate(hi)` is exactly 1, because equality is tested
  before any arithmetic. Then `y = 0 + 1·u = u`, and the measure is multiplied
  by exactly `1`.
- **Degenerate windows.** When `lo = hi`, the window becomes `[0, 0]` and both
  forms give `s = lo` with measure 0. When `lo > hi`, the window is still read
  as `[0, 1]`.
- **NaN.** NaN propagates in both forms.
- **What is new.** The per-point allocations are new: `GridPoint` per
  unweighting trial and the default-path `Vec`. These are a cost, not a value
  change, and I did not measure them.

**What would falsify the argument:** a moved cell in the after-run against the
baseline above, or any seeded golden failing. Run those first on resume.

### Not done

- Gates: `cargo test --workspace`, clippy (plain and with
  `extended-validation`), `kb-lint`, and the after-run banked comparison.
- New tests for the inverse on interior sub-windows. These would check that
  `T⁻¹(T(y)) ≈ y`, and that a sub-window draw is measure-preserving and
  reciprocal to `window_measure`. P2 depends on those properties and nothing
  tests them yet.
- A timing check of the driven path. It adds a `dyn` call per coordinate and
  some allocations.

## What I learned

- **The inversion is forced by draw addressing, and easy once you accept it.**
  The parallel block scheme seeks each chunk to `point · ndim` words, so a
  point must consume a fixed number of draws. The natural split is therefore
  eager uniforms with lazy mapping, not lazy draws. That split also makes
  bit-identity easy to keep, and it leaves room for P2's windowed draw.
- **Only the log map is window-shaped.** BW, flat and t maps are the same curve
  on any support, reparametrised. So "relative on an absolute transform" equals
  today's draw *mathematically*, though not bitwise. The two-piece log map is
  different: its floor (`min(10, t_hi/50)`), its kink and its tail fraction all
  depend on the support. On an absolute support it would be a different map,
  with a fixed floor and kink. That is what MadEvent does (pre-warp `xo`), and
  it is a physics change P2 must measure, not plumbing.
- **The brief's premise holds** with one correction: "every analytic map" should
  read "every invariant map". The τ map already has a fixed support
  `[τ_min, 1]`, so it is already absolute. The rapidity window and the
  shaped-split `cos θ*` window move with earlier draws, but they are not
  invariants. MadEvent's `sample_get_x` does not absolute-bin them either.

## P2 design note: plugging in absolute coordinates

- **Support.** In absolute mode, each invariant's transform is built on a
  support that does not move with the point: for example
  `[threshold², s_tot]` for a timelike invariant, and a fixed `[−s_tot, 0]` (or
  the cut-implied edges) for t. The point's kinematic window then inverts to a
  strict sub-window `[y_lo, y_hi]`. Callers change only in which support they
  pass to `InvariantMap::new`/`TransferMap::new`.
- **Grid source.** `GridPoint::within(lo, hi)` gains a per-grid mode. Relative
  mode stays as it is now, `lo + (hi − lo)·x`. Absolute mode restricts the
  dimension's draw to the bins covering `[y_lo, y_hi]`: it maps the pre-drawn
  uniform onto that bin range, splitting the edge bins. The weight becomes the
  grid density relative to uniform-in-window, which is MadEvent's "scale by the
  window's bin count". The recorded bin is the one hit, so refinement is
  unchanged.
- **What stays the same.** The draw count per point is still `ndim`, so the
  addressing holds. The channel's density stays `T′(y)·(y_hi − y_lo)`, a
  function of the window and the transform alone. This is what keeps the
  Kleiss–Pittau mixture well defined at a foreign point.
- **Replay.** Replay stores the returned in-window `y`, not the raw uniform. A
  replay source must therefore return stored values verbatim (an absolute
  `SliceCoords` variant). The current `SliceCoords` would re-window them.
- **Artifacts.** The mode must be recorded in the integrate artifact and in
  `MapChoices`, so that `generate` rebuilds the same supports.

### #48's bin-width bound K on an absolute axis

K bounds every bin at `K/nbins` of the full `[0, 1]` axis. On an absolute axis,
a point whose window covers a fraction `w = y_hi − y_lo` of that axis sees
roughly `w·nbins` bins in its range, and K bounds them in *absolute* width.
Two consequences follow:

- **Inside a narrow window the bound is loose.** A window covering 5% of the
  axis can hold a single bin of width `K/nbins` that is wider than the window
  itself. The draw is then flat in-window, and the grid gives no resolution
  there.
- **Window-edge bins dominate the acceptance rescale.** Points whose windows
  overlap differently weight the same absolute bins. The acceptance rescale
  ("non-zero points per bin") counts hits per bin, not window coverage, so a
  bin near a cut edge looks low-acceptance even when every point whose window
  reached it was accepted. A coverage-aware denominator is needed: the
  fraction of points whose window contains the bin.

So K keeps its meaning, a cap on absolute bin width, but stops guaranteeing
in-window resolution. Read it as a per-tier, per-axis quantity, not as one
number for every invariant.

### Fit with the multi-tier VEGAS the user is designing

The candidate design has three parts: a shared outer tier (τ, y; absolute),
per-channel inner tiers, and possibly a K per tier, with non-aligned cuts left
to the pluggable-sampler work.

- **What fits.** The coordinate-source seam is what a tiered sampler needs.
  `ProtonIntegrand::value_in_channel_driven` already asks for τ and y first,
  then hands the same source to the channel. A tiered source answers the first
  two requests from the shared outer grid and the rest from the channel's
  inner grid, with no change to the maps. `GridPoint`'s eager-uniforms,
  lazy-mapping split carries over per tier. Each tier draws its own fixed word
  count, so each tier can have its own stream and addressing.
- **Where it conflicts or needs a decision.**
  1. Today one `GridPoint` serves all of a channel's dimensions, including τ
     and y. A shared outer tier means the outer grid is refined from every
     channel's points. The block scheme's per-block histogram ordering would
     then need a cross-block reduction in fixed order to stay bit-for-bit, and
     the budget driver's allocation would interact with it.
  2. τ needs no absolute mode, since its support is already fixed. y's window
     moves with τ, so "absolute y" means binning on `[−½ ln τ_min, ½ ln τ_min]`
     with a τ-dependent window. That is the same windowed-grid mechanism P2
     builds for invariants, applied to a non-invariant.
  3. Per-tier K is natural with the reading above. On the outer tier, K is a
     bound on absolute widths that every channel shares.
  4. Non-aligned cuts (pT, ΔR, HT) do not map to any single coordinate's
     window, so this mechanism cannot express them. That is consistent with
     leaving them to the pluggable sampler.

## Found

- **No test pins the inverse transforms on interior points.** In relative mode
  only their exact edges run. Closing it: property tests that check, per map
  family, `T⁻¹∘T = id`, that a sub-window draw is measure-preserving, and that
  it is reciprocal to `window_measure`.
- **The acceptance rescale under windowed draws.** As argued in the K note
  above, under absolute coordinates it would need a coverage-aware
  denominator. Closing it: a design decision in P2, or in the multi-tier work.
- **A baseline taken in a worktree that is then edited mid-build is silently
  contaminated.** Session briefs could say so. Closing it: a line in the
  dev-agent discipline, "take baselines from a separate detached checkout".
