---
type: Measurement
title: Seed-sweep headroom of the enforced gate statistics (base 0538e2d)
description: "Five-seed headroom of every enforced tolerance and pull/chi2 threshold in the banked layer, read by threshold class; calibrations written before e73b158 no longer reproduce."
status: draft
tags: [validation, seed-sweep, tolerances, statistics, census]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  commit: 44e4e04
  pr: 6
  landed_in: 02e8b25
  command: "#[ignore] probes run with --ignored under --features extended-validation: probe_gate_row_seed_headroom (validate_sigma), probe_samples_p_floor_headroom (validate_samples), probe_hadronic_seed_headroom (validate_hadronic), plus the unweighting and cli_generate_proton sweeps"
sources:
  - {id: n36-b0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L86-L143", title: "Note 36 B0 (the census brief and outcome)"}
  - {id: n36a-classes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L12-L51", title: "Note 36a §0 (how to read headroom)"}
  - {id: n36a-sigma, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L52-L190", title: "Note 36a §1 (validate_sigma: 31 Plan::Gate rows)"}
  - {id: n36a-samples, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L191-L212", title: "Note 36a §2 (P_FLOOR)"}
  - {id: n36a-hadronic, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L213-L277", title: "Note 36a §3 (validate_hadronic)"}
  - {id: n36a-other, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L278-L325", title: "Note 36a §4–6 (unweighting, cli_generate_proton, samples_proton)"}
  - {id: n36a-thin, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L326-L397", title: "Note 36a §7–8 (the thin list; downstream)"}
---

A census of every enforced statistic in the banked validation layer: for each,
how many seeds form it, how many calibrated its threshold, and how far the
worst of five seeds sits from the threshold. It was taken on base `0538e2d`
(branch commit `44e4e04`, merged in PR #6 as `02e8b25`). Note 36a does not
record the host; the sprint plan budgeted the census for the project's 16-core
host. Rows touched by later sampling-stream changes have
moved since; the probes below are the instruments that re-measure
them.[^n36-b0]

## How to read "headroom"

The census splits thresholds into two classes, because the same ratio means
opposite things in each.[^n36a-classes] This is the reading
[gate thresholds](gate-thresholds.md) relies on.

- **(a) Tolerances**: `rel_tol`, `*_MAX_REL`, `SIGMA_REL_LIMIT`,
  `SIGMA_MAX_REL`. They bound a disagreement of physical size that does not
  shrink on its own. Headroom is bound ÷ measured; near 1, the next change that
  touches the row fails it. **Under 2× is thin.**
- **(b) Standardised thresholds**: `PULL_LIMIT`, the hadronic `|pull| < 3.0`,
  `SHAPE_PULL_LIMIT`, `SPECTRUM_MAX_PULL`, both `P_FLOOR`s, every
  `*_MAX_CHI2_PER_DOF`. Each is a false-positive rate against a trial count:
  the statistic is the extremum of N draws from a known null. The largest of
  five `|N(0,1)|` draws has expectation 1.57, exceeds 2.0 a fifth of the time
  and 3.5 once in 430, so a 3.5σ limit reading "2× above the worst of five" is
  the instrument working. **More room would mean the threshold had stopped
  rejecting.** The 2× rule does not apply, and "form the statistic over more
  seeds" can make an extremum-vs-floor statistic worse (more draws lower the
  expected minimum p). Seed count still matters through degrees of freedom:
  `χ²/dof < 4` is a 1.8% false-positive rate on 2 dof (three seeds) and 0.30%
  on 4 dof (five).

The census's per-row worst `|pull|` over the 31 gated σ rows averaged 1.562
(median 1.49) against the half-normal expectation 1.57: the pulls behave as the
standardised variable they are treated as.[^n36a-classes]

## `validate_sigma.rs`: 31 `Plan::Gate` rows

Every row forms its statistic at the single `SEED = 20260719`. The five-seed
column is `probe_gate_row_seed_headroom` (seeds `[SEED, 11, 22, 33, 44]` at
each row's own plan budget, 207 s for all 31), which reproduced the recorded
manifest numbers where they existed.[^n36a-sigma]

`rel_tol` headroom, worst of five (class a):

| headroom | rows (threshold → measured) |
|---|---|
| < 2× | `ddx_to_epemg` 0.01 → 6.30e-3 (1.6×); `gux_to_epemux` 0.005 → 2.63e-3 (1.9×) |
| 2–4× | `gg_to_ttx_smlimit` 2.4×, `uux_to_epemg` 2.4×, `ud_to_epemud_qcd0` 2.5×, `ee_to_wpwm_cw` 2.6×, `ee_to_mumua` 2.7×, `gu_to_epemu` 2.7×, `ee_to_mumu_tata_qcd0` 3.0×, `gg_to_ttx_smlimit_qcd2` 3.0×, `ee_to_ttx_dipole` 3.1×, `tata_to_ttx_tensor4f` 3.5×, `ll_to_qqx_toy_tensor` 3.6×, `ee_to_mumu_4f` 3.8× |
| 4–10× | `ee_to_ttx_smeft` 4.3×, `ee_to_ttx_smlimit` 4.4×, `ll_to_qqx_toy_dipole` 5.0×, `uux_to_ttx_4f` 7.5×, `ee_to_mumu_smlimit` 8.3×, `ee_to_tatah` 9.2×, `uux_to_mumu` 9.9× |
| > 10× | `ee_to_wpwm` 10.7×, `ee_to_zh_smeft` 11.4×, `gg_to_gg` 15.2×, `ee_to_mumu` 16.0×, `ll_to_qqx_toy_yukawa` 16.3×, `ee_to_zh` 18.9×, `uux_to_uux` 19.2×, `ee_to_ttx` 21.7×, `ee_to_ee` 27.9×, `gg_to_ttx` 29.9× |

`PULL_LIMIT = 3.5` (class b): worst-of-five pulls ran 0.51 (`gg_to_ttx`) to
2.65 (`ddx_to_epemg`) on the asserted rows; seven read 1.3×–1.9×, as a
correctly sized limit should. (Note 36a §7's prose says nine; its own table 1b
lists seven: `ddx_to_epemg`, `ee_to_mumu_tata_qcd0`, `ee_to_ttx_dipole`,
`uux_to_ttx_4f`, `tata_to_ttx_tensor4f`, `uux_to_mumu`, `ee_to_wpwm_cw`.) A single draw exceeds 3.5 with probability 4.7e-4, so the
gate's own one-seed reading over 31 rows flags spuriously about once in 70 full
runs.[^n36a-thin]

Findings:[^n36a-sigma]

- **Six rows had no recorded seed calibration**: `uux_to_mumu`, `ee_to_mumu`,
  `ee_to_ttx`, `ee_to_zh`, `ee_to_wpwm`, `ee_to_ee`. Measured, they clear
  `rel_tol` by 9.9×–27.9×: the loosest bounds on the smoothest integrands.
- **`ee_to_mumua`'s exemption is load-bearing.** Its worst pull over five
  seeds is 3.56, above `PULL_LIMIT`; without `PULL_REPORTED_NOT_ASSERTED` the
  gate would fail on a residual attributed to the reference. See
  [sigma-row gating exceptions](sigma-row-gating-exceptions.md).
- **`ddx_to_epemg`'s pull cannot run away with budget**: 2.65 at the gate
  budget, 2.62 at 4×. The combined error is the reference's (0.20%), so the pull
  saturates near `rel/σ_MG,rel ≈ 2.2`. Its 1.3× on `PULL_LIMIT` is bounded, the
  opposite of `ee_to_mumua`.
- **Calibrations written before `e73b158` no longer reproduced; all 13 written
  in it did**, to the digit, at the same seeds, budget and helper. The split is
  by when the comment was written, not by row: a sampling stream moved in the
  window `(5cc41de, e73b158]`, which holds the stream-touching draw-performance
  commits (`f85718d`, `c48fc69`, `f3d6e8b`). The falsifier for that attribution
  is one run of `probe_gate_row_seed_headroom` at `f85718d^`, not yet run. The
  re-record is
  [σ calibration comments stale](../backlog/validation/sigma-calibration-comments-stale.md).

## `validate_samples.rs`: `P_FLOOR`

Smallest p over 32 gating rows × `GEN_SEEDS` × 7–21 columns, against `1e-4`
(class b). `probe_samples_p_floor_headroom` repeats it over five seeds. Three
and five seeds give the same minimum, `1.573e-4` (`ee_to_wpwm` `pt(w+)`, seed
`0x5a4d0003`, 1.57×); the extra seeds read `4.8e-2` and `2.0e-2` on that row.
Runners-up: `ee_to_wpwm_cw` 7.8×, `qqx_to_o8o8_toy_dcolor` 9.4×,
`ddx_to_epemg` 18.5×, `uux_to_ttx_4f` 20.4×. No remedy applies: more seeds
raise the flag rate. The two `info` rows sat below the floor by construction
(`ee_to_mumua` `pt(a)`; `ud_to_epemud_qcd0` `ICOLUP`, whose `samples` cell was
`info` at this base).[^n36a-samples] For the floor's rationale see
[the samples gate](samples-gate.md).

## `validate_hadronic.rs`

`LLJ_SEEDS` was already five; `DY_SEEDS`, `BB_SEEDS`, `JJ_SEEDS` and
`RECARDED_SEEDS` were three. `probe_hadronic_seed_headroom` runs each
three-seed arm over its three gate seeds plus two. The `χ²/dof < 4` bounds were
calibrated on five-seed ladders but formed on three seeds; `DY_MAX_CHI2_PER_DOF`
had no independent calibration (its comment quoted the gate's own
readings).[^n36a-hadronic]

| row | `|rel|` headroom (class a) | notes |
|---|---|---|
| `pp_to_jj` | **1.5×** (+3.33e-3 vs 0.005) | thin; `JJ_SEEDS` raised 3 → 5 (below). Since moved: the vector-vertex sign fix took it to +0.18% (manifest), about 2.8× |
| `pp_to_bb_fixed` | 3.2× | |
| `llj_dyn` | 3.6× | χ²/dof 2.17 vs 4.0 (1.8×, class b) |
| `pp_to_bb` | 6.9× | |
| `pp_to_ll_scalefact2` | 14.8× | |
| `dy_default` | 17.0× | |
| `pp_to_bb_qcd2` | 18.6× | |
| `pp_to_llj` | 19.2× | |
| `llj_fixed` | 25× | |
| `dy_mmll_60_120` | 75.5× | |

**`pp_to_llj_dyn` is doubly stale.** Its manifest note and `LLJ_DYN_MAX_REL`'s
comment recorded +0.21% at χ²/dof 0.66 over five seeds; the census read +0.14%
at χ²/dof 2.17 at the same seeds and budget: σ barely moved and the seed scatter
tripled, the signature of a small per-point numerical change. `χ²_4/4 > 2.17`
has probability 0.070, a 1.5σ upward draw rather than a margin.[^n36a-hadronic][^n36a-thin]
Since then the per-group dynamic-scale fix moved the row again (paired shift
−0.176% ± 0.008% over twenty seeds), so neither recorded figure describes the
tip; its current reading is in its manifest note. See
[configuration-draw σ shifts](../scales-pdf/configuration-draw-sigma-shifts.md).

## Other files

| file | statistic | 5-seed reading | headroom |
|---|---|---|---|
| `validate_unweighting.rs` | `SIGMA_PULL_LIMIT` 3.5 | 1.56 (`ee_to_tatah`) | 2.2× (b) |
| `validate_unweighting.rs` | `SIGMA_REL_LIMIT` 0.03 | 5.77e-3 (`ee_to_mumua`) | 5.2× |
| `validate_unweighting.rs` | `SHAPE_CHI2_LIMIT` 3.0 | 1.25 | 2.4× |
| `validate_unweighting.rs` | `SHAPE_PULL_LIMIT` 5.0 over ~75 bins | 2.54 | 2.0× (b; ≈2.7 is the expected maximum) |
| `cli_generate_proton.rs` | `SIGMA_MAX_REL` 0.015, one seed | 4.28e-3 worst of five | 3.5× |
| `validate_samples_proton.rs` | min p, 10 rows | 1.439e-3 (`pp_to_jj` flavour χ²) | 14.4× (b) |

Before the census none of the four unweighting thresholds had a recorded
calibration. `cli_generate_proton`'s per-seed readings changed sign between the
recorded calibration and the re-measurement at the same magnitude, which is
what a single-seed cell is: which side of the integration a sample lands on is
not a property of the seed.[^n36a-other]

## The thin list, and what was done

No threshold was widened.[^n36a-thin]

| statistic | before | after | remedy |
|---|---|---|---|
| `pp_to_jj` `rel` vs `JJ_MAX_REL` | 1.5×, 3 seeds | 1.5×, **5 seeds** | `JJ_SEEDS` 3 → 5. The ratio does not move (a converged offset, not scatter); the χ²/dof now has the 4 dof its bound was calibrated on |
| `ddx_to_epemg` `rel_tol` | 1.6× vs a stale record | 1.6× recorded | a converged +0.45% offset against a 0.20% reference error, flat at 4× budget; margin is bought with points, not tolerance |
| `gux_to_epemux` `rel_tol` | 1.9× vs a stale record | 1.9× recorded | scatter, not offset: one seed at −2.63e-3, four inside 8.7e-4; 4× budget takes the worst to 1.37e-3 |
| unweighting `SIGMA_PULL_LIMIT` | 1.77× at 4 seeds | 2.2× | `GEN_SEEDS` 4 → 5: the fifth seed buys the denominator of a pull whose error comes from the same seeds |

The budget decision for the two σ rows is
[σ cells thin budget headroom](../backlog/validation/sigma-cells-thin-budget-headroom.md).
The general seed and ladder protocol is
[seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md).

## Running heavy suites

Run gate suites one at a time on a shared host. During the census a full
`pixi run --skip-deps validate` was SIGKILLed by the OS in `validate_samples`
while three worktrees ran heavy suites at once; the remaining suites passed
when re-run serially. A killed run leaves a partially cleared report directory,
and "the gate failed" and "the gate was killed" look the same if only the exit
code is read.[^n36a-thin] Host comparability for timings is
[benchmark hosts](../performance/benchmark-hosts.md).

[^n36-b0]: Note 36 B0, brief and "Landed B0".
[^n36a-classes]: Note 36a §0.
[^n36a-sigma]: Note 36a §1a–1c.
[^n36a-samples]: Note 36a §2.
[^n36a-hadronic]: Note 36a §3.
[^n36a-other]: Note 36a §4–6.
[^n36a-thin]: Note 36a §7–8.
