---
type: Session Report
title: "G-F4 report: refuse σ = 0 and a σ carried by too few points"
description: "integrate refuses a run with no point of weight, naming whether the cuts or the matrix element zeroed it, and a run whose kept points have an effective sample size below 20, measured against 320 banked integrations (minimum 1.1e5) and a 21-seed sweep of the smallest command-line budget (minimum 90); generate refuses a σ = 0 artifact, which it previously sampled."
status: draft
tags: [refusal, sigma, cli, cms-survey, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Branch `sprintg-refusals-f4`, worktree `/home/user/wt/G1-F4`, base `a113da8`.
Implementation commit `fad21ef`; this report is committed on top of it.

## What landed

1. **A per-point census in the integrands.** `support::PointCensus`
   (`evaluated`, `passed_cuts`, `nonzero`) is counted per thread in each
   integrand's existing evaluation scratch (`FixedBeamScratch`,
   `ProtonScratch`), so it costs a `Cell` update on a structure the point
   already fetched, and no shared atomic. `FixedBeamIntegrand::{census,
   clear_census}`, `ProtonIntegrand::{census, clear_census}` and
   `MultiplicitySum::{census, clear_census}` read and reset it. `integrate`
   clears it after the α-survey and reads it after the grid integration, so it
   counts exactly the integration's points, warm-up included. A point is
   "passed" when it got through `Cuts` (which includes a decay chain's
   `bwcutoff` window, `ForcedResonances`); "nonzero" when its weight was not
   exactly zero.
2. **Support counts in `ConvergenceReport`**: `kept_points`,
   `nonzero_points` and `effective_points`, all over the kept (post-warm-up)
   iterations the reported σ averages. `effective_points` is
   `(Σcᵢ)²/Σcᵢ²` with `cᵢ = wᵢ/Nⱼ`, `Nⱼ` channel `j`'s kept points, formed from
   `Σw`, `Σw²` per channel; `BlockIteration` carries the block's `Σw²`
   (`sum_sq`) for it.
3. **`support::check_support`** (typed `SupportError`), called by `integrate`
   after the existing kept-iterations check and before σ is printed or the
   artifact written; the CLI appends "; no artifact was written" and exits 1.
   - `CutsRejectAll`: no nonzero point and none passed the cuts.
   - `MatrixElementZero`: no nonzero point, some passed the cuts.
   - `TooFewEffective`: `effective_points < MIN_EFFECTIVE_POINTS = 20`
     (NaN refused too), with the nonzero, kept and cut-pass counts.
   An info line `support: N of M kept points carry weight (effective sample
   size E); P of Q integration points passed the cuts` is printed on every run.
4. **`generate`** refuses an artifact whose `sigma_pb == 0.0`. Lines touched:
   `vibegraph-cli/src/generate.rs` 339–349 (one `if` block right after
   `IntegrateArtifact::read_hashed`, nothing else in the file).
5. **Docs**: `docs/src/cli/overview.md` (a paragraph before "The artifact"
   and one sentence in it); two rows of the release-scope table
   (`research/kb/pipeline/release-scope.md`). `docs/src/cli/reference.md` is
   generated from `--help`, which did not change.

## The statistic and the threshold

**Choice: an effective sample size per run, over the kept points.** Measured
first as both a nonzero count and an ESS, then chosen. The deciding
measurement is `e+ e- > mu+ mu-` at `√s = 91.2` with `ptl = 45.599999` (only
`|cos θ| < 2·10⁻⁴` passes), `--fixed-budget --neval 4000 --niter 4`, which
fails in two shapes:

| seed | nonzero / kept | ESS |
|---|---|---|
| 1 | 575 / 8192 | 44 |
| 2 | 1166 / 8192 | 86 |
| 3 | 640 / 8192 | 25 |
| 4 | 2 / 8192 | 2 |
| 5 | 590 / 8192 | 5 |
| 20261104 | 1 / 8192 | 1 |

(same card at `ptl = 45.5999985` and `45.599998`: ESS 1–143, nonzero 1–1553.)
Either the grid never finds the window (1–2 nonzero points, the shape card 048
shows) or it finds it late and hundreds of points have weight while a handful
carry it (seed 5: 590 nonzero, ESS 5). A per-run count refuses the first
shape and passes the second; ESS ≤ nonzero count, so an ESS floor refuses the
first whenever a count floor would and also refuses the second. Per channel
was rejected because a multichannel run legitimately has channels with few
nonzero points that carry little of σ, and the run-level ESS already weighs a
channel by its share.

**Card 048** reproduced (`p p > vz, (vz > h z)`, its own param card, the
campaign's `--fixed-budget --neval 10000 --niter 3`, `etaonium = 0.6`; the
CMS PDF set 325500 is not on this host, so NNPDF23_lo 247000 stands in: the
pathology is the 1 MeV width against the 15-width `bwcutoff` window, not the
densities). Six seeds, `support:` lines verbatim:

```
seed 1: 1 of 10000 kept points carry weight (effective sample size 1); 1 of 30000 integration points passed the cuts
seed 2: 1 of 10000 kept points carry weight (effective sample size 1); 1 of 30000 integration points passed the cuts
seed 3: 35 of 10000 kept points carry weight (effective sample size 2); 41 of 30000 integration points passed the cuts
seed 4: 1 of 10000 kept points carry weight (effective sample size 1); 1 of 30000 integration points passed the cuts
seed 5: 0 of 10000 kept points carry weight (effective sample size 0); 0 of 30000 integration points passed the cuts
seed 6: 0 of 10000 kept points carry weight (effective sample size 0); 0 of 30000 integration points passed the cuts
```

Seeds 1, 2 and 4 are the survey's `σ ± σ` shape exactly (one point). The cut
count shows card 048's zero is the decay-chain window acting as a cut, not
`|M|²`; the refusal says so.

**Legitimate runs**, every integration the gates ran, recorded by a temporary
uncommitted logging patch in `integrate_channels` (diff below):

| source | integrations | min ESS | min nonzero |
|---|---|---|---|
| `validate_sigma` (all non-ignored tests, 5 seeds per row) | 245 | 110 731 | 118 195 |
| `validate_hadronic` (20 tests) | 75 | 158 047 (`sigma_jj_ht_window_vs_mg`) | 312 311 |
| `cli_generate_proton` (extended-validation) | 4 | 147.9 | 5 803 |
| `cargo test --workspace` (dev, every non-ignored test) | 220 | 22.6 lib / 995.8 CLI | 1 036 CLI |

The 22.6 is `hadronic::tests::multichannel_resolves_resonant_pole_flat_rambo_misses`,
a library test that integrates flat RAMBO across a pole precisely to show it
misses it (21% error); no CLI route reaches it. The 147.9 is
`a_dynamical_scale_card_runs_past_the_density_grid_and_past_the_coupling_table`
(`p p > l+ l- j`, six channels, `--neval 2000 --niter 2`, dynamical scales),
the smallest legitimate CLI budget measured, so that configuration was swept
over 21 seeds (AGENTS.md: the rung that matters, 20+ seeds):

```
ESS by seed 1..20, 20260719: 570 621 90 610 622 629 169 494 458 452 578 530 409 545 569 180 476 475 485 188 148
```

Median ≈ 480, heavy lower tail (90, 148, 169, 180, 188). The smallest
`e+ e- > mu+ mu-` CLI budget (`--neval 1000 --niter 2`) over six seeds:
ESS 976–1000.

**Threshold 20**: ten times card 048's worst (2), 4.5× below the lowest
legitimate run measured (90, the heavy-tail seed of the smallest budget), and
5 500× below the smallest banked row. 50 was the first value tried; the
21-seed sweep's 90 put it within 1.8×, too close for a heavy-tailed quantity,
so it was lowered.

**What the choice cannot see.** A region the sampler never reaches while the
reached region's weights are flat: a confidently wrong σ with a high ESS
(AGENTS.md's missed-region case). It is a property of the pooled kept points,
not of the inverse-variance combination of iterations a fixed budget may use.
And it does not separate "too few points" from "too small a budget": a run at a
budget far below the smallest measured here can legitimately fall under 20,
and is then refused with a message that says to look at the counts.

### Measurement patch (never committed)

`python3 G-F4-scripts/measure.py` inserted, before `(per_channel, total,
report)` at the end of `integrate_channels`, an append of
`G-F4-SUPPORT nonzero= kept= ess= sigma= err= channels= who=<thread> | <argv>`
to `$G_F4_SUPPORT_LOG`; `--remove` took it out (`grep -c G_F4
vibegraph-lib/src/budget.rs` → `0`, and `git show HEAD:vibegraph-lib/src/budget.rs
| grep -c G_F4` → `0`). It reads only the report and writes only to the file.

## Mutation evidence

`cargo test -p vibegraph --test cli_support_refusals -- --test-threads 1`,
the same test file in all four runs (`G-F4-mutants.log`):

```
=== A pre-change (a113da8 sources of vibegraph-lib/src and vibegraph-cli/src)
test a_coupling_zero_at_the_parameters_in_force_is_refused ... FAILED
test a_cross_section_carried_by_a_few_of_many_nonzero_points_is_refused ... FAILED
test a_cross_section_carried_by_one_point_is_refused ... FAILED
test cuts_that_reject_every_point_are_refused ... FAILED
test generate_refuses_an_artifact_recorded_at_zero_cross_section ... FAILED
test the_same_toy_process_at_its_default_coupling_integrates ... ok
test result: FAILED. 1 passed; 5 failed
=== B count-based floor (nonzero < 100 in place of the ESS floor)
test a_cross_section_carried_by_a_few_of_many_nonzero_points_is_refused ... FAILED
(the other five ok)
test result: FAILED. 5 passed; 1 failed
=== C census counts cut passes only where weight is nonzero
test a_coupling_zero_at_the_parameters_in_force_is_refused ... FAILED
(the other five ok)
test result: FAILED. 5 passed; 1 failed
=== D committed HEAD
test result: ok. 6 passed; 0 failed
```

On the pre-change code every refused card exited 0 with `σ = 0` (the
`iteration 3: 0.000000e0 ± 2.1e-154` lines), and `generate` on the hand-made
σ = 0 artifact **wrote 100 events** ("sample estimate 2.265062e3 ± 2.266439e2
against the integration's 0.000000e0 ± 0.000000e0") instead of refusing.
B pins the ESS-over-count choice; C pins that the census tells the cuts from
the matrix element.

Library tests added: `budget::tests::{the_support_counts_are_the_kept_iterations_nonzero_points,
a_run_without_weight_has_no_support, the_effective_sample_size_weighs_each_point_by_its_channel}`,
`proton::tests::the_census_separates_cut_rejections_from_zero_weights`,
`support::tests::{a_run_without_weight_names_the_cuts_or_the_matrix_element,
the_floor_is_on_the_effective_sample_size}`.

### What each test cannot detect

- `cuts_that_reject_every_point_are_refused` (`ptl = 50` at `E = 45.6`): a
  wrong cut-pass count that is nonzero would flip it to the matrix-element
  message, and it would fail; it cannot see the census on proton beams.
- `a_coupling_zero_at_the_parameters_in_force_is_refused` (toy `lt~ lt > vt vt`,
  `gl = 0`): whether a zero luminosity or a factorisation-scale floor would be
  reported the same way (it would: `MatrixElementZero` covers every zero after
  the cuts). Its control integrates the same process at the model's `gl`.
- the two too-few tests: pinned seeds of a chaotic adaptation (ESS 1 and 5
  against the floor of 20); another platform's last-ulp arithmetic can move
  them. They cannot calibrate the threshold; the table above does.
- `generate_refuses…`: only `σ = 0` exactly; a NaN or negative σ artifact is
  not refused by this check.
- `the_census_separates_cut_rejections_from_zero_weights`: one card
  (`p p > l+ l- j`, sub-threshold μF), single thread; it pins that
  `passed_cuts` is unchanged by a zero set after the cuts and that the counts
  equal an independent count of the returned values.

## Gates

All with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 RAYON_NUM_THREADS=1`, `--jobs 1`,
`--test-threads 1`, run detached on tree `7e8430c` (identical tree to
`fad21ef`, checked with `git rev-parse HEAD^{tree}` before and after the
squash). Logs in the scratchpad, prefixed `G-F4-final-`.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 (`Finished dev profile … in 56.64s`) |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings` | exit 0 (`Finished … in 1m 22s`) |
| `cargo test --workspace --no-fail-fast` | exit 0; 52 test binaries, 1843 passed, 0 failed, 25 ignored |
| `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_generate_proton -- --nocapture` (`validate-generate-proton`) | `test result: ok. 9 passed; 0 failed; 2 ignored; … finished in 67.66s` |
| `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --nocapture` (`validate-sigma`) | `test result: ok. 8 passed; 0 failed; 33 ignored; … finished in 1459.93s` |
| `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_hadronic -- --nocapture` (`validate-hadronic`) | `test result: ok. 20 passed; 0 failed; 26 ignored; … finished in 1410.50s` |
| `python3 scripts/kb.py lint` (after `python3 scripts/kb.py index` for this report) | `1056 concepts, 337 backlog items; 0 errors, 5 warnings` (the 5 are pre-existing backlog warnings) |

`validate_sigma` and `validate_hadronic` ran once, with the measurement patch
in place and the floor at 50 in `support.rs`; neither calls `check_support`,
and the changes after that run (the floor's value, its doc, formatting, test
code) touch nothing they execute, so they were not re-run.
`cli_generate_proton` does call it and was re-run on the final tree. The pixi
`depends-on` tasks (`extract-sigma`, `fetch-pdf`, `generate-hadronic-sigma`,
`build-diagrams`) were not run: the reference data were already present
(hard links), which is what `--skip-deps` means.

## Brief corrections

- **The banked library gates cannot trip the refusal.** `validate_sigma` and
  `validate_hadronic` drive the integrands directly and never reach
  `integrate`, so "still pass" is not the evidence that no banked row trips
  it; the measured ESS of every integration they ran is. Only
  `validate_generate_proton` (`cli_generate_proton`) runs the refusal, and its
  lowest run (ESS 147.9) is the one the threshold had to be lowered for.
- **Card 048's zero support is the cuts, not `|M|²`**: the decay chain's
  `bwcutoff` window is part of `Cuts`, and with 1 MeV widths almost no point
  lands in it (cut-pass count 0–41 of 30 000). The message names the window.
- **"Too few nonzero points" is the wrong statistic** for the case the brief
  names; see seed 5 above. G2's wording ("fewer nonzero points than a
  threshold") should read as an effective sample size.
- `CARGO_PROFILE_RELEASE_DEBUG_DEBUG=0` is not a usable override for the
  `release-debug` profile: cargo fails at start-up ("invalid type: Option
  value, expected a boolean…"). The banked gates ran without it.

## Found

- **The zero-weight cause stops at "after the cuts".** `MatrixElementZero`
  covers `|M|² = 0`, a parton luminosity of zero (an initial-state flavour the
  PDF set does not carry), the factorisation-scale floor and the on-shell
  veto alike. Evidence: `ProtonIntegrand::shape` returns `acc == 0` for all
  four; the census counts at that single point. Closing it: count `|M|² ≠ 0`
  separately inside `shared_scale_sum`/`per_group_sum` (and the veto) and let
  the message name which factor vanished.
- **No proton-beam CLI test of the refusals.** The proton census is pinned by
  a library test; the CLI refusal path is exercised on fixed-energy beams
  only, because a proton card needs the PDF set. Closing it: a
  `cli_generate_proton`-style extended-validation test with cuts that reject
  everything on `p p > e+ e-`.
- **The smallest-budget CLI tests sit 4.5× above the floor at their worst
  seed** (dynamical-scale `p p > l+ l- j` at `--neval 2000 --niter 2`: ESS 90
  on seed 3, 148 on the pinned seed). A change that worsens sampling at tiny
  budgets would trip `a_dynamical_scale_card_runs_past…` before any σ gate.
  Closing it: either accept that as the signal it is, or raise that test's
  budget.
- **A σ that is NaN or negative is not refused** by `generate`'s artifact
  check or by `check_support` (a NaN ESS is refused; a negative-weight
  cancellation to a finite σ with a large ESS is not). LO weights are
  non-negative, so this is a robustness gap, not an observed failure.
