---
type: Algorithm
title: "The VEGAS integrator: grid adaptation, damping, adapt and frozen phases"
description: "VegasGrid's Lepage importance grid, the damping exponent (1.5 raw, 0.5 on mapped multichannel integrands), adapt vs sample_frozen, serialisation, and what VEGAS+ would add."
status: draft
tags: [vegas, integration, importance-sampling, grid, serialisation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-vegas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L159-L182", title: "Note 01, VEGAS and VEGAS+ summaries"}
  - {id: n18-phases, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L258-L282", title: "Note 18 §2.4, VEGAS phases and serialisation"}
  - {id: n18-h5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5, decision records H5 and H8 (VEGAS split, serde, artifact format)"}
  - {id: n21-production, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L300-L381", title: "Note 21, putting the sampler into production (damping exponent)"}
  - {id: vegas-rs, resource: "vibegraph-lib/src/vegas.rs", title: "VegasGrid, adapt/sample_frozen/draw, parallel forms, refine_grid"}
  - {id: hadronic-rs, resource: "vibegraph-lib/src/hadronic.rs#L68-L87", title: "VEGAS_NBINS, VEGAS_ALPHA, VEGAS_ALPHA_MAPPED"}
  - {id: lepage, resource: "https://doi.org/10.1016/0021-9991(78)90004-9", title: "G. P. Lepage, A new algorithm for adaptive multidimensional integration (1978)"}
  - {id: vegasplus, resource: "https://arxiv.org/abs/2009.05112", title: "G. P. Lepage, Adaptive multidimensional integration: VEGAS enhanced (2020)"}
---

`vibegraph-lib/src/vegas.rs` is classic Lepage VEGAS:[^lepage] importance
sampling on a separable, piecewise-uniform grid over the unit hypercube, with no
stratification. Every integrand in the crate reaches VEGAS through a
phase-space map (`u ∈ [0,1]^d → point`), so VEGAS never knows what the
coordinates mean ([channel contract](channel-contract.md)).

## The grid and one iteration

`VegasGrid { ndim, nbins, alpha, xi }` holds, per dimension, `nbins + 1` bin
edges `xi[d][k]` with `xi[d][0] = 0` and `xi[d][nbins] = 1`. Every bin is
drawn with equal probability, so narrow bins concentrate points and the
Jacobian of a draw is `nbins · Δx` per dimension: this is importance sampling,
not stratification[^n01-vegas]. Production grids use `VEGAS_NBINS = 64`
(`hadronic.rs:68`).

An iteration draws `neval` points, accumulates the integral estimate and, per
dimension and bin, the histogram of `(f·w)²`, then `refine_grid`
(`vegas.rs:906`) reshapes the edges:

1. normalise each bin's sum by the expected hits per bin;
2. damp with Lepage's rule `m_k → avg · ((r_k − 1)/ln r_k)^α`, `r_k = m_k/avg`;
3. smooth with a three-bin nearest-neighbour average;
4. redistribute the edges so every new bin carries an equal share of the
   damped weight.

The density is a product `∏_d g_d(u_d)`. A correlation between coordinates is
invisible to it, which is why each multichannel term gets its own grid rather
than one grid over a channel-selection coordinate
([per-channel grids](per-channel-vegas-grids.md)). Adapting on `(f·w)²` also
starves bins that mostly fail the cuts, which is where the heavy weight tail
comes from ([weight tail](vegas-grid-weight-tail.md)).

How the per-iteration estimates are combined is its own decision: the default
is the unweighted mean after two warm-up iterations, not Lepage's `1/σ²`
weighting, which biases σ low at small budgets
([iteration combination](vegas-iteration-combination.md)). The paper
summary's "combine estimates weighted by 1/σ²" describes Lepage's method, not
this code's default.

## The damping exponent

The exponent `α` in step 2 is set per integrand, from code:

| constant | value | used on |
|---|---|---|
| `VEGAS_ALPHA` | 1.5 | a raw integrand: flat RAMBO, the single-grid Drell–Yan path |
| `VEGAS_ALPHA_MAPPED` | 0.5 | any integrand behind a resonance-aware multichannel map (`use_multichannel` and `use_multichannel_with_alphas` set it) |

Lepage's 1.5 assumes the grid has to discover the integrand's structure. A
converged multichannel map has already flattened the peaks it knows about, so
the per-bin `f²` statistics in the hypercube are mostly noise, and a high
exponent turns that noise into a real distortion: the grid collapses into a
corner and later iterations sample a smooth region with a small
variance[^n21-production]. On `e+ e- > mu+ mu- ta+ ta-` (25 channels), five
seeds:

| `α` | outcome | quoted error |
|---|---|---|
| 0.0 (frozen grid) | all stable, χ²/dof ≈ 1 | 1.15e-5 |
| 0.5 | all stable, χ²/dof ≈ 1 | 5.1e-6 |
| 1.5 | one seed at 36% of the banked σ, χ²/dof ≈ 580 | — |

The tell was in the iteration path (`probe_vegas_iteration_path`,
`validate_sigma.rs`): iterations 1–3 agree, iteration 4 drops and stays
there. `probe_grid_adaptation_is_the_residue` is the damping sweep. The 1.5
paths keep 1.5 so their banked numbers do not move.

These numbers were measured while iterations were still combined by `1/σ²`,
which is what turned a collapsed iteration into a confident wrong answer. The
exponent has not been re-measured under the unweighted default.

## Phases

The API splits adaptation from use:[^n18-phases][^vegas-rs]

- **`adapt(f, neval, niter, rng)`** samples and refines for `niter`
  iterations, combining all but the first `warmup` of them.
- **`sample_frozen(f, neval, rng)`** is one pass with no refinement, against a
  grid trained earlier and possibly deserialised. Its estimate goes through no
  multi-iteration combination at all.
- **`draw(rng, x)`** fills one point and returns its Jacobian weight `1/p(x)`.
  It is the accept/reject primitive ([unweighting](../events/unweighting.md)):
  a point's full weight is that Jacobian times the integrand, with no
  accumulation in between. It draws the same sequence `sample_frozen` draws
  from the same generator state.
- **`adapt_batched` / `sample_frozen_batched`** take a batch callback. `adapt`
  and `sample_frozen` *are* the batched forms at batch size 1, so batched and
  unbatched results are bit-identical for any batch size.

The parallel forms keep the result independent of the thread count
([RNG substreams](rng-substreams-and-parallel-determinism.md)):

- `adapt_parallel` / `sample_frozen_parallel` key each chunk's `ChaCha8`
  substream by `(iteration, chunk)` and reduce chunk partials in chunk order.
  They are thread-count invariant but do **not** reproduce `adapt`'s numbers;
  production does not call them.
- `adapt_parallel_seeded` (`vegas.rs:525`) is bit-for-bit the sequential
  `adapt` over a seekable substream: each chunk seeks to global point `p · ndim`
  draws, and per-point values and bin indices are reduced in global point
  order on one thread. `test_adapt_parallel_seeded_is_the_sequential_adapt`
  pins it.
- `adapt_blocks_iteration` lifts that iteration body over many grids at once,
  one block per channel, scheduled by `(block, chunk)` in one rayon region.
  The per-channel integration in `budget.rs` runs on it
  ([budget allocation](channel-budget-allocation.md)).

## Serialisation

`VegasGrid` derives `Serialize`; `Deserialize` goes through `from_raw`, which
rejects a zero dimension or bin count, a shape mismatch, endpoints other than
0 and 1, and non-increasing edges, so a corrupt grid fails at load rather than
mis-sampling later. `warmup` and `combination` are `#[serde(skip)]`: they
describe an adaptation, not a grid, and leaving them out kept banked artifact
bytes unchanged when they were added.

The integrate artifact stores one `ChannelGrid` per channel (`artifact.rs`,
`ChannelGrid`): the grid with its `alpha`, `neval`, `sigma_pb`,
`sigma_err_pb`, `chi2_per_dof`, channel key and sampler summary, written with
bincode + zstd, which stores `f64` as raw bytes. The schema and its version
history live in the `FORMAT_VERSION` doc comment
([integrate artifact](../pipeline/integrate-artifact.md),
[format versioning](../pipeline/artifact-format-versioning.md)).

A JSON round trip of a grid is not bit-exact under `serde_json`'s default
float parser, which moved some values by one ulp. The `float_roundtrip`
feature (enabled in `vibegraph-lib/Cargo.toml`) fixes it; a test asserting
bit equality on `f64` payloads through JSON needs it[^n18-h5].

`Vegas::new` / `Vegas::integrate` survive as a thin wrapper over `adapt`.

## What VEGAS+ would add

VEGAS+[^vegasplus] adds adaptive stratification inside the importance grid:
the hypercube is cut into stratification cells and evaluations are moved to
the cells with the largest variance. The paper reports 2–19× on integrands
with several peaks or diagonal structure[^n01-vegas]. It is not implemented.
Here the channel decomposition already takes the diagonal structure out of
each grid and the budget is already stratified across channels, so the gain
inside a channel is open: [VEGAS+ unmeasured](../backlog/feature/vegas-plus-stratification-unmeasured.md).
See also [VEGAS](../references/papers/vegas.md) and
[VEGAS+](../references/papers/vegas-plus.md).

Open behaviour of the grid itself:
[refinement noises a constant integrand](../backlog/performance/vegas-refinement-noises-constant-integrand.md),
[low-acceptance bins starve](../backlog/performance/vegas-grid-starves-low-acceptance-bins.md).

[^lepage]: Lepage 1978, the algorithm `vegas.rs` implements.
[^n01-vegas]: Note 01, VEGAS and VEGAS+ paper summaries.
[^n18-phases]: Note 18 §2.4, the adapt / frozen split and batched seam.
[^n18-h5]: Note 18 §5, H5 (parallel entry points, `float_roundtrip` footgun) and H8 (bincode + zstd artifact).
[^n21-production]: Note 21, the sampler-in-production addendum, "Defect 2".
[^vegas-rs]: `vibegraph-lib/src/vegas.rs`, module and method docs.
[^vegasplus]: Lepage 2020, VEGAS+.
