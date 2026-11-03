---
type: Session Report
title: "T-V4 report: pp_to_epemjj_qcd0 gated against its 20-seed record; the spine concepts describe the always-on pole floor"
description: "The EWK e+ e- j j no-pT-cut row is enforced at the long tier against refdata-13's seed record (3.593175 ± 0.001777 pb, 20 reported seeds). The gate reads the record or fails, evaluates pull, rel and this side's χ²/dof band together, and its verdict runs in the banked layer against shifted, narrowed and missing inputs. CI does not run the long tier, so the row does not yet gate vibegraph's σ in CI. The spine concepts now describe the pole floor, DEFAULT_POLE_SCALE, the remainder mixture and the version-15 digest."
status: draft
tags: [validation, spine, seeded-sigma, vbf, kb, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-02}
---
The session brief is [T-V4](T-V4.md). Worktree `/home/user/wt/T`, branch
`sprinte-spine-fallback`, from `5b6a6e87`. Nothing is pushed. Every cargo
command ran with `CARGO_INCREMENTAL=0`, debuginfo off and `-j 2`.

## CI does not run this gate

**CI does not run this long-tier target.** The row is enforced at tier `long`,
but it does not yet gate vibegraph's σ in CI, so by the brief's own condition
the item does not close on this session. I did not wire it into CI.

- `ci.yml`'s `banked` job runs `pixi run --skip-deps validate`, which is
  `validation/validate.sh`. That script runs
  `cargo test --workspace --profile release-debug --features … --no-fail-fast -- --nocapture`
  with no `--ignored`.
- `ewk_epemjj_without_pt_cuts_matches_madevents_seeds` is
  `#[ignore = "long tier: …"]`, so CI skips it. The other long-tier σ gates
  are the same: `validate-mlm-sigma`, `validate-sigma-2to6` and the ladders
  are ignored tests behind their own pixi tasks, which no workflow calls.
  `acceptance.yml` tests a released binary and nothing else.
- **What CI does run now:** the same file's four non-ignored tests are part of
  the banked layer. They exercise the verdict and the record reader against
  the banked record, a shifted copy, a narrowed band and an empty work area
  (see Mutations). CI therefore checks the reference and the gate logic, but
  not vibegraph's integration of this row.
- Running the integration in CI would take about 35–55 min of a four-core
  runner per banked run. Whether to spend that is the manager's call (Found F1).

## 1. The row is enforced

Commits `5ec22463` (gate) and the final commit (manifest, pixi, this report).

**`vibegraph-cli/tests/validate_ewk_lljj_sigma.rs`**

- `reference(output_dir)` reads `seed_record::reference(output_dir, ROW)`,
  through the same reader as every seeded row. When the record is missing, it
  calls `vibegraph::validation::require`, as `validate_param_card_sigma`
  does. The `AWAITING THE BANK` path and the `planned_rows()` lookup are gone.
  So is the `manifest` module include, which nothing else used.
- `verdict(side, reference, band)` evaluates `|pull| < 3`, `|rel| < 0.02` and
  `CHI2_PER_DOF_BAND = (0.023, 4.617)` together. It returns one line for each
  failing check, and the test asserts once over all of them. This resolves
  H17: before, the σ assertion fired first and hid a χ²/dof failure, and the
  planned path checked nothing.
- `RECORDED_SEEDS` holds the gate's five `(seed, σ, quoted)` readings to the
  printed digits. The four banked-layer tests read the verdict on them:
  - `the_recorded_seeds_pass_against_the_banked_record` is the control;
  - `a_reference_shifted_by_three_percent_fails_on_sigma` writes a copy of the
    record into a tempdir with every seed value ×1.03 and ×0.97, and reads it
    back through the same reader;
  - `a_band_that_excludes_the_measured_scatter_fails_on_it` sets the upper
    edge to half the measured χ²/dof;
  - `a_missing_record_fails` uses `should_panic`, expecting `needs the row's
    seed record`.

**`validation/manifest.toml`**

- `status = "planned"` is removed.
- The `integrals` note now records:
  - the reference and how it was banked: refdata-13; `bank_seeded_sigma.sh`
    `card` kind; 3.7.1 pinned tree; `hard_survey = 2`; reported estimator;
    20 seeds at 10000 give 3.593175 ± 0.001777, χ²/dof 0.66;
  - the two screening seeds (pulls +0.59 and +0.68, not flagged);
  - the CONFIRMED scatter with its four channels above χ²/dof 3;
  - the equal-weight reading, which stays on the record and is not read;
  - the gate rule;
  - this run;
  - the T-V3 measurements, which were already there.

**`pixi.toml`**

- The comment on `validate-ewk-lljj-sigma` no longer describes the
  AWAITING path. It now says to run the task between `validate` and
  `validation-report`.
- `validate-deep` lists the task with the other long-tier drivers.

**Matching the other seeded gates.**

- `validate_param_card_sigma` and the MLM σ gates call `require` when a row
  that is not planned has no record. This gate now does the same.
- `validate_hadronic`'s Drell-Yan path still has a `planned` arm, but it
  serves other rows and was not touched.

## 2. The gate on the committed seeds

The run used a detached, resumable driver (`T-V4-gate.sh` in the scratchpad)
on `5ec22463`, with a clean tree for the code:

```
$ CARGO_INCREMENTAL=0 cargo test -j 2 -p vibegraph --profile release-debug --config 'profile.release-debug.debug=0' \
    --features extended-validation --test validate_ewk_lljj_sigma -- --ignored --nocapture
[pp_to_epemjj_qcd0] MadEvent reference from seeded_sigma/pp_to_epemjj_qcd0/seed_record.json: 20 reported seeds at nevents 10000, 3.593175e0 +- 1.777e-3 pb (quoted 1.777e-3, spread 1.430e-3, chi2/dof 0.66)
[pp_to_epemjj_qcd0] import model sm-no_b_mass; generate p p > e+ e- j j QCD=0 QED=4 $ h on …/validation/madgraph/pp_to_epemjj_qcd0_run_card.dat, --fixed-budget --neval 40000 --niter 12, seeds [20261031, 20261032, 20261033, 20261034, 20261035]
  seed 20261031: σ = 3.6111418e0 ± 3.11e-2 pb (χ²/dof 3.07), 867 s
  seed 20261032: σ = 3.5834316e0 ± 3.63e-2 pb (χ²/dof 2.48), 675 s
  seed 20261033: σ = 3.5822541e0 ± 2.43e-2 pb (χ²/dof 4.55), 600 s
  seed 20261034: σ = 3.5210295e0 ± 2.14e-2 pb (χ²/dof 2.20), 606 s
  seed 20261035: σ = 3.6042065e0 ± 2.39e-2 pb (χ²/dof 2.53), 618 s
[pp_to_epemjj_qcd0] GATE vibegraph σ = 3.5804127e0 ± 1.59e-2 pb (5 seeds, quoted 1.25e-2, spread 1.59e-2, χ²/dof = 2.42, band (0.023, 4.617)), 3366 s | MG σ = 3.5931750e0 ± 1.78e-3 pb | pull = -0.80 | rel = -0.00355 | PASS
[report] wrote …/target/validation-report/integrals/pp_to_epemjj_qcd0.json
test ewk_epemjj_without_pt_cuts_matches_madevents_seeds ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 3366.60s
```

- **Verdict: PASS.**
  - σ = 3.5804127 ± 0.0159 pb, against the reference 3.5931750 ± 0.00178,
    which the gate read from `seed_record.json`.
  - pull −0.80, rel −0.355%.
  - χ²/dof 2.42, inside (0.023, 4.617).
- Every seed equals T-V3's reading to the printed digits.
- **Wall time.** 3366 s. The hermetic suite build ran alongside the first
  seed, which took 867 s. The later seeds took about 600 s on an otherwise
  idle host, against T-V3's 405 s.
- **Collator.** `cargo run -p vibegraph-validation-report` over this work
  area renders the cell as `✅ pull -0.80, chi2/dof 2.42`, with the
  manifest's declared mode `gate`. It exits 1 on 50 problems, all of the form
  "declared 'banked' and no gate wrote it". They are cells missing from a
  partial work area left by earlier sessions. None concerns this row, and no
  full `validate` was run here.

**Ten-seed tension, recorded in the note.**

- T-V3's ten seeds read 3.5760 ± 0.0085, which is −0.48% and pull −1.98
  against the record.
- The second five seeds alone read −0.60% and pull −1.85.
- This is within the gate but not comfortably so. T-V3 put the budget
  plateau at ±0.4% (12 → 24 iterations), which is the same size (Found F2).

## 3. Mutations

**Input mutations.** These go through the gate's own reader and verdict
(`cargo test -p vibegraph --profile release-debug --features extended-validation --test validate_ewk_lljj_sigma -- --nocapture`):

| case | reference read | verdict |
|---|---|---|
| banked record (control) | 3.593175 ± 1.777e-3 | pull −0.80, rel −0.00355, failures `[]` |
| record ×1.03 | 3.700970 ± 1.831e-3 | pull −7.54, rel −0.03257, fails on pull (7.54 ≥ 3) and rel (0.03257 ≥ 0.02), not on χ²/dof |
| record ×0.97 | 3.485380 ± 1.724e-3 | pull +5.95, rel +0.02727, fails on pull (5.95 ≥ 3) and rel (0.02727 ≥ 0.02), not on χ²/dof |
| band (0.023, 1.21), the measured 2.42 outside it | banked | failures `["this side's seed χ²/dof 2.42 lies outside (0.023, 1.210029972204778)"]`, σ checks pass |
| empty work area | — | panics: `` `validate_ewk_lljj_sigma` needs the row's seed record (/tmp/.tmpepzDPV/seeded_sigma/pp_to_epemjj_qcd0/seed_record.json) … `` |

- The verdict lines are the arithmetic on the five recorded seeds.
- The record path is the gate's real input. The doctored copies are read
  through `seed_record::reference`, including its `check()` validation.

**Code mutations.** The driver is `T-V4-mut.py`. It applies one mutation by
exact replacement, runs the target in the dev profile, and restores the file.
Afterwards `git diff --stat vibegraph-cli/` was empty.

| id | mutation | result |
|---|---|---|
| MV0 | none | 4 passed |
| MV1 | χ²/dof band never checked | `a_band_that_excludes_the_measured_scatter_fails_on_it` FAILED (`:388`) |
| MV2 | rel bound never checked | `a_reference_shifted_by_three_percent_fails_on_sigma` FAILED (`:367`) |
| MV3 | `MAX_REL` 0.02 → 0.04 | `a_reference_shifted_by_three_percent_fails_on_sigma` FAILED (`:367`) |

**Not covered by these mutations.**

- These tests do not see this side's integration. A fixed reading cannot
  detect a change in what `integrate` returns; only the long-tier run can.
- A pull-only mutation survives the shift test, because the rel check fails
  with it anyway. With the current errors, a 3% shift trips both checks, so
  no input separates them.

## 4. The spine concepts

Commit `293caa9f`. Each concept below describes the current code, with no
history.

**`phase-space/spacelike-floor.md`**

- The description is rewritten.
- The Decision section now says:
  - the pole floor is always on, at `10⁻³ ×` the cut-implied floor or
    `DEFAULT_POLE_SCALE = 10³ GeV²`;
  - `t_pole_shapes` holds on every window that is not degenerate;
  - the bound applies only from `Cuts::spacelike_floor > 0`;
  - a spine is built for every chain;
  - the remainder mixture (`REST_LOG_SHARE = 0.5`, `Spine::rest_mixed`);
  - `map_key`/`channel_map_digest` at version 15, with `generate` refusing a
    mismatch.
- The 2 → 2 flat-fallback line now says it applies to an *unfloored* spine.
- The link to the backlog item is removed, since its question is closed.
- The anchors `cuts-rs`, `dc-rs`, `dc-chain`, `dc-tmap` and `hadronic-mc`,
  and the two inline line numbers, are moved to the current ranges.

**Other concepts**

- **`phase-space/channel-contract.md`** (lines 132–143): the regulator
  paragraph.
- **`phase-space/t-channel-spine.md`**:
  - a new paragraph, "The remainder's invariant";
  - in "The `t` map", the floor means only a zero-width window draws flat;
  - the regulator sentence at line 116 is rewritten;
  - the description now mentions the mixture.
- **Found by grep, beyond the brief's three:**
  - `phase-space/diagram-channels.md:66-68` said spines are built only for
    two legs or a positive scale;
  - `hadronic/proton-integrand.md:99-103` said "more than two legs has no
    peripheral spine without it";
  - `references/codebases/madevent-phase-space-maps.md:45`: the t-map table
    cell.
- `python3 scripts/kb.py index` regenerated `phase-space/index.md` and the
  sessions index.

## Gate

Driver `T-V4-checks.sh` (scratchpad), `CARGO_INCREMENTAL=0
CARGO_PROFILE_{DEV,TEST}_DEBUG=0`, `-j 2`, on `293caa9f`. The later commit
changes only `validation/manifest.toml`, `pixi.toml` and this report.

```
fmt rc=0            cargo fmt --all --check
clippy rc=0         cargo clippy --workspace --all-targets -- -D warnings
clippy-ev rc=0      cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
clippy-ev-lib rc=0  cargo clippy -p vibegraph-lib --all-targets --features extended-validation -- -D warnings
test rc=0           cargo test --workspace --no-fail-fast -- --test-threads 2
                    49 suites, 1745 passed, 0 failed, 24 ignored
```

After the final manifest edit:

```
cargo test -p vibegraph-validation-report         45 + 20 + 13 passed, 0 failed (the committed manifest parses)
cargo test -p vibegraph --features extended-validation --test validate_ewk_lljj_sigma
                                                   4 passed; 0 failed; 1 ignored
python3 scripts/kb.py lint                        1022 concepts, 333 backlog items; 0 errors, 4 warnings
```

- The four kb warnings are the existing `state: blocked` items.
- The long-tier gate result is in §2.
- The full banked layer (`validate.sh`) was not run. This session's code
  change is a single CLI test target, and its banked part ran above.

## Brief corrections

- **Paths.** The concepts live under `research/kb/phase-space/`, not
  `phasespace/`. The line numbers were right.
- **The support bound.** The brief says it "comes only from a provable pT
  floor". The bound does come only from `Cuts::spacelike_floor`, but that
  floor is provable only up to three outgoing legs. Past three it is applied
  as a scale, and the coverage gates check it
  (`Cuts::spacelike_floor`'s doc). The concepts say so, as `spacelike-floor`
  already did.
- **The 2 → 2 flat fallback.** The brief's list did not mention it. With the
  floor always on, the collinear-edge fallback can no longer be reached in
  production. `spacelike-floor.md` and `t-channel-spine.md` stated it as
  current behaviour, and both now give the condition.
- **CI.** The brief's stop condition applies (see "CI does not run this gate"), and I report it here
  instead of wiring the target into CI.

## Found

- **F1. No long-tier σ gate runs in CI.**
  - `validate.sh` passes no `--ignored`, and no workflow calls
    `validate-ewk-lljj-sigma`, `validate-mlm-sigma`, `validate-sigma-2to6` or
    the ladders.
  - Every `long` gate cell is therefore enforced only when someone runs its
    task by hand.
  - To close, either:
    - a scheduled or `workflow_dispatch` job that runs the long-tier tasks
      and then `validation-report`, about 1 h of a four-core runner for this
      row alone; or
    - a cheaper CI tier for this row: fewer iterations on a grid banked as a
      fixture, with its own calibrated tolerance.
- **F2. The ten-seed reading sits at −1.98 pull.**
  - T-V3's ten seeds read 3.5760 ± 0.0085
    against 3.593175 ± 0.0018: −0.48%.
  - The 12 → 24 iteration step was measured at ±0.4%, so a residual
    under-convergence of that size is not excluded.
  - To close: 20 seeds at the gate's budget and 20 at 24 iterations, read as
    the seed policy says for a budget-rung comparison.
- **F3. The gate only sees the total σ.** The record carries `by_channel` and
  per-`LPRUP` values. A per-subprocess comparison (`part_reference`) would
  see a mis-partition between `qq`, `qg` and `bb` groups that the total
  cannot, and it is the finer oracle available here.
- **F4. Stale backlog text** (not edited):
  - `hygiene/cli-tests-define-pdf-dir-per-file` still counts eight files
    (T-HY1's H18);
  - `hygiene/kb-concepts-stale-after-sprint-d` cites `artifact.rs` at
    version 14.
