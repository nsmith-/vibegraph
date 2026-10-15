---
type: Session Report
title: "G1 report: t t~ + jet against MadGraph standalone, per flow"
description: "All five t t~ + jet matrix-element classes (g g, u g, g u, g u~, u u~) agree with MadGraph 3.7.1 standalone per flow to <= 6.9e-16, colour-assignment mutations fail them at O(1), and MadEvent's width pattern agrees too, so the @1 sigma excess is not in |M|^2."
status: draft
tags: [validation, amplitudes, color, ttx, mlm, standalone-jamps, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: 556ec90, resource: "https://github.com/nsmith-/vibegraph/commit/556ec90", title: "bank t t~ + jet standalone JAMP tables and their per-flow gates"}
---
The dev agent's report. Every number is copied from a command run in the
session. The gate commands and their output are under "Gates".

## Verdict

Every `p p > t t~ j @1` matrix-element class agrees with MadGraph standalone
(the pinned 3.7.1, through `mg5_pinned.sh`) per helicity and per flow, at REL_TOL
1e-12. Each table holds 3 RAMBO points at √s = 1000 GeV, with every width zeroed
on both sides.

| row | process | entries | G | worst flow | worst \|M\|² |
|---|---|---|---|---|---|
| `gg_to_ttxg` | `g g > t t~ g` | 576 | `+1.0000000000000000i` | 6.886e-16 | 1.327e-15 |
| `ug_to_ttxu` | `u g > t t~ u` | 384 | `−0.9999999999999993i` | 3.041e-16 | 5.120e-16 |
| `gu_to_ttxu` | `g u > t t~ u` | 384 | `−0.9999999999999996i` | 4.177e-16 | 5.040e-16 |
| `gux_to_ttxux` | `g u~ > t t~ u~` | 384 | `+0.9999999999999993i` | 3.550e-16 | 5.592e-16 |
| `uux_to_ttxg` | `u u~ > t t~ g` | 384 | `−0.9999999999999996i` | 4.322e-16 | 7.861e-16 |

The `pp_to_ttx_0j1j_mlm` MadEvent work area has three `@1` directories.
`P1_gg_ttxg` is `g g > t t~ g`. `P1_gq_ttxq` holds `matrix1` = `g u > t t~ u` and
`matrix2` = `g u~ > t t~ u~`. `P1_qq_ttxg` is `u u~ > t t~ g`. MadEvent folds
the c, d and s flavours into these matrix elements, and builds the
quark-first and antiquark-first orders by mirroring the beams. Neither is a
separate MadGraph matrix element. So `gu_to_ttxu`, `gux_to_ttxux`,
`uux_to_ttxg` and `gg_to_ttxg` are exactly MadEvent's four `@1` matrix
elements, and `ug_to_ttxu` is the mirrored order as this crate generates it.

**Consequence for `ttx-mlm-at1-sigma-high`:** at fixed momenta and
parameters, MadEvent's own width pattern included, this crate's `@1` |M|²
matches MadEvent's to rounding. The +1.28 % lies outside the matrix element.

## What was done

1. Five rows in `validation/madgraph/gen_standalone_jamps.py` (seeds
   4064–4068), with tables banked under `validation/madgraph/standalone/`. The
   generation log of every table names `VERSION 3.7.1`. Each row sets
   `zero_widths = True` for the reason the existing massive-t-channel rows do.
   In `g g > t t~ g` and the qg rows a top is exchanged between the two beams,
   so it is spacelike. MadEvent zeroes that top's width (`ZERO` in
   `matrix1_orig.f`) and this crate does too (`lower.rs`), but `output
   standalone` keeps `MDL_WT` there. `u u~ > t t~ g` has no spacelike top, and
   zeroing its widths only removes the timelike top width from both sides.
2. Five `#[test]`s in `vibegraph-lib/tests/standalone_jamps.rs`. All five agree,
   so none is listed in `KNOWN_DISAGREEMENT` and each is enforced from the
   start. There was no disagreement to localise.
3. The comment in `color_cf_oracle.rs` naming the per-flow gates that pin the
   four-gluon contact's colour-to-Lorentz assignment now includes `gg_to_ttxg`.
4. The scope was extended mid-session (the manager's message, approved by the
   planner) with the qg and qq~ rows. I added `gu_to_ttxu` and `gux_to_ttxux`
   beside the requested `ug_to_ttxu` because MadEvent's subprocess puts the
   gluon first.

## Falsifiers (temporary mutations, reverted, not committed)

- **Four-gluon contact, colour against Lorentz.** On the SM `gggg` vertex
  the couplings keys `(c, l)` were remapped to `(perm[c], l)` (in the test,
  on a clone of the model). All five non-identity permutations fail
  `gg_to_ttxg`:

  | perm | worst flow | worst \|M\|² |
  |---|---|---|
  | 1,2,0 | 1.293e-1 | 1.508e-1 |
  | 2,0,1 | 1.250e-1 | 1.077e-1 |
  | 1,0,2 | 4.372e-2 | 5.938e-2 |
  | 0,2,1 | 4.544e-2 | 2.853e-2 |
  | 2,1,0 | 1.695e-1 | 1.826e-1 |

  The correct assignment gives 6.886e-16. `gg_to_ggg` fails under every
  permutation too, at 3.8e-2 to 2.2e-1.
- **Triple-gluon `f^{abc}` orientation** (`f(1,2,3)` → `f(2,1,3)` on the
  `ggg` vertex, through an env-gated hook in `ufo/sm.rs`). It fails every row:
  worst flow 1.931 on gg, 2.056 on ug, 1.939 on gu, 1.468 on gux and 0.569 on
  uux.
- **`T^a T^b` order along a quark line** (the chain merge in
  `color/tensor.rs::t_t_pair` concatenating `[oadj, sadj]` in place of
  `[sadj, oadj]`). It fails every row: worst flow 2.129 on gg, 0.999 on ug,
  1.385 on gu, 1.309 on gux and 1.098 on uux. Under the same mutation
  `gg_to_ggg` still agrees, as it should, since it has no quark line.
- A vertex-level `T(a,i,j)` → `T(a,j,i)` swap is refused when the evaluator
  compiles (a panic in `AmplitudeEvaluator::compile`). It is not a usable
  falsifier and is not counted.

## Width diagnostic (not banked)

The standalone `matrix.f` was patched to each MadEvent `P1_*` matrix's width
arguments, with the real card (WT = 1.4915). Each patched table was compared
against vibegraph by swapping it in for the banked table temporarily. Each
control is the opposite width choice on the same lines.

| row | MadEvent pattern | worst flow / \|M\|² | control | worst flow / \|M\|² |
|---|---|---|---|---|
| gg_to_ttxg | 4 beam-attached top lines ZERO, 2 timelike WT | 6.860e-16 / 1.706e-15 | WT on every line | 4.236e-3 / 1.499e-3 |
| gu_to_ttxu | 2 beam-attached ZERO | 4.177e-16 / 5.040e-16 | WT kept | 1.070e-3 / 4.717e-4 |
| gux_to_ttxux | 2 beam-attached ZERO | 3.550e-16 / 5.592e-16 | WT kept | 8.832e-4 / 1.508e-3 |
| ug_to_ttxu | 2 beam-attached ZERO (mirrored) | 3.041e-16 / 5.120e-16 | WT kept | 1.621e-3 / 4.199e-4 |
| uux_to_ttxg | 2 timelike WT (`FK_MDL_WT`) | 3.899e-16 / 4.716e-16 | timelike ZERO | 7.616e-4 / 3.001e-4 |

MadEvent passes `FK_MDL_WT` on timelike lines. That equals `MDL_WT` whenever
WT exceeds MT times the small-width threshold, as it does here.

## Gates

All gates ran on the committed tree (556ec90's content) with
`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`, which only reduces the build's disk use. The log
is `G1-gates4.log` in the session scratchpad.

| command | result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo build --workspace` | `BUILD_EXIT 0` |
| `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture` | `test result: ok. 15 passed; 0 failed`; the five new rows print the numbers in the verdict table |
| `cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture` | `test result: ok. 49 passed; 0 failed` |
| `cargo test --workspace` | summed over suites: 1400 passed, 0 failed, 16 ignored (`TEST_EXIT 0`) |
| `pixi run --skip-deps validate` | the extended-validation clippy is clean; summed over suites 1785 passed, 0 failed, 84 ignored; `[report] 64 rows x 4 categories: the measured cells are the declared cells`; `VALIDATE_EXIT 0` |

`validate` exits non-zero on any enforced cell outside its tolerance, so its
exit 0 is what confirms the enforced cells are unmoved. No source outside the
tests changed.

`pixi run -e kb kb-lint` reports four stale generated indexes after this
report was added (`research/kb/index.md`, `sprints/index.md`,
`sprints/amplitude-signs/index.md`, `sprints/amplitude-signs/sessions/index.md`).
I left `pixi run kb-index` to the manager, since the brief limits my
`research/kb/` edits to this file.

## Blind spots

- **The global phase.** G is fitted per row (±i here), so a sign shared by
  every diagram of a subprocess is invisible.
- **The diagram split within a flow.** Two compensating errors in diagrams
  that reach the same flows with the same weights at every point would cancel.
- **Kinematic coverage.** There are 3 RAMBO points per row at √s = 1000 GeV.
  No soft, collinear or threshold region is visited, and every helicity is
  visited at those 3 points only.
- **Widths.** The banked tables are all at zero width. The MadEvent width
  pattern was checked only as a diagnostic, so a regression in spacelike-width
  zeroing on a massive t-channel line would not fail the banked gate.
- **Everything outside the compiled amplitude.** The flavour grouping, the
  beam mirroring, identical-particle and symmetry factors, PDFs, α_s and its
  MLM reweighting, cuts, matching and sampling are outside this test. Those are
  where the `@1` excess has to be.
- **The contact falsifier also moves |M|²** (by 2.9e-2 to 1.8e-1). A banked
  |M|² gate on this subprocess would therefore have caught the same mutation.
  No such gate existed, and the per-flow gate is strictly finer.

## Found

- **The spacelike-width convention is not banked for massive t-channel
  lines.** Every such standalone row zeroes all widths. The diagnostic above
  shows the per-flow gate sees the convention at 1e-3 when MadGraph's
  width pattern is matched, and agrees at 1e-16. A `madevent_widths` option
  in `gen_standalone_jamps.py` would close it. That option would zero the
  width argument of every top or W/Z propagator call attached to a spacelike
  line, the way MadEvent does, and keep the card's widths elsewhere.
- **The falsifiers are not permanent.** The contact permutation can be done
  from an integration test (`Vertex::couplings` is `pub`). A test asserting
  that the mutated model *disagrees* with `gg_to_ttxg.json` would keep the
  gate's sensitivity to the colour-to-Lorentz assignment from silently
  lapsing. The `f` and `T`-order mutations need crate-internal access.
- **Host disk.** The default `optimized + debuginfo` dev profile plus
  incremental data took this worktree's `target/debug` to 15 GB. That filled
  the shared container mid-gate (an ENOSPC Bus error in `ld`). Cloud briefs
  could set `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` and delete
  `target/debug` before `validate`.
