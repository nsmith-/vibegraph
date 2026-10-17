---
type: Session Report
title: "HY1 report: the MLM Pythia gate's tests were blind to its trial factors; hardened, build recorded"
description: "Hygiene review of V1 (df3bbf9): 16 of 41 one-line mutations of the verdict logic were missed; 14 Python and 5 Rust tests added so 43 of 46 are caught; the row records the Pythia build and the version prints as 8.312. Verdict, every printed number and the rest of the row are unchanged."
status: draft
tags: [mlm, pythia, matching, hygiene, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-17}
sources:
  - {id: 61a5af1, resource: "https://github.com/nsmith-/vibegraph/commit/61a5af1", title: "validation: harden the MLM Pythia gate's tests, record the Pythia build"}
---
Worktree `/home/user/wt/mlm-pythia-tolerance`, branch `mlm-pythia-tolerance`, base `11db076`.
Scratch (all `mlmpy-HY1-` prefixed) in the session scratchpad
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/`.

## Findings (written before any edit)

| id | pt | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| F1 | 2 | `test_mlm_match.py`, chi2 leg | Mutations `chi2_fail = p_min < alpha`, `2 * p_min < alpha`, `4 * p_min < alpha` all MISSED (`mlmpy-HY1-mut-before.log`). The only trial-factor test reads `p_min_x_trials`, which the mutants leave correct, and no test sits between alpha/4 and alpha/2. | checked | fixed |
| F2 | 2 | per-bin leg | MISSED: `bin_fail = p_bin * n_bins / 2 < alpha`, one-sided p (`0.5 * erfc`). The excess tests use 3.0 and 4.1 only, and z* is 3.978; nothing probes between. | checked | fixed |
| F3 | 2 | `null_split` | MISSED: ceil to floor split, side B's events sliced from the first half, comparison given the wrong events. The function has no test. | checked | fixed |
| F4 | 2 | `gated_pulls`, `main` wiring | MISSED: merged-sigma pull read from `merged_over_sigma_lhe`; `all` dropped from the dict; the verdict fed with side B's null instead of A's. | checked | fixed (wiring moved into `judge()`, tested) |
| F5 | 2 | `chi2_sf` | MISSED: continued fraction `b += 1` for `b += 2` (the test histograms sit in the series branch or are so far out that either answer fails); `a = (k+1)/2`. No test of `chi2_sf` exists although every p-value of the gate comes from it. | checked | fixed |
| F6 | 2 | `check_standalone_modes` | No test at all. V1 ran one manual negative control. | checked | fixed |
| F7 | 2 | `main()` exit code | MISSED: `return 0` for `return 0 if ... else 1`. No unit test reaches `main`. | checked | not unit-testable; negative control run (below) |
| F8 | 2 | `<` vs `<=` in the two legs | MISSED: equivalent in practice (needs `p x trials == alpha` to the last bit). | checked | leave |
| F9 | 1 | `mlm_match.cc` line 84 / row | Version printed as `8.3120e+00` (the stream is in scientific mode). | checked | fixed in Python (see below) |
| F10 | 1 | row, `manifest.toml:1546` | No build or platform in the row. The manifest note names build `py311h5756c45_105`, but the environment that produced the cache carries `py312hd50de4e_105` (`.pixi/envs/pythia/conda-meta/pythia8-8.312-py312hd50de4e_105.json`, dated 2026-10-16, before the cache's `mlm_match` binary of 2026-10-17 01:27). | checked | fixed |
| F11 | 4 | `chi2_sf` vs Rust | Hand-written incomplete gamma; `validation-report` and `vibegraph-lib` use `puruspe`. | checked | Found |
| F12 | 2 | `jet_rate_legs` | A histogram with dof <= 0 has `p_value` None, so `jet_rate_legs` raises `TypeError` (run: `unsupported operand type(s) for *: 'NoneType' and 'int'`). A loud failure, not a green. | checked | Found |

No comment-hygiene hits: `git diff origin/main...HEAD` for `\bT[0-9]\b|Phase [0-9]|used to|no longer|session|sprint|backlog` over `validation*` and `pixi.toml` found none. No second copy of the statistics inside `validation/pythia/` (`consume.py` has none); `NormalDist` is the standard primitive for z*. Visibility: `mlm_pythia_verdict` is private; `Standalone.mode` is `pub(crate)` and read by `check_standalone_modes`. Function lengths: `main()` was already long; V1 shortened it. Backlog check: `mlm-match-py-error-not-seed-policy` (`mean_of` reports the quoted error) still holds at `mlm_match.py` `mean_of`, and bears on the gate (V1 Found 1); `mlm-pythia-comparison-no-tolerance` is the item the close-out deletes; `restrict-card-lookup-triplicated` names no file I reviewed.

## The rule against D1, as read from the code

The rule matches D1 as amended on every point I checked: five pulls `not abs(pull) < 3` (so `>= 3` and a missing pull fail); chi2 leg `min p x len(p_values)` against 0.005; per-bin leg two-sided `erfc(|z|max/sqrt 2) x n_bins` against 0.005 with z* from `NormalDist().inv_cdf(1 - alpha/n_bins/2)` (3.978 at 72, scipy `norm.isf(0.005/72/2)` = 3.978181192219203); the null is side A's split and a failing null makes the jet rates `info` and rescales nothing; a failing null does not excuse a pull. The code was right; the tests were not able to say so (F1-F5).

## Fixed (`61a5af1`)

- **Tests, Python**: 16 to 30 (`pixi run -e pythia --skip-deps test-mlm-pythia`: `Ran 30 tests ... OK`). New: `Chi2Sf` (scipy values on both branches, even-dof closed form, degenerate arguments), `Thresholds` (chi2 leg probed at p = 0.0015 fail and p = 0.0019 pass at 23 dof with scipy `isf` values; per-bin leg at |z| = 3.93 pass and 4.03 fail, both signs; 36-bin total with z* 3.810), `Judge` (`gated_pulls` keys, missing pull, A's null decides and B's does not), `NullSplit` (5 files split 3+2 with files, events and runs sliced alike, 2 files 1+1, 1 file no null), `Build` (version string, conda-meta package, no-package case).
- **Tests, Rust**: 5 added (38 + 6 became 38 + 11): four for `check_standalone_modes` (matching mode; the other mode, naming key, both modes and file; no / unreadable / non-string mode; absent row, undeclared mode and no row left alone, which is the "omits `mode` stays as it was" case) and one for the build text.
- **Refactor**: `judge(comparison, nulls)` takes the three wiring lines out of `main`; `check_standalone_modes` takes `&[Standalone]`. Identity oracle chosen first: the full `validate-mlm-pythia` run read back from the cache, stdout and the whole row JSON. After the change, the row is identical except `pythia_version` (`8.3120e+00` to `8.312`) and the new `pythia_build` (Python: `rest identical: True` for the dict and for `json.dumps(sort_keys=True)`); stdout differs in the Pythia line and the output path only (`diff` of the two logs).
- **Version and build** (the brief's two Found items): `pythia_version()` formats the driver's float as `8.312` in Python. I did not edit `mlm_match.cc`: the run cache keys on the driver binary's sha, and rebuilding it would have invalidated the cache. `pythia_build()` records `platform`, `pythia8-config --version` and the conda package, channel and subdir. Recorded now: `{'platform': 'Linux x86_64', 'pythia8_config_version': '8.312', 'package': 'pythia8-8.312-py312hd50de4e_105', 'channel': 'https://conda.anaconda.org/conda-forge/', 'subdir': 'linux-64'}`. The collator prints `; Pythia 8.312 (pythia8-8.312-py312hd50de4e_105, conda-forge/linux-64, Linux x86_64)` (from `cargo run -q -p vibegraph-validation-report | grep mlm-pythia`).
- **Manifest note**: V1's `py311h5756c45_105` was the lock file's first linux-64 entry, not the environment's; corrected to `py312hd50de4e_105`.

## Mutations

Driver `mlmpy-HY1-mutate.py` applies each one-line mutation to a scratch copy and runs the unit tests; logs `mlmpy-HY1-mut-before.log` (old tests, 41 mutants) and `mlmpy-HY1-mut-after.log` (new tests, 46 mutants: the same plus the version and build ones and the `judge` rewrite of the wiring mutant).

- **Old tests: 25 caught, 16 missed**: alpha, pull limit and `<=` on pulls, `p_min` to `max`, sidedness of z*, bin-pull sign and error, null leg selection (both single-leg variants), a failing null excusing a pull, all caught. Missed: the three chi2-verdict factors (`p_min`, `2 p_min`, `4 p_min`), `<=` on both legs, per-bin `n_bins / 2`, one-sided per-bin p, the three `null_split` mutants, both `gated_pulls` mutants, null-B wiring, exit code, `chi2_sf` continued fraction, `chi2_sf` shifted dof.
- **New tests: 43 caught, 3 missed**. Missed: the two `<=` mutants (equivalent: equality of `p x trials` and alpha to the last bit is not constructible) and `main` returning 0 always.
- **Exit code, negative control instead of a unit test**: with `PULL_LIMIT = 1.5` temporarily in the tree and reverted from a backup, `validate-mlm-pythia --out <scratch>` exited 1 and printed `verdict: FAIL (pulls |pull| < 1.5: failed 1, 2; jet rates gate: pass)`.
- **Rust**, `mlmpy-HY1-rmut.py` (11 mutants, `cargo test -p vibegraph-validation-report`): all caught: `!=` to `==`, absent file as a problem, undeclared mode checked as gate, no row still checked, message without the key, declared mode printed in the written slot, channel segment, platform, subdir, version only with build, the ⚠️/✅ mark. Before this commit `check_standalone_modes` had no test, so none of its mutants could fail.
- **No mutation can reach**: the exit code (above) and the `<=` equalities. The five pulls and the legs are blind, by construction, to what D1 lists; I added no claim past that.

## Routed

None. No finding moves a computed number.

## Gate

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished dev profile [optimized] target(s) in 2m 46s` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | `Finished dev profile [optimized] target(s) in 1m 14s` |
| `cargo test --workspace` | 39 result lines, 1485 passed, 0 failed, 16 ignored (V1: 1480; the 5 are the Rust tests above) |
| `cargo test -p vibegraph-validation-report` | `38 passed`, `11 passed` (after the final edit) |
| `cargo doc -p vibegraph-validation-report --no-deps --document-private-items` | 0 warnings |
| `pixi run -e pythia --skip-deps test-mlm-pythia` | `Ran 30 tests ... OK` |
| `pixi run -e pythia --skip-deps validate-mlm-pythia` | exit 0, `verdict: PASS (pulls |pull| < 3: all pass; jet rates gate: pass)`; every number identical to the base run, 71 s from the cache |

`cargo test --workspace` and the two clippy runs started before the manifest note edit (one string in `manifest.toml`); `cargo test -p vibegraph-validation-report`, which parses the manifest, was re-run after it. No banked extended-validation target is touched (no amplitude, colour, coupling, diagram, phase-space, sampling, PDF, scale or LHEF code changed).

## Method

- Mutation of the verdict logic (about 60 % of the session): a scripted batch over a scratch copy found 16 misses in 41 for a 16-test suite that looked complete by reading. The shared pattern: the tests assert at the points the author thought of (3.0, 4.1) and the thresholds' factors live between them.
- Probing the tests' oracle (self-reference): the trial-factor test compares the output with a quantity computed from the same output; the histogram fixtures take their p-values from `chi2_sf`, so a wrong `chi2_sf` would move every fixture with it. External values (scipy, closed forms) break this.
- Identity oracle on the real cached run for the refactor: cheap (about 70 s) and complete.
- Did not pay off: comment-history greps, function-length and visibility checks (no hits).

## Found

1. **`chi2_sf` is a hand-written incomplete-gamma function** (`validation/pythia/mlm_match.py`), while `validation-report/Cargo.toml` and `vibegraph-lib` use `puruspe` for the same function. AGENTS.md says not to hand-write a standard primitive. It is now pinned by scipy values and closed forms, so it is correct, but it is a second copy. *To close:* add `scipy` to `[feature.pythia.dependencies]` (the environment has no scipy today) and use `scipy.stats.chi2.sf`, or have the driver emit chi2 and dof only and let the collator compute p.
2. **The recorded build describes the environment of the invocation, not of the cached runs.** `pythia_build()` reads the current environment; the cache key (`input ... driver <sha>`) does not include the Pythia library, and a rebuilt driver with the same sha could load a different `libpythia8`. So a cache read back after an environment change would be labelled with the new build. *To close:* put `pythia8-config --version` and the conda build string into the cache key (this invalidates the 410-run cache, about 5.6 h on this host), or have `mlm_match.cc` print the library's build info per run.
3. **`jet_rate_legs` raises `TypeError` on a histogram with dof <= 0** (`p_value` None, fewer than two bins after the >= 100-event merging). It is loud, not green, but a thin sample would crash the gate instead of reporting. *To close:* treat a histogram with no p-value as a failed leg or exclude it from the trial count, with a test.
4. **V1 report and the KB pages**: V1's report names the build `py311h5756c45_105` (the lock's first entry; the environment carries `py312hd50de4e_105`), and Found 3 of V1 (the KB pages still say `info`) stands. Not edited, per the brief's `research/kb/` rule.
5. **Per-file scatter of vibegraph's `@1`** (V1 Found 1) and the `mean_of` item are the same fact seen twice: the gate takes `error = quoted`. Not touched.

## Brief corrections

- "`pythia8-config --version` or equivalent": it prints only `8.312`, with no build string, so the record also reads `conda-meta` for the package build; the conda part is absent outside a conda environment (tested).
- "Print it as the version string" in `mlm_match.cc`: changing the C++ would invalidate the run cache, which the assignment says not to clear. The formatting is done where the version is read.
- "The row should record which Pythia build and platform ran": true for the invocation, not for cached runs (Found 2).
