---
type: Design Decision
title: One VEGAS grid per multichannel term
description: "Each channel gets its own frozen grid in its own map coordinates as MadEvent does, so errors and w_max are per channel; absolute grid coordinates are designed, not built."
status: draft
tags: [vegas, multichannel, unweighting, madevent-parity, grid-coordinates]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n21-grid, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L382-L511", title: "Note 21, one VEGAS grid vs MadGraph's grid-per-channel (design and outcome)"}
  - {id: n23-grid, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L898-L943", title: "Note 23, per-channel VEGAS grids: what event output inherits"}
  - {id: n37-abs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L360-L399", title: "Note 37 §5.2, absolute grid coordinates: the design"}
  - {id: channel-rs, resource: "vibegraph-lib/src/phasespace/channel.rs#L355-L375", title: "MultiChannel, 'Splitting the estimator by channel'"}
  - {id: unweight-rs, resource: "vibegraph-lib/src/unweight.rs#L1-L40", title: "unweight.rs, why the channel is drawn ∝ w_max_j"}
  - {id: artifact-rs, resource: "vibegraph-lib/src/artifact.rs#L255-L282", title: "ChannelGrid"}
  - {id: mg-sample-get-x, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/dsample.f#L1245", title: "MadEvent dsample.f, sample_get_x"}
---

## Decision

The multichannel estimator is integrated as a sum of per-channel terms, each
on its own VEGAS grid over that channel's own coordinates:

```text
∫dΦ f = Σⱼ ∫dΦ f·αⱼgⱼ/g = Σⱼ E_{p∼gⱼ}[ αⱼ·f(p)/g(p) ],   g = Σₖ αₖ gₖ
```

Term `j` is the integrand with the channel frozen: draw from channel `j` alone
(`MultiChannel::sample_channel`, no selection coordinate) and weight by `αⱼ/g`
with the same combined `g` the mixture uses[^channel-rs]. Every integrand
exposes it as `value_in_channel(j, u)` through the `ChannelIntegrand` trait
(`unweight.rs:213`; `hadronic.rs`, `proton.rs`, `multiplicity.rs`), and
`adapt_grids*` trains one grid per channel and sums the terms with their
errors in quadrature. This is MadEvent's arrangement — one job, one grid, one
`σⱼ ± Δσⱼ` per configuration (`G<config>/` directories) — reached from a
Kleiss–Pittau mixture rather than from single-diagram enhancement
([multichannel](multichannel.md),
[MadEvent's enhancement](madevent-single-diagram-enhancement.md)).

`FixedBeamIntegrand::adapt_grid` (one grid over `1 + channel_ndim`, `u[0]`
selecting the channel by cumulative `α`) survives as the comparison arm.
How many points each grid gets per iteration is
[budget allocation](channel-budget-allocation.md); the α-adaptation runs on
the mixture before the grids are trained and is untouched by the split.

## Why the split is the stronger arrangement

Both arrangements are unbiased; they differ in what a grid can learn:[^n21-grid]

1. **Coordinate semantics differ per channel.** `u[k]` is a Breit–Wigner
   invariant in one channel and a transfer `t` in another. A shared grid
   learns the α-weighted average density per slot, which is right for no
   single channel.
2. **VEGAS is a product density**, and the channel-selection coordinate is
   exactly the correlation it cannot represent: the best conditional density of
   `u[1..]` depends on which channel `u[0]` picked.
3. **Per-channel errors and maxima.** They let the budget follow channel
   variance and give each channel its own `w_max` for unweighting. A single
   global `w_max` over a mixture is set by the worst channel.

Where the channel maps already flatten their poles the residual integrand is
nearly featureless and the shared grid is adequate; that is also the regime of
over-adaptation, hence `VEGAS_ALPHA_MAPPED = 0.5`
([VEGAS integrator](vegas-integrator.md)).

## What it bought

Variance × CPU, `(Δσ²·T)_shared / (Δσ²·T)_split`, four seeds at gate budgets
(`probe_per_channel_grid_variance_cpu`, `validate_sigma.rs`):

| process | channels | Δσ ratio | err²·CPU |
|---|---|---|---|
| `ee_to_mumua` | 8 | 1.28–1.37× | 1.68× |
| `ee_to_mumu_tata_qcd0` | 25 | 1.24–1.36× | 1.46× |
| `gg_to_ttx` | 3 | 1.16× | 1.39× |
| `uux_to_uux` | 2 | 1.01–1.33× | 1.36× |
| `ee_to_tatah` | 5 | 1.05× | 1.07× |

CPU stays within ±13%; the gain is variance, and it is smallest where one
channel carries almost all of `α`, since then there is nothing conditional to
learn.

Unweighting efficiency (`probe_unweighting_weight_max`), per-channel maxima
against one global maximum:

| process | efficiency, global `w_max` | efficiency, per-channel `w_maxⱼ` | gain | largest channel's share of `Σⱼ w_maxⱼ` |
|---|---|---|---|---|
| `ee_to_mumua` | 3.3e-3 | 9.3e-3 | 2.86× | 29% |
| `gg_to_ttx` | 7.1e-2 | 1.7e-1 | 2.37× | 38% |
| `ee_to_mumu_tata_qcd0` | 5.0e-3 | 1.0e-2 | 2.04× | 24% |
| `uux_to_uux` | 1.6e-2 | 2.7e-2 | 1.69× | 90% |
| `ee_to_tatah` | 4.6e-2 | 4.2e-2 | 0.91× | 100% |

`ee_to_tatah` is the honest negative: one channel is the whole sum, and its
own scan, spending every draw where it lives, finds a slightly higher
extremum. The gain tracks the largest channel's share of `Σⱼ w_maxⱼ`, not the
channel count, and that share is the diagnostic to print.

The overall efficiency is `σ / Σⱼ w_maxⱼ` only because the generator draws
the channel **∝ `w_maxⱼ`**, not ∝ `σⱼ`: a trial in channel `j` is accepted
with mean probability `σⱼ/w_maxⱼ`, so drawing ∝ `w_maxⱼ` is what makes the
kept events carry cross section ∝ `σⱼ` (`unweight.rs` module doc,
[unweighting](../events/unweighting.md))[^unweight-rs]. Each `w_maxⱼ` is
estimated from a frozen scan of that channel's grid, so it is an estimate and
overweights are expected; how the maximum is chosen over a heavy weight tail
is the unweighting concept's, and why the tail exists is
[weight tail](vegas-grid-weight-tail.md).

Every gated σ row kept passing with a smaller error at the same budget
(0.74–0.96× the shared-grid error). One row moved the wrong way: sharper
per-channel grids stopped compromising with each other and covered the
spacelike collinear region of `uux_to_uux` less, roughly doubling its standing
negative bias (−0.17% → −0.30%). The cause was the flat transfer draw at the
collinear edge; with [the spacelike floor](spacelike-floor.md) the five-seed
mean reads +0.019%.

## The artifact

The integrate artifact stores a `Vec<ChannelGrid>`: grid, `alpha`, `neval`,
`sigma_pb`, `sigma_err_pb`, `chi2_per_dof`, the channel key and the sampler
summary (`artifact.rs`)[^artifact-rs]. A grid is meaningful only together
with its `alpha`, which is why both are banked, and a generator replays the
artifact's `α` rather than re-surveying (`use_multichannel_with_alphas`). The
format version is read from the payload prefix before the body is decoded and
an unsupported one is refused. A single-grid run banks one entry with
`alpha = 1`. Versions and their history:
[integrate artifact](../pipeline/integrate-artifact.md),
[format versioning](../pipeline/artifact-format-versioning.md).

## Absolute grid coordinates: designed, not built

MadEvent's `sample_get_x` (`dsample.f:1245`)[^mg-sample-get-x] bins each
invariant's VEGAS coordinate on the *absolute* dimensionless scale (`s/s_tot`,
`−t/s_tot`), restricts the draw to the point's own `[x_min, x_max]` window by
bin index and scales the weight by the window's bin count. A cut edge or a
pole is then a fixed grid location. Here a grid coordinate is the fractional
position inside the window, so the same feature drifts through the unit cube
as `ŝ` and earlier invariants move[^n37-abs]
([MadEvent maps](../references/codebases/madevent-phase-space-maps.md)).

Building it needs two changes, and one shortcut is rejected:

1. **Invert the VEGAS↔channel contract.** Today VEGAS draws the whole point
   (`VegasGrid::draw`) and hands the channel a slice. A windowed draw needs
   the channel to drive the grid one coordinate at a time, since the window of
   coordinate `k` depends on coordinates `< k`: a coordinate source threaded
   through `sample_branch`/`sample_spine` and the hadronic outer draw, a
   per-dimension `draw_in_window(dim, lo, hi)` on `VegasGrid` that reports the
   bin for refinement, and driven variants of `adapt`, `sample_frozen`, the
   `w_max` scan and the unweighting replay. The eager path draws the same
   uniforms in the same order, so it can stay bit-identical.
2. **Make every analytic map window-independent.** A BW, log or `t` map must
   be a fixed transform of the absolute coordinate over the full range, with
   the window inverted through it (`T⁻¹(lo)`, `T⁻¹(hi)`), so each channel's
   density stays grid-free. That is what keeps the mixture `Σⱼ αⱼ gⱼ` defined
   at a point another channel drew. MadEvent needs no such care, because its
   single-diagram-enhanced weight never evaluates one channel's density at
   another's point. Four map families, each needing an inverse.
3. **Rejected: the rejection form.** Reading the eager coordinate as absolute
   and zero-weighting draws outside the window is unbiased, but for a hadronic
   run whose windows sit at `ŝ ≪ s` it wastes most draws and would measure
   worse for reasons unrelated to the idea.

No flag is exposed until the code exists, so no artifact can claim a map that
was not run. Open work:
[VEGAS grid coordinates are not absolute](../backlog/performance/vegas-grid-coords-not-absolute.md).

[^channel-rs]: `vibegraph-lib/src/phasespace/channel.rs`, `MultiChannel` doc.
[^n21-grid]: Note 21, the grid-per-channel addendum and its outcome.
[^unweight-rs]: `vibegraph-lib/src/unweight.rs`, "Why the channel is drawn ∝ w_maxⱼ".
[^artifact-rs]: `vibegraph-lib/src/artifact.rs`, `ChannelGrid` and `FORMAT_VERSION` docs.
[^mg-sample-get-x]: MadEvent `dsample.f` at the pinned commit.
[^n37-abs]: Note 37 §5.2.
