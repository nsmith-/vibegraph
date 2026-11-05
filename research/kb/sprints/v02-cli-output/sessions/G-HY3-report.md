---
type: Session Report
title: "G-HY3 report: integrate prints σ to its error's digits, χ²/dof as n/a over one iteration, and names a non-constant μF"
description: "σ and Γ print value ± error with the error to two significant figures and the value to the same place; VegasResult::chi2_per_dof is Option<f64> and prints n/a below two entering iterations; μF is read from the fixed flags, named per beam or by its dynamical choice, and recorded as None (artifact format 19, 18 refused). The survey parser reads the new lines and, fixed on the way, the per-part lines it had not matched since parts carry masses. 13 of 13 mutations killed; every gate green."
status: draft
tags: [cli, output, statistics, scales, artifact, cms-survey, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
measured:
  host: "cloud container, linux-64, 2026-11-04"
  commit: "change ed17acb + c97f165 on base 9c573f9"
  command: "see Gate; logs under the session scratchpad, prefixed G-HY3-"
---
# G-HY3 report

Branch `sprintg-cli-output-hy3`, base 9c573f9. Commits: `ed17acb` (Rust),
`c97f165` (survey parser), and this report.

## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| F1 | 1 | `vibegraph-cli/src/integrate.rs:852`, `:1127` | result line `σ = {:.6} ± {:.6} pb`, per-multiplicity line `{:.6}` | checked | **fixed** |
| F2 | 1 | `vibegraph-lib/src/vegas.rs:1192` (`combine_iterations`), doc `:421`; `budget.rs:1175`, `:847`; `hadronic.rs:1916` | χ²/dof is `0.0` for one iteration / empty / zero dof; `integrate.rs:796` prints it `{:.3}`; `artifact.rs:320,395` are `f64` | checked | **fixed** (`Option<f64>` at the source) |
| F3 | 1 | `budget.rs:837` | per-iteration log `iteration N: … (χ²/dof {:.3})` prints 0.000 on the first kept iteration, same sentinel | checked | **fixed** with F2 |
| F4 | 1 | `progress.rs:91-94`, `tui/state.rs:132`, `tui/footer.rs:190` | progress doc "`0` until there are two"; the TUI decoded the sentinel (`unwrap_or(0.0)`, `chi2 > 0.0`) | checked | **fixed** with F2 |
| F5 | 1 | `integrate.rs:605-613`, `:1218`, `:786`; `artifact.rs:376` | `recorded_mu_f` returns `0.0` for a per-event or per-beam μF; fixed energy records `0.0`; summary prints `μF = 0 GeV` | checked | **fixed** |
| F6 | 1 | `integrate.rs:610` | `recorded_mu_f` keys on `RunningCouplingReport::constant_scales`, `None` whenever μR is per event or `xqcut > 0`/`ickkw = 1` (`hadronic.rs:324`, `ScaleChoice::is_constant`), though μF is the card's constant whenever both fixed flags are set (`scales.rs:522`, `:839`: the fixed value overrides on every path). `fixed_fac_scale = T, fixed_ren_scale = F` recorded μF = 0 | checked (unit test + banked CLI case) | **fixed** |
| F7 | 2 | `validation/cms_gridpacks/parse.py:521` | `VG_MULT` matches `(@N)` only; `integrate` prints `(@0; masses 0, 0 GeV)` and `(@0 @1; …)`. `P.VG_MULT.findall(<current line>)` → `[]`. The fixture `test_parse.py:57` is the old form, so the test was green on a parser that read no current part line | checked | **fixed** |
| F8 | 2 | `parse.py:522` | `VG_VEGAS` χ² group `[0-9.eE+-]+|NaN|nan`: an `n/a` line drops `n_evals` too | checked | **fixed** |
| F9 | 2 | `vibegraph-lib/tests/validate_sigma.rs:4682` | a one-seed cell carries the integration's own χ²/dof, the sentinel | settled: every plan runs `niter` ≥ 6, so no current row keeps one iteration | **fixed** to `expect` (a one-iteration plan now fails loudly instead of banking 0) |
| F10 | 3 | `scales.rs:129`, `ScaleChoice` | `DynamicalChoice::as_i64` was `pub(crate)`, `fixed_fac` had no accessor | — | widened, with a user in `integrate.rs` |

Backlog check (path tails `/integrate.rs`, `/artifact.rs`, `/vegas.rs`,
`/budget.rs`, `/progress.rs`, `tui/`, `coupling/scales.rs`,
`cms_gridpacks/parse.py`): beyond the three claimed items, none has a site
this change makes wrong or fixed except `vector-triplet-chain-one-nonzero-point`
(see Found).

## Fixed

**F1, `ed17acb`.** `value_and_error(value, error)` in `integrate.rs`: the
error to two significant figures (`{:.1e}`), the value in scientific notation
to the decimal place of the error's second figure, with a carry correction when
rounding moves the value into the next decade, a two-figure floor when the
error exceeds the value, `0e0` for a zero value, and the shortest exact form
when the error is zero or not finite (a constant integrand). The result line,
the per-multiplicity line and the Γ line all use it. A σ of 9e-6 pb now prints
`8.6812e-6 ± 2.3e-9` where it printed `0.000009 ± 0.000000`.
`cli_decay.rs`'s `PRINTED_WIDTH_REL` doc ("seven significant figures") is
rewritten to the new rule; its gates passed on it.

**F2–F4, `ed17acb`.** `VegasResult::chi2_per_dof: Option<f64>`;
`combine_iterations` → `(niter > 1).then(..)`; `running_estimate` and
`combine_channels` sum `Option`s (undefined unless every channel's is
defined). `vegas::Chi2PerDof` displays a defined value under the caller's
width and precision and an undefined one as `n/a`. `progress::vegas_iteration`
takes `Option<f64>` and omits the field when `None`; `UiState::chi2` is
`Option<f64>`, the footer shows it only when `Some`. Test-code readers print
through `Chi2PerDof`; numeric readers `expect` where the iteration count
guarantees a value. Identity oracle for the refactor: the seeded goldens in
`vegas.rs` (`test_pinned_seed_regression_grid_adapt`, `to_bits`) and the
thread-count `to_bits` equalities now compare `Option<u64>` and pass unchanged.

**F5–F6, `ed17acb`.** `FactorizationScale { NoDensities, Constant, PerBeam,
PerEvent { choice } }` in `integrate.rs`, built from
`ScaleChoice::fixed_factorisation_scales()` (new) and `DynamicalChoice::as_i64`
(now `pub`). The summary prints `μF = 91.188 GeV`, `μF = 91.188 GeV, 50 GeV by
beam`, `μF dynamical, choice -1` or `no parton densities`;
`IntegrateArtifact::mu_f` is `Some` only for `Constant`. The
`RunningCouplingReport` the hadronic path kept only for `recorded_mu_f` is no
longer collected.

**Artifact, `ed17acb`.** `FORMAT_VERSION = OLDEST_READABLE_VERSION = 19`;
`mu_f`, `IntegrateArtifact::chi2_per_dof` and `ChannelGrid::chi2_per_dof` are
`Option<f64>`. `a_version_18_artifact_is_refused` serializes version 18's real
layout (bare `f64`s, `0` for the sentinels), checks it differs from a
version-19 encoding by exactly its `2 + channels` Option tags, and asserts the
`UnsupportedVersion { found: 18, oldest: 19, expected: 19 }` refusal with the
regenerate remedy. The v16/v17 tests' docs claimed "version N's real shape";
they are this build's encoding with the missing field cut, and say so now.
Readers: `generate` and reweighting read neither field (grep: no production
reader of `artifact.mu_f` or `artifact.chi2_per_dof`); `cli_integrate`'s
replay takes `mu_f.expect(..)` on its fixed-scale card; the report collator's
`chi2_dof` cells are seed-scatter values except F9's one-seed branch.

**F7–F8, `c97f165`.** `VG_MULT` accepts `(@N[ @M…][; masses … GeV])` and
records the part under its first process number (the `(run_id, tag)` key
stays unique: parts' ids are disjoint); `VG_VEGAS` accepts `n/a`, which `_f`
stores as NULL. Old campaigns' output still parses: the base `test_parse.py`
passes against the new `parse.py`.

## Tests added (suite count +8 hermetic, +1 banked, +1 Python; none deleted)

Hermetic: `vegas::tests::an_undefined_chi2_prints_as_not_a_number`;
`artifact::tests::a_version_18_artifact_is_refused`,
`an_undefined_chi2_and_an_absent_scale_round_trip_as_none`;
`tui::state::tests::a_single_kept_iteration_leaves_the_chi2_unset`;
`integrate::tests::a_value_keeps_the_digits_its_error_supports`,
`the_factorization_scale_is_named_by_its_fixed_flags`;
`cli_fixed_energy::a_small_cross_section_prints_the_digits_its_error_supports`
(`e+ e- > mu+ mu-` at √ŝ = 100 TeV, σ ≈ 9e-6 pb; pins two error figures, the
error within 5% of the artifact's, σ's last digit at the error's second
figure, and σ within err/20 of the artifact's — the rounding bound of that
format), `a_chi2_over_one_iteration_is_reported_undefined` (`--fixed-budget
--niter 3`: `(χ²/dof = n/a)`, artifact and every channel `None`, no μF and
`mu_f = None` at fixed energy; `--niter 4`: the printed number equals the
artifact's at `{:.3}`). Banked:
`cli_integrate::the_summary_names_a_factorization_scale_with_no_constant`
(dynamical, fixed-μF-with-running-μR, and per-beam cards: line and artifact).
Python: `VgOutput.test_significant_figures_masses_and_an_undefined_chi2`.
Assertions extended: progress event omits `chi2` when `None`; artifact
round trip compares `chi2_per_dof`; the warm-up clamp test asserts `None`
where it asserted the `0.0` sentinel.

## Mutations

Driver `G-HY3-mutate.py` (apply, run, revert; log `G-HY3-mutations.log`).
"Killed" means `test result: FAILED`, not a build error. Tree identical after
the batch (`git diff --stat` unchanged).

| id | mutation | test | result |
|---|---|---|---|
| M1 | result line back to `{sigma_pb:.6} ± {sigma_err_pb:.6}` (the old code) | `cli_fixed_energy a_small_cross_section` | killed (`:413`, error figures) |
| M2 | carry correction disabled | `a_value_keeps…` | killed |
| M3 | error to three figures | `a_small_cross_section` | killed |
| M4 | `combine_iterations` defines χ² at one iteration | `a_chi2_over_one_iteration` | killed (`:449`) |
| M5 | `Chi2PerDof(None)` prints `0` | same | killed |
| M6 | `combine_channels` `.or(Some(0.0))` | same | killed |
| M7 | per-beam pair taken as `Constant(a)` | `the_factorization_scale…` | killed |
| M8 | `recorded()` returns `Some(0.0)` for no constant | same | killed |
| M9 | fixed energy as `Constant(0.0)` | `a_chi2_over_one_iteration` | killed (`:457`) |
| M10 | constant only when fully fixed (the old `constant_scales` semantics, F6) | `the_factorization_scale…` | killed |
| M11 | TUI state `chi2.or(Some(0.0))` | `a_single_kept_iteration…` | killed |
| M12 | progress emits `chi2.unwrap_or(0.0)` | `progress::` | killed |
| M13 | reader accepts version 18 | `a_version_18_artifact_is_refused` | killed |
| P1 | `VG_MULT` back to `\(@(\d+)\)` | `test_parse.VgOutput` | killed; base `test_parse.py` passes it |
| P2 | `VG_VEGAS` without `n/a` | same | killed; base `test_parse.py` passes it |

On the old tests: M1 is the base code, which every base test passed; the old
warm-up clamp test asserted the sentinel `chi2 == 0.0`, i.e. M4's behaviour.
No site unreachable by mutation in the change, except the `cli_integrate`
banked case was not mutation-run (M7, M8, M10 cover its logic at unit level;
its own pass is recorded under Gate).

## Gate (all on the final tree, `ed17acb` + `c97f165`)

- `cargo fmt --all --check` → exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0
  (covers `vibegraph/extended-validation` and the lib's)
- `cargo test --workspace` → exit 0; 54 result lines, 1897 passed, 0 failed,
  25 ignored. The base count was not taken on the base tree (a first listing
  ran after edits had begun); the delta is the 8 hermetic tests listed above.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items` → 8 warnings,
  all the pre-existing sites `rustdoc-warnings-ungated` lists (configs.rs:4,
  channel.rs ndim, alphas.rs, scales.rs ×2, lhef/mod.rs ×2, …); none new.
- `cargo doc -p vibegraph --no-deps --document-private-items` → 0 warnings
- `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_integrate`
  → `test result: ok. 6 passed; 0 failed` (reads the summary and the artifact;
  holds the new μF case)
- `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_generate_proton`
  → `test result: ok. 11 passed; 0 failed; 2 ignored` (parses the σ line)
- `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py' -v`
  → `Ran 22 tests … OK` (pytest is not installed; this is `pixi run test-gpdb`)
- `python3 scripts/kb.py lint` → `0 errors, 5 warnings`

Not run: the long banked tiers whose sources changed only in print or
`expect` lines (`validate_sigma`, `validate_hadronic`, `validate_samples`,
`validate_param_card_sigma`, `validate_ewk_lljj_sigma`, `diagram_channel`,
`decay_widths`); they compile under the all-features clippy above.

## Method

Reading the item sites and grepping every reader of the changed fields (~40%);
the compiler as the census of `chi2_per_dof` readers once the type changed
(~20%); hand-computed format cases before the unit test (~10%, one wrong: a
zero value took the carry branch, fixed before commit); a Python probe of the
parser regex against the current CLI format (found F7); the mutation batch
(~20%). The lead that `generate`/reweighting read `artifact.mu_f` did not hold:
nothing in production reads either changed field. F6 came from reading
`ScaleChoice::is_constant` while replacing the sentinel; a naive `Option` fix
of `recorded_mu_f` would have printed "dynamical" for a fixed μF.

## Found

1. **kb: artifact format pages are stale.**
   `research/kb/pipeline/artifact-format-versioning.md:103` says the floor is
   "its current version, 15"; the code is at 19 (18 before this change). The
   χ²/dof and μF fields' `Option` meaning is not in
   `phase-space/vegas-integrator.md:145` or `per-channel-vegas-grids.md:120`.
   Close: update the floor and add the version-19 entry.
2. **Backlog site: `vector-triplet-chain-one-nonzero-point`** (and
   `validation/cms-gridpack-survey-2026-10.md:89`) cites "μF = 0 logged" as a
   symptom of card 048. That line was this sentinel for a per-event scale, not
   evidence about the card. Close: drop it from the item's evidence (the
   one-nonzero-point symptom stands on its own).
3. **Survey schema keeps one process number per part.** A part line can name
   several (`@0 @1`); `vg_run_multiplicity` is keyed `(run_id, tag)` and the
   parser records the first. The σ is the part's, correct, but a query by
   process number misses `@1`. Close: a `vg_run_multiplicity_tag` table (or a
   tags column), with a `PARSER_VERSION` bump.
4. **Report collator schema has no "undefined" χ²/dof.**
   `vibegraph-lib/tests/common/report.rs:281` `chi2_dof: f64`, defaulted to
   `0.0` by `IntegralsRow::new` (`:312`), and `validation-report/src/cells.rs:150`
   prints it. Every current writer sets a seed-scatter value (12 sites checked),
   and the one-seed branch now `expect`s, so no 0 reaches the report today; a
   row that forgets to set it still reads `chi2/dof 0.00`. Close: `Option<f64>`
   in the row and the cell, `n/a` in the table.
5. **The survey records no factorization scale.** The `√s` line now names
   μF's kind, but `parse_vg_output` does not read it. Close only if the
   survey wants the column.

## Brief corrections

- The brief's gate says `python3 -m pytest validation/cms_gridpacks`; pytest
  is absent and the suite is `unittest` (`pixi.toml`'s `test-gpdb`).
- The brief lists `generate` and reweighting among the readers of the changed
  artifact fields; neither reads them (no production reader of `mu_f` or `chi2_per_dof`), so item 4
  of the work reduced to the tests that read them.
- The brief's per-multiplicity summary note missed that the survey parser
  already failed on it (F7).
