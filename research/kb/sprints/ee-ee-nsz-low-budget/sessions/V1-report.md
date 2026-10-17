---
type: Session Report
title: "V1 report: the ee_to_ee_nsz low-budget scatter is the |η| < 2.5 edge inside the t-channel map"
description: "The under-sampled region is the last 3e-4 in cos θ below the lepton rapidity cut, where the spine t-map, bounded only by pT_min², leaves a step at the integrand's maximum; moving the map's bound to the edge removes the scatter, and the CLI default reaches the plateau on all twenty seeds."
status: draft
tags: [validation, vegas, budget-ladder, bhabha, s-channel-restriction, t-channel-map, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: 54510cb, resource: "https://github.com/nsmith-/vibegraph/commit/54510cb", title: "test(sigma): probe_ee_nsz_channel_split"}
---
Diagnosis only. One `#[ignore]`d probe was added to
`vibegraph-lib/tests/validate_sigma.rs` (`probe_ee_nsz_channel_split`, with
`NSZ_COS_EDGES` and `env_or`); nothing else in the file changed, and no σ gate
constant or comment was touched. No MadGraph was run and nothing was banked.

Every library number below comes from

```
RAYON_NUM_THREADS=2 VG_NSZ_SEEDS=20 VG_NSZ_FROZEN=200000 VG_NSZ_SCALES=<rungs> [VG_NSZ_ROW=<card dir>] \
  cargo test -p vibegraph-lib --features extended-validation --test validate_sigma \
  -- --ignored --nocapture --exact probe_ee_nsz_channel_split
```

(test result `ok. 1 passed` on all four configurations; logs
`eensz-V1-final-{base,ptl40,ptl20,etal5}.log` in the session scratchpad). The
probe calls the same `with_integrand` + `adapt_grids(neval, 6, seed)` path as
`probe_grammar_seed_sweep`, so its seeds are that sweep's seeds. "err" is the seed
policy's `max(quoted, spread/√n)` (`combine`); "spread" below is `spread/√n`.

## 1. Reproduction: the scatter is still there

`VG_NSZ_SCALES=0.25,1,4,16`, row `ee_to_ee_nsz`, seeds 1–20:

| rung | mean (pb) | err | quoted | spread/√n | seed χ²/dof | p(χ² ≥) at 19 dof |
|---|---|---|---|---|---|---|
| ×¼ 10k×6 | 157.6252 | 0.2308 | 0.2308 | 0.1820 | 3.843 | 2.9e-8 |
| ×1 40k×6 | 157.4303 | 0.1222 | 0.0976 | 0.1222 | 2.013 | 5.5e-3 |
| ×4 160k×6 | 157.5758 | 0.0530 | 0.0479 | 0.0530 | 1.228 | 0.22 |
| ×16 640k×6 | 157.5467 | 0.0289 | 0.0244 | 0.0289 | 1.531 | 0.065 |

These are the backlog item's numbers to every printed digit. The tail
probabilities are `scipy.stats.chi2.sf(chi2·19, 19)`.

## 2. Location

### Per channel

The row has two sampling channels: `ch0`, the s-channel photon on the
all-timelike tree, and `ch1`, a one-rung spine for the t-channel lines
(`ChannelSampler { topology: Spine, t_channels: [mass 0], spine_poles_gev2: [0.1] }`).

| rung | `ch0` mean ± err, χ²/dof | `ch1` mean ± err, χ²/dof |
|---|---|---|
| ×¼ | 0.029752 ± 0.000094, 0.965 | 157.5955 ± 0.2308, 3.843 |
| ×1 | 0.029752 ± 0.000094, 0.965 | 157.4006 ± 0.1223, 2.013 |
| ×4 | 0.029752 ± 0.000094, 0.965 | 157.5461 ± 0.0529, 1.226 |
| ×16 | 0.029742 ± 0.000095, 0.678 | 157.5170 ± 0.0289, 1.533 |

`ch0` sits at the 512-point channel floor on the first three rungs, so it is the
same computation there. The whole excess scatter is in `ch1`.

### Per region (`cos θ` of the outgoing e+, frozen passes)

Each seed's final `ch1` grid was sampled frozen on 200 000 fresh points
(`NSZ_FROZEN`, `NSZ_BIN`). The `cos θ` bins narrow toward the card's
`|η| < 2.5` edge at `cos θ = tanh 2.5 = 0.986614`. Typical seeds, ×¼:

| `cos θ` bin | seed 15 (read 155.675 ± 0.248): hits / σ_bin (pb) / var share / w_max/⟨w⟩ | seed 10 (read 158.647 ± 1.092) | ptl40 card, seed 15 |
|---|---|---|---|
| [0.9800, 0.9830) | 24849 / 19.55 / 0.016 / 2.2 | 23582 / 19.47 / 0.094 / 2.4 | 24507 / 19.71 / 0.120 / 2.2 |
| [0.9850, 0.9860) | 13085 / 10.61 / 0.009 / 2.5 | 12298 / 10.51 / 0.053 / 2.5 | 13097 / 10.51 / 0.064 / 2.2 |
| [0.9860, 0.9863) | 2182 / 2.60 / 0.081 / 184 | 6132 / 3.38 / 0.011 / 2.2 | 4276 / 3.46 / 0.021 / 2.2 |
| [0.9863, 0.9865) | **17** / 2.79 / **0.509** / **396** | 485 / 2.14 / 0.154 / 38 | 1708 / 2.36 / 0.038 / 8.6 |
| [0.9865, 0.9866) | **14** / 2.00 / **0.288** / 188 | 93 / 1.27 / 0.100 / 38 | 400 / 1.33 / 0.034 / 8.7 |
| [0.9866, 1.0) | 0 / 0 / 0 / – | 9 / 0.12 / 0.012 / 39 | 49 / 0.17 / 0.005 / 8.7 |
| frozen total | 157.83 ± 0.93; implied 4-iteration error 2.09 against the run's quoted 0.248 | 157.25 ± 0.25; implied 0.56, quoted 1.09 | 157.75 ± 0.11; implied 0.256, quoted 0.260 |

The region is **`cos θ ∈ [0.9863, 0.986614)`, i.e. `|t| ∈ [1673, 1713]` GeV², the
last 3·10⁻⁴ in `cos θ` below the rapidity cut**. It holds about 2.2% of σ (the
frozen seed-10 grid: 3.53 of 157.25 pb; `∫dt/t²` over the window gives 2.3%) and,
on grids like seed 15's, 80% of the per-point variance with weights up to 400× the
mean. Seed 15's final grid puts 31 of 200 000 draws there, about 1.5 per
10 000-point iteration.

Across all twenty seeds (medians over seeds; "edge" = `cos θ ≥ 0.985`):

| rung | `ch1` point rel. sd | edge variance share | edge w_max/⟨w⟩ |
|---|---|---|---|
| ×¼ | 1.448 | 0.684 | 195 |
| ×1 | 0.994 | 0.493 | 101 |
| ×4 | 1.159 | 0.575 | 176 |
| ×16 | 1.221 | 0.611 | 184 |
| ptl40 card, every rung | 0.313 | 0.155 | 8.5 |

So the grid does not learn the edge with budget: at every rung most seeds' grids
leave weights near 200× the mean there, and a minority (×¼ seeds 6, 9, 10; ×16
seeds 4, 16) resolve it to 5–40×. What budget buys is points in the sliver per
iteration, which is why the scatter settles by ×4 while the grids do not change.

### Missed region or honest scatter

Both faces of one heavy tail, not a bias. Every low seed has an unresolved edge
and a quoted error well below what its own grid implies:

| rung | seeds with pull < −1.5 vs the ×16 mean (pull; implied/quoted error; edge var share) |
|---|---|
| ×¼ | 1 (−1.60; 2.30; 0.85), 4 (−1.67; 1.66; 0.66), 15 (−7.55; 8.42; 0.89) |
| ×1 | 1 (−2.72; 1.91; 0.69), 9 (−4.21; 1.82; 0.63), 13 (−2.32; 1.56; 0.64), 19 (−1.97; 1.96; 0.88) |

A seed whose iterations draw no point in the sliver reads low **and** quotes a
small error (seed 15 at ×¼: 155.675 ± 0.248, frozen-implied error 2.09). A seed
that draws one shows instead as iteration χ²/dof 10–17 at a normal value (×¼
seeds 2, 13, 19, 20). The frozen pass over seed 15's own grid reads
157.83 ± 0.93, on the plateau: the grid is unbiased, the estimator is skewed.

### Mechanism

The spine draws `t` with density `∝ 1/(m² − t)` (`draw_t`) over
`[t_min, −fiducial_scale]`, with `fiducial_scale = Cuts::spacelike_floor() =
ptl² = 100` GeV² and pole floor `10⁻³ · 100 = 0.1` GeV² (the `0.1` printed
above). The binding cut is not `pT > 10` but `|η| < 2.5`, which at √s = 500 GeV
with lab = CM is `|t| ≥ (s/2)(1 − tanh 2.5) = 1673.3` GeV². In the map coordinate
`x`, with `N = ln(250000.1/100.1) = 7.823`, that edge is at
`x_c = ln(250000.1/1673.4)/N = 0.640`: 36% of the coordinate carries zero
integrand. Under a `1/t` density a `1/t²` integrand leaves `f/p ∝ 1/|t|`,
largest at `x_c`. So the integrand in the grid's coordinate rises
exponentially to its maximum and drops to zero there. A 50-bin VEGAS grid
that has not put a bin edge on that step spreads the step bin's density over
the zero side, and the sliver just below `x_c` (`x ∈ [0.637, 0.640)`, 0.3% of
the coordinate) is sampled so thinly that its weights run about 100× those of
the neighbouring bins (w_max/⟨w⟩ 180–400 against 2.2).

### MadEvent's channels

The refdata-10 seed record (`seeded_sigma/ee_to_ee_nsz`, `hard_survey = 2`, five
seeds) has two channels: `G1` 0.304148 pb (mean of 0.30404, 0.30415, 0.30403,
0.30443, 0.30409) and `G2` 157.236 pb (157.29, 157.15, 157.32, 157.14, 157.28). The
total is 157.536 ± 0.044 at χ²/dof 0.80. This side's plateau (×16) is
`ch0` 0.029742 + `ch1` 157.517 = 157.547 ± 0.029, pull +0.21 on the total.

The terms do not map one to one. MadEvent weights a configuration by single-diagram
enhancement (`|M_i|²/Σ|M_j|²`), this side by the balance heuristic
(`αⱼgⱼ/Σαᵢgᵢ`). The split is a sampler convention, not physics. The falsifier
cards below move this side's `ch0` from 0.02975 (ptl 10) to 0.01375 (ptl 20) and
0.00531 (ptl 40), while the total stays put. Only the sum compares, and it agrees.

## 3. The CLI default budget

`target/release-debug/vibegraph` (built `--profile release-debug --config
'profile.release-debug.debug=0'`), proc card `import model sm` +
`generate e+ e- > e+ e- $$ z`, the row's run card, defaults throughout (120 000
per iteration, target 0.1%, Neyman, min 6 iterations):

```
RUST_LOG=info vibegraph integrate eensz-V1-proc.dat --run-card validation/madgraph/output/ee_to_ee_nsz/Cards/run_card.dat \
  --out <dir> --force --seed <1..20> -j 2
```

Pulls are against the library plateau (×16, 157.5467 ± 0.0289) and MadEvent's
refdata-10 mean (157.536 ± 0.0436). "scaled" is the run's consistency-scaled error
from its `target:` line.

| seed | σ (pb) | quoted | scaled | pull vs ×16 | pull vs MadEvent | iterations | evaluations |
|---|---|---|---|---|---|---|---|
| 1 | 157.385 | 0.144 | 0.144 | −1.10 | −1.00 | 10 | 1 204 181 |
| 2 | 157.641 | 0.146 | 0.146 | +0.63 | +0.69 | 12 | 1 444 964 |
| 3 | 157.770 | 0.126 | 0.157 | +1.73 | +1.76 | 15 | 1 806 231 |
| 4 | 157.477 | 0.094 | 0.154 | −0.71 | −0.57 | 15 | 1 806 255 |
| 5 | 157.577 | 0.150 | 0.150 | +0.20 | +0.26 | 6 | 722 481 |
| 6 | 157.830 | 0.125 | 0.150 | +2.22 | +2.23 | 13 | 1 565 371 |
| 7 | 157.468 | 0.152 | 0.152 | −0.51 | −0.43 | 10 | 1 204 159 |
| 8 | 157.282 | 0.144 | 0.144 | −1.80 | −1.69 | 10 | 1 204 118 |
| 9 | 157.535 | 0.154 | 0.154 | −0.08 | −0.01 | 10 | 1 204 141 |
| 10 | 157.411 | 0.110 | 0.152 | −1.20 | −1.06 | 16 | 1 926 628 |
| 11 | 157.594 | 0.153 | 0.153 | +0.31 | +0.37 | 11 | 1 324 590 |
| 12 | 157.442 | 0.155 | 0.155 | −0.66 | −0.58 | 9 | 1 083 742 |
| 13 | 157.816 | 0.141 | 0.151 | +1.87 | +1.90 | 12 | 1 444 959 |
| 14 | 157.618 | 0.149 | 0.149 | +0.47 | +0.53 | 12 | 1 444 967 |
| 15 | 157.696 | 0.156 | 0.156 | +0.94 | +0.99 | 10 | 1 204 141 |
| 16 | 157.589 | 0.134 | 0.144 | +0.31 | +0.37 | 10 | 1 204 146 |
| 17 | 157.635 | 0.146 | 0.146 | +0.59 | +0.65 | 13 | 1 565 380 |
| 18 | 157.612 | 0.155 | 0.155 | +0.42 | +0.47 | 10 | 1 204 146 |
| 19 | 157.521 | 0.145 | 0.145 | −0.17 | −0.10 | 11 | 1 324 605 |
| 20 | 157.642 | 0.152 | 0.152 | +0.62 | +0.67 | 12 | 1 444 972 |

Twenty seeds: mean 157.5771 ± 0.0319 (quoted 0.0319, spread/√n 0.0318),
χ²/dof 1.13, pull +0.71 against the ×16 plateau and +0.76 against MadEvent. The
range is 157.282–157.830, every run reported `target ... met`, and the worst
seed is +2.22 (+1.86 on its scaled error). For the largest of twenty normal
pulls that is unremarkable (p ≈ 0.4). **The CLI default reaches the plateau on
every one of the twenty seeds.**

Its consistency scaling is what makes this hold. On seeds 3, 4, 6, 10 and 13 the
iterations disagreed (quoted 0.06–0.09% widened to 0.095–0.10%), and the run kept
going: 6–16 iterations, median 11, 0.72M–1.93M evaluations.

The same twenty seeds on the ptl40 card (below) all stop at the 6-iteration
minimum (722 930–722 970 evaluations), each at 0.047% where the target asks for
0.1%. They read 157.5436 ± 0.0167, χ²/dof 0.64, pull −0.09 against ×16 and +0.16
against MadEvent. On this row the edge costs the CLI 1.8× the evaluations at the
median (1.32M against 0.72M), at twice the error.

## 4. Attribution and its falsifier

**Attribution.** The scatter comes from the spine t-map's support bound. It is
taken from the transverse-momentum cut alone (`|t| ≥ ptl²`), so the binding
`|η| < 2.5` edge (`|t| ≥ 1673` GeV²) sits inside the map coordinate at
`x_c = 0.640`, exactly where `f/p` peaks. The VEGAS grid does not resolve that
step at any rung up to ×16. A 2%-of-σ sliver below the edge is therefore
sampled by O(1) points per iteration at ×¼–×1, and the per-seed estimate there
is Poisson-skewed with an under-quoted error.

**Falsifiers run.** Each one moves only the map's bound (`ptl`), or removes the
edge (`etal`), on a copy of the row's cards (`VG_NSZ_ROW`). The ptl cards leave
the accepted region, and so σ, unchanged: pT ≥ 40 GeV at 250 GeV is
`|cos θ| ≤ 0.98712`, looser than `tanh 2.5 = 0.98661`.

| card | map bound (GeV²) | zero-integrand share of `x` | ×¼ mean ± err, χ²/dof | ×1 | ×4 |
|---|---|---|---|---|---|
| row (ptl 10) | 100 | 36.0% | 157.625 ± 0.231, 3.84 | 157.430 ± 0.122, 2.01 | 157.576 ± 0.053, 1.23 |
| ptl 20 | 400 | 22.2% | 157.259 ± 0.179, 1.43 | 157.513 ± 0.081, 1.53 | – |
| ptl 40 | 1600 | 0.9% | 157.655 ± 0.058, 0.77 | 157.585 ± 0.031, 1.12 | 157.540 ± 0.0145, 0.61 |
| etal 5 (edge removed; σ is a different number) | 100 | ≈0 | 2771.68 ± 1.08, 0.66 | 2771.79 ± 0.54, 0.90 | 2771.59 ± 0.27, 0.28 |

The response follows the dose: as the step moves out of the coordinate, χ²/dof
falls 3.84 → 1.43 → 0.77 and the ×¼ mean error 0.231 → 0.179 → 0.058, at equal
budget. ptl 40 at ×4 reads 157.540 ± 0.0145, against the row's ×16 at
157.547 ± 0.029 (−0.2σ), so σ did not move. Under ptl 40 every rung's frozen
grids read point rel. sd 0.31 and edge w_max/⟨w⟩ 8.5, against 1.0–1.45 and
100–195 on the row's card.

**What would falsify it.** Any of these would:
- a card whose map bound sits on the edge and still scatters at ×¼;
- a low-reading seed whose frozen grid shows no edge tail;
- an implemented η-aware bound that leaves χ²/dof at ×¼ above ~1.5 on twenty
  seeds.

None of the first two was observed in eighty seeds.

## 5. Fix design (for a performance session; not landed)

**A. Bound the transfer by the rapidity cut where the bound is provable.** For a
2 → 2 rung emitting one massless leg at fixed beams with lab = partonic CM
(`ebeam1 = ebeam2`), `|t| = (s/2)(1 − cos θ)`. The leg's `|η| ≤ η_max` gives
`cos θ ≤ tanh η_max`, hence `|t| ≥ (s/2)(1 − tanh η_max)`. The rung's cap
becomes `max(pT_min², (s/2)(1 − tanh η_max))`, with `η_max` the tightest
rapidity cut over the legs on both sides of the rung: the recoil leg's
`cos θ = −cos θ_leg` bounds the same `t` from the other end.

For this row that is `max(100, 1673.3) = 1673.3` GeV². That gives
`N = ln(250001.7/1675.0) = 5.006` and puts the edge at `x_c = 1.000`, with no
zero-integrand strip. The residual `f/p ∝ 1/|t|` then spans
`1673 → 248 327` GeV², a smooth ×148 the grid learns without a step.

Where it lives: a new bound beside `Cuts::spacelike_floor`, rather than a change
to it, since the pole floor `10⁻³ · scale` should not move. It is passed to
`DiagramChannel::with_fiducial_t_max` per rung in the fixed-beam
`use_multichannel` construction.

Scope limits, each to be stated in the doc and tested:
- massless legs only (a massive leg's rapidity is not `atanh(cos θ)`);
- symmetric fixed beams only (an asymmetric fixed beam has a known boost `y_b`,
  so `tanh(η_max − y_b)` applies leg by leg; a hadronic beam's per-event boost
  gives no construction-time bound);
- one-rung 2 → 2 spines (a longer chain's rung does not have a fixed leg energy;
  the generic `|t| = 2E_a p_T e^{−y} ≥ 2E_a pT_min e^{−η_max}` = 410 GeV² here
  is weaker but valid for a single massless emitted leg off a massless beam in
  the CM frame).

It is a support bound, so the doc's "unbiased only while the channel set covers
every accepted point" applies. It is provable here because the rapidity cut is a
per-leg `Cuts::pass` condition.

Predicted effect, from the ptl 40 proxy (bound 1600 against the exact 1673; 0.9%
of the coordinate still zero):
- ×¼ seed χ²/dof near 1;
- ×¼ mean error 0.231 → about 0.058, i.e. 4× smaller, or about 16× fewer
  points for equal error;
- CLI default at the 6-iteration minimum on every seed, about 0.72M
  evaluations.

The exact bound should do at least as well. The `$$ z` row and the unrestricted
`ee_to_ee` row and control share the card, so both move.

**Gate for the fix.** Five things:
- this probe's base configuration on twenty seeds at ×¼ and ×1: χ²/dof inside
  the 19-dof band, means within 2σ of 157.547;
- the `the_grammar_rows_match_madevents_seeds` cells unchanged in status;
- a unit test that the bound never cuts an accepted point: draw at the bound's
  edge and check `Cuts::pass`;
- a control card where η is not binding (etal 5) whose map is unchanged;
- the sampling stream moves, so every fixed-seed σ gate on a fixed-beam t-channel
  row must be re-read. That re-recording belongs to whoever owns those gates
  (PR #42 is moving the fixed-beam gate to five seeds).

**B (second order, unmeasured).** With A in place the residual `f/p ∝ 1/|t|`
could also be flattened by drawing a massless spacelike photon `∝ 1/t²`. Without
A that would make things worse: the `1/t²` density puts 94% of `x` above the
η edge. It needs its own ladder; do not bundle it with A.

**C (no change).** This is not a correctness defect at the shipped budgets. The
gate runs at ×4, and the CLI default reaches the plateau on every seed. What A
buys is low-budget robustness, plus a factor of 1.8 in median CLI evaluations
on this row.

## Gates

On `54510cb`, with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

| command | result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo build` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo clippy -p vibegraph-lib --features extended-validation --test validate_sigma -- -D warnings` | exit 0 |
| `cargo test` | exit 0; summed over its `test result:` lines, 1478 passed, 0 failed, 16 ignored |
| `pixi run kb-lint` | 0 errors (two pre-existing warnings on performance backlog items) |

`pixi run --skip-deps validate` was not run. No amplitude, evaluation or coverage
code changed, and the only edit to a banked suite is an `#[ignore]`d probe.

## Brief and backlog corrections

- "MadEvent's own seeds have the same low tail" (backlog item; `GRAMMAR_NEVAL`'s
  doc) holds for `grammar_sigma_reference.json`. That is MadEvent's default
  survey, 6–10 s a run: seed 20260929 at 156.48 ± 0.11. It does not hold for the
  refdata-10 seed record the gate's MadEvent side now reads (`hard_survey = 2`,
  25–47 s a run): five seeds at 157.44–157.62, χ²/dof 0.80. Whether MadEvent's
  low seed comes from the same η edge inside its t-channel map is untested.
- "Compare the plateau channel split with MadEvent's `G*/results.dat` where
  channels map": they map only as a sum. The split is the multichannel weighting
  convention, as the ptl cards show on this side.
- Building the CLI with `CARGO_PROFILE_RELEASE_DEBUG_DEBUG=0` fails: cargo
  rejects it with "invalid type: Option value", apparently because the
  dash-to-underscore env mapping is ambiguous for the `release-debug` profile.
  `--config 'profile.release-debug.debug=0'` works.

## Found

1. **The spine t-map's support bound ignores the rapidity cut.**
   - What is wrong: `Cuts::spacelike_floor()` is `pT_min²` alone. On fixed-beam
     2 → 2 t-channel processes with a binding `etal`/`etaj`, the cut edge sits
     inside the map coordinate at the integrand's maximum. On `ee_to_ee_nsz` that
     is 36% of the coordinate, 2% of σ in the last 0.3%, and the cause of the
     ×¼/×1 scatter.
   - Evidence: §2 and the §4 falsifier table.
   - To close: fix A above, with its gate. The `ee-ee-nsz-sigma-low-at-low-budget`
     item's closing condition is met by §2 and §3; this is the separate sampler
     item.
2. **Seeds scatter below their quotes on clean configurations at 160k×6.**
   - What is wrong: etal 5 at ×4 reads χ²/dof 0.280 over twenty seeds,
     `chi2.cdf(0.28·19, 19) = 8.9e-4`; spread/√n 0.1435 against quoted 0.2710.
     ptl 40 at ×4 reads 0.61. AGENTS.md treats a χ²/dof far below 1 as loud as
     one far above. A plausible cause is anticorrelated iterations through grid
     refinement, which `combine_iterations` treats as independent, but it is
     unmeasured.
   - To close: a 40-seed ensemble per rung on one of these cards, plus the
     covariance of successive kept iterations within a seed.
3. **A possible low-budget offset on the ptl 40 card.**
   - What is wrong: ×¼ 157.655 ± 0.058 and ×1 157.585 ± 0.031 against ×4
     157.540 ± 0.0145, i.e. +1.9σ and +1.4σ. Twenty seeds cannot calibrate a
     rung difference, so this is not a claim.
   - To close: 40 seeds on ×¼ and ×4 (`probe_ee_nsz_channel_split` with
     `VG_NSZ_SEEDS=40`) before fix A's ladder is read.
4. **MadEvent's 156.48 seed on this row is unattributed.**
   - What is wrong: the default-survey low seed may be the same η edge inside
     MadEvent's t-channel generation.
   - To close: a diagnostic MadEvent run, throwaway and unbanked, with the
     pinned 3.7.1 on the default survey for seeds 20260926–20260930. Run it
     first with `ptl = 40` (σ unchanged), then with the row's card. If the
     156.48-type seed disappears only under `ptl = 40`, the mechanism is shared.
     No banked data is needed for this item's closure.
