---
type: Session Report
title: "F1 report: the fixed-beam σ rows gate on five seeds"
description: "The 34 Plan::Gate rows of validate_sigma now read the unweighted mean of HEADROOM_SEEDS against their seed records. All 34 pass with limits unmoved. The gate's own mean pull over them is +0.410 ± 0.163, and the 48-row population is +0.475 (clustered se 0.148). Both reproduce V1's sweep exactly."
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources: [6ab4e04]
---
Scope: [F1](F1.md), the design [D1](../decisions/D1-fixed-beam-seeds-and-ew.md)
point 1. One code commit, `6ab4e04`, touches only
`vibegraph-lib/tests/validate_sigma.rs`. The report commit adds this file and, by
`pixi run kb-index`, its one line in `sessions/index.md`. The logs are in the session
scratchpad as `sigmalean-F1-*` (directory `/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad`).

## What changed

- **The gate (`gate_dir`).** A `Plan::Gate` row is integrated on each seed of
  `HEADROOM_SEEDS = [SEED, 11, 22, 33, 44]`, in that order, through the same
  `integrate_reported` as before. The seeds are read with `combine` (the seed
  policy: unweighted mean, `err = max(quoted, spread/√n)`). The pull and the
  rel are taken against the row's seed record through
  `BankedSigma::madevent_reference`, exactly as before. `PULL_LIMIT = 3.5`,
  every `rel_tol` and `PULL_REPORTED_NOT_ASSERTED` are untouched.
- **What the cell carries.** It holds all five `per_seed` entries and `seeds`.
  Its `chi2_dof` is the seeds' χ²/dof about their mean, which is how the
  report schema defines it for a sweep. The subsampler summary is the gate
  seed's. Each seed's own line is printed, and the GATE line now names its
  seed count and labels the column `seed chi2/dof`. Nothing new is asserted
  on the χ²/dof.
- **Rows that stay on one seed.** `Plan::Info` (`gg_to_gg_cg`) and `Plan::Long`
  (the two 2 → 6 rows) still run on `SEED` alone. The brief's scope is the 34
  gate rows. See **Found**.
- **Seed-count guards.**
  - `gate_dir` asserts that a gated row was read on at least `MIN_GATE_SEEDS`
    (5) seeds, with `SEED` first.
  - A compile-time `const` assert pins `MIN_GATE_SEEDS >= 5` and
    `HEADROOM_SEEDS.len() >= MIN_GATE_SEEDS`.
  - The hermetic test `every_gated_row_is_read_on_five_seeds` checks the
    plan-to-seeds map for every row in `sigma_reference.json`: gated rows get
    at least five distinct seeds with `SEED` first, and all other rows get
    `[SEED]`.
- **The probe.** `probe_gate_row_seed_headroom` reads each row's reference
  through `madevent_reference`, the gate's own reader. Its header names the
  source, and its census line adds the five seeds' mean pull.
- **Doc comments.** The module's "Statistical gate" section, `SEED`,
  `HEADROOM_SEEDS` (moved beside `SEED`), the probe's doc, and the 2 → 6 plan
  comment now state the seed count. The per-row calibration figures in
  `plan_for` and in `PULL_LIMIT`'s doc are untouched, as the brief says.

## Mutation: a gate that goes back to one seed fails

Mutation: `plan_seeds` returns `&[SEED]` for `Plan::Gate`. The file was
restored from a saved copy afterwards.

- `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --exact every_gated_row_is_read_on_five_seeds`
  gives `[ddx_to_epemg] gated on [20260719]` and `test result: FAILED. 0 passed; 1 failed`
  (`sigmalean-F1-mut1.log`).
- The same mutation under `-- --exact sigma_gate_matches_madgraph --nocapture`
  integrates one seed of `ddx_to_epemg` and then panics:
  `[ddx_to_epemg] a gated row is read on 5 seeds or more, the gate's seed first; it was read on [20260719]`,
  `test result: FAILED … finished in 4.48s` (`sigmalean-F1-mut1-gate.log`).
- The unmutated hermetic test passes: `test every_gated_row_is_read_on_five_seeds ... ok`.

## Gates on `6ab4e04`

The host was a 4-core container shared with two sibling Monte Carlo sessions,
at load average 8 to 14 throughout. Wall times are therefore loaded-host
figures, not a cost model.

**Pass 1: `pixi run --skip-deps validate`.** It started at
2026-10-17T01:47:48Z and exited with 101 at 04:13:28Z
(`sigmalean-F1-validate.log`, `sigmalean-F1-status.log`).

- The extended-feature `cargo clippy … -D warnings` it runs first was clean
  (`Finished dev profile … in 4m 18s`, with no warning or error lines).
- 49 test suites reported `ok`, among them:

  | suite | result | time |
  |---|---|--:|
  | `validate_hadronic` | `14 passed; 0 failed; 22 ignored` | 1044.24 s |
  | `validate_samples_proton` | `11 passed` | 1549.28 s |
  | `validate_madgraph_diagrams` | `57 passed` | — |
  | `amplitude_oracle` | `49 passed` | — |
  | `validate_lhef` | `3 passed` | — |
  | `cli_onshell_veto` | `4 passed` | 632.81 s |

- **`validate_pdf_grid` failed: `7 passed; 13 failed`.** All 13 failures panic
  with `failed to load PDF set at …/validation/pdf/NNPDF31_lo_as_0130: … No such file or directory`
  (13 of 13 panics). That is a missing input in this worktree, not this
  change: only `NNPDF23_lo_as_0130_qed` was copied in. `fetch.sh` refuses to
  download without `VIBEGRAPH_FETCH_CONSENT=1`, and I did not give that
  consent on the user's behalf.
- `cargo test` stops at the first failed suite, so pass 1 never reached
  `validate_samples`, `validate_scale_couplings`, `validate_scales`,
  `validate_sigma`, `validate_unweighting` or `validate_vegas`.

**Pass 2: the suites pass 1 did not reach**, on the same commit, adding to the
same report directory (`sigmalean-F1-rest.log`):

```text
cargo test -p vibegraph-lib --profile release-debug --features extended-validation --no-fail-fast \
  --test validate_sigma --test validate_samples --test validate_scale_couplings \
  --test validate_scales --test validate_unweighting --test validate_vegas -- --nocapture
```

| suite | result | time |
|---|---|--:|
| `validate_samples` | `6 passed; 0 failed; 2 ignored` | 450.17 s |
| `validate_scale_couplings` | `1 passed` | 4.21 s |
| `validate_scales` | `10 passed; 0 failed; 1 ignored` | 176.68 s |
| **`validate_sigma`** | **`8 passed; 0 failed; 31 ignored`** | **1184.72 s** |
| `validate_unweighting` | `1 passed` | 32.06 s |
| `validate_vegas` | `2 passed` | 0.93 s |

`validate_sigma` printed: `sigma gate: 34 process(es) asserted against banked
MadGraph sigma, 0 awaiting the bundle (none), 2 in the oracle layer`.

**The collator.** `cargo run --profile release-debug -p
vibegraph-validation-report` exited 0 (`sigmalean-F1-collate.log`):

- `256 cells: 203 measured in the layers this run drove (197 ✅, 6 ⚠️, 13 ⏳, 40 — / uncovered)`
- `every gate cell passed`
- `the measured cells are the declared cells`

The base's CI report (V1's `ci_report`) has the same tally: 197 ✅, 6 ⚠️,
13 ⏳. The same five rows carry the ⚠️, and they are informational
or amplitude-tolerance cells. Only the fixed-beam integrals readings moved. The
report is copied to `sigmalean-F1-report-final/`.

**Hermetic, lints and fmt** (`sigmalean-F1-status.log`):

| command | result |
|---|---|
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | exit 0: 40 suites, 1498 passed, 0 failed |
| `cargo fmt --all --check` | exit 0 |

The banked layer is green on this commit, except `validate_pdf_grid`. That
suite is not touched by this change and fails here on a missing PDF set.

## The fixed-beam σ rows over five seeds

These are the 34 gate cells as `validate_sigma` wrote them in pass 2, read from
`target/validation-report/integrals/<row>.json` by `sigmalean-F1-table.py`:

- σ_vg ± err is the seed-policy mean.
- Seed χ²/dof is the five seeds' scatter about that mean, on 4 dof.
- σ_MG ± err is the seed record's reading.
- Wall is the row's `duration_s`, covering all five seeds.

| row | status | σ_vg (pb) | ± | seed χ²/dof | σ_MG (pb) | ± | rel | pull | wall (s) |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|
| ddx_to_epemg | pass | 6.204896e-01 | 3.458e-04 | 0.79 | 6.191320e-01 | 4.572e-04 | +2.193e-03 | +2.37 | 43.6 |
| ee_to_ee | pass | 1.557558e+02 | 1.338e-01 | 2.48 | 1.557620e+02 | 4.428e-02 | -3.949e-05 | -0.04 | 16.4 |
| ee_to_mumu | pass | 2.024072e+03 | 7.528e-01 | 0.70 | 2.024080e+03 | 3.389e-01 | -4.052e-06 | -0.01 | 5.7 |
| ee_to_mumu_4f | pass | 7.383640e+00 | 2.306e-03 | 0.48 | 7.386020e+00 | 1.487e-03 | -3.222e-04 | -0.87 | 9.4 |
| ee_to_mumu_smlimit | pass | 4.209301e-01 | 1.323e-04 | 0.46 | 4.209500e-01 | 9.737e-05 | -4.716e-05 | -0.12 | 6.9 |
| ee_to_mumu_tata_qcd0 | pass | 1.378080e-03 | 1.632e-06 | 1.90 | 1.374320e-03 | 1.265e-06 | +2.736e-03 | +1.82 | 65.4 |
| ee_to_mumua | pass | 1.006968e-01 | 7.748e-05 | 0.45 | 1.007732e-01 | 1.871e-05 | -7.579e-04 | -0.96 | 21.5 |
| ee_to_tatah | pass | 1.884990e-03 | 1.019e-06 | 1.47 | 1.883720e-03 | 8.179e-07 | +6.742e-04 | +0.97 | 17.3 |
| ee_to_ttx | pass | 5.486753e-01 | 1.949e-04 | 1.18 | 5.485880e-01 | 7.933e-05 | +1.590e-04 | +0.41 | 6.2 |
| ee_to_ttx_dipole | pass | 5.471693e-01 | 1.547e-04 | 0.77 | 5.472540e-01 | 9.020e-05 | -1.548e-04 | -0.47 | 18.0 |
| ee_to_ttx_smeft | pass | 2.222948e+00 | 2.926e-04 | 0.50 | 2.222840e+00 | 2.031e-04 | +4.880e-05 | +0.30 | 50.2 |
| ee_to_ttx_smlimit | pass | 5.500376e-01 | 1.555e-04 | 0.77 | 5.501360e-01 | 7.904e-05 | -1.788e-04 | -0.56 | 2.5 |
| ee_to_wpwm | pass | 7.196299e+00 | 1.585e-03 | 0.54 | 7.193640e+00 | 5.080e-03 | +3.696e-04 | +0.50 | 7.0 |
| ee_to_wpwm_cw | pass | 7.287796e+00 | 1.616e-03 | 0.40 | 7.285660e+00 | 5.197e-03 | +2.932e-04 | +0.39 | 22.6 |
| ee_to_zh | pass | 5.720837e-02 | 1.604e-05 | 1.28 | 5.720960e-02 | 4.969e-06 | -2.143e-05 | -0.07 | 1.7 |
| ee_to_zh_smeft | pass | 1.410991e-01 | 1.672e-05 | 0.84 | 1.410940e-01 | 5.099e-06 | +3.607e-05 | +0.29 | 16.2 |
| gg_to_gg | pass | 1.429502e+05 | 7.984e+01 | 1.39 | 1.428180e+05 | 4.272e+01 | +9.258e-04 | +1.46 | 19.4 |
| gg_to_ttx | pass | 1.351556e+01 | 2.987e-03 | 0.61 | 1.351560e+01 | 7.089e-03 | -3.322e-06 | -0.01 | 21.2 |
| gg_to_ttx_smlimit | pass | 1.354508e+01 | 4.288e-03 | 1.20 | 1.353960e+01 | 7.132e-03 | +4.048e-04 | +0.66 | 14.0 |
| gg_to_ttx_smlimit_qcd2 | pass | 1.349862e+01 | 4.874e-03 | 1.48 | 1.349300e+01 | 7.020e-03 | +4.162e-04 | +0.66 | 519.8 |
| gu_to_epemu | pass | 1.087792e-01 | 6.767e-05 | 1.12 | 1.087300e-01 | 8.244e-05 | +4.526e-04 | +0.46 | 23.9 |
| gux_to_epemux | pass | 1.087709e-01 | 6.323e-05 | 0.97 | 1.086480e-01 | 8.142e-05 | +1.131e-03 | +1.19 | 24.2 |
| ll_to_qqx_toy_dipole | pass | 7.814920e+01 | 1.885e-02 | 1.23 | 7.816020e+01 | 9.329e-03 | -1.408e-04 | -0.52 | 3.0 |
| ll_to_qqx_toy_tensor | pass | 8.038211e+03 | 5.197e+00 | 2.77 | 8.032840e+03 | 1.945e+00 | +6.687e-04 | +0.97 | 8.9 |
| ll_to_qqx_toy_yukawa | pass | 4.026599e+00 | 5.130e-04 | 0.66 | 4.026520e+00 | 1.523e-04 | +1.967e-05 | +0.15 | 9.9 |
| p3r3_to_p3r3_toy_epsilon | pass | 2.216058e-03 | 6.656e-07 | 0.99 | 2.215000e-03 | 3.474e-07 | +4.775e-04 | +1.41 | 1.6 |
| p3r3_to_p3r3_toy_sextet | pass | 2.414726e-03 | 7.242e-07 | 0.68 | 2.413820e-03 | 3.898e-07 | +3.754e-04 | +1.10 | 1.7 |
| qqx_to_o8o8_toy_dcolor | pass | 8.403635e-01 | 2.369e-04 | 0.98 | 8.404440e-01 | 4.745e-04 | -9.581e-05 | -0.15 | 2.5 |
| tata_to_ttx_tensor4f | pass | 5.549736e-01 | 1.581e-04 | 0.54 | 5.550480e-01 | 7.654e-05 | -1.341e-04 | -0.42 | 11.9 |
| ud_to_epemud_qcd0 | pass | 1.412474e-02 | 1.005e-05 | 0.65 | 1.409740e-02 | 1.922e-05 | +1.940e-03 | +1.26 | 53.8 |
| uux_to_epemg | pass | 5.567234e-01 | 3.852e-04 | 1.25 | 5.550540e-01 | 4.480e-04 | +3.008e-03 | +2.83 | 21.7 |
| uux_to_mumu | pass | 7.673951e-02 | 3.062e-05 | 1.18 | 7.673200e-02 | 2.071e-05 | +9.794e-05 | +0.20 | 2.3 |
| uux_to_ttx_4f | pass | 1.390811e+01 | 1.794e-03 | 0.64 | 1.391160e+01 | 4.732e-04 | -2.510e-04 | -1.88 | 40.1 |
| uux_to_uux | pass | 2.827152e+04 | 1.059e+01 | 0.92 | 2.826260e+04 | 9.616e+00 | +3.157e-04 | +0.62 | 10.7 |

`rows 34; summed row wall 1101 s; mean pull +0.410 sd 0.953 se 0.163; positive
21; |pull|max 2.83`. Seed χ²/dof runs from 0.40 to 2.77, and no row is outside
0.13–3.1.

- **No row newly fails.** All 34 are `pass`. The closest to a limit is
  `uux_to_epemg`, at pull +2.83 (1.24× inside 3.5) and rel +3.01e-3 against
  `rel_tol` 0.01. Next is `ddx_to_epemg`, at +2.37 (1.48×). These are the two
  rows whose EW − R is resolved, MadEvent's iteration weighting (D1 point 2,
  V2), so their margin hangs on V2's outcome.
- **The readings are bit-for-bit V1's sweep.** Every per-seed value equals
  V1's `probe_gate_row_seed_headroom` log. For example,
  `gg_to_ttx_smlimit_qcd2` seed 22 is `1.348506e1 ± 8.905e-3` in both. All 34
  five-seed pulls equal V1's "R" column. Five seeds took the 34 rows' mean
  pull from +0.607 (one seed, V1) to **+0.410**.
- **Cost.** `validate_sigma` took 1184.72 s, against V1's 274.76 s on the
  single-seed base. D1 estimated about 753 s on an unloaded host. This run's
  load makes it no test of that figure. `gg_to_ttx_smlimit_qcd2` alone is
  519.8 s of the 1101 s summed over rows (see **Found**).

## The population re-read

```text
cargo run -p vibegraph-validation-report --bin sigma-pulls -- --report target/validation-report \
  --output validation/madgraph/output --manifest validation/manifest.toml
```

The output is `sigmalean-F1-pop.txt`, run with no `--this-side`. The fixed-beam
cells now carry five seeds themselves (`n 5`).

| reference | N | mean pull | clustered se | positive |
|---|--:|--:|--:|--:|
| R | 48 | **+0.475** | 0.148 over 42 clusters (+3.2) | 32 |
| EW | 48 | +0.285 | 0.128 (+2.2) | 34 |
| EW, median on heavy rows | 48 | +0.209 | 0.137 (+1.5) | 33 |
| R4x | 48 | +0.374 | 0.145 (+2.6) | 35 |
| EW4x | 48 | +0.110 | 0.140 (+0.8) | 29 |

| class (R) | N | mean | se |
|---|--:|--:|--:|
| fixed-beam SM | 17 | +0.765 | 0.235 |
| toy/SMEFT | 18 | +0.060 | 0.216 (cl.) |
| grammar | 3 | −0.129 | 0.771 |
| hadronic | 10 | +0.911 | 0.290 (cl.) |

The relative offset is R unweighted +0.0523 % ± 0.0126 % and weighted
+0.0034 % ± 0.0050 % (χ²/dof 1.13). The correlation of the R pull with the
EW − R shift is +0.396 over 48 rows.

Each figure equals V1's "whole population, the swept rows read over five
seeds" table, which V1 built by substituting its sweep with `--this-side`.
The gate now measures what V1 predicted. The population lean that is left,
+0.475 at +3.2 clustered se, is V1's attribution: about 0.19 from the
iteration weighting on the multi-leg rows (V2), about 0.18 from budget
movement, and `pp_to_bb`. The tool still reports one row with n = 1,
`gg_to_gg_cg (info)`.

## The probe now reads the gate's reference

```text
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_sigma -- --ignored --nocapture --exact probe_gate_row_seed_headroom
```

Result: `test result: ok. 1 passed … finished in 1000.78s`
(`sigmalean-F1-headroom.log`).

- **`ee_to_ttx`, the case V1 found.** The header now reads
  `MG 5.485880e-1 ± 7.93e-5 … MadEvent side: 5 reported seeds of seeded_sigma/ee_to_ttx/seed_record.json under the seed policy`.
  Its gate seed reads `pull +1.21`, which is the gate's value. Before, it read
  +0.96 against `sigma_reference.json`'s 5.4866e-1.
- **The probe matches the gate on every row.** Over all 34 rows, its census
  mean pull and χ²/dof equal the gate cells' to print precision (0
  mismatches).
- **Per-seed headroom against the gate's reference.** This is the input PR #45
  re-records from. These figures are not edited into the code:

| row | worst seed \|rel\| | rel_tol / worst | worst seed \|pull\| | PULL_LIMIT / worst |
|---|--:|--:|--:|--:|
| ddx_to_epemg | 3.969e-3 | 2.5x | 2.72 | 1.3x |
| ee_to_ee | 2.970e-3 | 3.4x | 2.09 | 1.7x |
| ee_to_mumu | 7.964e-4 | 6.3x | 0.94 | 3.7x |
| ee_to_mumu_4f | 8.409e-4 | 5.9x | 1.15 | 3.0x |
| ee_to_mumu_smlimit | 5.806e-4 | 8.6x | 0.78 | 4.5x |
| ee_to_mumu_tata_qcd0 | 5.383e-3 | 3.7x | 2.33 | 1.5x |
| ee_to_mumua | 2.361e-3 | 12.7x | 1.47 | 2.4x |
| ee_to_tatah | 2.254e-3 | 8.9x | 2.07 | 1.7x |
| ee_to_ttx | 9.030e-4 | 5.5x | 1.21 | 2.9x |
| ee_to_ttx_dipole | 9.749e-4 | 5.1x | 1.48 | 2.4x |
| ee_to_ttx_smeft | 3.032e-4 | 6.6x | 0.98 | 3.6x |
| ee_to_ttx_smlimit | 9.981e-4 | 5.0x | 1.53 | 2.3x |
| ee_to_wpwm | 7.514e-4 | 10.6x | 0.87 | 4.0x |
| ee_to_wpwm_cw | 5.938e-4 | 13.5x | 0.68 | 5.1x |
| ee_to_zh | 1.066e-3 | 4.7x | 1.89 | 1.9x |
| ee_to_zh_smeft | 3.421e-4 | 14.6x | 1.28 | 2.7x |
| gg_to_gg | 2.322e-3 | 3.4x | 2.08 | 1.7x |
| gg_to_ttx | 5.672e-4 | 8.8x | 0.79 | 4.4x |
| gg_to_ttx_smlimit | 1.602e-3 | 3.1x | 1.92 | 1.8x |
| gg_to_ttx_smlimit_qcd2 | 1.543e-3 | 3.2x | 1.82 | 1.9x |
| gu_to_epemu | 1.866e-3 | 2.7x | 1.23 | 2.9x |
| gux_to_epemux | 2.476e-3 | 2.0x | 1.66 | 2.1x |
| ll_to_qqx_toy_dipole | 1.023e-3 | 4.9x | 2.05 | 1.7x |
| ll_to_qqx_toy_tensor | 2.300e-3 | 2.2x | 2.52 | 1.4x |
| ll_to_qqx_toy_yukawa | 3.146e-4 | 15.9x | 1.09 | 3.2x |
| p3r3_to_p3r3_toy_epsilon | 1.202e-3 | 4.2x | 1.74 | 2.0x |
| p3r3_to_p3r3_toy_sextet | 9.934e-4 | 5.0x | 1.44 | 2.4x |
| qqx_to_o8o8_toy_dcolor | 1.056e-3 | 4.7x | 1.25 | 2.8x |
| tata_to_ttx_tensor4f | 5.974e-4 | 8.4x | 0.92 | 3.8x |
| ud_to_epemud_qcd0 | 3.695e-3 | 2.7x | 1.77 | 2.0x |
| uux_to_epemg | 4.193e-3 | 2.4x | 2.61 | 1.3x |
| uux_to_mumu | 1.244e-3 | 4.0x | 1.43 | 2.4x |
| uux_to_ttx_4f | 6.530e-4 | 7.7x | 2.26 | 1.5x |
| uux_to_uux | 1.269e-3 | 3.9x | 1.41 | 2.5x |

## The brief

- **"`|pull| < 3`".** The fixed-beam gate's limit is `PULL_LIMIT = 3.5`
  (`|pull| ≤ 3.5`). `< 3` is the Drell-Yan hadronic gate's. Neither moved.
- **"34 fixed-beam σ rows".** That is exactly the `Plan::Gate` set (34
  asserted). The Info row `gg_to_gg_cg` and the two `Plan::Long` 2 → 6 rows
  are also fixed-beam and stay on one seed. The brief named only the 34, so I
  kept to that.
- **"`pixi run --skip-deps validate`".** On this worktree it cannot pass, for
  the missing `NNPDF31_lo_as_0130` set. Because cargo fails fast, it also
  hides every later suite. I therefore recorded it as two passes on one
  commit, as stated above.

## Found

- **The worktree lacks `validation/pdf/NNPDF31_lo_as_0130`.**
  - Evidence: `validate_pdf_grid` fails 13 of 20 tests on
    `failed to load PDF set … NNPDF31_lo_as_0130`. The main checkout lacks it
    too (`ls /home/user/vibegraph/validation/pdf`).
    `bash validation/pdf/fetch.sh NNPDF31_lo_as_0130` refuses without
    `VIBEGRAPH_FETCH_CONSENT=1`.
  - To close: worktree setup fetches (with the user's consent) or copies the
    multigrid set beside `NNPDF23_lo_as_0130_qed`.
- **`validate.sh` runs `cargo test` without `--no-fail-fast`.**
  - Evidence: one environmental failure in `validate_pdf_grid` stopped the
    banked layer before `validate_samples`, `validate_scales`,
    `validate_sigma`, `validate_unweighting` and `validate_vegas`. The
    collator then reports their cells as missing rather than measured.
  - To close: add `--no-fail-fast` to the banked `cargo test` in
    `validation/validate.sh`. The exit status already propagates.
- **`gg_to_gg_cg` (Info) and the 2 → 6 Long rows still read one seed.**
  - Evidence: `sigma-pulls` lists `gg_to_gg_cg (info)` with `n 1` in the
    48-row population, and `plan_seeds` gives Info and Long `[SEED]`.
  - To close: decide whether Info rows also read `HEADROOM_SEEDS`.
    `gg_to_gg_cg` is cheap at 40 000 × 6, but it is a `SCALE_FALLBACK_ROWS`
    row, so measure its cost first. The Long rows' cost argues against it.
- **`SCALE_FALLBACK_ROWS`'s cost note is stale, and that row now dominates the
  suite.**
  - Evidence: the doc says `gg_to_ttx_smlimit_qcd2` "runs in 17.3 s at its gate
    budget (40 000 × 6)". V1's sweep measured 69.0–91.4 s per seed. This
    session's gate row took 519.8 s for five seeds (`duration_s`), and the probe
    took 102–108 s per seed, both under load. That is 47 % of the summed row
    wall of `validate_sigma`.
  - To close: re-time on an idle host, then correct the doc or cut the row's
    re-evaluation cost.
- **The σ-gate knowledge page describes a single-seed fixed-beam gate.**
  - Evidence: `research/kb/validation/sigma-gate.md`, "The statistic", gives
    the multi-seed `combine` reading only for "the hadronic gates".
  - To close: the close-out session states that the fixed-beam `Plan::Gate`
    rows read five seeds under the seed policy, with Info and Long on one.
- **`PULL_LIMIT`'s rationale is now about a different statistic.**
  - Evidence: its doc argues 3.5 as a bound on "the maximum of five roughly
    standard-normal draws". The gate now asserts one draw per row, the mean's
    pull, whose error is `max(quoted, spread/√n)`. The 3.5 bound is
    therefore looser per row than the doc's argument says.
  - To close: PR #45's re-record restates the rationale for the mean's pull.
    The limit is not changed here (brief, expose-don't-fix).
