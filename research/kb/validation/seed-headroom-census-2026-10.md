---
type: Measurement
title: Seed-sweep headroom of the σ gates (2026-10, refdata-10)
description: "Five-seed headroom of every gated fixed-beam σ row and the hadronic σ gates under soft-all, MadEvent's τ rule and the K = 6 bin bound, against refdata-10's seed records; 20- and 80-seed spreads beside the five-seed calibrations; what moved the 2026-09 census figures."
status: draft
tags: [validation, seed-sweep, tolerances, statistics, census]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-19}
measured:
  - commit: 508e094
    pr: 45
    host: "4-core cloud container"
    command: "cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --ignored --nocapture --exact probe_gate_row_seed_headroom (also probe_llj_parton_seed_stability; validate_hadronic probe_llj_fixed_budget_ladder, probe_llj_dyn_budget_ladder, probe_jj_budget_ladder, probe_llj_fixed_scatter_guard_calibration)"
  - commit: 3cfd7b8
    pr: 45
    host: "4-core cloud container"
    command: "probe_resonant_seed_stability, probe_smeft_capstone_seed_stability, probe_non_sm_seed_stability (validate_sigma); probe_llj_dyn_scatter_guard_calibration, sigma_llj_mlm_vs_madevent, sigma_llj_xqcut_only_vs_madevent (validate_hadronic); vibegraph integrate --fixed-budget over seeds 20261101-20"
  - commit: 6ef026f
    pr: 45
    host: "4-core cloud container"
    command: "probe_bb_budget_ladder, probe_recarded_budget_ladder (validate_hadronic)"
  - commit: 5782665
    pr: 45
    host: "4-core cloud container"
    command: "pixi run --skip-deps validate (the hadronic gates' five-seed readings)"
sources:
  - {id: v1-report, resource: "../sprints/sigma-calibration-rerecord/sessions/V1-report.md", title: "sigma-calibration-rerecord V1 report (commands, logs, tree hashes)"}
  - {id: census-09, resource: "seed-headroom-census-2026-09.md", title: "The 2026-09 census this page re-records"}
  - {id: p1, resource: "../sprints/vegas-low-acceptance-bins/sessions/P1-report.md", title: "vegas-low-acceptance-bins P1 (the rows whose 20-seed spreads are re-measured here)"}
  - {id: p2, resource: "../sprints/vegas-low-acceptance-bins/sessions/P2-report.md", title: "vegas-low-acceptance-bins P2, Found 4 (pp_to_llj_mlm's quoted error above its spread)"}
---

The [2026-09 census](seed-headroom-census-2026-09.md) re-measured on the tree
that carries refdata-10, the soft-all split angle with MadEvent's τ rule, and
the VEGAS bin-width bound `K = 6`. Every reading is against the MadEvent seed
record the gate reads (`BankedSigma::madevent_reference`), not the single
MadEvent run. The probes' code is unchanged from the gates' at `5782665`;
`508e094` and `3cfd7b8` add only probe reads of that reference. How to read
headroom, class (a) tolerances against class (b) standardised thresholds, is
the [2026-09 page](seed-headroom-census-2026-09.md)'s and
[gate thresholds](gate-thresholds.md)'.[^v1-report]

## `validate_sigma.rs`: 35 `Plan::Gate` rows

`probe_gate_row_seed_headroom`: the gate's seeds `[20260719, 11, 22, 33, 44]`
at each row's plan budget, 970 s for the 35 rows. The gate asserts
`|pull| ≤ 3.5` and `|rel| ≤ rel_tol` on the five seeds' unweighted mean; the
worst single seed is how far inside `rel_tol` the individual draws sit. Each
row's mean rel, mean pull and seed χ²/dof equal the banked gate's own cell on
the same tree to every printed digit.

| row | budget | `rel_tol` | worst \|rel\| | headroom | worst seed \|pull\| | mean rel | mean pull | seed χ²/dof |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| `gg_to_gg_cg` | 40000x6 | 0.005 | 2.55e-03 | 2.0× | 1.97 | +4.28e-04 | +0.68 | 0.97 |
| `gux_to_epemux` | 60000x8 | 0.005 | 2.43e-03 | 2.1× | 1.70 | +1.15e-03 | +1.24 | 1.05 |
| `ll_to_qqx_toy_tensor` | 40000x6 | 0.005 | 2.29e-03 | 2.2× | 2.49 | +6.56e-04 | +0.95 | 2.73 |
| `ud_to_epemud_qcd0` | 120000x8 | 0.01 | 3.80e-03 | 2.6× | 1.52 | +2.35e-03 | +1.47 | 0.37 |
| `gu_to_epemu` | 60000x8 | 0.005 | 1.73e-03 | 2.9× | 1.22 | +3.94e-04 | +0.41 | 1.20 |
| `gg_to_gg` | 40000x6 | 0.008 | 2.59e-03 | 3.1× | 2.34 | +1.21e-03 | +1.76 | 1.65 |
| `gg_to_ttx_smlimit` | 40000x6 | 0.005 | 1.60e-03 | 3.1× | 1.92 | +4.05e-04 | +0.66 | 1.20 |
| `uux_to_epemg` | 60000x8 | 0.01 | 3.21e-03 | 3.1× | 2.06 | +1.85e-03 | +1.85 | 0.59 |
| `gg_to_ttx_smlimit_qcd2` | 40000x6 | 0.005 | 1.54e-03 | 3.2× | 1.82 | +4.16e-04 | +0.66 | 1.48 |
| `uux_to_uux` | 40000x6 | 0.005 | 1.55e-03 | 3.2× | 1.57 | +2.89e-04 | +0.48 | 1.45 |
| `ddx_to_epemg` | 60000x8 | 0.01 | 2.80e-03 | 3.6× | 2.01 | +2.09e-03 | +2.29 | 0.27 |
| `ee_to_mumu_tata_qcd0` | 100000x8 | 0.02 | 4.77e-03 | 4.2× | 2.51 | +1.40e-03 | +1.06 | 1.65 |
| `p3r3_to_p3r3_toy_epsilon` | 40000x6 | 0.005 | 1.20e-03 | 4.2× | 1.74 | +4.78e-04 | +1.41 | 0.99 |
| `uux_to_mumu` | 30000x6 | 0.005 | 1.20e-03 | 4.2× | 1.39 | +1.02e-04 | +0.21 | 1.14 |
| `ee_to_zh` | 30000x6 | 0.005 | 1.07e-03 | 4.7× | 1.89 | -2.14e-05 | -0.07 | 1.28 |
| `qqx_to_o8o8_toy_dcolor` | 40000x6 | 0.005 | 1.06e-03 | 4.7× | 1.25 | -9.58e-05 | -0.15 | 0.98 |
| `ee_to_ttx_smlimit` | 40000x6 | 0.005 | 9.98e-04 | 5.0× | 1.53 | -1.79e-04 | -0.56 | 0.77 |
| `ll_to_qqx_toy_dipole` | 40000x6 | 0.005 | 1.01e-03 | 5.0× | 2.04 | -1.46e-04 | -0.55 | 1.19 |
| `ee_to_ttx_dipole` | 40000x6 | 0.005 | 9.75e-04 | 5.1× | 1.48 | -1.55e-04 | -0.47 | 0.77 |
| `ee_to_ttx` | 30000x6 | 0.005 | 9.03e-04 | 5.5× | 1.21 | +1.59e-04 | +0.41 | 1.18 |
| `p3r3_to_p3r3_toy_sextet` | 40000x6 | 0.005 | 8.94e-04 | 5.6× | 1.30 | +3.75e-04 | +1.11 | 0.61 |
| `ee_to_mumu_4f` | 40000x6 | 0.005 | 8.41e-04 | 5.9× | 1.15 | -3.22e-04 | -0.87 | 0.48 |
| `ee_to_mumu` | 30000x6 | 0.005 | 7.96e-04 | 6.3× | 0.94 | -4.05e-06 | -0.01 | 0.70 |
| `ee_to_ttx_smeft` | 160000x8 | 0.002 | 3.03e-04 | 6.6× | 0.98 | +4.88e-05 | +0.30 | 0.50 |
| `uux_to_ttx_4f` | 160000x8 | 0.005 | 6.53e-04 | 7.7× | 2.26 | -2.51e-04 | -1.88 | 0.64 |
| `tata_to_ttx_tensor4f` | 40000x6 | 0.005 | 5.97e-04 | 8.4× | 0.92 | -1.34e-04 | -0.42 | 0.54 |
| `ee_to_mumu_smlimit` | 40000x6 | 0.005 | 5.81e-04 | 8.6× | 0.78 | -4.72e-05 | -0.12 | 0.46 |
| `gg_to_ttx` | 60000x8 | 0.005 | 5.67e-04 | 8.8× | 0.79 | -3.32e-06 | -0.01 | 0.61 |
| `ee_to_mumua` | 80000x8 | 0.03 | 3.06e-03 | 9.8× | 2.02 | -3.64e-04 | -0.42 | 1.60 |
| `ee_to_wpwm` | 40000x8 | 0.008 | 7.18e-04 | 11.1× | 0.84 | +3.69e-04 | +0.50 | 0.54 |
| `ee_to_wpwm_cw` | 40000x6 | 0.008 | 5.92e-04 | 13.5× | 0.68 | +2.92e-04 | +0.39 | 0.40 |
| `ee_to_zh_smeft` | 160000x8 | 0.005 | 3.42e-04 | 14.6× | 1.28 | +3.61e-05 | +0.29 | 0.84 |
| `ll_to_qqx_toy_yukawa` | 160000x8 | 0.005 | 3.15e-04 | 15.9× | 1.09 | +1.97e-05 | +0.15 | 0.66 |
| `ee_to_tatah` | 60000x8 | 0.02 | 7.87e-04 | 25.4× | 0.86 | +6.23e-05 | +0.11 | 0.42 |
| `ee_to_ee` | 80000x10 | 0.01 | 3.56e-04 | 28.1× | 0.54 | +2.27e-04 | +0.58 | 0.07 |

- **Class (a), under 2×:** none. `gg_to_gg_cg` sits at 2.0× exactly (seed
  scatter, 0.12% a seed, not offset), `gux_to_epemux` at 2.1× and
  `ll_to_qqx_toy_tensor` at 2.2× (seed χ²/dof 2.73).
- **`ddx_to_epemg` is no longer thin on `rel_tol`.** Against the single run it
  read 1.6×; against its seed record (`0.6191 ± 0.074%`, five MadEvent seeds at
  `hard_survey = 2`) it reads 3.6×. Its five-seed mean pull, `+2.29`, is the
  largest of the 35, and `uux_to_epemg`'s `+1.85` is second.
- **Class (b), the pull.** The gate's statistic is one draw per row. Over the
  35 rows the mean pulls have mean `+0.38`, standard deviation `0.84`, median
  `|pull|` `0.50` and maximum `2.29`; 23 of 35 are positive. The first seed's
  own pulls average `+0.55` (sd `0.69`). A simulation of the seed-policy pull
  with correct quoted errors on both sides gives sd `0.91` and
  `P(|pull| > 3.5) = 2.4e-4`, `4.6e-3` when one side's seeds scatter at twice
  their quoted error. The worst single-seed `|pull|` per row averages `1.49`
  (median `1.48`) against `1.57` expected for the largest of five `|N(0,1)|`.
- **Seed χ²/dof** ranges `0.07` (`ee_to_ee`) to `2.73`
  (`ll_to_qqx_toy_tensor`), mean `0.91`. `ee_to_ee`'s `0.07` is a seed spread
  a quarter of the quoted error (1.5e-4 against 6.0e-4 of σ).

### The thin pair, at the gate budget and at four times it

`probe_llj_parton_seed_stability`, the gate's five seeds, against the seed
records:

| row | budget | mean rel | mean pull | seed χ²/dof | worst \|rel\| | `rel_tol` headroom | wall, five seeds |
|---|---|--:|--:|--:|--:|--:|--:|
| `ddx_to_epemg` | 60000×8 | +2.09e-3 | +2.29 | 0.27 | 2.80e-3 | 3.6× | 13.3 s |
| | 240000×8 | +1.48e-3 | +1.69 | 3.15 | 2.66e-3 | 3.8× | 34.8 s |
| `gux_to_epemux` | 60000×8 | +1.15e-3 | +1.24 | 1.05 | 2.43e-3 | 2.1× | 19.0 s |
| | 240000×8 | +0.90e-3 | +1.08 | 1.93 | 2.27e-3 | 2.2× | 40.4 s |
| `uux_to_epemg` | 60000×8 | +1.85e-3 | +1.85 | 0.59 | 3.21e-3 | 3.1× | 13.3 s |
| | 240000×8 | +2.28e-3 | +2.65 | 0.98 | 2.99e-3 | 3.3× | 31.7 s |
| `gu_to_epemu` | 60000×8 | +0.39e-3 | +0.41 | 1.20 | 1.73e-3 | 2.9× | 16.5 s |
| | 240000×8 | +0.52e-3 | +0.61 | 2.14 | 1.31e-3 | 3.8× | 41.0 s |

Four times the points costs about 2.5 times the wall time (the α survey is a
fixed cost) and buys no `rel` margin on either thin row: `gux_to_epemux`'s
worst seed moves from 2.43e-3 to 2.27e-3, a different seed. The two
`q q̄ → e⁺e⁻ g` rows sit about `+0.2%` high at both budgets, and on
`uux_to_epemg` the pull grows with budget (`+1.85` → `+2.65`).

## `validate_hadronic.rs`: the σ gates

The banked run's five-seed readings (`pixi run --skip-deps validate`, tree
`e1fe8ae`), against the seed records:

| row | budget | rel | pull | seed χ²/dof | `*_MAX_REL` headroom |
|---|---|--:|--:|--:|--:|
| `pp_to_llj_fixed` | 150k×10 | +0.284% | +1.29 | 0.55 | **1.8×** |
| `pp_to_llj` | 150k×10 | +0.197% | +1.05 | 0.49 | 2.5× |
| `pp_to_llj_dyn` | 150k×10 | +0.226% | +1.01 | 0.53 | 2.2× |
| `pp_to_bb_fixed` | 75k×10 | +0.169% | +1.96 | 1.00 | 3.0× |
| `pp_to_bb_qcd2` | 75k×10 | +0.098% | +1.87 | 1.12 | 5.1× |
| `pp_to_bb` | 75k×10 | +0.064% | +1.33 | 1.71 | 7.8× |
| `pp_to_ll` m_ll window | 120k×12 | +0.050% | +1.06 | 0.16 | 10.1× |
| `pp_to_jj` | 75k×10 | −0.046% | −0.51 | 1.03 | 10.9× |
| `pp_to_ll` default | 120k×12 | +0.037% | +0.73 | 0.80 | 13.6× |
| `pp_to_ll_scalefact2` | 75k×10 | −0.023% | −0.19 | 0.65 | 21.8× |

`pp_to_llj_fixed`'s gate quintet is a high draw of a real offset: eighty seeds
at the same budget read `+0.23%` (2.2×).

The ℓℓj budget ladders, five seeds a rung (`probe_llj_fixed_budget_ladder`,
`probe_llj_dyn_budget_ladder`), with the five seeds' standard deviation and
mean quoted error per rung, both as fractions of σ:

| row | rung | σ (pb) | rel | χ²/dof | seed sd | quoted |
|---|---|--:|--:|--:|--:|--:|
| `pp_to_llj_fixed` (MadEvent 423.066 ± 0.898) | 75k | 424.759 | +0.40% | 1.15 | 0.20% | 0.19% |
| | 150k | 424.269 | +0.28% | 0.55 | 0.097% | 0.13% |
| | 300k | 423.749 | +0.16% | 1.15 | 0.10% | 0.092% |
| | 600k | 424.006 | +0.22% | 0.49 | 0.046% | 0.065% |
| `pp_to_llj_dyn` (MadEvent 414.716 ± 0.895) | 75k | 416.176 | +0.35% | 0.93 | 0.18% | 0.19% |
| | 150k | 415.653 | +0.23% | 0.53 | 0.095% | 0.13% |
| | 300k | 415.293 | +0.14% | 0.88 | 0.087% | 0.093% |
| | 600k | 415.536 | +0.20% | 0.39 | 0.041% | 0.065% |

`probe_jj_budget_ladder` reads `−0.05%`, `−0.03%`, `−0.00%`, `−0.02%` at 75k
to 600k (χ²/dof 1.03, 0.55, 0.93, 1.27; seed sd 0.12%, 0.061%, 0.056%, 0.046%).
`probe_bb_budget_ladder` (`pp_to_bb_fixed`) reads `+0.17%`, `+0.06%`, `+0.02%`,
`+0.06%` (χ²/dof 1.00, 2.33, 0.31, 0.45), and `probe_recarded_budget_ladder`
reads `pp_to_bb` `+0.06%`, `+0.06%`, `+0.07%`, `+0.04%` (1.71, 0.32, 0.37,
0.32), `pp_to_bb_qcd2` `+0.10%`, `+0.04%`, `+0.02%`, `+0.06%` (1.12, 0.25,
0.46, 0.46), `pp_to_llj` `+0.28%`, `+0.20%`, `+0.22%`, `+0.21%` (1.32, 0.49,
0.49, 1.43) and `pp_to_ll_scalefact2` `−0.02%`, `+0.00%`, `+0.03%`, `+0.05%`
(0.65, 1.43, 0.25, 0.49), all at 75k, 150k, 300k and 600k against the seed
records (commit `6ef026f`, whose gate code equals `5782665`'s).

## Spreads beside the calibrations

The rows `vegas-low-acceptance-bins` P1 and P2 measured over twenty seeds,
re-measured on this tree. "Gate" is the row's own gate (its seeds and
harness); "20 seeds" is `vibegraph integrate --fixed-budget` at the gate's
budget over seeds 20261101-20 (the P1/P2 driver, `--no-scan`); the 80-seed
rows are `llj_scatter_calibration` at the gate's budget from the gate's
first seed. Spread is one seed's sample standard deviation, quoted one seed's
mean quoted error, both over σ.[^p1][^p2]

| row | budget | gate: seeds, spread / quoted (ratio), χ²/dof | many seeds: n, spread / quoted (ratio), χ²/dof, disjoint quintets' χ²/dof |
|---|---|---|---|
| `pp_to_llj_mlm` | 150k×10 | 10, 0.123% / 0.136% (0.90), 0.81 | 20, 0.099% / 0.138% (**0.72**), 0.52, 0.23-1.00 |
| `pp_to_llj_xqcut_only` | 150k×10 | 5, 0.161% / 0.130% (1.24), 1.52 | 20, 0.095% / 0.131% (**0.72**), 0.52, 0.24-0.99 |
| `pp_to_llj` | 150k×10 | 5, 0.110% / 0.158% (0.70), 0.49 | 20, 0.159% / 0.158% (1.01), 1.01, 0.17-3.01 |
| `pp_to_jj` | 75k×10 | 5, 0.119% / 0.117% (1.01), 1.03 | 20, 0.125% / 0.117% (1.07), 1.14, 0.15-2.20 |
| `uux_to_epemg` | 60k×8 | 5, 0.102% / 0.132% (0.77), 0.59 | 20, 0.151% / 0.132% (1.14), 1.31, 0.18-2.78 |
| `ee_to_mumua` | 80k×8 | 5, 0.191% / 0.150% (1.27), 1.60 | 20, 0.160% / 0.152% (1.05), 1.08, 0.57-1.69 |
| `pp_to_llj_fixed` | 150k×10 | 5, 0.097% / 0.131% (0.74), 0.55 | 80, 0.123% / 0.131% (0.94), 0.89, 0.15-2.00 |
| `pp_to_llj_dyn` | 150k×10 | 5, 0.095% / 0.131% (0.72), 0.53 | 80, 0.125% / 0.131% (0.95), 0.91, 0.14-2.02 |

Read against AGENTS.md's warning that few-seed scatter understates a
heavy-tailed row's spread: on this tree the five-seed gate quintets read
0.70-1.27 of the quoted error where twenty or eighty seeds read 0.72-1.14 on
the same rows; the five seeds understate the many-seed spread on five of the
eight rows, the direction the warning predicts. The two
matched `ℓℓj` rows are the exception in the other direction: over twenty
seeds their quoted error is 39% above their seed spread (P2 measured 26% on
`pp_to_llj_mlm` under `K = 6` alone), so their pull is conservative and their
seed χ²/dof reads low (0.52); the gate's ten-seed `pp_to_llj_mlm` read is
0.90 of quoted. Under `max(quoted, spread)` an over-quoted error widens the
gate rather than narrowing it.

## What moved the 2026-09 figures

The 2026-09 census found every calibration comment written before `e73b158`
drifted and attributed the drift to the note-34 draw-performance commits
(`f85718d`, `c48fc69`, `f3d6e8b`), with one run of
`probe_gate_row_seed_headroom` at `f85718d^` as the falsifier.[^census-09]
That probe did not exist at `f85718d^`, and the 13 figures written in
`e73b158` are SMEFTsim and toy-model rows that did not exist there either, so
the falsifier was run as a throwaway probe printing each gated row's per-seed
σ to nine digits at its plan budget, built at each commit in a detached
worktree against refdata-10's run cards and the commit's own
`sigma_reference.json` (the SM single runs are the same numbers in every
bundle). A moved sampling stream moves a first-seed σ at the `1e-4` level or
more, the scale of its error; a change in the ninth digit is re-associated
arithmetic, and identical digits are an unmoved stream.

**The attribution is falsified.** At `f85718d^` (`c9993b4`) 14 of 17 SM rows
reproduce the 2026-09 census's worst \|rel\| to every printed digit
(`gg_to_gg` 1.972e-3, `ddx_to_epemg` 6.296e-3, `gux_to_epemux` 2.628e-3,
`ee_to_ee` 1.435e-3, ...), and `f85718d` itself is bit-identical to its
parent on all four rows tried. `f3d6e8b` lands between two bisection points
(`e99b05c`, `443f6bc`) and `c48fc69` between two others (`a38ac3a`,
`c9993b4`), and each pair's first-seed σ are identical on every row. What moved the figures, bisected over the 108 first-parent
commits from `35ab3f1` to `c9993b4` with nine signature rows:

| commit | change | rows it moved |
|---|---|---|
| `e99b05c` (`ea58ab9`, 2026-08-04) | unbiased VEGAS iteration combination | every row tried, `2 → 2` included |
| `7664ff9` (`cd49f40`, 2026-08-06) | cut-implied timelike floors for the invariant draw | every row with three or more final-state particles; no `2 → 2` row |
| `b6a0b88` (`09cfb61`, 2026-08-07) | the channel floor denominated in accepted points | `ee_to_mumu_tata_qcd0`, `ee_to_tatah`, `ud_to_epemud_qcd0` (the many-channel rows) |

`35ab3f1` itself reproduces the figures its era's comments recorded
(`gg_to_gg` 1.438e-3 against 1.4e-3, `gg_to_ttx` 8.5e-4 against 8.6e-4,
`uux_to_epemg` 3.93e-3 against 3.9e-3, `ddx_to_epemg` 5.58e-3 against
5.6e-3, `ud_to_epemud_qcd0` 2.64e-3 against 2.6e-3), so the instrument reads
those comments' own statistic. `gu_to_epemu` and `gux_to_epemux` read
1.2e-2 and 1.0e-2 there: their recorded 1.35e-3 and 1.51e-3 were written
after the per-point AMP2 configuration draw (chain B, `2d60d1b`), which moved
them before the two commits above did. Between `b6a0b88` and `e73b158` the
four rows tried agree to eight digits (`gg_to_gg` moves in the ninth), and the
2026-09 census found every figure written in `e73b158` reproducing.

**The drift after the census** (`ee_to_ee` 1.44e-3 → 3.24e-3, `gg_to_gg`
1.97e-3 → 2.66e-3, `ll_to_qqx_toy_tensor` 1.40e-3 → 2.23e-3, the
hygiene sprint's re-read) is `c2c5407`, "integrate on MadGraph's configuration
channel set", a later commit of the census's own PR (#6, squashed as
`02e8b25`): the census commit `44e4e04` and `c2c5407^` agree on the three
rows' first seed (`ee_to_ee` and `gg_to_gg` to every digit,
`ll_to_qqx_toy_tensor` to `1.8e-6`), `c2c5407` moves all three by `5e-4` or
more, and `02e8b25` and `669f3fa`
reproduce the October readings exactly.

Every figure since then moved with deliberate sampling changes (#42's
five-seed gates and seed records, #44's refdata-10, #51's map defaults, #48's
bin bound), and this page's tables are the re-record after them. The `2 → 2`
coloured rows are the control: `gg_to_ttx`'s first-seed σ, 13.52203 pb, is the
same at `c9993b4`, `02e8b25` and this tree.

[^v1-report]: The session report carries every command, log and tree hash.
[^census-09]: The 2026-09 page's tables, measured at `0538e2d`.
[^p1]: P1 report, "Tail rows" and "Fixed-beam controls".
[^p2]: P2 report, the ladder tables and Found 4.
