---
type: Session Report
title: "M1 report: where the weight tail meets the cut edges, per channel"
description: "Measured on pp_to_llj_fixed, pp_to_jj and pp_to_llj_mlm: near-edge shares of the weight tail, how each edge sits in the grid coordinates, cross-channel (τ, y) grid distances, and accepted against rejected point cost. The jj tail sits on the jet pT edge, which is pinned by the earlier coordinates and not fixed in absolute coordinates. Most of the llj tail is away from every edge. Rejected points cost 1–2% of evaluation time, yet removing rejection would still be worth 1.3–1.5× in variance × CPU."
status: draft
agent: performance-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## What was done

Measurement only. No production behaviour changed.

- **Probe code:** `vibegraph-lib/src/proton/edge_probe.rs`, declared in
  `proton.rs` as `#[cfg(test)] mod edge_probe;`. It has two `#[ignore]`d tests,
  `probe_cut_edge_weight_tail` (M1, M2 and M4) and `probe_rejected_point_cost`
  (M5). The code commit is `ffefe94`.
- **Why the probe is in the crate and not in `tests/validate_hadronic.rs`.**
  This corrects the brief. Three things the brief asks for are not reachable
  through the public interface:
  - a rejected point's momenta, needed for the cut that failed first
    (`event_in_channel` returns `None`, and `combiner`, `map_point` and
    `build_frames` are private);
  - a trained grid's bin edges, needed for M4 (`VegasGrid::xi` is `pub(crate)`);
  - the mixture weight `αⱼ/g` on its own, needed for M5 (`channel_weight` is
    private).

  Extending the integration-test probe would have needed new public accessors,
  which would be a production change. The child module sees private items
  instead. It is compiled only under `cfg(test)`, so non-test builds are
  unchanged: the `cargo build -p vibegraph-lib --lib` line under Gates builds
  without the module.
- **Self-checks the probe asserts on every point:**
  - The restated cut margins agree with the compiled `Cuts::pass` on every
    drawn point (`mismatch == 0`, accepted and rejected). The margin columns
    therefore measure the filter itself, not a copy of it.
  - The probe's sample is the production `w_max` scan. For every channel, the
    truncated maximum read off its first `neval` draws is bit-equal to
    `Unweighter::scan_with(…, MaxRule::default())`'s `w_max`. The "scan" columns
    below are therefore the scan's own points.
- **MLM row: `pp_to_llj_mlm`.** It is the same `p p > e+ e- j` kinematics as
  `pp_to_llj_fixed`, with the `xqcut = 20` rewrites: `ptj → 20` and `drjl`
  removed. It adds the clustering scale and `rewgt`. It is the controlled
  comparison: any difference from `_fixed` is matching, not process content.
  `pp_to_ll_0j2j_mlm`, where `vegas-grid-weight-tail.md` saw the outer-bin tail,
  is a three-part `MultiplicitySum`. The probe's per-channel draw does not cover
  that yet (see Found).
- **Budgets:** each row's gate budget. llj and MLM: survey 8 000 × 5, VEGAS
  150 000 × 10. jj: survey 8 000 × 5, VEGAS 75 000 × 10. The grids are
  trained by the probe (`adapt_grids`). No banked artifact was read.

### Tree, seeds and commands

- **Tree:** HEAD `b0a9312` plus the probe files. The runs used `edge_probe.rs`
  blob `b6aa6221` and `proton.rs` blob `d7e6c951`, both recorded with
  `git hash-object` before the runs.
- **Committed blob.** `ffefe94` commits `edge_probe.rs` as blob `191d0f29`. It
  differs from the run blob in two places only:
  - a `type NearTest` alias, for clippy's `type_complexity`;
  - quoting of the edge name in the near-edge CSV.

  To check that the measurements did not change, the committed blob was rebuilt
  and `pp_to_jj` seed `20261020` was re-run at 200 000 draws. Its `TAIL`,
  `REJECT`, `GRID`, `GRIDCH`, `COND` and `TPICK` records are byte-identical to
  the five-seed run's (`diff` exit 0, log `gridcoords-M1-recheck-jj.log`).
- **Build:**
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --profile release-debug -p vibegraph-lib --lib --no-run`
- **M1, M2 and M4.** Five seeds, `20261020`–`20261024`, 200 000 draws a
  channel. Run under load from a concurrent banked run, which does not affect
  these numbers:

  ```
  VG_EDGE_SEEDS=5 VG_EDGE_DRAWS=200000 VG_EDGE_DUMP=$S/gridcoords-M1-dump \
    target/release-debug/deps/vibegraph-41e899bf1a677758 --ignored --nocapture \
    --test-threads=1 proton::edge_probe::probe_cut_edge_weight_tail
  ```

  It finished `test result: ok. 1 passed … finished in 1814.82s`. The trained
  σ̂ per seed in pb:

  | row | seed 1 | seed 2 | seed 3 | seed 4 | seed 5 |
  |---|---|---|---|---|---|
  | `pp_to_llj_fixed` | 424.87 | 424.34 | 424.12 | 424.39 | 423.41 |
  | `pp_to_jj` (×10⁸) | 6.7989 | 6.7975 | 6.8007 | 6.8025 | 6.8079 |
  | `pp_to_llj_mlm` | 268.81 | 268.39 | 268.37 | 268.93 | 267.96 |

- **M5.** One seed, `20261020`, 50 000 draws a channel, best of 3 passes, one
  thread. The host was quiet: the banked run had exited and the container had
  just restarted (load 0.50 at start, 1.10 at end). It ran detached:

  ```
  VG_EDGE_SEEDS=1 VG_EDGE_DRAWS=50000 VG_EDGE_REPS=3 \
    target/release-debug/deps/vibegraph-41e899bf1a677758 --ignored --nocapture \
    --test-threads=1 proton::edge_probe::probe_rejected_point_cost
  ```

  It finished `test result: ok. 1 passed … finished in 307.75s`.
- **Aggregation:** the five-seed figures below are the unweighted mean ± the
  sample standard deviation over seeds. They come from the probe's
  tab-separated `TAIL`, `REJECT`, `GRID` and `GRIDCH` records, aggregated by a
  scratch script. The edge geometry (M2) comes from a nearest-neighbour script
  over seed 1's near-edge dump (method below).

### Channel identities

Channels `0`–`5` on both llj rows: even = γ*, odd = Z.

- **Even channels are γ*:** α ≈ 2%, `N_total/N_j` ≈ 47–60, and they carry
  points on the `mmll` floor.
- **Odd channels are Z:** α ≈ 0.28–0.35.
- **Topology, from the probe's `TPICK` correlations:**
  - `0` and `1` are all-timelike (`q* → q Z/γ*`): `x3` correlates equally with
    both transfers.
  - `2` and `3` are spines drawing `t` off beam 0.
  - `4` and `5` are spines off beam 1.

`pp_to_jj` has three channels:

- `0`: the s-channel gluon, all-timelike, α ≈ 2%.
- `1` and `2`: the `t` and `u` spines, α ≈ 0.49 each.

The grid dimensions are as follows:

- llj: `x0 = τ`, `x1 = y`, `x2 = m_ll²`, `x3 =` polar (`t` or `cos θ`),
  `x4 = φ`, `x5 = cos θ*_ll`, `x6 = φ*_ll`.
- jj: `x0 = τ`, `x1 = y`, `x2 =` polar, `x3 = φ`.

## M1: how much of the tail sits near each edge

"Near an edge" means within ε of the threshold, relative to it, on the
accepted side:

- `pT/pT_min − 1 < ε`;
- `(η_max − |η|)/η_max < ε`;
- `ΔR/R − 1 < ε`;
- `m_ll/mmll − 1 < ε`.

For η this is relative to `η_max`, where the brief had an absolute margin.

### Row level, five seeds, mean ± sd

The σ share weights each channel by its term. The variance share weights each
channel's second moment by `1/N_j`, its production allocation.

| row | ε | edge | σ share | variance share |
|---|---|---|---|---|
| `pp_to_jj` | 5% | any | 0.179 ± 0.001 | 0.299 ± 0.009 |
| | 5% | `ptj` | 0.160 ± 0.001 | 0.266 ± 0.011 |
| | 5% | `etaj` (either jet) | 0.008 / 0.017 | 0.011 / 0.037 |
| | 10% | any | 0.325 ± 0.001 | 0.458 ± 0.006 |
| `pp_to_llj_fixed` | 5% | any | 0.120 ± 0.001 | 0.197 ± 0.008 |
| | 5% | `ptj` | 0.050 ± 0.000 | 0.087 ± 0.005 |
| | 5% | `etal` (ℓ⁺ / ℓ⁻) | 0.032 / 0.034 | 0.047 ± 0.008 / 0.053 ± 0.003 |
| | 5% | `ptl`, `drll`, `drjl`, `etaj` | ≤ 0.002 each | ≤ 0.004 each |
| | 10% | any | 0.230 ± 0.001 | 0.355 ± 0.013 |
| `pp_to_llj_mlm` | 5% | any | 0.127 ± 0.001 | 0.217 ± 0.014 |
| | 5% | `ptj` | 0.059 ± 0.000 | 0.108 ± 0.007 |
| | 10% | any | 0.240 ± 0.001 | 0.378 ± 0.014 |

`mmll` is absent from the llj row table because it carries 0 in the Z channels
and ≈ 0.05 of σ in the γ* channels. It is a map edge (M2).

### The tail itself, per channel, ε = 5%, five seeds, mean ± sd

The columns are:

- **pts:** the share of accepted points near any edge;
- **top 0.1% M₂:** the share of the top-0.1% weights' second moment;
- **top 20:** the second-moment share of the 20 largest weights;
- **scan excess:** the share of the scan's `Σ(w − w_max)₊`.

| row / ch | pts | M₂ | top 0.1% M₂ | top 20 | scan excess | of which `ptj` (top 0.1%) |
|---|---|---|---|---|---|---|
| jj 1 (t) | 0.129 | 0.294 ± 0.009 | **0.710 ± 0.022** | 0.82 ± 0.08 | 0.60 ± 0.10 | 0.646 ± 0.043 |
| jj 2 (u) | 0.130 | 0.305 ± 0.015 | **0.741 ± 0.040** | 0.93 ± 0.03 | 0.69 ± 0.04 | 0.676 ± 0.083 |
| jj 0 (s) | 0.063 | 0.253 ± 0.023 | 0.300 ± 0.098 | 0.29 ± 0.16 | 0.28 ± 0.19 | 0.250 ± 0.101 |
| llj 1 (Z, timelike) | 0.064 | 0.166 ± 0.008 | 0.239 ± 0.068 | 0.23 ± 0.13 | 0.20 ± 0.03 | 0.104 ± 0.034 |
| llj 3 (Z, spine) | 0.076 | 0.197 ± 0.014 | 0.280 ± 0.083 | 0.22 ± 0.12 | 0.35 ± 0.05 | 0.152 ± 0.061 |
| llj 5 (Z, spine) | 0.077 | 0.210 ± 0.011 | 0.319 ± 0.066 | 0.29 ± 0.15 | 0.25 ± 0.12 | 0.145 ± 0.056 |
| llj 0 (γ*, timelike) | 0.090 | 0.275 ± 0.074 | 0.399 ± 0.163 | 0.42 ± 0.21 | 0.23 ± 0.31 | 0.166 ± 0.230 |
| llj 2 (γ*, spine) | 0.102 | 0.309 ± 0.072 | 0.431 ± 0.260 | 0.41 ± 0.31 | 0.33 ± 0.27 | 0.091 ± 0.031 |
| llj 4 (γ*, spine) | 0.102 | 0.318 ± 0.042 | 0.469 ± 0.170 | 0.47 ± 0.26 | 0.37 ± 0.32 | 0.122 ± 0.065 |
| mlm 1 / 3 / 5 (Z) | 0.067–0.081 | 0.18–0.23 | 0.27–0.38 | 0.27–0.34 | 0.21–0.43 | 0.12–0.20 |
| mlm 0 / 2 / 4 (γ*) | 0.091–0.101 | 0.30–0.34 | 0.44–0.52 | 0.48–0.52 | 0.22–0.49 | 0.12–0.26 |

The full per-channel, per-edge table at ε ∈ {1%, 5%, 10%} is in the probe's
`TAIL` records. At ε = 1%, the any-edge top-0.1% share is 0.19–0.22 on jj
channels 1 and 2, and 0.06–0.11 (llj) or 0.07–0.20 (MLM) elsewhere. At
ε = 10% it is 0.84–0.85 on jj channels 1 and 2, and 0.46–0.73 on llj and MLM.

**Reading.**

- **`pp_to_jj`'s tail is the jet `pT` edge.**
  - In the two channels that carry 97% of σ, two thirds of the top-0.1% second
    moment sits within 5% of `ptj`.
  - That is 12% of the points.
  - It is consistent over five seeds (sd 0.04–0.08).
- **On `pp_to_llj_fixed` and `pp_to_llj_mlm`, most of the tail is away from
  every edge.**
  - Within 5% of any edge sits 24–47% of the top-0.1% second moment, against
    6–10% of the points.
  - The rest (53–76%) sits in the interior.
  - The near-edge part is split between `ptj` (9–17%), the two `etal` edges
    (5–14% each) and, in the γ* channels, the `mmll` floor (9–12%), which is
    already a map edge.
  - `ptl`, `drll`, `drjl` and `etaj` each carry ≤ 2%.
- **Matching (MLM) shifts little.** Its near-edge shares are within a few
  points of `_fixed`. `ptj` is slightly heavier, since `drjl` is removed and the
  jet sits at `xqcut = 20`.
- **The γ* channels' shares scatter widely across seeds** (sd up to 0.3 on the
  top-20 share). Those channels are starved (≈ 3 000 points an iteration), and
  their top weights are a handful of points. Read them as "edges carry under
  half", not as a number.

### Rejected points: Q and the cut that failed first (five seeds, mean ± sd)

| row / ch | Q | rejected by (share of rejections) |
|---|---|---|
| jj 0 | 0.706 ± 0.006 | `ptj` 0.947, `etaj` 0.053 |
| jj 1 / 2 | 0.815 / 0.816 ± 0.001 | `ptj` 0.846, `etaj` 0.155 |
| llj Z channels 1 / 3 / 5 | 0.561 / 0.546 / 0.546 | `etal` 0.49–0.51, `ptj` 0.40–0.42, `ptl` 0.07–0.08, `drjl` 0.01 |
| llj γ* channels 0 / 2 / 4 | 0.527–0.538 | `etal` 0.45–0.46, `ptj` 0.43–0.45, `ptl` 0.09, `drjl` 0.01 |
| MLM | 0.513–0.553 | as llj, `ptj` 0.42–0.47 |

`mmll`, `drll` and `etaj` (llj) reject nothing to three decimals.

`zero_after_cuts` is 0.000 on every row: no point that passed the cuts
carried zero weight. On `pp_to_llj_mlm` this means the kT clustering and
`rewgt` never zeroed a point in 6 × 200 000 draws × 5 seeds. A single
multiplicity is its own highest multiplicity, so there is no veto to fire, as
expected.

## M2: is each edge fixed in relative coordinates, absolute coordinates, or neither?

### Method (seed 1, one-seed descriptive)

Take every point within 1% of an edge on either side; there are 200–7 800
points a channel and edge. Three statistics are read off them:

- **outside:** the share on the rejected side. For an edge the map already
  enforces it is 0, since no draw lands beyond it. That is a direct test of
  "map edge", stronger than clustering at `x ∈ {0, 1}`.
- **R(k | S):** pair each near-edge point with its nearest near-edge neighbour
  in the coordinates `S ⊂ {0..k−1}` (rank-standardised, `|S| ≤ 3`). Then
  compare the median `|Δx_k|` over those pairs with the median over random
  pairs. `R → 0` means the edge's position in `x_k` is a function of `S`. The
  best `S` is reported.
- **Absolute position.** The ratio `|t|_min / ptj²` at the edge, and the
  interquartile ratio of the absolute variable (`R_abs` in the probe's `COND`
  records).

Since `x_k` determines everything once all earlier coordinates are fixed, the
criterion for a D candidate is that a **small** `S`, at most 3 earlier
coordinates, pins the edge.

| row / edge | outside | best R(k \| S) | absolute position | class |
|---|---|---|---|---|
| llj `mmll` (all channels) | **0.000** | `x2`, 0.25–0.51 given `τ` | `m_ll = 50` by definition | **map edge** (relative, `x2 = 0`) |
| jj `ptj`, ch 1 / 2 (spines) | 0.49–0.50 | **`x2`: 0.081 / 0.085 given `τ` alone** | `|t|_min/ptj²` 1.09–1.30 (IQR), 1.03–1.56 (5–95%) at √ŝ ≈ 56 GeV | **D** (drifts in absolute `t`) |
| jj `ptj`, ch 0 (timelike) | 0.49 | `x2`: 0.081 given `τ` | 1.13–1.38 (IQR) | **D** |
| jj `etaj`, ch 0 / 1 / 2 | 0.47–0.51 | **`x2`: 0.03 / 0.11 / 0.10 given (`τ`, `y`)** | — | **D** (conditional on the outer `y`) |
| llj `ptj`, spine ch 2–5 | 0.50–0.52 | **`x3`: 0.035–0.048 given (`τ`, `m_ll²`)** | `|t|_min/ptj²` 1.7–3.3 (IQR), 1.3–5.5 (5–95%) | **D** |
| llj `ptj`, timelike ch 0 / 1 | 0.51 | `x3`: 0.21 / 0.20 given (`τ`, `m_ll²`) | 2.2–4.2 (IQR) | **D** (two branches `±cos θ`, which pairs straddle) |
| llj `etaj`, all channels | ≈ 0.5 | `x3`: 0.06–0.23 given (`τ`, `y`) | — | D, but carries ≤ 2% of anything |
| llj `etal` (both leptons) | 0.49–0.50 | `x6`: 0.48–0.64 given (`y`, polar, `cos θ*`); every other `k` ≥ 0.63 | — | **C** (non-aligned) |
| llj `ptl` | 0.45–0.54 | `x6`: 0.14–0.35 given (`m_ll²`, polar, `cos θ*`) | — | C, or a weak D at the last coordinate; carries ≤ 2% |
| MLM: `ptj`, `etal`, `ptl` | as llj | `ptj`: `x3` 0.035–0.046 (spines), 0.24–0.27 (timelike); `etal`: `x6` 0.42–0.60 | — | as llj |

Two coordinate checks:

- **No live edge clusters at a window edge.** For every edge except `mmll`,
  the share of near-edge points with `x_k < 0.01` or `> 0.99` is at most 0.06
  in its own coordinate (`at_window_edge` in `COND`; baseline ≤ 0.02).
- **`mmll` sits at `x2 = 0`, but the trained grid stretches it.** Its
  near-edge points sit at `x2 < 0.01` only 4–22% of the time: the grid spreads
  the first 1% of `m_ll` over more than 0.01 of `x2`. The `outside = 0`
  column is the proof that it is a map edge.

**Answer to "is any high-weight edge fixed in absolute invariants?": no.**

- The one aligned, high-weight edge is jet `pT`.
- In absolute `−t` it drifts:
  - by 1.03–1.56 × `ptj²` on jj, where its near-edge points sit near threshold
    at √ŝ ≈ 56 GeV;
  - by 1.3–5.5 × on llj, where the massive recoil shifts it.
- It is pinned by `τ` (and `m_ll²`) at the 4–8% level in relative coordinates.
- Absolute coordinates would not make it sharper. A window conditional on
  `(ŝ, m_ll²)` (design D) makes it a map edge.

## M4: (τ, y) similarity across channels (five seeds, mean ± sd)

The columns are:

- **L1:** the integrated `|p_a − p_b|` of two channels' piecewise-constant bin
  densities on `[0, 1]`, ranging 0–2;
- **to pool:** the L1 from one channel to the `N_j`-weighted pooled density;
- **seed noise:** the same channel's L1 between seed 1 and seeds 2–5 (n = 4).
  This is the floor any distance has to clear.

| row | dim | pairwise L1 mean / max | starved channels: `N_tot/N_j`, to pool, seed noise | large channels: `N_tot/N_j`, to pool, seed noise |
|---|---|---|---|---|
| `pp_to_llj_fixed` | τ | 0.252 ± 0.012 / 0.379 ± 0.028 | 47–60, **0.32–0.34**, 0.12–0.15 | 3.0–3.6, 0.04–0.05, 0.05–0.06 |
| | y | 0.129 ± 0.010 / 0.195 ± 0.032 | 47–60, 0.10–0.12, 0.13–0.19 | 3.0–3.6, 0.04–0.07, 0.05–0.07 |
| `pp_to_llj_mlm` | τ | 0.260 ± 0.009 / 0.400 ± 0.018 | 48–57, **0.33–0.35**, 0.13 | 2.9–3.4, 0.04–0.07, 0.05 |
| | y | 0.131 ± 0.005 / 0.201 ± 0.031 | 48–57, 0.10–0.13, 0.13–0.16 | 2.9–3.4, 0.04–0.07, 0.05–0.06 |
| `pp_to_jj` | τ | 0.163 ± 0.024 / 0.229 ± 0.035 | 53, **0.22 ± 0.04**, 0.12 | 2.0–2.1, 0.017, 0.03 |
| | y | 0.148 ± 0.027 / 0.168 ± 0.029 | 53, 0.13 ± 0.04, 0.18 | 2.0–2.1, 0.07, 0.04 |

**What this says for a shared outer grid (A1).**

- **`y` is shared already, in effect.** Every channel's trained `y` marginal is
  within its own seed noise of the pooled one. Sharing it costs nothing
  measurable. It would also give the starved channels a `y` grid trained on
  ~50× their points.
- **`τ` is not the same across channels.**
  - The large (Z, or `t`/`u`) channels agree with the pool to within their
    seed noise.
  - The starved channels differ from the pool by 0.22–0.35 in L1, which is
    1.8–2.8× their seed noise.
  - On llj those are the γ* channels, whose `τ` marginal is pulled to the
    `mmll` floor. On jj it is the s-channel.
- **So a naively pooled outer grid (`N_j`-weighted) would hand the starved
  channels a `τ` grid shaped by the Z channels.**
  - The draft's optimal `q* ∝ √(Σ H_j/N_j)` up-weights starved channels by
    `1/N_j`, and may land elsewhere. These distances cannot say which.
  - Deciding A1 needs the variance of the starved terms under each pooled `q`.
    That is a cheap offline replay, but it is not done here (see Found).
- `N_total/N_j` is ≈ 50 for the starved channels and 2–3.6 for the rest.
  Starved channels are 2% of α and one third of the channels.

## M5: what a rejected point costs (one seed `20261020`, quiet host, one thread)

The cost is ns per point, best of 3 passes, over 50 000 trained-grid draws a
channel. The columns are:

- **acc `g`:** the `αⱼ/g` mixture weight alone, on accepted points;
- **acc rest:** accepted − rejected − `g`, which is the matrix elements, PDFs
  and scales.

| row | ch | Q | draw | rejected | accepted | acc `g` | acc rest | rejected share of eval time |
|---|---|---|---|---|---|---|---|---|
| `pp_to_llj_fixed` | 0–5 | 0.507–0.560 | 87–163 | 535–627 | 28 015–30 263 | 1 231–1 305 | 26 192–28 428 | **0.0165** |
| `pp_to_jj` | 0–2 | 0.702–0.818 | 71–79 | 230–289 | 98 886–103 144 | 176–180 | 98 421–102 735 | **0.0006** |
| `pp_to_llj_mlm` | 0–5 | 0.502–0.553 | 95–145 | 563–634 | 249 312–265 454 | 1 225–1 321 | 247 507–263 595 | **0.0020** |

The row columns come from the probe's `TIMEROW` records (allocation-weighted):

| row | accepted mean (ns) | rejected mean (ns) | Q at allocation |
|---|---|---|---|
| `pp_to_llj_fixed` | 29 024 | 596 | 0.550 |
| `pp_to_jj` | 100 696 | 286 | 0.814 |
| `pp_to_llj_mlm` | 260 732 | 621 | 0.540 |

The jj and MLM accepted cost is dominated by the per-point scale clustering,
not the matrix element proper. The probe does not split those.

**The draft's §5 M5 inference is wrong. I correct it here.**

- The draft says that with `T_ME ≫ T_ps`, a sharp window's variance × CPU gain
  "tends to 1".
- With the shape unchanged, a perfect window gives `w' = Q·w`, `Var' = Q·M₂ − σ²`
  and time `T_acc`, against `M₂ − σ²` and `Q·T_acc + (1−Q)·T_rej`. The gain is

  ```text
  G = (1 − r)(Q + (1−Q)·T_rej/T_acc) / (Q − r),   r = σ²/M₂
  ```

  At `T_rej → 0` this is `(1 − r)/(1 − r/Q)`, which is 1 only when `r → 0`.
- On these rows `r` is 0.20–0.51. From the five-seed `Q` and `r` and the timing
  above:

  | row | channels | r = σ²/M₂ | G (ceiling) |
  |---|---|---|---|
  | `pp_to_jj` | 1 / 2 | 0.507 / 0.505 | **1.305 ± 0.011 / 1.300 ± 0.011** |
  | `pp_to_jj` | 0 | 0.287 | 1.22 ± 0.09 |
  | `pp_to_llj_fixed` | Z: 1 / 3 / 5 | 0.27–0.29 | **1.49–1.50** (sd ≤ 0.04) |
  | `pp_to_llj_fixed` | γ*: 0 / 2 / 4 | 0.22 | 1.36–1.37 (sd ≤ 0.08) |
  | `pp_to_llj_mlm` | Z / γ* | 0.26–0.28 / 0.20–0.21 | 1.45–1.46 / 1.30–1.33 |

- So rejected volume is nearly free in time (0.06–1.7% of evaluation time).
  Even so, an exact window is worth up to 1.3× on jj and 1.5× on llj in
  variance × CPU, through the variance alone.
- That is a ceiling for the volume effect. M1's tail shares are additional:
  they say where the shape-dependent part lives.

## Answers to the design draft's questions (§6), from these numbers

1. **Target.** On jj the tail and the rejection are the same edge (`ptj`), so
   both targets point the same way. On llj the tail is mostly interior, so
   "sharp edges" would address under half of it.
2. **Absolute coordinates: no.** No high-weight edge is fixed in an absolute
   invariant on these rows (M2). `mmll` is already a map edge.
3. **A1:**
   - Share `y`: free on these rows.
   - Share `τ`: only with a measured variance check on the starved channels,
     whose `τ` marginal differs at 1.8–2.8× seed noise.
4. **D first on `pp_to_jj`: supported.**
   - Its `ptj` edge holds 65–68% of the top-0.1% second moment in the two main
     channels.
   - It is pinned by `τ` alone to `R = 0.08`.
   - `etaj` is pinned by (`τ`, `y`).
   - Together they are all of jj's rejections, and the ceiling gain is 1.3×.
   - On llj, a `(τ, m_ll²)`-conditional `ptj` window (`R = 0.04` in spine
     channels) would remove ~42% of rejections and 9–17% of the tail.
5. **Non-aligned edges.** `etal` is C territory (best `R ≥ 0.42`). It causes
   ~half of llj's rejections and 5–14% of its tail.

## Gates

```
cargo fmt --all --check                                                  → fmt exit=0
cargo clippy --workspace --all-targets -- -D warnings                     → clippy-plain exit=0
cargo clippy --workspace --all-targets \
  --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings → clippy-ext exit=0
cargo test --workspace                                                    → test exit=0: 40 `test result: ok` lines, 1538 passed, 0 failed, 18 ignored
cargo build -p vibegraph-lib --lib   (non-test build, no probe module)    → lib-build exit=0
pixi run --skip-deps kb-lint                                              → kblint exit=0 ("799 concepts, 269 backlog items; 0 errors, 3 warnings"; the warnings are pre-existing backlog items)
```

All cargo commands ran with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on
the committed blobs (`git hash-object` printed at the head of
`gridcoords-M1-gates.log`).

The first clippy run failed on the run blob with `type_complexity` at the
near-edge test closures. The committed alias fixes it.

The `lib-build` line builds the crate without `cfg(test)`, so `edge_probe` is
not compiled into it. Non-test builds carry nothing from this session.

The seed-1 dump the M2 analysis read was written by the run blob, whose
pair-cut names such as `mmll[2,3]` were unquoted. The analysis script
re-joins that field. The committed writer quotes it.

## Corrections to the brief

- **Merge base.** The brief says "main `e3f543f` merged in". The merged main is
  `2ad9720`, through the merge `90b6215`. HEAD `b0a9312` is not itself a merge.
  `git merge-base --is-ancestor 2ad9720 HEAD` holds.
- **Probe location.** The brief puts the probe in `validate_hadronic.rs`. That
  is not possible without new public accessors (see What was done). It is an
  in-crate `#[cfg(test)]` module.
- **The draft's M5 inference.** "The variance×CPU gain tends to 1" is wrong
  (see M5). The gain from an exact window at fixed shape is
  `(1 − r)/(1 − r/Q)`, which is 1.3–1.5 on these rows.
- **The draft's statement about the `pT` edge.** The draft says it "moves with
  `ŝ` and `M`", and that holds. Measured, it sits at 1.03–1.56 × `ptj²` in
  `|t|` on jj and 1.3–5.5 × on llj. It is never fixed.

## Found

- **The MLM tail on a multi-multiplicity row is unmeasured.** The probe's
  per-channel draw goes through one `ProtonIntegrand`.
  `pp_to_ll_0j2j_mlm` is a `MultiplicitySum`. There, `vegas-grid-weight-tail.md`
  saw the outer `(τ, y)` edge bins carry the tail before `K = 6`.
  - Evidence: `rows()` in `edge_probe.rs` covers single integrands only.
  - Closes with: the same probe driven per part through
    `MultiplicitySum::value_in_channel`, with `lab_frame` per part.
- **A1's decision needs a variance replay, not an L1.** The starved channels'
  `τ` marginal differs from the pool at 1.8–2.8× seed noise (M4). Whether that
  costs variance under `q* ∝ √(Σ H_j/N_j)` is not known.
  - Evidence: the M4 table.
  - Closes with: re-weighting each channel's recorded points (the probe already
    holds them) from its own `τ` density to the pooled `q*`, offline, and
    reading the starved terms' variance at fixed `N_j`. No sampler is needed.
- **About half of the llj tail is in the interior, and its location is
  unmeasured.** Within 5% of any edge sits only 24–47% of the top-0.1% second
  moment.
  - Evidence: the M1 tail table.
  - Closes with: `probe_llj_weight_tail_regions` binning, applied to the
    non-edge remainder at the trained grids.
- **The `ptj` window for llj all-timelike channels is two-branched.** In
  channels 0 and 1, `ptj` sits at `±cos θ` (`R = 0.20` against 0.04 in spines).
  A D window there is a union of two intervals, not one interval.
  - Evidence: the M2 table.
  - Closes with: a design note for D's timelike-split case.
- **A dead-code warning in the release-debug test build.**
  `NodeAnalysis::resolve_mom` (`helas/eval/analysis.rs:208`) warns
  "never used" when the lib's tests are built under `--profile release-debug`.
  It is not from this session.
  - Closes with: a `cfg` on it, or removing it.
