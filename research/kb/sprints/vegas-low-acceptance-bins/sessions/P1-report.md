---
type: Session Report
title: "P1 report: acceptance-aware VEGAS grid rules, measured opt-in"
description: "Three opt-in grid rules (MadEvent's acceptance rescale, Σ|w| adaptation, a bin-width bound) behind a default-inert option; on the banked rows the bin-width bound K=4 raises unweighting efficiency 1.2-1.6x and the tail index on every tail row at unchanged σ and variance per CPU, Σ|w| loses everywhere, and the default is unchanged."
status: draft
tags: [vegas, grid-adaptation, weight-tail, unweighting, mlm, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: 1ed69b7, resource: "https://github.com/nsmith-/vibegraph/commit/1ed69b7", title: "vegas: a grid-rule option for refinement, defaulting to today's rule"}
  - {id: 97a5927, resource: "https://github.com/nsmith-/vibegraph/commit/97a5927", title: "vegas: acceptance rescale, |f·w| histograms and a bin-width bound as opt-in grid rules"}
---
Host: the 4-core cloud container, shared with sibling sessions at load 6-10
throughout. Every timing below is a same-sitting ratio on that host; absolute
seconds are not comparable to anything else.

## What landed

Two code commits on `vegas-low-acceptance-bins`:

1. **`1ed69b7`, the plumbing, inert.** `GridRule { weight: BinWeight }` on
   `VegasGrid` (`serde(skip)`, so artifacts keep their bytes). Both histogram
   builders (`run_iter_batched`, `adapt_blocks_iteration`) accumulate through
   one `BinHistogram` in the same point order and arithmetic as before. The rule
   reaches `integrate_channels` through a defaulted
   `ChannelIntegrand::grid_rule()`, set on `FixedBeamIntegrand` and
   `MultiplicitySum` (`set_grid_rule`); the CLI flag is
   `vibegraph integrate --grid-bin-weight squared` (its only value then).
2. **`97a5927`, the rules.** Each opt-in, default off:
   - (a) `--grid-acceptance-rescale`: each bin's histogram entry times
     `min(N_acc / n_acc(bin), 10⁴)`, the iteration's non-zero points over the
     bin's, a bin with none left at zero. MadEvent `dsample.f:2106-2124` (the
     `use_cut = 2` default, `:753`), on whichever quantity (b) selects.
   - (b) `--grid-bin-weight absolute`: accumulate `|f·w|` instead of `(f·w)²`.
     MadEvent `dsample.f:1890` (`grid(1,i,j) + abs(wgt)`).
   - (c) `--grid-max-bin-width K` (`K > 1`): no refined bin wider than
     `K/nbins`. Implemented in `refine_grid` after damping and smoothing, as a
     density floor `ρ·width` on the importance function with `ρ = Σm'/K`
     solved exactly (`bound_bin_width`); a bound `K` caps each coordinate's
     Jacobian factor at `K`. No MadEvent or Lepage source has it: it is the
     backlog item's own candidate, motivated by the 10-40× edge bins note 41
     F-A localised the tail in.
   - Measurement tooling: the frozen `w_max` scan records Hill's tail index of
     each channel's 20 largest weights (`hill_tail_index`,
     `ChannelScan::tail_index`), printed on the existing per-channel debug line
     of `vibegraph generate`.

Unit tests (`vegas.rs`): every rule's histogram equals a hand replay of the same
draws bit for bit; the rescale cap; the bound holds through refinement at
`K = 1.5, 4, 10` and binds (the free grid's widest bin exceeds `4/nbins`); the
bound's fixed point; the block-split and sequential adaptations agree bit for
bit under every rule; and the low-acceptance toy (`u₀ > 0.75` passes only
`|u₁ − ½| < 0.02`, a 4% acceptance), whose widest strip bin is, in units of the
uniform width, **default 7.52, `|f·w|` 8.98, rescale 1.78, `|f·w|` + rescale
2.16, bound 4.00** (`cargo test -p vibegraph-lib --lib vegas:: -- --nocapture`),
every rule integrating to 0.76 within 5σ. `unweight.rs` gains a Hill test on
exact Pareto quantiles (index 1, 2, 5 recovered within `a/√20`).

**Classification: sampling (statistical).** The default rule is bit-for-bit
inert (below); every other rule changes the sampling and gates statistically.

## Inertness of the default

Before = `09cc6d4` (sprint base), built from `git archive` into the shared
target; after = `1ed69b7` and the `97a5927` tree.

| check | command | result |
|---|---|---|
| artifact bytes | `vibegraph integrate <card> --run-card <row card> --fixed-budget --neval 40000 --niter 5 --seed 20261017 -j 4` on `pp_to_llj`, `uux_to_epemg`; `--neval 20000 --niter 4` on `pp_to_llj_mlm` | `grid.bin.zst` byte-identical before / step 1 / step 2 (`cmp`); sha256 `6690801d…`, `28ab904e…`, `a99e89e8…`; σ = 508.468398 ± 3.282975, 0.555443 ± 0.001188, 268.358326 ± 1.706633 pb on all three builds |
| seeded golden, `to_bits` | the `test_pinned_seed_regression_grid_adapt` configuration with an exact-bits print, run on both trees (temporary test, not committed) | before and after both `4605706486304428084 4564401184564159161 4605496727683903400` |
| hermetic, step 1 | `cargo test --workspace --no-fail-fast` | 1478 passed, 16 ignored, 1 failed: the temporary exact-bits test, against the *committed* constants (next line) |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 0 on step 1 and step 2 |
| hermetic, step 2 | `cargo test --workspace --no-fail-fast` | 1485 passed, 0 failed, 16 ignored |
| banked, step 2 | `pixi run --skip-deps validate` (`.cargo/config.toml` `debug = 0`, untracked, removed after) | exit 0: clippy with the extended-validation features clean; 1870 passed, 0 failed, 84 ignored; collator: 203 cells measured (197 ✅, 6 ⚠️, 13 ⏳), "the measured cells are exactly the cells the manifest declares, every gate cell passed" |

The committed golden constants are not this host's bits even at the base:
`std_dev` differs by 11 ulps and `chi2_per_dof` by 811 ulps
(`…159150`/`…902589` committed against `…159161`/`…903400` here, before and
after alike). That is the cross-platform drift the golden's own `1e-12`
tolerance documents; the before/after identity on one host is the inertness
evidence. The first step-1 run also caught the CLI reference chapter going
stale; `docs/src/cli/reference.md` is regenerated in both commits.

## Measurements

**Commands.** Per (row, arm, seed), one run at a time, detached:

```
vibegraph integrate <proc card> --run-card <row card> --fixed-budget \
    --neval N --niter K --seed s -j 4 --no-tui [arm flags]
vibegraph --log-level debug generate grid.bin.zst <proc card> --run-card <row card> \
    --nevents 100 --scan-points 30000 --seed s -j 4 --no-tui
```

Proc card `import model sm / generate <process>`; row cards are
`validation/madgraph/output/<row>/Cards/run_card.dat`, or
`validation/madgraph/<row>_run_card.dat` for the matched rows. Budgets are the
gates': `pp_to_llj_mlm`, `pp_to_llj_xqcut_only`, `pp_to_llj` 150k × 10;
`pp_to_jj` 75k × 10; `uux_to_epemg` 60k × 8; `ee_to_mumua` 80k × 8. Seeds
20261101-20261105 (5) or -20261120 (20). Arms: `base` (no flag), `abs`
(`--grid-bin-weight absolute`), `rescale` (`--grid-acceptance-rescale`), `me`
(both, MadEvent's pair), `bound4` (`--grid-max-bin-width 4`).

**Columns.** σ: unweighted seed mean. spread: the seeds' sample s.d. over σ.
quoted: one seed's mean quoted error over σ. seed χ²/dof: the seeds about
their mean in their own quoted errors. pull: against the MadEvent seed record
(`output/seeded_sigma/<row>/seed_record.json`, the five seeds at its budget,
unweighted mean, error `max(quoted, spread/√n)`), with this side's error
`max(quoted, spread/√n)`. ε_unw: `generate`'s predicted efficiency
`σ / Σ_j w_max_j` at the default `--max-truncation 0.01` over a fixed 30 000-point
scan per channel (± s.e.m. over seeds). Hill: the channels' tail indices from
the same scan (top 20 of the non-zero weights), median and worst channel per
seed, averaged (± s.e.m.); one reading's own spread is `a/√20 ≈ 22%`. CPU:
user + system seconds of the `integrate` process. `quoted²·CPU` and
`spread²·CPU`: relative variance times CPU, as a ratio to `base` (the figure of
merit at fixed precision; the first uses each run's own quoted error and is the
steadier one).

### Tail rows

| row | arm | n | σ (pb) | Δσ vs base | spread | quoted | seed χ²/dof | pull vs MG | ε_unw | Hill median | Hill worst | CPU (s) | quoted²·CPU | spread²·CPU |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `pp_to_llj` | base | 20 | 504.925 | — | 1.90e-3 | 1.87e-3 | 1.01 | +0.67 | 0.0181 ± 0.0017 | 2.21 ± 0.04 | 1.57 ± 0.05 | 93.9 | 1 | 1 |
| | abs | 5 | 503.843 | −0.21% | 4.32e-3 | 3.47e-3 | 2.22 | −0.37 | 0.0030 ± 0.0009 | 1.63 ± 0.03 | 1.25 ± 0.05 | 106.6 | 3.91 | 5.90 |
| | rescale | 20 | 505.428 | +0.10% | 2.38e-3 | 2.33e-3 | 1.05 | +1.21 | 0.0209 ± 0.0015 | 2.56 ± 0.06 | 1.94 ± 0.06 | 49.8 | 0.82 | 0.84 |
| | me | 20 | 505.275 | +0.07% | 3.10e-3 | 2.31e-3 | 1.78 | +1.01 | 0.0134 ± 0.0011 | 2.66 ± 0.06 | 2.02 ± 0.07 | 57.4 | 0.93 | 1.64 |
| | **bound4** | 20 | 504.966 | +0.01% | 1.48e-3 | 1.88e-3 | 0.62 | +0.71 | **0.0255 ± 0.0027** | 2.43 ± 0.09 | 1.78 ± 0.07 | 85.2 | 0.92 | 0.55 |
| | MG record | 5 | 504.322 | | 3.61e-3 | | 0.90 | | | | | | | |
| `pp_to_llj_mlm` | base | 20 | 268.550 | — | 0.97e-3 | 1.40e-3 | 0.47 | +0.69 | 0.0364 ± 0.0034 | 2.23 ± 0.06 | 1.72 ± 0.04 | 201.1 | 1 | 1 |
| | abs | 5 | 268.598 | +0.02% | 2.80e-3 | 2.41e-3 | 1.32 | +0.68 | 0.0052 ± 0.0011 | 1.74 ± 0.07 | 1.20 ± 0.06 | 209.9 | 3.11 | 8.70 |
| | rescale | 20 | 268.630 | +0.03% | 2.00e-3 | 1.73e-3 | 1.34 | +0.82 | 0.0412 ± 0.0021 | 2.78 ± 0.08 | 1.73 ± 0.03 | 117.7 | 0.90 | 2.48 |
| | me | 5 | 268.425 | −0.05% | 1.71e-3 | 1.72e-3 | 1.03 | +0.46 | 0.0308 ± 0.0026 | 2.67 ± 0.16 | 1.90 ± 0.14 | 129.6 | 0.98 | 2.01 |
| | **bound4** | 20 | 268.601 | +0.02% | 1.36e-3 | 1.43e-3 | 0.91 | +0.78 | **0.0590 ± 0.0017** | 2.93 ± 0.08 | 2.21 ± 0.06 | 174.2 | 0.91 | 1.70 |
| | MG record | 5 | 268.136 | | 4.19e-3 | | 1.08 | | | | | | | |
| `pp_to_llj_xqcut_only` | base | 5 | 212.854 | — | 1.03e-3 | 1.39e-3 | 0.52 | +1.19 | 0.0429 ± 0.0069 | 2.26 ± 0.11 | 1.74 ± 0.12 | 115.1 | 1 | 1 |
| | abs | 5 | 212.698 | −0.07% | 1.90e-3 | 2.10e-3 | 0.76 | +0.81 | 0.0053 ± 0.0014 | 1.73 ± 0.08 | 1.22 ± 0.07 | 128.2 | 2.54 | 3.80 |
| | rescale | 5 | 212.849 | −0.00% | 1.46e-3 | 1.66e-3 | 0.75 | +1.16 | 0.0476 ± 0.0014 | 3.02 ± 0.15 | 1.90 ± 0.11 | 73.6 | 0.91 | 1.28 |
| | me | 5 | 212.936 | +0.04% | 2.06e-3 | 1.65e-3 | 1.63 | +1.30 | 0.0342 ± 0.0027 | 2.93 ± 0.21 | 2.06 ± 0.18 | 82.8 | 1.02 | 2.88 |
| | **bound4** | 5 | 212.918 | +0.03% | 0.92e-3 | 1.37e-3 | 0.45 | +1.33 | **0.0627 ± 0.0038** | 2.93 ± 0.08 | 2.19 ± 0.07 | 101.3 | 0.85 | 0.70 |
| | MG record | 5 | 212.304 | | 2.14e-3 | | 0.24 | | | | | | | |
| `pp_to_jj` | base | 20 | 6.79805e8 | — | 1.64e-3 | 1.34e-3 | 1.49 | −0.50 | 0.1048 ± 0.0027 | 2.76 ± 0.09 | 2.33 ± 0.08 | 68.3 | 1 | 1 |
| | abs | 5 | 6.79278e8 | −0.08% | 1.76e-3 | 1.71e-3 | 1.06 | −1.10 | 0.0438 ± 0.0046 | 2.21 ± 0.16 | 1.83 ± 0.06 | 67.6 | 1.60 | 1.14 |
| | rescale | 20 | 6.79946e8 | +0.02% | 1.34e-3 | 1.23e-3 | 1.21 | −0.26 | 0.1381 ± 0.0056 | 3.21 ± 0.13 | 2.44 ± 0.09 | 64.2 | 0.78 | 0.63 |
| | me | 5 | 6.79485e8 | −0.05% | 1.59e-3 | 1.43e-3 | 1.31 | −0.86 | 0.0698 ± 0.0109 | 2.50 ± 0.13 | 1.93 ± 0.13 | 64.3 | 1.06 | 0.89 |
| | **bound4** | 20 | 6.79673e8 | −0.02% | 1.51e-3 | 1.41e-3 | 1.15 | −0.75 | **0.1238 ± 0.0012** | 3.39 ± 0.10 | 2.78 ± 0.06 | 68.6 | 1.11 | 0.85 |
| | MG record | 5 | 6.80086e8 | | 1.60e-3 | | 0.91 | | | | | | | |

### Fixed-beam controls (5 seeds)

| row | arm | σ (pb) | Δσ vs base | spread | seed χ²/dof | pull vs MG | ε_unw | Hill median | Hill worst | quoted²·CPU |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `uux_to_epemg` | base | 0.556604 | — | 1.11e-3 | 0.65 | +2.75 | 0.1541 ± 0.0037 | 3.72 | 2.79 | 1 |
| | abs | 0.556751 | +0.03% | 0.91e-3 | 0.30 | +2.79 | 0.0859 ± 0.0043 | 2.94 | 2.34 | 1.43 |
| | rescale | 0.556490 | −0.02% | 0.68e-3 | 0.25 | +2.56 | 0.1809 ± 0.0020 | 4.16 | 3.11 | 0.92 |
| | me | 0.556575 | −0.01% | 1.20e-3 | 0.62 | +2.59 | 0.1183 ± 0.0023 | 3.80 | 2.80 | 1.20 |
| | bound4 | 0.556523 | −0.01% | 1.34e-3 | 0.93 | +2.60 | 0.1612 ± 0.0026 | 4.07 | 3.33 | 0.99 |
| `ee_to_mumua` | base | 0.100845 | — | 1.64e-3 | 0.91 | +0.88 | 0.0578 ± 0.0148 | 2.61 | 1.51 | 1 |
| | abs | 0.100723 | −0.12% | 1.46e-3 | 0.35 | −0.45 | 0.0125 ± 0.0025 | 1.73 | 1.38 | 2.05 |
| | rescale | 0.100875 | +0.03% | 0.77e-3 | 0.22 | +1.30 | 0.0820 ± 0.0192 | 3.10 | 1.97 | 0.87 |
| | me | 0.100822 | −0.02% | 0.60e-3 | 0.10 | +0.55 | 0.0583 ± 0.0084 | 2.76 | 2.17 | 1.10 |
| | bound4 | 0.100843 | −0.00% | 1.28e-3 | 0.64 | +0.93 | 0.1035 ± 0.0018 | 3.19 | 2.37 | 0.78 |

`uux_to_epemg` reads +0.28% (pull +2.6 to +2.8) in every arm, the base
included: it is not a rule effect, and it is the size the manifest records for
its down-type partner on this side (`ddx_to_epemg`, +0.43%).

### Evaluations to a target (`pp_to_llj`, 5 seeds)

`vibegraph integrate … --target-rel 0.002 --neval 150000 --seed s -j 4`
(Neyman allocation, the mode's default), every run `met`:

| arm | evaluations (M) | CPU (s) | σ (pb) |
|---|--:|--:|--:|
| base | 1.50 ± 0.13 | 98 ± 11 | 504.28 ± 0.39 |
| rescale | 2.85 ± 0.14 | 87 ± 5 | 504.82 ± 0.17 |
| bound4 | 1.65 ± 0.16 | 96 ± 10 | 504.48 ± 0.42 |

## Reading

- **σ moves under no rule.** The largest arm-to-arm difference is
  `rescale − base` on `pp_to_llj`, +0.50 ± 0.34 pb at 20 seeds (1.5σ); every
  other is within 1.2σ, and every arm's pull against the MadEvent record is
  inside ±1.4 on the tail rows.
- **`|f·w|` alone loses on every row**: ε_unw falls 1.8-8×, the tail index falls
  (`pp_to_llj` median 2.21 → 1.63), and variance per CPU rises 1.4-3.9×. The
  toy test pins why: a wide bin's few accepted points carry large weights,
  which `(f·w)²` credits more than `|f·w|` does, so `|f·w|` starves the
  low-acceptance bins harder (strip bin 8.98× against 7.52×). These are large
  differences, read at 5 seeds.
- **MadEvent's pair** (`me`) inherits that: ε_unw below the base on every tail
  row (`pp_to_llj` 0.0134 against 0.0181 at 20 seeds). The pair is not
  MadEvent's whole rule — MadEvent damps relative to the sum, after
  smoothing, at exponent 1.5 (`dsample.f:2157-2174`, `average_grid`), and this
  crate's `refine_grid` damps relative to the average, before smoothing, at
  0.5 on mapped integrands — so the result says Σ|w| does not suit *this*
  refinement, not that MadEvent's grid is wrong.
- **The acceptance rescale** buys CPU, not efficiency: it widens the grids
  into rejected regions (scan acceptance `pp_to_llj` 0.45 → 0.19,
  `pp_to_llj_mlm` 0.65 → 0.35), and rejected points cost almost nothing under
  the cut-first evaluation, so a run costs 0.53-0.94× the CPU while its quoted
  error grows. Net variance per CPU 0.78-0.92; ε_unw +11-32%; the tail
  index's median rises but the worst channel does not on `pp_to_llj_mlm`
  (1.72 → 1.73). It takes 1.9× the evaluations to a 0.2% target for the same
  CPU. On the current tree it moves `pp_to_llj_mlm`'s median Hill index
  2.23 → 2.78; note 41 F-A's probe read 1.8 → 2.2 on a tree before channel
  merging, so the base itself has moved.
- **The bin-width bound `K = 4` is the one clear gain.** At 20 seeds ε_unw
  rises ×1.41 on `pp_to_llj` (2.3σ), ×1.62 on `pp_to_llj_mlm` (5.9σ), ×1.18
  on `pp_to_jj` (6.4σ), and ×1.46 on `pp_to_llj_xqcut_only` at 5 seeds; the
  tail index rises on every channel of `pp_to_llj`, `pp_to_llj_mlm` and
  `pp_to_jj` (per-channel means over the first five seeds; `pp_to_llj`'s
  worst channel 1.94 → 2.73, every channel ≥ 2.7); variance per CPU is
  0.85-1.11 of the base; σ and the evaluations to 0.2% are unmoved. On `pp_to_llj_mlm` it also brings the
  seeds' χ²/dof from 0.47 (the base's seeds scatter less than they quote,
  the under-dispersion a heavy tail gives) to 0.91.

## Proposal (for the manager and the user; the default is unchanged)

Adopt `--grid-max-bin-width` as a default only after two more measurements,
since a default change moves every integration and every σ calibration and so
sequences before the calibration re-record (PR #45):

1. **The `K` ladder.** Only `K = 4` was measured. `K = 2, 3, 6, 10` on
   `pp_to_llj`, `pp_to_llj_mlm` and `pp_to_jj` at 20 seeds each picks the
   value; the bound costs nothing in variance at 4, so a tighter one may buy
   more efficiency before it costs variance.
2. **The banked layer under the chosen `K`.** Every σ gate and seed sweep run
   with the bound on (a library-side default flip on a branch), read against
   the measured seed spreads, before the flip lands.

No new MadEvent reference data is needed for any of this: every comparison
is against the existing seed records. **Banking request: none.**

## Deviations from the brief

- Step 1 was committed with `BinWeight::Squared` as the option's only value,
  then the three rules on top: the brief's "plumbing first, provably inert"
  as its own commit.
- The rule is set on `FixedBeamIntegrand` and `MultiplicitySum` (the two
  integrands the CLI integrates through). `ProtonIntegrand`'s own
  `adapt_grids_budget`, which tests drive directly, takes the trait default.
- The rescale is applied to whichever quantity `--grid-bin-weight` selects
  (power 1, as MadEvent applies it to `grid(1,·,·)`). The cap is MadEvent's
  `10⁴` on `N_acc/n_acc`, which with 64 bins (MadEvent has 96) caps the
  relative inverse acceptance at about 156.
- "Evaluations to target" was read on `pp_to_llj` only, at 0.2% and five
  seeds; `pp_to_llj_mlm` at a target was not run.

## Corrections to the brief

- The sprint table makes this session depend on `vegas-constant-integrand`
  merged; it is not (PR #50 open). Nothing here needed it. Overlap with it is
  one trait (`ChannelIntegrand`: it adds `is_constant` after
  `scale_draw_ndim`, this adds `grid_rule` after `value_in_channel`) and
  `integrate_channels` (it edits the warm-up block, this the grid
  construction): adjacent, not overlapping.
- "`to_bits` equality of the seeded goldens" cannot mean against the
  committed constants: they are not this host's bits at the base either
  (above). The goldens are tolerance-gated by design; the evidence is
  before/after identity on one host.
- The F-A Hill numbers the brief and the backlog item quote (median 1.8 → 2.2)
  were taken before channel merging; the base now reads 2.23 on the same row,
  budget and scan.

## Found

- **The bin-width bound's `K` is untuned.** Only `K = 4` measured (the table
  above). Close: the `K` ladder in the proposal.
- **`refine_grid`'s damping is neither Lepage's nor MadEvent's.** It forms
  `r = m/avg` and damps before the nearest-neighbour smoothing; Lepage and
  MadEvent (`dsample.f:2157-2174`) form `xo = m/Σm`, smooth first
  (`average_grid`, `dsample.f:2500`), then damp with `((xo−1)/ln xo)^1.5`,
  which compresses the importance function logarithmically. Σ|w| losing here
  may be this refinement's property rather than the histogram's. Close: a
  `GridRule` damping option and the same five-arm measurement.
- **The rescale combined with the bound is unmeasured.** The rescale's CPU
  gain and the bound's efficiency gain act on different things (acceptance
  against width). Close: one more arm in the `K` ladder.
- **The grid rule is not recorded in the artifact.** An artifact integrated
  under a non-default rule cannot say so (the rule is `serde(skip)` to keep
  banked bytes). Harmless while the option is diagnostic; needed if a rule
  becomes a default or a banked run uses one. Close: an artifact metadata
  field, with a format-version decision.
- **A heavy tail shows as under-dispersed seeds.** `pp_to_llj_mlm`'s base
  seeds scatter at χ²/dof 0.47 over 20 seeds (spread 0.97e-3 against 1.40e-3
  quoted), and the 5-seed `pp_to_llj` base read 0.43 where 20 seeds read 1.01:
  the quoted error is not the seed spread on these rows, which bears on every
  5-seed gate of them. Close: the 20-seed spreads above beside the gates'
  calibrations in the re-record.
