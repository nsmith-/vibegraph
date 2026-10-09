---
type: Codebase Survey
title: MadEvent phase-space maps (genps/myamp/dsample/transpole)
description: "Every importance-sampling map MadEvent 3.7.1 applies, read at b7687064, beside vibegraph's: transpole, setgrid pre-warps, width floors, tstrategy, absolute grid coordinates, channel weights."
resource: "https://github.com/mg5amcnlo/mg5amcnlo/tree/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO"
status: draft
tags: [madevent, phase-space, importance-sampling, vegas, external-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n37-survey, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L26-L68", title: "Note 37 §1, MadEvent's maps against ours"}
  - {id: mg-genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f", title: "genps.f: one_tree (710), gen_s (1363), GENCMS (1621), get_channel_cut (1817)"}
  - {id: mg-myamp, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/myamp.f#L207-L594", title: "myamp.f: set_peaks, width floor, BW and setgrid branches"}
  - {id: mg-dsample, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/dsample.f", title: "dsample.f: setgrid (938), sample_get_x (1245), transpole call (1396)"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L5504-L5760", title: "export_v4.py: tstrategy and reorder_tchannels"}
  - {id: mg-matrix, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/matrix_madevent_group_v4.inc#L214-L228", title: "matrix_madevent_group_v4.inc: multichannel AMP2 and get_channel_cut by sde_strat"}
  - {id: mg-banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4447", title: "banner.py: tmin_for_channel default -1 (hidden)"}
  - {id: maps-rs, resource: "vibegraph-lib/src/phasespace/maps.rs", title: "TauMap, SplitAngle, RungOrder and the auto rules"}
  - {id: channel-rs, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs", title: "draw_invariant, log_scale, draw_t, spine_chain"}
---

MadEvent's phase-space generator (`one_tree`, `genps.f:710`) is the same
recursive two-body decomposition as vibegraph's `DiagramChannel`: s-channel
invariants from the outside in, then a t-channel chain, then the s-channel
decays. It importance-samples in two ways: analytic transforms (`transpole`,
called from `sample_get_x` at `dsample.f:1396` and from `gen_s` at
`genps.f:1406`), and VEGAS grids pre-warped by `setgrid` (`dsample.f:938`) where
there is no analytic map. Everything below is read at `b7687064` (v3.7.1)
unless a note is cited[^n37-survey].

vibegraph's map choices and the measurements behind them are
[map choices](../../phase-space/map-choices.md); the multichannel combination
is [multichannel](../../phase-space/multichannel.md). The rest of MadGraph is
[the MadGraph survey](madgraph5-amcnlo.md).

## The maps, variable by variable

| Variable | MadEvent | vibegraph |
|---|---|---|
| `τ = ŝ/s` (hadronic) | `transpole(pole = −2, width = xo)`: density `∝ 1/τ²` above `xo` (the cut-implied `ŝ_min/s`), flat below; a Breit–Wigner `tan` map instead when one finite-width resonance spans the whole final state | `TauMap::Log` by default (`τ = τ_min^(1−u)`, density `∝ 1/τ`); MadEvent's rule is `TauMap::InverseSquare`, selectable, not the default[^maps-rs] |
| rapidity `y` | flat in `[½ ln τ, −½ ln τ]` (`GENCMS`, `genps.f:1621`) | the same |
| s-channel invariant, finite width | `transpole(pole > 0)`: `s = m² + mΓ tan θ`, `θ` uniform; only when the pole is reachable (`m + 5Γ ≥ xm`, `bwcutoff` widths for a decay-chain-forced line) and the line is not identical-particle radiation (`iden_part = 0`) | the same `tan` map (`draw_invariant`) whenever the line has a width; no reachability test |
| s-channel invariant, zero width | `setgrid(itype = 1, xo)`: 90% of the grid bins log-spaced from 1 down to `xo` (`grid = xo^(1−i/ngu)`), 10% linear below; `gen_s` itself is flat (`spole = 0`) | the same two-piece shape as an analytic map (`log_scale`), floored by the cut-implied timelike floor |
| s-channel invariant, no pole at all | `setgrid(itype = 1)` always | flat; reached only for auxiliary invariants of vertices with more than two subsystems |
| t-channel transfer | `setgrid(itype = 1, xo)` on the absolute `−t/s`, `xo = min(ET₁, ET₂)²/s` from the two sides' cut-implied energies, invented floor `1/10000` | `draw_t`: density `∝ 1/(m² − t)`, pole floored at a fraction of the fiducial scale, window capped at `t ≤ −scale` |
| t-channel ladder order | `tstrategy` per configuration (`export_v4.py:5504`, `reorder_tchannels` at 5569): one side eats all, or ping-pong (`reorder_tchannels_pingpong`, 5747) when both outermost exchanged lines are massless and the ladder has three or more transfers | rungs from beam 0 (`spine_chain`); `RungOrder` selects the order |
| s-channel decay angles | flat `cos θ` and `φ` (`costh = 2x − 1`, `jac·4π`), built with `mom2cx` and boosted along the parent | isotropic, or the soft-shaped split angle where a split has a single gluon or photon daughter ([soft split angle](../../phase-space/soft-shaped-split-angle.md)) |
| t-channel rung azimuth | flat | flat |

`xo` for a zero-width or pole-less s-channel line is
`max(xm², s_min cuts, 0.8·ET₁ET₂ΔR², xqcut)/s`, or the invented
`min(10/s, s/50, ½)` when nothing applies (`myamp.f`, `set_peaks`).

## Width floors

`small_width_treatment` floors a **nonzero** width at `mass × small_width_treatment`
before the map is built (`myamp.f:131–135` and `:329–333`):

```fortran
if (prwidth(i,iconfig) .gt.0d0)then
   prwidth_tmp(i,iconfig) = max(prwidth(i,iconfig), prmass(i,iconfig)*small_width_treatment)
else
   prwidth_tmp(i,iconfig) = 0d0
endif
```

A line whose width is exactly zero (a UFO with `WZ = 0`) keeps zero and takes
the `setgrid` branch, as vibegraph's `log_scale` branch does for `mΓ ≤ 0`. The
floor matters only for tiny positive widths, which MadEvent maps as a
Breit–Wigner of width `m × small_width_treatment` while vibegraph maps the
model's own width. Both are unbiased; only the variance differs.

## Floors come from the cuts

`set_peaks` (`myamp.f:207`) derives every `xo` from the run card's cuts (`ptj`,
`ptl`, `etmin`, the `ΔR·ET` product, `xqcut`, `mmjj`), as
`Cuts::timelike_floor` and `Cuts::spacelike_floor` do here
([cut-implied floors](../../phase-space/cut-implied-timelike-floors.md),
[spacelike floor](../../phase-space/spacelike-floor.md)). MadEvent's floors only
pre-warp the grid: the grid can still move below them, which is what the
reserved 10% of bins is for. vibegraph's timelike floor is the support edge of
the log map (with the same linear tail below), and its transfer bound is a hard
edge.

## Grid coordinates

`sample_get_x` (`dsample.f:1245`) draws each invariant on a grid over the
**absolute** dimensionless invariant (`s/s_tot`, `−t/s_tot`, on
`[xgmin, xgmax] = [−1, 1]`), restricts the draw to the point's own
`[xmin, xmax]` window by bin index, and scales the weight by the window's bin
count. vibegraph's coordinate is the **fractional** position in each
invariant's own `[lo, hi]` window, so a feature at a fixed absolute invariant
moves in the unit cube as the other draws move `lo` and `hi`. For a
Breit–Wigner or log-mapped invariant the analytic map already pins the feature,
so the difference matters for flat-drawn invariants and residual shape. It is
open as [vegas-grid-coords-not-absolute](../../backlog/performance/vegas-grid-coords-not-absolute.md).

## Channels and grids

MadEvent integrates **one configuration per job**: in a `G<config>/` directory
`gen_mom` sets `nconfigs = 1`, `mincfig = maxcfig = iconfig`
(`genps.f:683–687`), and the per-configuration results are summed. Each job has
its own VEGAS grid. The channel weight partitions `|M|²` among configurations:

- `sde_strategy = 1`: `AMP2_c · CC_c / Σ_d AMP2_d · CC_d`, in the grouped
  matrix-element template (`matrix_madevent_group_v4.inc:214–228`).
  `CC = get_channel_cut` (`genps.f:1817`) is 1 at the default
  `tmin_for_channel = −1` and for configurations with fewer than two t-channel
  lines, so the weight is the familiar `|A_c|² / Σ_d |A_d|²`. The non-grouped
  template (`matrix_madevent_v4.inc:174–185`) has only that `AMP2` ratio.
- `sde_strategy = 2`: `CC_c` alone, the product of the configuration's
  propagator denominators.

Under either strategy, a `tmin_for_channel` above its default −1 multiplies
`CC` by `exp((t − t_min)/(t + 1))` for each t-channel line with
`t/s_tot < t_min` (`t` in units of `s_tot`).

The details are [single-diagram enhancement](../../phase-space/madevent-single-diagram-enhancement.md).
vibegraph instead samples a Kleiss–Pittau mixture `Σ αⱼ gⱼ` with
variance-minimising `α` and one VEGAS grid per channel
([per-channel grids](../../phase-space/per-channel-vegas-grids.md)). The
[loop-induced MG5 paper](../papers/loop-induced-madgraph5.md) §2.2 summarises
this diagram-enhancement scheme in prose; the code above is the authority on
its details.

`nzoom` re-draws inside the last bin during unweighting refinement; it is an
unweighting device, not a map.

## Open candidates from this survey

- `1/τ²` as the default hadronic map: measured, not adopted; see
  [map choices](../../phase-space/map-choices.md).
- Ping-pong ordering for ladders of three or more rungs (`p p > j j j j` class);
  inert for every gated row, since llj and the 2 → 3 QCD rows have at most two
  transfers: [tchannel-pingpong-for-three-rung-ladders](../../backlog/performance/tchannel-pingpong-for-three-rung-ladders.md).
- The Breit–Wigner reachability test (`m + 5Γ < xm` drops the map): low
  priority, the affected region carries little σ.
- A `1/s` pre-warp for a massive line drawn flat: rare here, since a massive
  s-channel line has a width and takes the Breit–Wigner map.

[^n37-survey]: Note 37 §1; the cited lines re-read at `b7687064`. Note 37 says the width floor also gives zero-width lines a Breit–Wigner map; `myamp.f:131–135` shows it does not.
[^maps-rs]: `vibegraph-lib/src/phasespace/maps.rs`, `TauMap` and `MapOptions::resolve`.
