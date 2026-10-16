---
type: Session Report
title: "J-H report: hygiene review of the jioxxx chirality and longitudinal test"
description: "Findings of the four-point hygiene review of PR #34 (test_eval_jioxxx and the jioxxx reference port), written before any edit, with their dispositions."
status: draft
tags: [hygiene, helas, review, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/f9b0b05", title: "PR #34 tip reviewed"}
  - {id: findings, resource: "https://github.com/nsmith-/vibegraph/commit/dff72b8", title: "findings written before any edit"}
  - {id: fix, resource: "https://github.com/nsmith-/vibegraph/commit/b93acb7", title: "ALOHA-check count and shared diagram builders"}
---
Review of `git diff main...HEAD` on `amp-signs/jioxxx-chiral-oracle`
(`vibegraph-lib/src/helas/eval/run.rs`, `vibegraph-lib/src/helas/vertex.rs`).
The findings table was committed (`dff72b8`) before any code edit; the final
dispositions are in the last column and the sections below it.

## Findings (before any edit)

| id | point | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `run.rs` ALOHA cross-check, `match (m_in != 0.0, case.current_terms)` then `if let Some(aloha)` | The check is optional per case and nothing counts how many ran. A defect that makes the match fall through (a changed guard, a renamed term) leaves the ALOHA comparison, the only independent pin of the longitudinal denominator, silently empty while the test stays green. Mutation: `(true, [("FFV2", c)])` to `(false, ...)`. | checked: the mutation passes the old test (below) | **fixed** (`b93acb7`): count the comparisons and assert the independently derived number (2 structures x 2 propagators x 4 helicity pairs = 16) |
| H2 | 1, 4 | `run.rs` `test_eval_jioxxx` (326 lines, `awk` over the fn) and the two tests after it | The fn builds four `ExtLegInfo` literals and the e+e- current diagram inline. The same 4-node current diagram and the same two `ExtLegInfo` literals recur in `test_eval_ffv2_4_3` and `test_longitudinal_z_current_transverse_for_massless_fermions` (`grep -n "ExtLegInfo {"`: 2299, 2306, 2553, 2560, 2720, 2727). The full-amplitude diagram is a 40-line inline closure. | checked | **fixed** (`b93acb7`): shared `fermion_leg` and `s_channel_current_diagram` helpers, amplitude-diagram builder out of the test; identity oracle is the printed residuals |
| H3 | 1 | `run.rs` comments in `test_eval_jioxxx` | (a) the `ref_amp` comment is hard-wrapped mid-sentence ("... to pin. Read reversed, the line couples" runs past its neighbours); (b) "Both comparisons are relative to the largest reference over the helicities" now sits above only the current scale, `amp_scale` is computed far below. | checked | **fixed** (`b93acb7`) |
| H4 | 1, 2 | `run.rs` closures `helas_gc` and `aloha_coupling` | Both repeat `evaluated.coupling(model.coupling_id(c).unwrap()).im`. The `.im` silently drops a real part, so the claim that the SM FFV couplings are pure imaginary is an unchecked convention. | checked (duplicate); suspected (real part is exactly 0 for GC_3/50/51/58/59): settle with an assert | **fixed** (`b93acb7`): one closure that asserts the real part vanishes |
| H5 | 3 | `vertex.rs` `pub fn jioxxx`/`iovxxx`, `pub use` in `helas/mod.rs` | `grep -rln "jioxxx\|iovxxx" --include=*.rs .` lists `vibegraph-lib/tests/helas_kernel_composition.rs`, an integration target, and `docs/src/guide/04-helicity-amplitudes.md:44` links the rustdoc page. Both need `pub`. | checked | **rejected** (public API needed by an integration test and the book) |
| H6 | 1 | `ffv2_3`/`ffv4_3` and `ffv_chiral_weights` duplicating something in the tree | `grep -rn "fn ffv2_3\|fn ffv4_3"` gives only `vertex.rs:132,170`, both `#[cfg(test)]` and unchanged by the PR. `grep -rn '"FFV4"\|"FFV5"' src tests` finds no other chiral-weight table. | checked | **not reproduced** |
| H7 | 1 | history references in the diff | `git diff main...HEAD -U0 -- src \| grep -E '\bT[0-9]\b\|Phase [0-9]\|used to\|no longer\|the old\|session'` is empty | checked | **not reproduced** |
| H8 | 2 | M5 (`chiral_weighted`) not reached by this test | stated blind spot in the brief and in the J1 report; S5 shows `helas::` tests catch it | checked | **rejected** (stated blind spot) |
| H9 | 1 | `research/kb/amplitudes/wavefunctions-and-propagators.md:108-115` | The page says `jioxxx` divides by complex `m^2 - i m Gamma` and that `test_eval_jioxxx` uses massless fermions and cannot tell the forms apart. Both statements are false after this PR (`vertex.rs:61-67`; the massive regime). | checked | **filed** (Found 1) |

Backlog check: `grep -rln "run.rs\|vertex.rs" research/kb/backlog` lists
`hygiene/comment-leftovers-after-hygiene-sprint`,
`validation/qcd-2to3-rows-unbanked` and two others; none names a line of
`test_eval_jioxxx` or `jioxxx`, so none is a finding here.

H2 leaves `test_eval_jioxxx` at 255 lines (it was 326, of which about 70 are doc
and comments). It is one loop body over the case table whose reference closures
share the kinematics of the case; splitting it would pass about fifteen locals
through a helper signature. That is the named reason it stays one function.

The brief's two specific questions: the massless and massive regimes share
their setup through the `JioxxxCase` table (one loop, no duplicated setup), and
`ffv2_3`/`ffv4_3` duplicate nothing (H6).

## Mutation evidence

All run with `cargo test -p vibegraph-lib --lib test_eval_jioxxx`, applied
temporarily and reverted (file compared byte-for-byte afterwards).

| mutation | old test (`f9b0b05`) | new test |
|---|---|---|
| H1: `match (m_in != 0.0, ..)` becomes `match (false, ..)`, the ALOHA comparison never runs | **passes** (`test result: ok. 1 passed`) | fails: `ALOHA comparisons run`, left 0, right 16 |
| H1b: the `"FFV4"` arm rewritten to `"FFV2"`, so FFV4 is never ALOHA-checked | passes (the FFV4 arm is then unreachable, no check on it) | fails: left 8, right 16 |
| H4: the pure-imaginary bound made unsatisfiable (liveness of the assert) | n/a, no such assert | fails: `GC_3 = -0-0.3079537672443688i is not imaginary` |
| helper: `(first, second)` swapped in `s_channel_amplitude_diagram` | n/a (helper is new) | fails in the amplitude comparison, as the pre-existing M4-class reversed-orientation check requires |

The H4 assert guards an assumption (no real part) that no one-line production
defect reaches; the liveness mutation only shows it is not dead. No mutation
reaches a coupling whose real part is nonzero, since none exists in the SM
restrict used here.

## Identity check for the diagram-builder refactor

Oracle chosen before the first edit: the per-assertion residuals. A temporary
`eprintln!` after each of the four residual computations (current, ALOHA,
amplitude, divergence guard) printed `residual / scale` at full precision for
the base test and for the refactored test.

```
base:      534 RESID lines (56 current, 16 ALOHA, 448 amplitude, 14 divergence)
refactored: 534 RESID lines
cmp base.resid new.resid  ->  IDENTICAL
```

The instrumentation was removed before the commit. The 16 ALOHA lines also
independently confirm the count the new assert pins (2 x 2 x 4).

## Gate

All with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_{DEV,TEST}_DEBUG=0`
at `b93acb7`:

```
cargo fmt --all --check                                          exit 0
cargo clippy --workspace --all-targets -- -D warnings            exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings   exit 0
cargo test --workspace                                           exit 0  (38 suites: 1395 passed, 0 failed, 16 ignored)
cargo test -p vibegraph-lib --lib helas::                        test result: ok. 247 passed; 0 failed; 8 ignored
cargo doc -p vibegraph-lib --no-deps --document-private-items    exit 0, 8 warnings, none in helas/
```

The suite count is unchanged from J1 (1395): no test was added or deleted. No
banked target was run: the change is `#[cfg(test)]` code only and touches no
amplitude, colour, coupling, diagram or coverage-list code.

## Method

About 40% reading the diff against the test's own claims (what each branch can
miss), 25% grep for duplicates and visibility users, 20% mutation and identity
runs, 15% gating. The productive lead was "what happens if an optional branch
never executes" (H1); the checklist greps for history words, duplicate
primitives and `pub` users found nothing here (H5, H6, H7).

## Found

1. **`research/kb/amplitudes/wavefunctions-and-propagators.md:108-115` is stale.**
   It says `jioxxx` divides by complex `m^2 - i m Gamma` and that
   `test_eval_jioxxx` uses massless fermions and cannot tell the forms apart.
   After PR #34 `jioxxx` divides by real `m^2` (`vertex.rs:61-67`) and the test
   has a massive regime with an ALOHA cross-check. Closes when the bullet
   describes the aligned port and the massive test.
2. **Three tests in `run.rs` build the same e+e- current diagram** (`test_eval_jioxxx`,
   `test_eval_ffv2_4_3`, `test_longitudinal_z_current_transverse_for_massless_fermions`);
   they now share the helpers. The `ExtLegInfo` literals at `run.rs:~4126-4301`
   (`leg_mup`, `leg_mum`, `leg_ep`) in the whole-amplitude tests are the same
   pattern with different leg data and were left as they are (outside the change);
   `fermion_leg` would fit them.
3. **Brief correction.** The brief's gate lists `cargo test -p vibegraph-lib --lib helas::`
   which passes (247), but it names no doc gate; `cargo doc` was run per the
   agent definition. The brief's "about 250 lines" for the test growth is right
   for the diff (`+264/-122`); the function is 326 lines before and 255 after.
