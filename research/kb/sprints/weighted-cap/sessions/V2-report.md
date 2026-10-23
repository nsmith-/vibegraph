---
type: Session Report
title: "V2 report: the Trying line is printed before the trial, and vibegraph's search accepts MadGraph's bound on every row"
description: "MadGraph prints each Trying coupling order line before its trial, so a successful trial is a printed line (rows 2 and 3 of the M3 build); vibegraph's loop accepts the same WEIGHTED value on all four rows, now pinned by a test of the loop's own record; V1's section 4 is superseded by a new section 5."
status: draft
tags: [diagrams, coupling-orders, weighted, madgraph-parity, sprint-d, report, validation]
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-21}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-21}
---
Worktree `/home/user/wt/E`, branch `sprintd-weighted-cap`, verified first
(toplevel `/home/user/wt/E`, branch `sprintd-weighted-cap`, HEAD `c6e900b`; the
brief says `50dddf0`, the branch tip after the brief commit is `c6e900b`).
MadGraph source: `madgraph/core/diagram_generation.py` at `b7687064`, fetched
from raw.githubusercontent.com (2206 lines, saved as
`.../scratchpad/dg.py`). Host: shared 4-core container, sibling cargo jobs
running; cargo with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`.

## 1. The source, lines cited

`MultiProcess.find_optimal_process_orders` (`diagram_generation.py`):

- `:1978-1980` a one-particle process that is not a decay-chain decay returns
  its (empty) orders: no loop, hence no `Trying` line. **Row 4.**
- `:1998-1999` `max_WEIGHTED_order = (len(fsids + isids) - 2) * int(model.get_max_WEIGHTED())`:
  the cap. For the toy `int(3.5) = 3`, so the cap is `3 (n_ext - 2)`.
- `:1986` the start comes from `get_minimum_WEIGHTED` (the sum of the `n_ext - 2`
  largest leg hierarchy values; every toy field is in the class of value 2).
- `:2013` `while max_order_now < max_WEIGHTED_order:` a trial runs at every
  bound **strictly below** the cap.
- `:2014` `logger.info("Trying coupling order WEIGHTED<=%d: ..." % max_order_now)`
  is the first statement of the loop body: the line is printed **before** the
  trial (the trial is `:2023-2109`).
- `:2104-2107` a trial with diagrams returns `{WEIGHTED: max_order_now}` from
  inside the loop; `:2111` otherwise the bound rises by one.
- `:2115` after the loop (bound reached the cap, nothing found) it returns
  `{WEIGHTED: max_order_now}`, i.e. the cap, with no trial and no line at the cap.

Row by row, with the toy's hierarchies QED = 2, NP = 1, vertex weights as in
`weighted_search.rs`:

| row | start | cap | trials (each printed first) | outcome |
|---|---|---|---|---|
| `fw~ fw > xw xw` | 4 | 2 x 3 = 6 | 4, 5, both empty (diagrams weigh 3 + 3 = 6) | `:2013` fails at 6, loop exits, `:2115` returns 6, caller generates at 6: 2 diagrams. Lines 4, 5 |
| `fw~ fw > xw xw vw` | 6 | 3 x 3 = 9 | 6, 7 empty, **8 succeeds** (diagrams weigh 3 + 3 + 2 = 8) | `:2107` returns 8 from inside the loop, 1 below the cap. Lines 6, 7, 8 |
| `xw xw > xw xw` | 4 | 2 x 3 = 6 | **4 succeeds** (contact term and `yw` exchanges, weights 2 and 4, all at most 4) | `:2107` returns at the first trial. Line 4 |
| `z > u u~ d d~` | -- | -- | none | `:1978-1980` |

**Row 2: why the loop reaches a successful trial at 8 below the cap 9.** V1
predicted "stop at 7, generate at the cap". Both halves were wrong: the six
diagrams each weigh 8 (not 7), and a trial runs at every bound below the cap
(`:2013`), so the trial at 8 runs and succeeds; the loop never reaches `:2115`.
It also printed 8, because `:2014` precedes the trial.

**Row 3: why a first trial that succeeds prints its `Trying` line.** `:2014` is
before `:2023`; the line records an attempt, not a failure. V1's "none (the first
trial, 4, returns)" read the line as printed on failure. The row's one line, `<=4`,
is its first and only trial.

The error is V1 section 4's only. F1's own report (section on the M3 request,
"should read 4, 5 / 6, 7, 8 / 4 / none") had the right sequence, and the test
docstring of `a_two_order_vertex_is_found_by_the_search_below_the_cap` already
said "returns 8 from inside its loop, below the cap". V1's section 4 dropped the
failed-trial-only reading into the notes.

## 2. Does vibegraph accept the same WEIGHTED value?

Before: no test could tell. The diagram set at a bound is the set of diagrams
weighing at most the bound, so for row 2 a stop at 8 and a generation at the
cap 9 are the same 6 diagrams; the three count tests
(`a_two_order_vertex_is_found_*`, `the_search_starts_where_madgraph_starts`) are
blind to the accepted bound and to the trial sequence. The value was visible
only in two `debug!` lines.

How established:

1. **By reading** `generate_undecayed` (`vibegraph-lib/src/diagrams/mod.rs`,
   the `auto_weighted` branch): `w = start`, `while w < cap` (MadGraph `:2013`),
   generate at `w`, return on the first non-empty set (`:2107`), else `w += 1`
   (`:2111`), and after the loop generate at `w`, the cap (`:2115`). A top-level
   one-particle process has `auto_weighted == false` (`:1978-1980`). Start and cap
   come from `minimum_weighted` / `maximum_weighted`.
2. **By measurement**: new test `the_search_accepts_the_bound_madgraph_does` in
   `vibegraph-lib/tests/weighted_search.rs` reads the loop's own `debug!` record
   through a scoped `tracing` subscriber (on `EnumerationPool::Ambient`; the
   default `Serial` pool runs the loop on a rayon thread the scoped subscriber
   does not see, which is why a first draft recorded nothing). It asserts the full
   sequence per row, with the exact messages:

   | row | recorded | = MadGraph |
   |---|---|---|
   | `fw~ fw > xw xw` | no diagrams at 4; no diagrams at 5; no diagrams below the cap 6, generating at 6 | `Trying` 4, 5; generate at 6 |
   | `fw~ fw > xw xw vw` | no diagrams at 6; no diagrams at 7; lowest WEIGHTED order with diagrams: 8 | `Trying` 6, 7, 8; accepted 8 |
   | `xw xw > xw xw` | lowest WEIGHTED order with diagrams: 4 | `Trying` 4; accepted 4 |
   | SM `z > u u~ d d~` | (empty) | no `Trying` line |

   The test mapping is: each "no diagrams at w" or "lowest ... w" is a trial at
   w and so a `Trying` line; the "below the cap" message is `:2115`, no trial.
   The M3 measured MadGraph's lines (V2 brief); the vibegraph sequence is the
   same on all four rows.

Blind spot of the test: it pins the loop's decisions, not which diagrams are
found (the count tests and the M3 `NGRAPHS` comparison do that), and it reads
log text, so a reworded message fails it loudly rather than silently.

**Mutation evidence** (each applied, run, restored with `cp` from a saved copy;
`git diff --stat vibegraph-lib/src` empty after each). Test binary
`weighted_search`, 8 tests plus 1 ignored:

- `let cap = (search.maximum_weighted(n_ext) as usize).min(8);`: the row-2 cap
  becomes 8, so MadGraph's trial at 8 never runs and the loop generates at 8 with
  the same 6 diagrams. Every count test passes; the new test fails:
  `left: [..., "no diagrams below the WEIGHTED cap 8; generating at 8"]`
  `right: [..., "lowest WEIGHTED order with diagrams: 8"]`
  (`wcapE-V2-mut1.log`: 7 passed, 1 failed, the failure only in the new test).
- `w += 2` for `w += 1`: skips 7 and 5; count tests all pass (the diagrams
  weigh 8, 6, 4: all reached with step 2 from the even starts); the new test
  fails (`wcapE-V2-mut2.log`: 7 passed, 1 failed).
- `auto_weighted = process.orders.is_empty()` (drop `&& !top_level_decay`): the
  decay runs the search; the new test fails along with the two existing decay
  tests (`wcapE-V2-mut3.log`: 5 passed, 3 failed).

No test, comment or doc in the branch encodes V1's predicted sequence
(grep for `Trying`, `WEIGHTED<=` and the toy row names: only the briefs, F1's
report, V1's report and the test file); the only text carrying it is V1's
section 4, superseded below.

## 3. The banking text

Added as section 5 of `V1-report.md`; section 4 is not rewritten, and section
5 states which of its lines it supersedes. Contents: the corrected `Trying`
lines (4, 5 / 6, 7, 8 / 4 / none) with start, cap and outcome; the four
manifest `diagrams` notes in full, with the two missing toy rationale lines; the
`z_to_uuxddx_decay` script without `import model`; and the M3 instruction to
commit its tree with these notes after checking each `build.log` shows exactly
those `Trying` values.

The notes quote the `WEIGHTED<=N` values and not the constant `: WEIGTHED IS
<definition>` suffix MadGraph appends (`:2014`, with its typo). I did not have
the M3's logs, only the brief's summary of them, so the notes are written from the
source reading and the brief's measured lines, and the instruction tells the
M3 session to stop and report if a log differs.

## 4. Brief corrections

- The brief's worktree line says `50dddf0`; HEAD at start was `c6e900b`
  (the brief commit itself).
- The brief says V1 "expected `6, 7`" for row 2 and "none" for row 3. V1
  section 4 did say that; F1's report did not (it had 6, 7, 8 and 4).
- Nothing else.

## Gate

Logs under `.../scratchpad/wcapE-V2-*.log`; run as one script on the final
tree (all with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`). No banked layer: no amplitude, eval or coverage
code changed, and the reference bundle is absent here.

```
$ cargo fmt --all --check
fmt_exit=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 35.12s
clippy_exit=0

$ cargo test --workspace
41 binaries: 1551 passed, 0 failed, 19 ignored     (V1: 1548 / 0 / 19)
  lib                          test result: ok. 1014 passed; 0 failed; 12 ignored
  validate_madgraph_diagrams   test result: ok. 57 passed; 0 failed; 0 ignored
  weighted_search              test result: ok. 8 passed; 0 failed; 1 ignored
test_exit=0

$ python3 scripts/kb.py index; python3 scripts/kb.py lint
840 concepts, 294 backlog items; 0 errors, 4 warnings    (the four pre-existing `state: blocked` warnings)
kb_exit=0
```

`weighted_search` was 6 passed / 1 ignored at V1; the new test makes 7 here
plus HY1's earlier additions (8 total), the ignored test unchanged.

## Found

1. **F1's and V1's reports keep two forms of the trial sequence.** F1's report
   has the right one (4, 5 / 6, 7, 8 / 4 / none); V1 section 4 has the wrong
   one, now superseded by section 5. Nothing to close beyond the M3 commit; the
   risk is a reader copying section 4's table. Close: none needed once section 5
   is the one banked.
2. **The search's accepted bound is exposed only as a `debug!` message.** The
   new test reads message text. Close: if the bound is ever needed elsewhere
   (a refusal message quoting MadGraph's bound, as V1 Found 3 mentions), return
   it from the loop and test it directly instead of through the log.
3. **`EnumerationPool::default()` runs enumeration on a rayon worker thread**,
   so a scoped `tracing` subscriber (`with_default`) sees nothing from it. Not a
   bug; a trap for any test that captures logs (cost me one failed draft).
   Close: a one-line note on `EnumerationPool` if more such tests appear.
4. **The shared scratchpad holds unrelated `V2-*.py` files** (another sprint's
   V2). My files are prefixed `wcapE-V2-`; noted only so nobody sweeps the
   directory by pattern.
