---
type: Session Report
title: "V1 report: a quarter of the σ pull lean is this side's fixed seed, the rest MadEvent's iteration weighting and budget dependence"
description: "Re-measured at d4b8bb3, the 48 σ pulls that read a seed record lean +0.615. With five seeds on this side, the fixed-beam rows go from +0.61 to +0.41. Read against equal-weight references the population is at +0.29 ± 0.13, and at 4× budget +0.11 ± 0.14. The inverse-variance weighted mean offset is +0.003 % ± 0.005 %, so σ is not shifted as a whole: the lean sits in a few imprecise rows."
status: draft
tags: [validation, sigma, madevent-reference, seeds, iteration-combination, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/d4b8bb3", title: "Measured at d4b8bb3 (sigma-pulls-lean-high base)"}
  - {id: tool, resource: "../../../../../validation-report/src/bin/sigma-pulls.rs", title: "sigma-pulls: the population tool committed by this session"}
---
Diagnosis only. No gate, reader, tolerance or reference changed. The one
committed change is a read-only tool, `sigma-pulls` (see [The tool](#the-tool)).
No MadEvent run was made. Every number below is a recorded measurement on this
host (4 vCPU cloud container, linux-64) at `d4b8bb3`, except where a line names
the CI run.

Commands, by the label the tables cite them with:

- **[G]** the gates: `cargo test -p vibegraph-lib --profile release-debug
  --features extended-validation --test validate_sigma -- --nocapture`, then the
  same with `--test validate_hadronic`. These are the `pixi run validate-sigma`
  and `validate-hadronic` commands, run without their dependency tasks
  (`--skip-deps` semantics).
- **[S]** this side's seed sweep: the existing probe
  `cargo test … --test validate_sigma -- --ignored --nocapture --exact
  probe_gate_row_seed_headroom`. It runs every fixed-beam gate row at its own
  plan budget on seeds `[20260719 (the gate's), 11, 22, 33, 44]`. Its log
  becomes `record seed sigma err` lines (an awk one-liner over the `── row` and
  `seed …: vg` lines).
- **[P]** the population tool: `sigma-pulls --report target/validation-report
  --output validation/madgraph/output --manifest validation/manifest.toml
  [--this-side sweep.tsv]`.

## 1. This side on the current commit: the population reproduces

**[G]** both suites pass:
- `validate_sigma`: `test result: ok. 7 passed; 0 failed; 31 ignored … finished in 274.76s`.
- `validate_hadronic`: `test result: ok. 14 passed; 0 failed; 22 ignored … finished in 547.71s`.

They wrote 52 σ row files. Every this-side value, error and pull equals CI run
38923646457's (`c7bb711`) to the printed digit; **[P]** on both sets gives
identical tables. The fixed-seed integrations are deterministic across the two
hosts and commits.

The backlog's population is the report's integrals column. Collating the local
cells (`validation-report`) and reading the first pull of each integrals cell
gives **47 cells, mean +0.651, sd 0.884, mean/se = 5.05**. That is the brief's
+0.65; its "spread 0.87" is 0.884 at the sample sd. Those 47 are not one
measurement each against a seed record:

- The report shows the **worst |pull| over a row's variants**. Three cells are
  not the restricted row's own pull. `ee_to_ee_nsz` shows its `unrestricted`
  control (+1.63; the restricted row is +1.09). `ee_to_wp0wm` shows its
  `transverse` control (+0.77; the row is +0.07). `pp_to_ll` shows
  `mmll_60_120` (+0.97; `default` is +0.13).
- The two grammar controls shown read `grammar_sigma_reference.json`, not a seed
  record (**[P]**: "not read from a seed record: ee_to_ee_nsz__unrestricted,
  ee_to_mumu_zonly__unrestricted, ee_to_wp0wm__transverse,
  ee_to_wp0wm__unpolarized").

So the population this report reads is the **48 row files whose MadEvent value
is a seed record's gate reading**. **[P]** joins each one by value and checks
that the reported re-read reproduces the gate's pull to 1e-9. These are 34
fixed-beam rows (`gg_to_gg_cg` is `info`), 3 grammar rows and 11 hadronic
measurements on 10 records. The seven records no default-tier gate reads are
the five MLM rows and the two 2 → 6 rows. **R, as measured: N 48, mean +0.615,
sd 0.873, se 0.126 (+4.9 se).**

How many seeds stand behind each pull on this side (the brief says one; that is
true only of the fixed-beam rows):

| rows | this side |
|---|---|
| 34 fixed-beam (SM, toy, SMEFT) | **one** seed (`SEED = 20260719`), shared by every row |
| 3 grammar | 10 seeds at 160000 × 6 |
| 10 hadronic | 5 seeds (rows within a family share their seed list) |

## 2. Three re-reads of the same population

References, all under the seed policy (unweighted mean, max(quoted, spread/√n)):

- `R`: the reported values at the budget, which is what the gates read;
- `EW`: the equal-weight values at the budget;
- `R4x`, `EW4x`: the two seeds at 4× budget;
- `EWmed`: the median of the equal-weight values, error × √(π/2). It is shown
  on rows flagged heavy-tailed, meaning their EW seed spread is more than 2×
  their reported spread.

"Clustered se" joins rows that drew the same MadEvent fluctuations into one
cluster. Every record ran iseeds 20261101–05, so rows whose processes differ
only by a parameter repeat MadEvent's draws. **[P]** finds 5 such clusters, all
physically twins:

- `ee_to_ttx`, `ee_to_ttx_dipole`, `ee_to_ttx_smlimit`;
- `ee_to_wpwm`, `ee_to_wpwm_cw`;
- `gg_to_ttx`, `gg_to_ttx_smlimit`;
- the two `p3r3` toys;
- `pp_to_bb`, `pp_to_bb_qcd2`.

### Per row, as the gates measured them (**[P]** on **[G]**'s cells)

`rel R` is σ_vg/σ_R − 1 with its combined error. `EW−R` is the record's paired
per-seed difference, relative to R, with its significance z. `MG var` is
MadEvent's share of the pull's variance.

| row | class | n | R | EW | EWmed | R4x | EW4x | rel R % | ± % | EW−R % | z | MG var |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| ddx_to_epemg | SM | 1 | +1.52 | +0.01 | | +1.62 | +0.37 | +0.220 | 0.145 | +0.219 | +3.5 | 0.26 |
| ee_to_ee | SM | 1 | −1.16 | −1.05 | | −1.19 | −1.09 | −0.135 | 0.116 | −0.013 | −1.1 | 0.06 |
| ee_to_ee_nsz | grammar | 10 | +1.09 | +1.22 | | +0.75 | +0.97 | +0.065 | 0.060 | −0.009 | −0.8 | 0.21 |
| ee_to_mumu | SM | 1 | +0.76 | +0.78 | | +0.91 | +0.87 | +0.065 | 0.085 | −0.002 | −0.5 | 0.04 |
| ee_to_mumu_4f | BSM | 1 | +0.15 | +0.45 | | +0.63 | +0.62 | +0.011 | 0.072 | −0.023 | −1.1 | 0.08 |
| ee_to_mumu_smlimit | BSM | 1 | +0.24 | +0.37 | | +0.48 | +0.58 | +0.018 | 0.074 | −0.010 | −0.7 | 0.10 |
| ee_to_mumu_tata_qcd0 | SM | 1 | +1.10 | −0.02 | | +0.40 | −0.49 | +0.228 | 0.208 | +0.232 | +2.2 | 0.20 |
| ee_to_mumu_zonly | grammar | 10 | −1.55 | −0.87 | | −0.79 | −0.73 | −0.029 | 0.019 | −0.012 | −1.2 | 0.66 |
| ee_to_mumua | SM | 1 | −0.42 | −0.49 | | −0.38 | −0.42 | −0.074 | 0.177 | +0.013 | +4.4 | 0.01 |
| ee_to_tatah | SM | 1 | −0.08 | −0.14 | | −0.25 | −0.18 | −0.009 | 0.108 | +0.007 | +0.4 | 0.16 |
| ee_to_ttx | SM | 1 | +1.21 | +1.09 | | +1.03 | +0.96 | +0.090 | 0.075 | +0.009 | +3.5 | 0.04 |
| ee_to_ttx_dipole | BSM | 1 | +0.55 | +0.44 | | +0.29 | +0.23 | +0.036 | 0.065 | +0.007 | +1.7 | 0.06 |
| ee_to_ttx_smeft | BSM | 1 | +0.60 | +0.48 | | +0.90 | +0.81 | +0.019 | 0.031 | +0.004 | +0.9 | 0.09 |
| ee_to_ttx_smlimit | BSM | 1 | +0.52 | +0.41 | | +0.33 | +0.24 | +0.034 | 0.064 | +0.007 | +1.9 | 0.05 |
| ee_to_wp0wm | grammar | 10 | +0.07 | +1.02 | +0.08 | +1.08 | −0.18 | +0.007 | 0.099 | −0.379 | −1.1 | 0.99 |
| ee_to_wpwm | SM | 1 | +0.49 | +0.09 | −0.95 | +1.07 | +1.11 | +0.043 | 0.086 | +0.030 | +0.3 | 0.67 |
| ee_to_wpwm_cw | BSM | 1 | −0.16 | −0.34 | −1.30 | +0.40 | +0.40 | −0.014 | 0.087 | +0.036 | +0.4 | 0.67 |
| ee_to_zh | SM | 1 | −0.83 | −0.88 | | −0.51 | −0.40 | −0.046 | 0.055 | +0.004 | +0.5 | 0.02 |
| ee_to_zh_smeft | BSM | 1 | +0.61 | +0.68 | | +0.72 | +0.60 | +0.016 | 0.027 | −0.002 | −1.2 | 0.02 |
| gg_to_gg | SM | 1 | +0.27 | +0.27 | | +0.19 | −0.03 | +0.030 | 0.111 | −0.001 | −0.1 | 0.07 |
| gg_to_gg_cg (info) | BSM | 1 | +0.15 | +0.29 | | +0.06 | +0.14 | +0.019 | 0.126 | −0.017 | −2.3 | 0.07 |
| gg_to_ttx | SM | 1 | +0.66 | +0.79 | | +0.34 | +0.18 | +0.048 | 0.072 | −0.009 | −0.3 | 0.53 |
| gg_to_ttx_smlimit | BSM | 1 | +1.92 | +2.03 | | +1.82 | +1.61 | +0.160 | 0.083 | −0.008 | −0.3 | 0.40 |
| gg_to_ttx_smlimit_qcd2 | BSM | 1 | +1.82 | +1.51 | | +1.27 | +0.88 | +0.154 | 0.085 | −0.002 | −0.0 | 0.38 |
| gu_to_epemu | SM | 1 | +0.16 | +0.04 | | −0.05 | −0.53 | +0.025 | 0.152 | +0.018 | +0.4 | 0.25 |
| gux_to_epemux | SM | 1 | +0.60 | +0.68 | | −0.28 | −0.28 | +0.090 | 0.150 | −0.015 | −0.4 | 0.25 |
| ll_to_qqx_toy_dipole | BSM | 1 | −0.48 | −0.54 | | +0.23 | +0.34 | −0.024 | 0.051 | +0.003 | +0.4 | 0.06 |
| ll_to_qqx_toy_tensor | BSM | 1 | +0.48 | +0.39 | | +0.07 | +0.17 | +0.043 | 0.090 | +0.007 | +0.7 | 0.07 |
| ll_to_qqx_toy_yukawa | BSM | 1 | +0.59 | +0.56 | | +0.66 | +0.66 | +0.017 | 0.029 | +0.001 | +0.4 | 0.02 |
| p3r3_to_p3r3_toy_epsilon | BSM | 1 | +1.74 | +1.69 | +1.63 | +1.49 | +1.66 | +0.120 | 0.069 | +0.004 | +0.3 | 0.05 |
| p3r3_to_p3r3_toy_sextet | BSM | 1 | +1.44 | +1.56 | +1.57 | +1.28 | +1.41 | +0.099 | 0.069 | −0.009 | −0.7 | 0.05 |
| pp_to_bb | hadronic | 5 | +1.92 | +1.54 | | +1.51 | +1.33 | +0.104 | 0.055 | +0.018 | +0.8 | 0.37 |
| pp_to_bb_fixed | hadronic | 5 | +1.92 | +1.65 | | +0.74 | +1.05 | +0.162 | 0.084 | +0.014 | +0.4 | 0.51 |
| pp_to_bb_qcd2 | hadronic | 5 | +1.68 | +1.34 | | +1.08 | +0.89 | +0.083 | 0.049 | +0.015 | +0.6 | 0.45 |
| pp_to_jj | hadronic | 5 | −0.21 | −0.73 | | −0.93 | −0.90 | −0.020 | 0.096 | +0.062 | +1.2 | 0.59 |
| pp_to_ll (default) | hadronic | 5 | +0.13 | −0.20 | | +0.02 | −0.35 | +0.007 | 0.056 | +0.019 | +2.7 | 0.13 |
| pp_to_ll (mmll_60_120) | hadronic | 5 | +0.97 | +0.43 | | +0.51 | +0.25 | +0.052 | 0.053 | +0.029 | +5.1 | 0.12 |
| pp_to_ll_scalefact2 | hadronic | 5 | −0.24 | −0.54 | | +1.40 | +0.79 | −0.029 | 0.120 | +0.041 | +0.6 | 0.70 |
| pp_to_llj | hadronic | 5 | +1.22 | +1.29 | | +1.20 | +0.14 | +0.234 | 0.192 | −0.033 | −0.2 | 0.82 |
| pp_to_llj_dyn | hadronic | 5 | +0.82 | +0.71 | +0.12 | +0.21 | −0.17 | +0.192 | 0.234 | +0.007 | +0.0 | 0.85 |
| pp_to_llj_fixed | hadronic | 5 | +0.90 | +0.77 | +0.94 | −0.40 | −1.19 | +0.202 | 0.226 | +0.007 | +0.0 | 0.88 |
| qqx_to_o8o8_toy_dcolor | BSM | 1 | +0.59 | +0.59 | | −0.09 | −0.06 | +0.050 | 0.084 | +0.000 | +0.0 | 0.45 |
| tata_to_ttx_tensor4f | BSM | 1 | +0.92 | +0.97 | | +0.97 | +0.86 | +0.060 | 0.065 | −0.004 | −0.9 | 0.05 |
| ud_to_epemud_qcd0 | SM | 1 | +1.77 | +1.24 | | +1.43 | +0.87 | +0.370 | 0.208 | +0.073 | +1.1 | 0.43 |
| uux_to_epemg | SM | 1 | +2.45 | +1.11 | | +1.96 | +1.22 | +0.391 | 0.159 | +0.202 | +3.5 | 0.26 |
| uux_to_mumu | SM | 1 | +1.43 | +1.44 | | +1.51 | +1.53 | +0.124 | 0.087 | −0.001 | −0.0 | 0.10 |
| uux_to_ttx_4f | BSM | 1 | −0.30 | −0.47 | | −0.28 | −0.47 | −0.009 | 0.029 | +0.005 | +3.2 | 0.01 |
| uux_to_uux | SM | 1 | −0.55 | −0.58 | | −0.65 | −0.88 | −0.050 | 0.091 | +0.004 | +0.5 | 0.14 |

The fixed-beam rows' `MG var` is mostly below 0.3. For those rows the pull
mostly measures **this side's single seed**, not the reference. Heavy-tailed EW
(spread ratio > 2): `ee_to_wp0wm` 5.35, `ee_to_wpwm` 2.02, `ee_to_wpwm_cw`
2.02, `p3r3` epsilon 2.69 and sextet 4.03, `pp_to_llj_dyn` 2.34,
`pp_to_llj_fixed` 2.23. Their EW χ²/dof are 9.60, 1.98, 2.14, 0.59, 0.76,
1.24 and 1.19 (from the records).

### Population, as the gates measured (**[P]** on **[G]**)

| reference | N | mean pull | sd | se | clustered se (42 clusters) | median | positive |
|---|--:|--:|--:|--:|--:|--:|--:|
| R (the gates) | 48 | **+0.615** | 0.873 | 0.126 (+4.9) | 0.134 (+4.6) | +0.598 | 37 |
| EW | 48 | +0.482 | 0.782 | 0.113 (+4.3) | 0.124 (+3.9) | +0.465 | 35 |
| EW, median on heavy rows | 48 | +0.410 | 0.836 | 0.121 (+3.4) | 0.139 (+2.9) | +0.439 | 34 |
| R4x | 48 | +0.516 | 0.764 | 0.110 (+4.7) | 0.114 (+4.5) | +0.498 | 36 |
| EW4x | 48 | +0.320 | 0.738 | 0.107 (+3.0) | 0.114 (+2.8) | +0.294 | 31 |

Relative offsets σ_vg/σ_ref − 1:

| reference | unweighted | inverse-variance weighted | χ²/dof |
|---|--:|--:|--:|
| R | +0.0677 % ± 0.0151 % | +0.0216 % ± 0.0084 % | 1.01 |
| EW | +0.0563 % ± 0.0146 % | +0.0196 % ± 0.0086 % | 0.74 |
| R4x | +0.0445 % ± 0.0120 % | +0.0200 % ± 0.0077 % | 0.71 |
| EW4x | +0.0189 % ± 0.0107 % | +0.0148 % ± 0.0079 % | 0.57 |

The reference against itself, relative to R. No this-side value enters:

| shift | unweighted | weighted | χ²/dof |
|---|--:|--:|--:|
| EW − R, paired per seed | +0.0114 % ± 0.0115 % | +0.0031 % ± 0.0007 % | 2.40 |
| R4x − R | +0.0232 % ± 0.0094 % | −0.0017 % ± 0.0022 % | 0.67 |
| EW4x − EW | +0.0374 % ± 0.0144 % | −0.0006 % ± 0.0023 % | 0.63 |
| EW4x − R | +0.0487 % ± 0.0143 % | +0.0005 % ± 0.0023 % | 1.05 |

**What the 4× seeds can and cannot show.** Each row has two 4× seeds. Their
spread estimate carries one degree of freedom. A single row's 4× pull cannot
tell a budget shift from a draw, and calibrating one needs the 20 + 20 ladder.
Across 48 rows they do one thing: they screen whether the population's lean
moves with budget. Here it moves in the expected direction:

- the reported reference rises with budget by +0.023 % ± 0.009 % unweighted;
- R4x pulls are lower than R pulls by 0.10 on average.

The **weighted** budget shift is zero (−0.002 % ± 0.002 %), though, so the
budget dependence lives in the imprecise rows. Two seeds cannot say which rows
those are.

## 3. Splits

By class (**[P]** on **[G]**):

| class | N | R | EW |
|---|--:|--:|--:|
| fixed-beam SM | 17 | +0.553 ± 0.237 | +0.258 ± 0.183 |
| toy/SMEFT | 18 | +0.632 ± 0.184 (cl.) | +0.616 ± 0.189 (cl.) |
| grammar | 3 | −0.129 ± 0.771 | +0.459 ± 0.665 |
| hadronic | 10 | +0.911 ± 0.290 (cl.) | +0.627 ± 0.302 (cl.) |

By the record's paired EW − R (resolved: |z| > 2), as the gates measured:

| rows | N | R | EW |
|---|--:|--:|--:|
| EW − R resolved | 9 | +0.758 ± 0.314 | +0.194 ± 0.198 |
| EW − R not resolved | 39 | +0.581 ± 0.139 | +0.548 ± 0.130 |

The correlation of the R pull with the EW − R shift in pull units is +0.27
over 48 rows. **The lean tracks EW − R on the nine rows where that difference is
resolved, and not elsewhere.** On 39 rows EW − R is zero within its error,
and the +0.58 lean there cannot be the iteration weighting.

## 4. This side's own share: five seeds on every fixed-beam row

**[S]** ran all 34 fixed-beam gate rows (`test result: ok. 1 passed … finished
in 753.22s`). Seed 20260719 reproduces each gate value exactly; for example
`ee_to_ttx` gives 5.490834e-1 ± 4.014e-4 in both. Every row's this-side χ²/dof
over the five seeds is inside the 0.13–3.1 band for 4 dof. The range is 0.40
(`ee_to_wpwm_cw`) to 2.77 (`ll_to_qqx_toy_tensor`). So the single seed is a
draw from an estimator whose quoted errors are right.

Rows with this side over five seeds (**[P]** `--this-side`); the cell's
single-seed R pull is beside them:

| row | cell R | R | EW | EWmed | R4x | EW4x | rel R % | ± % |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| ddx_to_epemg | +1.52 | **+2.37** | **+0.00** | | +2.84 | +0.60 | +0.219 | 0.093 |
| ee_to_ee | −1.16 | −0.04 | +0.10 | | −0.07 | +0.06 | −0.004 | 0.091 |
| ee_to_mumu | +0.76 | −0.01 | +0.03 | | +0.28 | +0.20 | −0.000 | 0.041 |
| ee_to_mumu_4f | +0.15 | −0.87 | −0.24 | | +0.06 | +0.04 | −0.032 | 0.037 |
| ee_to_mumu_smlimit | +0.24 | −0.12 | +0.13 | | +0.34 | +0.54 | −0.005 | 0.039 |
| ee_to_mumu_tata_qcd0 | +1.10 | **+1.82** | **+0.23** | | +0.88 | −0.38 | +0.274 | 0.150 |
| ee_to_mumua | −0.42 | −0.96 | −1.11 | | −0.88 | −0.98 | −0.076 | 0.079 |
| ee_to_tatah | −0.08 | +0.97 | +0.79 | | +0.77 | +0.86 | +0.067 | 0.069 |
| ee_to_ttx | +1.21 | +0.41 | +0.17 | | +0.05 | −0.09 | +0.016 | 0.038 |
| ee_to_ttx_dipole | +0.55 | −0.47 | −0.72 | | −1.09 | −1.21 | −0.016 | 0.033 |
| ee_to_ttx_smeft | +0.60 | +0.30 | +0.07 | | +0.90 | +0.72 | +0.005 | 0.016 |
| ee_to_ttx_smlimit | +0.52 | −0.56 | −0.79 | | −1.01 | −1.18 | −0.018 | 0.032 |
| ee_to_wpwm | +0.49 | +0.50 | +0.05 | −1.02 | +1.32 | +1.36 | +0.037 | 0.074 |
| ee_to_wpwm_cw | −0.16 | +0.39 | −0.04 | −1.10 | +1.41 | +1.39 | +0.029 | 0.075 |
| ee_to_zh | −0.83 | −0.07 | −0.19 | | +0.53 | +0.74 | −0.002 | 0.029 |
| ee_to_zh_smeft | +0.61 | +0.29 | +0.43 | | +0.53 | +0.28 | +0.004 | 0.012 |
| gg_to_gg | +0.27 | +1.46 | +1.44 | | +1.38 | +0.98 | +0.093 | 0.063 |
| gg_to_ttx | +0.66 | −0.01 | +0.16 | | −0.52 | −0.72 | −0.000 | 0.057 |
| gg_to_ttx_smlimit | +1.92 | +0.66 | +0.80 | | +0.46 | +0.12 | +0.041 | 0.062 |
| gg_to_ttx_smlimit_qcd2 | +1.82 | +0.66 | +0.51 | | −0.35 | −0.91 | +0.042 | 0.063 |
| gu_to_epemu | +0.16 | +0.46 | +0.26 | | +0.14 | −0.62 | +0.045 | 0.098 |
| gux_to_epemux | +0.60 | +1.19 | +1.26 | | −0.21 | −0.19 | +0.113 | 0.095 |
| ll_to_qqx_toy_dipole | −0.48 | −0.52 | −0.62 | | +0.85 | +1.05 | −0.014 | 0.027 |
| ll_to_qqx_toy_tensor | +0.48 | +0.97 | +0.84 | | +0.46 | +0.59 | +0.067 | 0.069 |
| ll_to_qqx_toy_yukawa | +0.59 | +0.15 | +0.08 | | +0.28 | +0.29 | +0.002 | 0.013 |
| p3r3_to_p3r3_toy_epsilon | +1.74 | +1.41 | +1.31 | +1.16 | +0.90 | +1.25 | +0.048 | 0.034 |
| p3r3_to_p3r3_toy_sextet | +1.44 | +1.10 | +1.36 | +1.35 | +0.78 | +1.06 | +0.038 | 0.034 |
| qqx_to_o8o8_toy_dcolor | +0.59 | −0.15 | −0.15 | | −1.12 | −1.09 | −0.010 | 0.063 |
| tata_to_ttx_tensor4f | +0.92 | −0.42 | −0.31 | | −0.36 | −0.59 | −0.013 | 0.032 |
| ud_to_epemud_qcd0 | +1.77 | **+1.26** | **+0.63** | | +0.72 | −0.15 | +0.194 | 0.154 |
| uux_to_epemg | +2.45 | **+2.83** | **+0.81** | | +2.19 | +1.00 | +0.301 | 0.106 |
| uux_to_mumu | +1.43 | +0.20 | +0.21 | | +0.31 | +0.35 | +0.010 | 0.048 |
| uux_to_ttx_4f | −0.30 | −1.88 | −2.24 | | −1.82 | −2.18 | −0.025 | 0.013 |
| uux_to_uux | −0.55 | +0.62 | +0.48 | | +0.56 | +0.10 | +0.032 | 0.051 |

The 34 swept rows, single seed against five seeds (**[P]** `--this-side`):

| reference | cell (1 seed) | 5 seeds | 5 seeds, clustered se |
|---|--:|--:|--:|
| R | +0.607 ± 0.145 | **+0.410** ± 0.163 | ± 0.172 (+2.4) |
| EW | +0.447 ± 0.130 | +0.169 ± 0.127 | ± 0.142 (+1.2) |
| EW, median on heavy rows | +0.386 ± 0.143 | +0.102 ± 0.136 | ± 0.156 (+0.7) |
| R4x | +0.538 ± 0.131 | +0.339 ± 0.161 | ± 0.182 (+1.9) |
| EW4x | +0.392 ± 0.123 | +0.096 ± 0.148 | ± 0.175 (+0.5) |

| swept class, 5 seeds | N | R | EW |
|---|--:|--:|--:|
| fixed-beam SM | 17 | +0.765 ± 0.235 | +0.314 ± 0.141 |
| toy/SMEFT | 17 | **+0.054** ± 0.229 (cl.) | +0.024 ± 0.249 (cl.) |

On the swept rows the correlation of the R pull with the EW − R shift rises
from +0.37 (single seed) to **+0.60** (five seeds).

**The whole population, the swept rows read over five seeds** (the grammar and
hadronic rows were already 10 and 5 seeds):

| reference | N | mean pull | clustered se | positive |
|---|--:|--:|--:|--:|
| R | 48 | **+0.475** | 0.148 (+3.2) | 32 |
| EW | 48 | **+0.285** | 0.128 (+2.2) | 34 |
| EW, median on heavy rows | 48 | +0.209 | 0.137 (+1.5) | 33 |
| R4x | 48 | +0.374 | 0.145 (+2.6) | 35 |
| EW4x | 48 | **+0.110** | 0.140 (+0.8) | 29 |

| relative offset | unweighted | weighted | χ²/dof |
|---|--:|--:|--:|
| R | +0.0523 % ± 0.0126 % | **+0.0034 % ± 0.0050 %** | 1.13 |
| EW | +0.0409 % ± 0.0124 % | +0.0006 % ± 0.0051 % | 0.71 |
| R4x | +0.0291 % ± 0.0094 % | +0.0054 % ± 0.0047 % | 0.91 |
| EW4x | +0.0036 % ± 0.0078 % | +0.0012 % ± 0.0047 % | 0.70 |

| class | N | R | EW | R4x | EW4x |
|---|--:|--:|--:|--:|--:|
| fixed-beam SM | 17 | +0.765 ± 0.235 | +0.314 ± 0.141 | +0.606 ± 0.227 | +0.183 ± 0.162 |
| toy/SMEFT | 18 | +0.060 ± 0.216 | +0.039 ± 0.235 | +0.070 ± 0.233 | +0.017 ± 0.273 |
| grammar | 3 | −0.129 ± 0.771 | +0.459 ± 0.665 | +0.347 ± 0.575 | +0.018 ± 0.500 |
| hadronic | 10 | +0.911 ± 0.290 | +0.627 ± 0.302 | +0.535 ± 0.280 | +0.183 ± 0.301 |

(Errors are the clustered se.)

| EW − R | N | R | EW |
|---|--:|--:|--:|
| resolved | 9 | +0.649 ± 0.510 | −0.180 ± 0.312 |
| not resolved | 39 | +0.435 ± 0.143 | +0.392 ± 0.134 |

### The hadronic rows: `pp_to_bb` on five fresh seeds

The hadronic rows already read five seeds on this side, and the lean there
survives EW. The `pp_to_bb` family carries most of it. Its rows share their
seed list: `pp_to_bb` and `pp_to_bb_qcd2` run `RECARDED_SEEDS` and also draw
the same MadEvent fluctuations.

I ran the three rows on five **fresh** seeds, 20261201–05. This used
`probe_hadronic_seed_headroom` with a throwaway, uncommitted edit of its
`HEADROOM_ARMS` to these three arms and seeds; the edit was reverted, and `git
diff --stat` is empty. Result: `test result: ok. 1 passed … finished in
115.91s`. The probe's own lines:

| row | fresh 5 seeds σ (pb) | χ²/dof | rel vs R | pull vs R | gate's 5 seeds: rel, pull |
|---|---|--:|--:|--:|---|
| pp_to_bb_fixed | 2.149227e6 ± 1.277e3 | 0.58 | +0.216 % | +2.55 | +0.162 %, +1.92 |
| pp_to_bb | 3.763044e8 ± 1.265e5 | 0.77 | +0.071 % | +1.51 | +0.104 %, +1.92 |
| pp_to_bb_qcd2 | 3.763191e8 ± 1.360e5 | **0.03** | +0.078 % | +1.60 | +0.083 %, +1.68 |

**[P]** over all ten seeds on this side (gate's five + fresh five):

| row | R | EW | R4x | EW4x | rel R % | ± % |
|---|--:|--:|--:|--:|--:|--:|
| pp_to_bb | +2.11 | +1.60 | +1.68 | +1.39 | +0.088 | 0.042 |
| pp_to_bb_fixed | +2.57 | +2.20 | +1.27 | +1.59 | +0.189 | 0.073 |
| pp_to_bb_qcd2 | +1.93 | +1.51 | +1.28 | +1.02 | +0.081 | 0.042 |

Two independent five-seed sets on this side agree with each other on every
`bb` row. **The `bb` excess is not this side's draw.** EW (+0.015 % to
+0.018 % on these records) does not remove it, and it is not the iteration
weighting.

One piece of evidence against a this-side offset, from the previous sprint's
V1 (`sigma-reference-scatter`, step 2): six independent default-survey
MadEvent draws had `pp_to_bb` at 376.27e6 ± 0.13e6 and `pp_to_bb_fixed` at
2146600 ± 1600. This side's ten-seed means sit +0.01 % and +0.09 % above
those. refdata-10's `hard_survey = 2` five-seed means sit 0.06 % and 0.09 %
below them. Those are different survey settings, so this is not a
combinable measurement, but it is the same size as the excess. **The `bb`
rows are open.** The candidates are a low five-seed MadEvent draw, which
twenty seeds would show, and a 0.1–0.2 % effect on either side, which only an
adaptation-free estimate would show.

### The long-tier rows

The three llj MLM gates (`cargo test … --test validate_hadronic -- --ignored
--nocapture --test-threads 1 sigma_llj_xqcut_only_vs_madevent sigma_llj_mlm`:
`test result: ok. 3 passed … finished in 1339.53s`), at d4b8bb3, which
includes #23's partonic-CM MLM clustering:

| row | this side | n | R | EW | EWmed | R4x | EW4x | rel R % | ± % | MG var |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| pp_to_llj_mlm | 2.684138e2 ± 1.193e-1 | 10 | +0.46 | +0.70 | | +0.41 | −0.59 | +0.104 | 0.226 | 0.96 |
| pp_to_llj_mlm_alps2 | 2.412404e2 ± 1.531e-1 | 5 | +1.36 | +0.85 | +0.06 | +1.49 | +0.30 | +0.309 | 0.227 | 0.92 |
| pp_to_llj_xqcut_only | 2.126082e2 ± 1.259e-1 | 5 | +0.66 | +0.20 | −0.91 | −0.96 | −1.07 | +0.143 | 0.217 | 0.93 |

MadEvent carries over 90 % of these pulls' variance, so they read the
reference. Its budget shift is EW4x − EW +0.25 % ± 0.03 % unweighted over
the three. It is the same family, so this is not three independent rows. It
is the size of the previous sprint's 20-seed finding that the llj MLM offset
was the reference's.

Left out:
- `pp_to_ll_0j2j_mlm`: one seed took 12 min on this host (10 seeds ≈ 2 h,
  past the session's compute budget). The run was stopped by PID.
- `pp_to_ttx_0j1j_mlm` (`info`), and the two 2 → 6 rows (`info`, already
  flagged by the scatter rule).

All three MLM rows are in the combined population below.

### Combined: 51 rows, every this-side improvement applied

This combines the 48 default-tier rows, the 3 llj MLM rows, the fixed-beam
sweep and the `bb` ten-seed reads (**[P]** `--this-side`, sweep + `bb` ten
seeds):

| reference | N | mean pull | clustered se (44) | positive |
|---|--:|--:|--:|--:|
| R | 51 | +0.517 | 0.147 (+3.5) | 35 |
| EW | 51 | +0.318 | 0.126 (+2.5) | 37 |
| EW, median on heavy rows | 51 | +0.209 | 0.136 (+1.5) | 35 |
| R4x | 51 | +0.388 | 0.144 (+2.7) | 37 |
| EW4x | 51 | **+0.092** | 0.139 (+0.7) | 30 |

| relative offset | unweighted | weighted | χ²/dof |
|---|--:|--:|--:|
| R | +0.0603 % ± 0.0131 % | +0.0046 % ± 0.0049 % | 1.19 |
| EW | +0.0485 % ± 0.0128 % | +0.0014 % ± 0.0050 % | 0.74 |
| EW4x | −0.0015 % ± 0.0087 % | +0.0017 % ± 0.0047 % | 0.73 |

| class | N | R | EW | R4x | EW4x |
|---|--:|--:|--:|--:|--:|
| hadronic | 10 | +1.020 ± 0.342 | +0.704 ± 0.335 | +0.625 ± 0.306 | +0.257 ± 0.330 |
| MLM | 3 | +0.826 ± 0.273 | +0.585 ± 0.197 | +0.313 ± 0.710 | −0.453 ± 0.401 |

## 5. Attribution

Over the 48 default-tier rows, each step removes one cause and is measured
above. The errors are clustered se.

| step | mean pull | change | what it removes |
|---|--:|--:|---|
| as the gates measured (R) | +0.615 ± 0.134 | | |
| five seeds on this side on the 34 fixed-beam rows | +0.475 ± 0.148 | −0.14 | this side's one shared seed |
| read against EW | +0.285 ± 0.128 | −0.19 | MadEvent's mean²/variance iteration weighting |
| read against EW at 4× | +0.110 ± 0.140 | −0.18 | MadEvent's budget (adaptation) dependence, at screen grade |
| 51 rows, plus llj MLM and `bb` on ten seeds, EW4x | +0.092 ± 0.139 | | |

**1. About a quarter (−0.14) is this side's fixed seed. It is luck, not
bias.** All 34 fixed-beam rows ran one seed, 20260719. Over five seeds:

- every row's χ²/dof is in band (0.40–2.77 on 4 dof);
- the toy/SMEFT lean goes from +0.66 to +0.05;
- individual pulls move by up to 1.6: `uux_to_ttx_4f` −0.30 → −1.88,
  `ee_to_ee` −1.16 → −0.04, `gg_to_ttx_smlimit` +1.92 → +0.66.

The iteration combination on this side is unweighted after a two-iteration
warm-up (`vegas.rs`, `IterationCombination::Unweighted`), so it has no
mechanism for a sign-definite bias, and the weighted offset says none is
there. Read over five seeds, σ_vg/σ_R − 1 weighted by inverse variance is
**+0.0034 % ± 0.0050 %** (48 rows); with all 51, it is +0.0046 % ± 0.0049 %.
**A common shift of σ on this side is bounded below about 0.013 % (2σ).**

**2. About a third (−0.19) is MadEvent's iteration weighting, on four
multi-leg fixed-beam rows.** On the nine rows where the record's paired EW −
R is resolved (|z| > 2), the five-seed R pull is +0.65 and the EW pull −0.18.
On the 39 where it is not, EW moves nothing (+0.44 → +0.39). On the 34 swept rows,
the correlation of the pull with the EW − R shift rises from +0.37 to +0.60
once this side's seed noise is averaged down (+0.27 to +0.40 over all 48).

The rows doing the work are `ddx_to_epemg`, `uux_to_epemg`,
`ee_to_mumu_tata_qcd0`, and `ud_to_epemud_qcd0` (less resolved). Five-seed R
pull → EW pull: +2.37 → 0.00, +2.83 → +0.81, +1.82 → +0.23, +1.26 → +0.63.
Their paired EW − R is +0.22 % (3.5σ), +0.20 % (3.5σ), +0.23 % (2.2σ) and
+0.07 % (1.1σ). This is the size and the family of #22's fixed-beam
`u u~ > e+ e- g g` (−0.1 to −0.2 %), a bias the concept
`madevent-iteration-weighting-bias` already records.

**3. About a third (−0.18) moves with MadEvent's budget, and two seeds can
only screen it.** The reference rises with budget in the imprecise rows:

- R4x − R is +0.030 % ± 0.010 % unweighted and −0.002 % ± 0.002 % weighted;
- EW4x − EW over the three llj MLM rows is +0.25 % ± 0.03 %.

That is the direction of an adaptation bias that shrinks with points per
iteration. Two 4× seeds per row cannot locate it row by row, and cannot
calibrate it: the ladder rule asks for 20 + 20.

**4. What is left (+0.11 ± 0.14, or +0.09 ± 0.14 with 51 rows) is consistent
with zero.** The largest residuals after all three steps are listed below.
Among 51 pulls of unit width, a few at 1–1.6 are expected. The `bb` family is
the exception, because it reproduced on independent seeds.

- the `bb` family, ten seeds this side, EW4x pulls +1.39 (`pp_to_bb`), +1.59
  (`_fixed`), +1.02 (`_qcd2`);
- the `p3r3` toys, +1.25 and +1.06 (one cluster);
- `uux_to_epemg`, +1.00;
- `gg_to_gg`, +0.98.

**Verdict.** The lean is reference-side wherever it is resolved: about 0.37 of
the 0.615. That part is MadEvent's mean²/variance weighting together with its
budget dependence. About 0.14 is this side's sampling luck through one shared
fixed seed. This side carries no sign-definite offset. The `bb` rows are the
one open residual, and they are not explained by EW.

**What would falsify it.**

- **Step 2.** An adaptation-free estimate of `uux_to_epemg` at MadEvent's
  reported mean, 0.555054 ± 0.000448 pb, rather than its EW mean, 0.556177 ±
  0.000559, or this side's five-seed 0.556723.
- **Step 3.** A 20 + 20 ladder on the multi-leg rows in which neither R nor EW
  rises with budget.
- **Step 1.** Five further fresh seeds on the fixed-beam rows that put the
  population back near +0.6. It is a measurement over five seeds that include
  the gate's, so this would mean the five are unrepresentative.

### Where an adaptation-free estimate would decide (not built)

The lean survives EW at +0.285 ± 0.128 (2.2 clustered se), and survives EW4x
only within its error. Two estimates would close the open parts:

1. **`uux_to_epemg`, with `ddx_to_epemg` as its free control**: fixed-beam
   2 → 3. #22's fixed-density estimator for `u u~ > e+ e- g g` (linked to
   MadGraph's standalone `matrix.o`) loses one gluon. About half a session,
   and under an hour of CPU here at #22's 40 × 5M-point batches. It decides
   step 2 on the rows that carry it, and whether EW should be validated for
   that family.
2. **`pp_to_bb_fixed`, with `pp_to_bb` as control**: hadronic 2 → 2. It needs
   fixed densities in (τ, y, cos θ) with a 1/(t − m_b²) component, this side's
   |M|² and the LO PDF set. About one session; minutes of CPU. Do it only if
   the banking request's twenty seeds do not already put refdata-10's
   five-seed mean as a low draw.

`ee_to_mumu_tata_qcd0` (2 → 4, 28 channels) would be a third, but it costs
much more than either and its EW − R is only 2.2σ.

### Proposal for the manager (a design decision for the user)

- **Do not read EW on more rows yet.** On the 42 rows without a resolved EW
  − R, EW changes the population by −0.06. On the seven heavy-tailed rows it
  is noisier than R: `ee_to_wp0wm`'s EW χ²/dof is 9.6, with spread 5.35× R's.
  D1's first amendment, which says EW only where an adaptation-free check
  validated it, is supported by this data. If estimate 1 above lands on EW,
  set `validated_ew` on `uux_to_epemg`, `ddx_to_epemg` and
  `ee_to_mumu_tata_qcd0`. That is three record flags, and those rows' R pulls
  of +2.4 to +2.8 then fall to 0.0–0.8.
- **Gate the fixed-beam σ rows on five seeds on this side**, as the hadronic
  (5) and grammar (10) rows already are. One seed shared by 34 rows put +0.14
  on the population mean, and moved single rows by up to 1.6σ. Five seeds
  took 753 s on this host against 275 s for the whole `validate_sigma` suite.
  This is a gate change, so it is out of this session's scope.

### Banking request

These are diagnostics for the user on the M3, with `bank_seeded_sigma.sh` as
for refdata-10: fresh directories, `hard_survey = 2`, both values recorded.

1. `uux_to_epemg` and `ddx_to_epemg`: **20 seeds at 10 000 events and 20 at
   40 000**. Expected output: two seed records of 40 seeds each, and the
   fourth amendment's ladder verdict on R and on EW (measured-spread σ). It
   decides step 3 on the rows that carry step 2. At the refdata-10 wall
   times (20–27 s at 1×, 41–61 s at 4×) this is about 1 h for both rows.
2. `pp_to_bb` and `pp_to_bb_fixed`: **20 seeds at 10 000 events** (4× is
   optional). Expected output: whether refdata-10's five-seed means
   (376036000 ± 124491 pb; 2144600 ± 1291 pb) are a low draw against twenty.
   This side's ten-seed means are 376.30e6 and 2.14865e6. At 33–51 s a run,
   this is about 30 min.

Nothing here is to be banked until the user decides.

### Compute spent

This-side compute on this host:

| run | time |
|---|--:|
| **[G]** `validate_sigma` | 275 s |
| **[G]** `validate_hadronic` | 548 s |
| **[S]** sweep | 753 s |
| llj MLM | 1340 s |
| `bb` fresh seeds | 116 s |
| `pp_to_ll_0j2j_mlm`, stopped after one seed | ~12 min |
| total | about 1 h 3 min |

Left out: `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`, the 2 → 6 rows, and
five-seed sweeps of the grammar and hadronic rows beyond `bb`, which already
read 10 and 5 seeds.

## The tool

`validation-report/src/bin/sigma-pulls.rs`, a third binary of the
`vibegraph-validation-report` crate beside `seed-record`. It includes
`seeds.rs` and `seed_record.rs` by path, as `seed-record` does. It is a
sibling binary and not a `seed-record` subcommand because the sibling PR
`seed-record-tooling` is editing `seed-record.rs`. A separate file cannot
conflict with that, and it shares the same record reader.

It is not an `#[ignore]`d probe because a probe would integrate again. The
population is a reading of measurements that already exist: the row files any
banked run writes, including CI's `validation-report` artifact. Regenerating
the population on a later commit therefore costs nothing beyond the banked run
itself:

```text
cargo run -p vibegraph-validation-report --bin sigma-pulls -- \
  --report target/validation-report --output validation/madgraph/output \
  --manifest validation/manifest.toml [--this-side sweep.tsv]
```

What it does:

- It joins each σ row file to the record whose gate reading equals the file's
  MadEvent value, to 1e-12 relative, over totals and every `by_lprup` part. It
  does not join by name, because the hadronic cells do not name their
  reference (`hadronic-sigma-cells-unnamed-reference`).
- It refuses to run if the reported re-read does not reproduce the gate's
  pull to 1e-9.
- It lists the row files no record reproduces and the records no row file
  read.
- It prints the per-row table, the populations (naive and clustered se), the
  relative means, the reference-against-itself shifts and the splits. With
  `--this-side`, it prints them again with the swept rows read over their
  seeds.

Hermetic tests: 7, all pass (`cargo test -p vibegraph-validation-report --bin
sigma-pulls`: `test result: ok. 7 passed`). They cover the value join, the
reported re-read reproducing a pull, the EW and 4× reads, same-draw
clustering, the clustered standard error against a hand computation, the
weighted relative mean, the class rules, and the sweep-file parser.

Hermetic gates at the commit:

- `cargo build --workspace`: finished.
- `cargo test --workspace`: 40 suites, 1485 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `pixi run kb-lint`: 0 errors. The base's sprint indexes were stale (no
  `sessions/index.md`), and `pixi run kb-index` regenerated the two.

**The clustering's blind spot.** It sees MadEvent's shared draws only.
Rows on this side that share an RNG seed and a near-identical integrand are
also correlated, and nothing here detects that. Every fixed-beam row runs
`SEED`; `pp_to_bb`/`pp_to_bb_qcd2`/`pp_to_llj`/`pp_to_ll_scalefact2` share
`RECARDED_SEEDS`. So the clustered errors are lower bounds where such twins
exist. The `pp_to_bb` pair is already one cluster through MadEvent's draws.

## Found

- **The backlog's "47 pulls" are read off the report's integrals column,
  which mixes references.** The column shows the worst |pull| over a row's
  variants. For `ee_to_ee_nsz` and `ee_to_wp0wm` that is an unrestricted or
  transverse control read against `grammar_sigma_reference.json`, not a seed
  record (+1.63 and +0.77 against the rows' own +1.09 and +0.07). To close it:
  read population statistics with `sigma-pulls`, which joins each cell to the
  record it read, and say so wherever the report column is quoted as a
  population.
- **The fixed-beam σ gate's one shared seed biases population reads.** Every
  fixed-beam row runs `SEED = 20260719`. Five seeds took the 34 rows' mean
  pull from +0.61 to +0.41, and single rows by up to 1.6. To close it: a
  decision on gating those rows on five seeds (see the proposal); cost 753 s
  here.
- **`pp_to_bb`, `pp_to_bb_fixed` and `pp_to_bb_qcd2` read +0.08 %, +0.19 %
  and +0.08 % above refdata-10 on two independent five-seed sets**. Ten-seed
  pulls are +2.11, +2.57 and +1.93 (R), and +1.39, +1.59 and +1.02 (EW4x).
  EW − R is about +0.015 %, so it is not the iteration weighting. To close it:
  the banking request's twenty MadEvent seeds, then an adaptation-free
  estimate if those do not move the reference.
- **`pp_to_bb_qcd2`'s five fresh seeds scatter 5× below their quoted
  errors**: χ²/dof 0.03 (spread 2.524e4 against quoted 1.360e5). Over ten
  seeds χ²/dof is 0.27, and the gate's own five read 0.58. A χ²/dof far below
  one is a warning, per AGENTS.md: either the quoted error of this side's
  unweighted iteration mean overstates the spread on this row, or the seeds are
  correlated. To close it: twenty seeds of the row, and a check of its
  per-iteration variance against the iteration-to-iteration scatter.
- **`probe_gate_row_seed_headroom` prints pulls against `sigma_reference.json`,
  the pre-refdata-10 single-run bank, not against the seed records the gate
  reads.** `ee_to_ttx` seed 20260719 prints pull +0.96 against MG 5.4866e-1,
  where the gate reads +1.21 against 5.48588e-1. Its rel and pull columns and
  its HEADROOM census are therefore stale. To close it: read the reference
  through `madevent_reference` as `gate_dir` does.
- **`pp_to_ll_0j2j_mlm`'s σ gate costs about 12 min per seed on a 4-core
  host**, so about 2 h for its ten seeds. `validate-mlm-sigma` is not
  runnable inside a dev session here, and its rows went unmeasured on this
  commit. To close it: record the per-host cost beside the task, or give the
  row a cheaper diagnostic budget.

## The brief

- "The pulls are one fixed seed per row" holds for the 34 fixed-beam rows only.
  The grammar rows read 10 seeds on this side, and the hadronic rows 5.
- The "47 σ pulls" are report cells, not seed-record measurements. Three show a
  variant other than the row's own, and two of those read a non-record
  reference. The seed-record population is 48 (51 with the llj MLM rows).
  The report's 47 reproduce exactly: mean +0.651, sd 0.884 (the brief has
  0.87).
- "The hadronic/MLM σ gates that read refdata-10 records" includes the MLM
  rows, which are long tier and were not in the 47. Three of the five were
  measured here.
