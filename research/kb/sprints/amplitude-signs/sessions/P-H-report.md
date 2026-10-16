---
type: Session Report
title: "P-H report: hygiene review of the probe-production bridge"
description: "Review of PR #31 on four points. The bridge's one function is split, three blind spots are closed (comparison counts, colour-factor source, row counts), the subsumed eemumu test is deleted, and two cross-module items are filed."
status: draft
tags: [hygiene, evaluator, oracle, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: fix, resource: "https://github.com/nsmith-/vibegraph/commit/3d84f53", title: "test(eval): tighten the probe bridge and delete the test it subsumes"}
---
Scope: `git diff origin/main...HEAD` code, that is `helas/eval/run/probe_bridge.rs`
and its two touches in `run.rs`. Nothing outside `#[cfg(test)]` changed. The findings
below were identified by reading before the first edit; the table is the record of what
the review found, and the fixes came after.

## Findings

| id | point | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 1 | `probe_bridge.rs` `bridge` | 160 lines (`awk '/^fn bridge/,/^}/' \| wc -l`) holding setup, three readouts and five comparison loops | checked | **fixed**: `Case` struct with `probe_amplitudes`, `check_colour_free`, `probe_jamps`, `contract`, plus `expanded_amp_rows`, `check_level`. `bridge` is now 123 lines (the rest is assert text) |
| H2 | 2 | `probe_bridge.rs` m2 check | the probe contraction used `bound.cf_weights()`, the same array `eval_m2` contracts with, so a wrong rational-to-float conversion in `BoundAmplitude::bind` cancels | checked | **fixed**: weights from `eval.cf_matrix()` converted with `ToPrimitive`. Mutation M1 below |
| H3 | 2 | the three loops | no count of comparisons made, so a truncated loop (skipped helicity or amplitude) stays green unless `|M|²` happens to see it | checked | **fixed**: `n_amp == 2·points·helicities·n_amps`, `n_flow == 2·points·helicities·n_flows`. Mutations M4, M6 |
| H4 | 2 | `expanded_amp_rows` | `chunks_exact(n).zip(...)` drops a remainder and a surplus row silently | suspected, settled by reading | **fixed**: asserts `amp_locs.len() == helicities·n_amps`, `hel_jamps.len() == helicities·n_flows`, `amp_rows.len() == helicities`. No mutation reaches these (they need a production program with a wrong row count); they are shape guards |
| H5 | 2 | m2 comparison | `m2_probe.abs().max(MIN_POSITIVE)` divisor: an all-zero set of probes and a zero production value would be green | checked | **fixed**: asserts `m2_probe > 0` per point. A per-combination non-zero assert is wrong (helicity-forbidden combinations exist; a first attempt failed on `e+ e- > mu+ mu-` hel `[-1,-1,-1,-1]`), so it sits at the point level |
| H6 | 1 | `probe_bridge.rs` | `std::sync::Arc<...>`, `crate::helas::color::colorize::Contribution`, `crate::diagrams::diagram::FermionLine` spelled in full at 5 sites | checked | **fixed**: imports |
| H7 | 2/1 | `run.rs` `test_whole_amplitude_equals_diagram_sum_eemumu` | P1 says the bridge subsumes it. It checks `eval_amplitude` against the sum of the colour-free probes for `e+ e- > mu+ mu-` | checked | **fixed**: deleted, and the comment in `test_eval_jioxxx` that named it now names `probe_bridge`. Evidence below |
| H8 | 4 | `probe_bridge.rs::points` | near copy of `AmplitudeEvaluator::generic_probe_points` (`compile.rs:566`, private): ±z beams, `rambo_massive`, two scales. Different seed, scales and floor | checked | **Found** (see Found) |
| H9 | 1 | comments | `grep -nE '\bT[0-9]\b\|\bC[0-9]\b\|Phase [0-9]\|note [0-9]+\|used to\|no longer\|P1\|session'` on the file: no hits | checked | no finding |
| H10 | 3 | visibility | the change adds no `pub`; `cf_weights` is `pub(crate)` with a production reader (`reweight/engine.rs:393`) | checked | no finding |
| H11 | 1 | `lower::lower` doc claim "test-only" | `#[cfg(test)]` on `lower.rs:39` | checked | claim holds |
| H12 | 1 | `bridge` still 123 lines | linear sequence of one setup and five comparisons, with assert messages | checked | **leave**: further splitting moves assert text, not logic |
| H13 | 1 | literal `3` for Nc in `colour_coeff` | `const NC: i64 = 3` exists privately in `lower.rs:56` and `colorize.rs:59` | checked | **leave**: the literal is the test's own statement of the convention it checks against; sharing the constant would couple the oracle to what it checks. The two production copies go to Found |
| H14 | backlog | filed items naming `run.rs` / `compile.rs` / `eval/` | `grep -rn "eval/run\|/run\.rs\|compile\.rs" research/kb/backlog`: `spin-2-*` (`build_external_core`), `jioxxx-reference-port-comparison-has-no-teeth` (`test_eval_jioxxx`), `generate-artifact-not-self-contained`, `config-tag-uses-signed-colour` | checked | none names a site this change touches or leaves wrong |

## Identity oracle for the refactor (H1)

Before editing, the residual table was printed at full precision (`{:e}`, which
round-trips an `f64`) from the original file. The table is five residual columns by
eleven processes plus the comparison counts and the `Features` line. After the split
the same diff over `grep -E '^(e|w|g|u|z)|Features'` is empty: every residual and
count is bit-identical. The first split attempt moved the m2 sum to per-combination
partial sums, and the diff showed it (m2 changed at 1e-16 on four multi-flow rows).
`contract` therefore returns the terms so the accumulation order is the original.
Switching the weights to `cf_matrix` (H2) left the table bit-identical too.

## Mutations

Driver: `/tmp/claude-0/ph/mutate.py` (applies one string replacement, runs
`cargo test -p vibegraph-lib --lib -- probe_bridge test_whole_amplitude_equals_diagram_sum_eemumu`, reverts).
"orig" is the bridge as PR #31 had it, with the old test alongside. "new" is the
bridge after this commit.

| mutation | old test | orig bridge | new bridge |
|---|---|---|---|
| M1 `BoundAmplitude::bind`: CF numerator × 2 (`run.rs`) | passes | **passes** | fails: `e+ e- > mu+ mu-: eval_m2 7.77e-2 vs probe contraction 3.89e-2 (rel 1.00e0)` |
| M2 `lower_flows`: drop the last term of a multi-term flow | fails | fails (`flow 0 ... rel 3.21e-2`) | fails (same row) |
| M3 `lower_flows`: symmetry × Fermi factor × 2 | fails | fails (`rel 9.68e-1`) | fails (same row) |
| M4 test loop skips the last helicity combination | passes | fails, late: `e- a > e- a: eval_m2 ... rel 7.63e-1` (the skipped `ee>mumu` combination contributes zero to `\|M\|²`) | fails on the first process, at the count assert |
| M5 test `colour_coeff` ignores `i` | passes | fails (`g g > u u~: flow 0 ... rel 1.41e0`) | fails (same row) |
| M6 test skips the last configuration amplitude | passes | **passes** | fails at the count assert |

- **H7:** M2 and M3 fail the old test and the bridge's first row alike. No mutation
  was found that fails the old test only. The old test also pinned the colour-free
  probes' sum, and the bridge asserts each colour-free probe equals its chain-zero
  amplitude to the bit before summing those. The old test used a single massless-card
  point; the bridge uses four points per process on the default card.
- **Not reached:** the shape guards of H4 (they need a production program with a wrong
  row count) and the non-zero guard of H5 (it needs a point where every amplitude vanishes on both sides; I did not
  construct one).
- **Blind spots that remain:** the transpose of the CF matrix (`cf[j·n+i]` vs
  `cf[i·n+j]`), because CF is symmetric. Anything the two paths share, as P1 lists.

## Gates

Every cargo command ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo fmt --all --check`: exit 0 (after `cargo fmt --all` reformatted two spots in the new code).
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile ... in 2m 05s`, exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `Finished dev profile ... in 42.10s`, exit 0.
- `cargo test --workspace`: exit 0; 38 suites, 1395 passed, 0 failed, 16 ignored. P1 reported 1396 passed; the one fewer is the deleted test.
- `cargo test -p vibegraph-lib --lib probe_bridge -- --nocapture`: `1 passed; 0 failed`, residual table as P1's (worst 2.4e-14 per flow, `g g > g g`).
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: exit 0, 8 warnings, none in a file this change touches (`coupling/cluster/configs.rs`, `phasespace/channel.rs`, `coupling/alphas.rs`, `coupling/scales.rs`, `ufo/parameters.rs`, `lhef/mod.rs`). The branch base has the same files unchanged.
- Banked targets: none run. The change is `#[cfg(test)]` only, so no production code, and no banked gate, reads it.

## Method

- Reading the test against the P1 report's own claims, 30%: gave H2 (a shared array on both sides), H4, H7.
- Mutation design, 40%: H3 and H6 came from asking what a truncated loop does. The orig bridge's M4 was caught only because of a later process; M6 was not caught at all.
- Grep for the helper's body in the workspace, 10%: H8.
- A "zero probes" guard failed on first try because helicity-forbidden combinations are real; the point-level guard is the right level.
- The comment-leftover and `pub` greps found nothing.

## Found

- **`points` duplicates `AmplitudeEvaluator::generic_probe_points`.** `probe_bridge.rs:120` and `compile.rs:566` (private) both build ±z beams plus seeded `rambo_massive` at two scales. Another copy of the ±z shape is at `reweight/engine.rs:962`. To close it, make the compile.rs generator `pub(super)` with its seed and scales as arguments, and call it from the bridge. That edits non-test code in the evaluator module, which this session was not allowed to do.
- **`NC = 3` is declared twice.** `helas/eval/lower.rs:56` and `helas/color/colorize.rs:59` are private constants of the same value. To close it, export one from the colour module.
- **Contact diagrams still have no per-amplitude readout** (P1's Found, unchanged). The bridge sees them only inside flow sums.
- **The brief's "about 400 lines in one test" is wrong in the useful way:** the `#[test]` function is 41 lines, `bridge` was 160 and the file was 395 lines. It is 515 now, with the new helpers.
