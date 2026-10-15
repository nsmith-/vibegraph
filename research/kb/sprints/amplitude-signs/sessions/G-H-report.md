---
type: Session Report
title: "G-H report: hygiene review of the t t~ + jet per-flow gate"
description: "PR #29's five standalone rows, five tests and one comment hold up on all four points; one test is added so the four-gluon contact sensitivity of the gg_to_ttxg gate stays under test, and two stale row lists in the kb are filed."
status: draft
tags: [hygiene, review, standalone-jamps, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-15}
sources:
  - {id: df93d59, resource: "https://github.com/nsmith-/vibegraph/commit/df93d59", title: "keep the gg_to_ttxg gate's colour-assignment sensitivity under test"}
---
Scope: `git diff main...HEAD` code and comments, that is the five rows in
`validation/madgraph/gen_standalone_jamps.py`, the five tests in
`vibegraph-lib/tests/standalone_jamps.rs`, and the comment in
`color_cf_oracle.rs`. The findings were written before the edit.

## Findings

| id | point | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `standalone_jamps.rs` (`gg_to_ttxg`) | The gate's colour-to-Lorentz sensitivity at the four-gluon contact was shown only by temporary mutations. Mutation the gate must keep catching: remap the contact's couplings keys `(c, l)` to `(perm[c], l)`. Nothing in the tree fails if a table or tolerance change makes the gate blind to it. | checked | **fixed** (df93d59) |
| H2 | 1 | the five new `#[test]`s | Five 3-line tests with per-row doc comments, matching the ten existing ones. A table or macro would drop the doc comments that say what each row covers, and no run today can use the saved lines. | checked | **rejected** (see below) |
| H3 | 1 | comments in the five rows and tests | `grep -nE '\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer'` on both files returns nothing. The claims were re-read against the process: top lines of `u u~ > t t~ g` are timelike; the beam-attached top in the other four is spacelike (`zero_widths` as for `ee_to_wpwmz`). | checked | no defect |
| H4 | 1 | row lists in `research/kb/amplitudes/vector-vertex-signs.md:177` and `research/kb/validation/amplitude-oracle.md:203` | Both list the rows "at `787070e`" or without the five new ones (`vector-vertex-signs.md` omits them; `amplitude-oracle.md:208-209` names them). | checked | **Found** (kb edits barred) |
| H5 | 2 | tests | Each new test calls `check(key)`, so a missing table, a non-matching process, NCOLOR and helicity-set mismatches panic and a zero-entry table cannot pass (G is fitted over the entries; `measure` asserts the sets). No `catch_unwind`, no OR of bounds. REL_TOL is relative to the table's largest amplitude. | checked | no defect |
| H6 | 3 | all | The change adds no `pub` item and no `cfg`. | checked | no defect |
| H7 | 4 | all | The five rows share the `zero_widths=True` knob with 6 older rows; no new default-only knob. | checked | no defect |
| H8 | 2 | no table-to-test coverage check | A banked `validation/madgraph/standalone/*.json` with no `#[test]` would never run. 15 tables, 15 `check` tests today. The check would need the test names as data, i.e. the macro/table H2 rejects. | checked | **Found** |

Backlog: no filed item names `standalone_jamps.rs` or `gen_standalone_jamps.py`
except `wpwm-to-epem-neutrino-exchange-sign` (the unrelated `KNOWN_DISAGREEMENT` row),
which this PR does not touch.

## Fixed

**H1.** `measure` became `measure_with(key, alter)`, `measure` calling it with a no-op;
`alter` edits a clone of the model before the evaluator compiles. The new test
`gg_to_ttxg_sees_the_contact_colour_assignment` finds the all-gluon four-point
vertex, permutes its colour indices, and requires worst flow and worst |M|² both
above 1e-2 for all five non-identity permutations. The floor comes from the
readings (weakest 4.372e-2 flow, 2.853e-2 |M|², matching the G1 report's table),
a factor of about three below, and orders of magnitude above `REL_TOL`.

This is the one G1 Found item that is a hygiene fix. The `f^{abc}` and
`T^a T^b` mutations need crate-internal hooks; those stay filed.

## Mutation evidence

| mutation (temporary, reverted) | new test |
|---|---|
| none: readings printed for each permutation | passes. Flow / \|M\|²: [1,2,0] 1.293e-1 / 1.508e-1; [2,0,1] 1.250e-1 / 1.077e-1; [1,0,2] 4.372e-2 / 5.938e-2; [0,2,1] 4.544e-2 / 2.853e-2; [2,1,0] 1.695e-1 / 1.826e-1 |
| identity permutation `[0,1,2]` added to the list | fails: `colour permutation [0, 1, 2]: worst flow 6.886e-16, worst \|M\|² 1.327e-15` |
| `alter(&mut altered)` replaced by `let _ = alter;` (edit never reaches the model) | fails: `colour permutation [1, 2, 0]: worst flow 6.886e-16, worst \|M\|² 1.327e-15` |

The old tests cannot see these mutations: they edit the model inside the new test only. The
asserted vertex count (`contacts.len() == 1`) failed on my first selector (0 found) before
I corrected it, so the selector cannot match nothing silently. Blind spot: the test
moves colour against Lorentz only at this one vertex; a permutation that happens to be a
symmetry of the contact's coefficients would be undetected, but all five are not.

## Rejected: table-driven tests (H2)

The brief asked whether five near-identical tests read better table-driven while keeping
one test name per row. Only a macro keeps the names, and the existing ten tests and these
five carry a doc comment each that says what the row covers (the non-obvious part, e.g.
why `ug_to_ttxu` and `gu_to_ttxu` both exist). A macro over `(name, docs)` saves about
two lines per row and obscures `cargo test <row>`'s source. Not worth its code.

## Gates

Run in the worktree with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished dev profile`, exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | `Finished dev profile`, exit 0 |
| `cargo test --workspace` | summed over suites: 1401 passed, 0 failed, 16 ignored (G1 had 1400: +1 is the new test) |
| `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test standalone_jamps --test color_cf_oracle` | `97 passed` (color_cf_oracle), `16 passed` (standalone_jamps), 0 failed |
| `cargo doc -p vibegraph-lib --no-deps --document-private-items` | Finished; 8 warnings, all in `src/` docs this change does not touch |

## Found

1. `research/kb/amplitudes/vector-vertex-signs.md:177` lists the `standalone_jamps`
   rows without `gg_to_ttxg`, `ug_to_ttxu`, `gu_to_ttxu`, `gux_to_ttxux`, `uux_to_ttxg`.
   Closing: add them to the row list.
2. `research/kb/validation/amplitude-oracle.md:203` still lists the rows "at `787070e`"
   and `zero_widths` rows without the five new ones (lines 208-209 name them).
   Closing: update the sentence.
3. The `f^{abc}` orientation and `T^a T^b` order mutations of G1 stay unpinned as permanent
   falsifiers; they need a crate-internal hook (G1 Found 2 remainder).
4. Nothing ties the table directory to the test list (H8): a banked table without a
   `#[test]` is not run. Closing: a test comparing `standalone/*.json` file stems to a
   `ROWS` list the test functions are generated from.
5. Brief correction: `color_cf_oracle` is gated behind `extended-validation`, so the brief's
   gate command `cargo test -p vibegraph-lib --test standalone_jamps --test color_cf_oracle`
   errors without the feature. It is a comment-only change there.
