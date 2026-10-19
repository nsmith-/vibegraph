---
type: Session Report
title: "P2 report: a K ladder for the bin-width bound"
description: "The bin-width bound measured at K = 2, 3, 6, 10 beside P1's K = 4 over 20 seeds on pp_to_llj, pp_to_llj_mlm and pp_to_jj; K = 6 keeps K = 4's efficiency and tail-index gains at no variance cost per evaluation, and is proposed as the default. No code changed; the default is unchanged."
status: draft
tags: [vegas, grid-adaptation, weight-tail, unweighting, mlm, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: 97a5927, resource: "https://github.com/nsmith-/vibegraph/commit/97a5927", title: "vegas: acceptance rescale, |f·w| histograms and a bin-width bound as opt-in grid rules (the binary measured)"}
  - {id: 5316a3b, resource: "https://github.com/nsmith-/vibegraph/commit/5316a3b", title: "kb: vegas-low-acceptance-bins P2 brief (K ladder), log (the tree this session ran on)"}
---
Host: the 4-core cloud container, shared with one sibling session running the
banked layer; load 3-11 over the session. Worktree
`/home/user/wt/vegas-low-acceptance-bins`, branch `vegas-low-acceptance-bins`,
verified before any action. **Classification: sampling (statistical).** Every
arm is an opt-in grid rule; no code changed and the default is untouched.

## What was run

**Binary.** P1's `vibegraph-step2` (built from `97a5927`, sha256
`fe07749d…59181c6a`), the binary P1's recorded arms came from, so the new rungs
and the reused ones differ only in the flag. The tree has moved since only in
tests, a private constant and a log statement's formatting
(`git diff --stat 97a5927 HEAD -- vibegraph-lib/src vibegraph-cli/src`:
`unweight.rs`, `vegas.rs`; the non-test hunks are `pub const` → `const` and
the merged `tracing::debug!` with identical text).

**Driver.** P1's `drive.py`, copied to the scratchpad's `vegasacc-P2/drive.py`
with arms `boundK` (`--grid-max-bin-width K`), `rescale_boundK`
(`--grid-acceptance-rescale --grid-max-bin-width K`) and `base_ctl` (no flag)
added; per (row, arm, seed), one run at a time, both campaigns detached:

```
vibegraph integrate <proc card> --run-card <row card> --fixed-budget \
    --neval N --niter 10 --seed s -j 4 --no-tui [arm flags]
vibegraph --log-level debug generate grid.bin.zst <proc card> --run-card <row card> \
    --nevents 100 --scan-points 30000 --seed s -j 4 --no-tui
```

Budgets are the gates' (`pp_to_llj`, `pp_to_llj_mlm` 150k × 10; `pp_to_jj`
75k × 10), seeds 20261101-20. Campaign 1 (`vegasacc-P2/campaign1.sh`, 255
runs, 15:47-20:04 UTC, 4 h 17 min): `bound2, bound3, bound6, bound10` × 20 seeds × 3 rows,
plus `base_ctl` × 5 seeds × 3 rows. Campaign 2 (`campaign2.sh`, 85 runs, 20:05-20:57 UTC,
52 min): `rescale_bound6` × 20 seeds × 3 rows, then evaluations to target
(below). No run failed (`grep "exit [1-9]"` over every run log prints nothing).
`base`, `bound4` and `rescale` are P1's records (`vegasacc-P1/c1/results.jsonl`),
not re-run.

**Reuse check.** `base_ctl` re-runs P1's base on seeds 20261101-05: σ and its
error are identical to P1's records on all 15 (row, seed) pairs, so the
reused arms are this binary's output on these seeds. The same runs measure
the CPU drift between sittings: this sitting / P1's = **1.061** (`pp_to_llj`),
**1.049** (`pp_to_llj_mlm`), **1.247** (`pp_to_jj`). Every CPU column that
mixes P1's arms with this session's carries that drift; the CPU-free column
is `quoted²` (relative quoted variance at the fixed evaluation budget).

**Summary.** `python3 vegasacc-P2/ladder.py` (P1's `combine`/`mg_reference`
plus rung-to-rung reads; output `vegasacc-P2-ladder2.md`). Columns as P1's:
σ the unweighted seed mean; spread the seeds' s.d. over σ; quoted one seed's
mean quoted error over σ; seed χ²/dof the seeds about their mean in their own
errors; pull against the MadEvent seed record with `max(quoted, spread/√n)`
on both sides; ε_unw `generate`'s predicted efficiency (30 000-point scan,
± s.e.m.); Hill the channels' tail indices (median and worst channel per seed,
averaged ± s.e.m.); CPU the `integrate` process's user + system seconds;
`quoted²·CPU`, `spread²·CPU` as ratios to `base`.

Scratchpad: `/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/vegasacc-P2/`
(`c2/results.jsonl`, `t2/results.jsonl`, per-run `log.txt` and `grid.bin.zst`).

## The ladder (20 seeds per arm)

### `pp_to_llj`

| arm | σ (pb) | Δσ vs base | spread | quoted | seed χ²/dof | pull vs MG | ε_unw | Hill median | Hill worst | CPU (s) | quoted²·CPU | spread²·CPU |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| base (P1) | 504.925 | — | 1.90e-3 | 1.87e-3 | 1.01 | +0.67 | 0.0181 ± 0.0017 | 2.21 ± 0.04 | 1.57 ± 0.05 | 93.9 | 1 | 1 |
| K = 2 | 504.824 | −0.020% | 2.27e-3 | 2.05e-3 | 1.22 | +0.55 | **0.0347 ± 0.0016** | 2.45 ± 0.06 | 1.81 ± 0.05 | 81.4 | 1.05 | 1.24 |
| K = 3 | 504.752 | −0.034% | 1.71e-3 | 1.92e-3 | 0.78 | +0.48 | 0.0282 ± 0.0023 | 2.53 ± 0.06 | 1.76 ± 0.06 | 90.0 | 1.02 | 0.78 |
| K = 4 (P1) | 504.966 | +0.008% | 1.48e-3 | 1.88e-3 | 0.62 | +0.71 | 0.0255 ± 0.0027 | 2.43 ± 0.09 | 1.78 ± 0.07 | 85.2 | 0.92 | 0.55 |
| K = 6 | 504.951 | +0.005% | 2.01e-3 | 1.83e-3 | 1.19 | +0.70 | 0.0250 ± 0.0025 | 2.40 ± 0.07 | 1.81 ± 0.06 | 97.3 | 1.00 | 1.16 |
| K = 10 | 504.777 | −0.029% | 1.90e-3 | 1.79e-3 | 1.11 | +0.50 | 0.0216 ± 0.0024 | 2.29 ± 0.07 | 1.68 ± 0.06 | 99.9 | 0.98 | 1.07 |
| rescale (P1) | 505.428 | +0.100% | 2.38e-3 | 2.33e-3 | 1.05 | +1.21 | 0.0209 ± 0.0015 | 2.56 ± 0.06 | 1.94 ± 0.06 | 49.8 | 0.82 | 0.84 |
| rescale + K = 6 | 505.405 | +0.095% | 1.94e-3 | 2.32e-3 | 0.71 | +1.18 | 0.0296 ± 0.0014 | **3.05 ± 0.06** | **2.30 ± 0.04** | 49.9 | 0.82 | 0.55 |
| MG record (5) | 504.322 | | 3.61e-3 | | 0.90 | | | | | | | |

### `pp_to_llj_mlm`

| arm | σ (pb) | Δσ vs base | spread | quoted | seed χ²/dof | pull vs MG | ε_unw | Hill median | Hill worst | CPU (s) | quoted²·CPU | spread²·CPU |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| base (P1) | 268.550 | — | 0.97e-3 | 1.40e-3 | 0.47 | +0.69 | 0.0364 ± 0.0034 | 2.23 ± 0.06 | 1.72 ± 0.04 | 201.1 | 1 | 1 |
| K = 2 | 268.471 | −0.029% | 1.73e-3 | 1.65e-3 | 1.10 | +0.56 | **0.0790 ± 0.0017** | **3.60 ± 0.07** | 2.46 ± 0.08 | 139.5 | 0.97 | 2.20 |
| K = 3 | 268.561 | +0.004% | 1.66e-3 | 1.49e-3 | 1.24 | +0.71 | 0.0610 ± 0.0016 | 3.08 ± 0.06 | 2.47 ± 0.08 | 157.4 | 0.90 | 2.31 |
| K = 4 (P1) | 268.601 | +0.019% | 1.36e-3 | 1.43e-3 | 0.91 | +0.78 | 0.0590 ± 0.0017 | 2.93 ± 0.08 | 2.21 ± 0.06 | 174.2 | 0.91 | 1.70 |
| K = 6 | 268.574 | +0.009% | 1.07e-3 | 1.35e-3 | 0.63 | +0.73 | 0.0580 ± 0.0019 | 2.81 ± 0.06 | 2.20 ± 0.07 | 174.9 | 0.81 | 1.06 |
| K = 10 | 268.599 | +0.018% | 1.07e-3 | 1.33e-3 | 0.66 | +0.77 | 0.0594 ± 0.0022 | 2.60 ± 0.05 | 1.93 ± 0.05 | 184.4 | 0.84 | 1.13 |
| rescale (P1) | 268.630 | +0.030% | 2.00e-3 | 1.73e-3 | 1.34 | +0.82 | 0.0412 ± 0.0021 | 2.78 ± 0.08 | 1.73 ± 0.03 | 117.7 | 0.90 | 2.48 |
| rescale + K = 6 | 268.641 | +0.034% | 1.15e-3 | 1.76e-3 | 0.43 | +0.84 | 0.0493 ± 0.0008 | 3.29 ± 0.08 | **2.50 ± 0.10** | 111.0 | 0.88 | 0.78 |
| MG record (5) | 268.136 | | 4.19e-3 | | 1.08 | | | | | | | |

### `pp_to_jj`

| arm | σ (pb) | Δσ vs base | spread | quoted | seed χ²/dof | pull vs MG | ε_unw | Hill median | Hill worst | CPU (s) | quoted²·CPU | spread²·CPU |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| base (P1) | 6.79805e8 | — | 1.64e-3 | 1.34e-3 | 1.49 | −0.50 | 0.1048 ± 0.0027 | 2.76 ± 0.09 | 2.33 ± 0.08 | 68.3 | 1 | 1 |
| K = 2 | 6.79879e8 | +0.011% | 1.90e-3 | 1.67e-3 | 1.30 | −0.36 | 0.1118 ± 0.0008 | **4.61 ± 0.16** | **3.42 ± 0.15** | 71.6 | 1.62 | 1.41 |
| K = 3 | 6.79689e8 | −0.017% | 1.76e-3 | 1.49e-3 | 1.40 | −0.70 | 0.1188 ± 0.0009 | 3.34 ± 0.12 | 2.88 ± 0.10 | 72.8 | 1.32 | 1.24 |
| K = 4 (P1) | 6.79673e8 | −0.019% | 1.51e-3 | 1.41e-3 | 1.15 | −0.75 | 0.1238 ± 0.0012 | 3.39 ± 0.10 | 2.78 ± 0.06 | 68.6 | 1.11 | 0.85 |
| K = 6 | 6.79899e8 | +0.014% | 1.46e-3 | 1.34e-3 | 1.20 | −0.34 | 0.1280 ± 0.0012 | 3.31 ± 0.10 | 2.85 ± 0.10 | 74.0 | 1.08 | 0.86 |
| K = 10 | 6.79921e8 | +0.017% | 1.38e-3 | 1.32e-3 | 1.09 | −0.30 | 0.1246 ± 0.0012 | 3.23 ± 0.14 | 2.75 ± 0.12 | 75.2 | 1.07 | 0.78 |
| rescale (P1) | 6.79946e8 | +0.021% | 1.34e-3 | 1.22e-3 | 1.21 | −0.26 | 0.1381 ± 0.0056 | 3.21 ± 0.13 | 2.44 ± 0.09 | 64.2 | 0.78 | 0.63 |
| rescale + K = 6 | 6.79828e8 | +0.003% | 1.11e-3 | 1.25e-3 | 0.78 | −0.48 | **0.1809 ± 0.0013** | 3.33 ± 0.11 | 2.32 ± 0.08 | 62.2 | 0.80 | 0.41 |
| MG record (5) | 6.80086e8 | | 1.59e-3 | | 0.91 | | | | | | | |

On `pp_to_jj` the P2 arms' CPU is inflated by the sitting drift (1.247 on the
base's own seeds), so its `quoted²·CPU` column against P1's base overstates
their cost by up to that factor; read `quoted²` below instead.

## Rung against rung

Every Δ is over `√(s₁²/20 + s₂²/20)` from the two arms' 20 per-seed values
(never the quoted errors). `quoted²` is the ratio of mean per-seed relative
quoted variance at the fixed budget (variance per evaluation; its z is large
because a seed's quoted error scatters little, and it inherits the quoted
error's tail bias). `spread²` is the ratio of the seeds' relative variances
with its two-sided F(19, 19) p-value; at 20 seeds the F test cannot resolve
a ratio inside about [0.40, 2.53].

| row | pair | σ Δ (z) | ε_unw Δ (z) | Hill median Δ (z) | Hill worst Δ (z) | quoted² ratio (z) | spread² ratio (p) |
|---|---|--:|--:|--:|--:|--:|--:|
| `pp_to_llj` | base → 2 | −0.020% (−0.3) | +0.0166 (+7.0) | +0.24 (+3.4) | +0.23 (+3.5) | 1.21 (+7.8) | 1.43 (0.44) |
| | 2 → 3 | −0.014% (−0.2) | −0.0065 (−2.3) | +0.08 (+1.0) | −0.04 (−0.6) | 0.88 (−13.5) | 0.57 (0.23) |
| | 3 → 4 | +0.042% (+0.8) | −0.0027 (−0.8) | −0.10 (−0.9) | +0.02 (+0.2) | 0.96 (−2.1) | 0.75 (0.54) |
| | 4 → 6 | −0.003% (−0.1) | −0.0004 (−0.1) | −0.03 (−0.2) | +0.02 (+0.3) | 0.95 (−2.1) | 1.84 (0.20) |
| | 6 → 10 | −0.034% (−0.6) | −0.0034 (−1.0) | −0.11 (−1.1) | −0.13 (−1.4) | 0.96 (−1.7) | 0.90 (0.82) |
| | base → 3 | −0.034% (−0.6) | +0.0101 (+3.5) | +0.32 (+4.8) | +0.19 (+2.6) | 1.06 (+2.1) | 0.81 (0.66) |
| | base → 4 | +0.008% (+0.2) | +0.0074 (+2.3) | +0.22 (+2.2) | +0.21 (+2.3) | 1.02 (+0.5) | 0.61 (0.29) |
| | base → 6 | +0.005% (+0.1) | +0.0069 (+2.3) | +0.20 (+2.4) | +0.23 (+2.9) | 0.96 (−1.2) | 1.12 (0.81) |
| | base → 10 | −0.029% (−0.5) | +0.0035 (+1.2) | +0.09 (+1.1) | +0.11 (+1.3) | 0.92 (−2.5) | 1.01 (0.99) |
| | 6 → rescale + 6 | +0.090% (+1.4) | +0.0045 (+1.6) | +0.64 (+6.8) | +0.50 (+6.5) | 1.60 (+27.7) | 0.93 (0.88) |
| | rescale → rescale + 6 | −0.005% (−0.1) | +0.0087 (+4.2) | +0.48 (+5.4) | +0.36 (+4.6) | 1.00 (−0.2) | 0.66 (0.38) |
| `pp_to_llj_mlm` | base → 2 | −0.029% (−0.7) | +0.0425 (+11.2) | +1.37 (+15.1) | +0.74 (+8.6) | 1.39 (+29.3) | 3.17 (0.016) |
| | 2 → 3 | +0.034% (+0.6) | −0.0180 (−7.6) | −0.51 (−5.4) | +0.00 (+0.0) | 0.82 (−47.7) | 0.93 (0.88) |
| | 3 → 4 | +0.015% (+0.3) | −0.0020 (−0.8) | −0.15 (−1.5) | −0.26 (−2.6) | 0.91 (−11.9) | 0.67 (0.39) |
| | 4 → 6 | −0.010% (−0.3) | −0.0010 (−0.4) | −0.12 (−1.2) | −0.01 (−0.2) | 0.89 (−11.3) | 0.62 (0.31) |
| | 6 → 10 | +0.010% (+0.3) | +0.0014 (+0.5) | −0.21 (−2.7) | −0.26 (−3.1) | 0.98 (−1.9) | 1.01 (0.98) |
| | base → 3 | +0.004% (+0.1) | +0.0246 (+6.5) | +0.86 (+9.8) | +0.75 (+8.5) | 1.15 (+11.1) | 2.95 (0.023) |
| | base → 4 | +0.019% (+0.5) | +0.0225 (+5.9) | +0.70 (+7.1) | +0.49 (+6.6) | 1.05 (+3.1) | 1.97 (0.15) |
| | base → 6 | +0.009% (+0.3) | +0.0215 (+5.5) | +0.58 (+7.0) | +0.48 (+6.0) | 0.93 (−4.8) | 1.22 (0.67) |
| | base → 10 | +0.018% (+0.6) | +0.0229 (+5.7) | +0.37 (+4.9) | +0.21 (+3.1) | 0.91 (−5.8) | 1.23 (0.66) |
| | 6 → rescale + 6 | +0.025% (+0.7) | −0.0087 (−4.1) | +0.48 (+4.9) | +0.30 (+2.5) | 1.71 (+68.7) | 1.17 (0.74) |
| | rescale → rescale + 6 | +0.004% (+0.1) | +0.0081 (+3.6) | +0.51 (+4.5) | +0.77 (+7.4) | 1.04 (+3.8) | 0.33 (0.021) |
| `pp_to_jj` | base → 2 | +0.011% (+0.2) | +0.0070 (+2.5) | +1.84 (+9.9) | +1.10 (+6.7) | 1.55 (+79.9) | 1.35 (0.52) |
| | 2 → 3 | −0.028% (−0.5) | +0.0070 (+6.1) | −1.27 (−6.3) | −0.54 (−3.1) | 0.80 (−132) | 0.86 (0.75) |
| | 3 → 4 | −0.002% (−0.0) | +0.0050 (+3.5) | +0.05 (+0.3) | −0.10 (−0.9) | 0.89 (−57.2) | 0.73 (0.50) |
| | 4 → 6 | +0.033% (+0.7) | +0.0043 (+2.6) | −0.08 (−0.6) | +0.07 (+0.6) | 0.90 (−46.3) | 0.94 (0.90) |
| | 6 → 10 | +0.003% (+0.1) | −0.0035 (−2.1) | −0.08 (−0.5) | −0.11 (−0.7) | 0.97 (−6.2) | 0.88 (0.79) |
| | base → 3 | −0.017% (−0.3) | +0.0139 (+4.9) | +0.58 (+3.7) | +0.56 (+4.5) | 1.24 (+34.4) | 1.16 (0.75) |
| | base → 4 | −0.019% (−0.4) | +0.0190 (+6.4) | +0.63 (+4.5) | +0.46 (+4.6) | 1.10 (+14.7) | 0.84 (0.72) |
| | base → 6 | +0.014% (+0.3) | +0.0232 (+7.9) | +0.54 (+4.0) | +0.53 (+4.1) | 1.00 (−0.7) | 0.80 (0.63) |
| | base → 10 | +0.017% (+0.4) | +0.0198 (+6.7) | +0.46 (+2.7) | +0.42 (+3.0) | 0.97 (−4.0) | 0.70 (0.45) |
| | 6 → rescale + 6 | −0.010% (−0.3) | +0.0528 (+30.9) | +0.02 (+0.2) | −0.53 (−4.1) | 0.88 (−63.1) | 0.57 (0.23) |
| | rescale → rescale + 6 | −0.017% (−0.4) | +0.0427 (+7.5) | +0.12 (+0.7) | −0.12 (−1.0) | 1.05 (+13.1) | 0.68 (0.41) |

### Evaluations to a target (`pp_to_llj`, 0.2 %, 5 seeds)

`python3 drive.py … --rows pp_to_llj --arms bound6,bound2,bound3,bound10,rescale_bound6 --seeds 20261101-05 --target 0.002 --no-scan`
(`integrate --target-rel 0.002 --neval 150000`), every run `met`; base, K = 4
and rescale are P1's (`vegasacc-P1/t1`):

| arm | evaluations (M) | CPU (s) | σ (pb) |
|---|--:|--:|--:|
| base (P1) | 1.50 ± 0.13 | 98 ± 11 | 504.28 ± 0.39 |
| K = 2 | 1.86 ± 0.14 | 88 ± 6 | 504.89 ± 0.54 |
| K = 3 | 1.77 ± 0.16 | 93 ± 8 | 504.79 ± 0.39 |
| K = 4 (P1) | 1.65 ± 0.16 | 96 ± 10 | 504.48 ± 0.42 |
| K = 6 | 1.53 ± 0.15 | 89 ± 8 | 504.29 ± 0.40 |
| K = 10 | 1.65 ± 0.12 | 98 ± 7 | 504.58 ± 0.33 |
| rescale (P1) | 2.85 ± 0.14 | 87 ± 5 | 504.82 ± 0.17 |
| rescale + K = 6 | 2.70 ± 0.16 | 79 ± 4 | 504.64 ± 0.27 |

The stop is quantised at one 150k iteration and five seeds resolve about
±10 %; CPU to target is flat across every bound within that (and within
the 5-25 % sitting drift, since base, K = 4 and rescale are P1's), the same
as P1 found at K = 4.

## Reading

- **σ moves under no K.** The largest Δσ between adjacent rungs or against
  the base is 0.8 measured-spread σ (`pp_to_llj` K = 3 → 4); every rung's
  pull against the MadEvent record is inside ±0.8. No rung is a σ finding.
  The rescale family reads +0.10% (rescale) and +0.095% (rescale + 6) on
  `pp_to_llj`, 1.5σ and 1.4σ from base and K = 6 respectively, pulls +1.2
  against MadEvent: below the brief's 2σ bar, and on the same seeds, so not
  two independent readings.
- **Tighter bounds buy efficiency on the Drell-Yan + jet rows and cost
  variance everywhere.** ε_unw at K = 2 is ×1.92 the base on `pp_to_llj` and
  ×2.17 on `pp_to_llj_mlm` (against ×1.38 and ×1.59 at K = 6), but the quoted
  variance per evaluation rises by 21 %, 39 % and 55 % on the three rows; the
  `pp_to_llj_mlm` spread at K = 2 and K = 3 is ×3.2 and ×3.0 the base's
  (F p = 0.016, 0.023; part of that is the base's own under-dispersion,
  χ²/dof 0.47). On `pp_to_jj` K = 2 is the *worst* bound for ε_unw (0.112,
  against 0.128 at K = 6) while its tail index is the best of any arm
  (median 4.61): a bound that tight flattens the grid toward uniform, which
  thins the tail and lowers the peak-to-mean ratio less than it costs.
- **K = 4 → 6 is free on efficiency and the tail, and cheaper on variance.**
  ε_unw: −0.1σ, −0.4σ, +2.6σ (K = 6 higher on `pp_to_jj`); Hill median and
  worst: every |z| ≤ 1.2; quoted variance per evaluation: ×0.95, ×0.89, ×0.90.
  Against the base, K = 6 is the only rung at or below the base's quoted
  variance per evaluation on all three rows (0.96, 0.93, 1.00) while keeping
  ε_unw ×1.38, ×1.59, ×1.22 and the worst-channel tail index +0.23, +0.48,
  +0.53 (z 2.9, 6.0, 4.1).
- **K = 6 → 10 gives back the tail.** Hill worst −0.26 (−3.1σ) and median
  −0.21 (−2.7σ) on `pp_to_llj_mlm`, ε_unw −2.1σ on `pp_to_jj`, for a 2-4 %
  quoted-variance gain. The bound's work is done by K = 6.
- **Rescale + K = 6** is the best arm on `pp_to_jj` by far (ε_unw 0.181,
  ×1.73 the base, +30.9σ over K = 6) and gives the highest worst-channel tail
  index on `pp_to_llj` (2.30) and `pp_to_llj_mlm` (2.50), at the rescale's
  half CPU per run. It costs quoted variance per evaluation ×1.60 and ×1.71
  on the two Drell-Yan rows (the rescale widens grids into rejected regions,
  as P1 found), and its ε_unw on `pp_to_llj_mlm` is *below* K = 6 alone
  (0.049 against 0.058, −4.1σ). Its `quoted²·CPU` against rescale alone is
  1.00, 0.98, 1.02 — the bound adds nothing to the rescale's cost. Mixed
  across rows; not proposed as a default here.

## Proposal (the default is unchanged in this session)

**K = 6.** The deciding table, 20 seeds each, ratios to the base:

| | ε_unw `llj` / `mlm` / `jj` | Hill worst Δ `llj` / `mlm` / `jj` | quoted² per evaluation `llj` / `mlm` / `jj` |
|---|--:|--:|--:|
| K = 2 | ×1.92 / ×2.17 / ×1.07 | +0.23 / +0.74 / +1.10 | 1.21 / 1.39 / 1.55 |
| K = 3 | ×1.56 / ×1.68 / ×1.13 | +0.19 / +0.75 / +0.56 | 1.06 / 1.15 / 1.24 |
| K = 4 | ×1.41 / ×1.62 / ×1.18 | +0.21 / +0.49 / +0.46 | 1.02 / 1.05 / 1.10 |
| **K = 6** | **×1.38 / ×1.59 / ×1.22** | **+0.23 / +0.48 / +0.53** | **0.96 / 0.93 / 1.00** |
| K = 10 | ×1.19 / ×1.63 / ×1.19 | +0.11 / +0.21 / +0.42 | 0.92 / 0.91 / 0.97 |

K = 6 keeps K = 4's tail-index gain on every row (every |z| ≤ 1.2), equals
K = 4's efficiency within 0.4σ on the Drell-Yan rows and exceeds it on
`pp_to_jj`, and is the tightest rung that costs no variance per evaluation on
any row; K = 10, the only looser rung, gives back the tail on
`pp_to_llj_mlm`. A tighter K trades σ precision for unweighting efficiency on the
Drell-Yan rows only; if event-generation throughput on those rows matters
more than σ precision, K = 3 is the alternative (×1.56-1.68 ε_unw for 6-24 %
variance), but it is a per-row trade, not a default. Evaluations and CPU to a
0.2 % target are unmoved at every K.

Sequencing, for the manager: the flip is a library default (the CLI's
`--grid-max-bin-width` then needs an "off" spelling), it moves every
integration's sampling, and so needs its banked run with the bound on — every
σ gate and seed sweep read against the measured seed spreads — before the
calibration re-record (PR #45), and its order against PR #51's map-default
flip, since both change every grid. The grid rule must be recorded in the
artifact before or with the flip (P1 Found / HY1 F1).

**Banking request: none.** Every comparison here is against the existing
MadEvent seed records; the default flip's banked run is this crate's own
layer, not new MadEvent data.

## The power-2 rescale (scope item 3): not added — the brief's premise is wrong

HY1's F3 read MadEvent as rescaling its `Σwgt²` refinement grid by the
acceptance factor squared. In `dsample.f` the factor squared is applied to
`grid2` (`:2115-2117`), but `grid2` is not what MadEvent refines from:

- under the default `use_cut = 2` the refinement reads `grid(1,·,·)` only —
  `Σ|wgt|` (`:1890`) times the factor to power **1** (`:2112-2114`), then
  `average_grid` (`:2157`, whose body `:2529-2547` reads and writes only
  `grid(1,·,j)`) and the log damping (`:2170-2174`, again `grid(1,·,·)`);
- under `use_cut = 3`, `grid(1) = grid2` is copied *before* the rescale
  (`:2110`), so MadEvent refines from `Σwgt²` times the factor to power **1**
  — exactly this crate's `Squared` + `--grid-acceptance-rescale`;
- `grid2 · factor²` is read only by `use_cut = 4`'s `reliable` diagnostic
  (`:2128`).

So no MadEvent mode refines from `Σwgt² · factor²`, and the existing arm is
already MadEvent's `use_cut = 3` rule. Adding a power-2 variant would test a
rule with no reference behind it; it is left out, and nothing in the tree
changed.

## Deviations and corrections to the brief

- **Item 3's F3 premise** (above): MadEvent's factor² never reaches its
  refinement; no variant added.
- **The rescale + bound arm is at K = 6**, the proposal's K, chosen after
  the ladder as the brief says.
- **CPU across sittings is not a same-sitting ratio** (the protocol's §1
  asks for one; reusing P1's arms, as the brief directs, cannot give it).
  The `base_ctl` re-runs measure the drift at 1.05-1.25. Rung-to-rung variance reads use `quoted²` (per evaluation,
  CPU-free); every CPU-weighted column carries the drift and is marked so.
- K = 10 was run: K = 2, 3, 6 plus the controls cost 3.2 h of run wall time
  (sum of the drive's per-run `t_int + t_scan`), under the brief's 5 h threshold.

## Found

- **The rescale + bound pair is row-dependent and only measured at one K.**
  On `pp_to_jj` it raises ε_unw ×1.41 over K = 6 alone; on `pp_to_llj_mlm` it
  lowers it ×0.85; it raises the quoted variance per evaluation ×1.6-1.7 on
  the Drell-Yan rows (offset by half the CPU per run). Close: the pair at K = 3 and K = 10 on the same three rows
  (20 seeds), and the fixed-beam controls, before considering it as a
  default.
- **The best K differs by row.** ε_unw peaks at K = 2 on the Drell-Yan + jet
  rows and at K = 6 on `pp_to_jj`; a uniform K across all dimensions and
  channels is a compromise. Close: a study of which grid dimension carries
  the widest bins on each row (the per-dimension widths of the trained grids
  in `grid.bin.zst`, already on disk for every run here) to see whether a
  per-dimension bound — tight on the dimensions the cuts reject, loose on the
  rest — buys K = 2's efficiency without its variance.
- **This report's CPU-weighted columns mix sittings**, which
  `end-to-end-timing-protocol` §1 ("same host, same sitting, ratios only")
  rules out: the new rungs ran in a later sitting than P1's base, K = 4 and
  rescale, and the `base_ctl` re-run shows CPU ×1.049-1.247 on identical work
  (σ bit-identical). The proposal rests on the CPU-free `quoted²` and on
  ε_unw and Hill, so it does not depend on them. Close: K = 6 against base
  timed interleaved in one sitting (20 seeds on the three rows, about 2 h
  here) before the flip quotes a variance-per-CPU figure.
- **Under-dispersion persists at the looser bounds on `pp_to_llj_mlm`.**
  Seed χ²/dof 0.63 (K = 6), 0.66 (K = 10), 0.43 (rescale + 6), against 1.10
  and 1.24 at K = 2 and 3: the quoted error still overstates the seed spread
  by about 20-30 % there under the proposed default. Close: the 20-seed
  spreads beside the gate calibrations in the re-record (P1's Found, now with
  the K = 6 numbers).
- **The default flip needs an "off" spelling for the bound.** The CLI flag
  takes only `K > 1`; once a library default is set, a user needs a way back
  to the unbounded grid for comparison runs. Close: a `--grid-max-bin-width
  none` value (or equivalent) in the flip's commit.
