---
type: Session Report
title: "L-Z8 report: hygiene pass on multicore generate's blocked accept/reject"
description: "Findings, mutation evidence and the -j 1 / -j 4 LHE identity oracle against main for the review of the blocked accept/reject pass (#180)."
status: draft
tags: [hygiene, unweighting, generate, parallel, review]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-generate` off main 8b3af5ab. The change under review is
#180 (`5b1f9f71`, [L-S1](L-S1.md), [report](L-S1-report.md)):
`vibegraph-lib/src/unweight.rs`, the new `unweight/blocks.rs`,
`generate/source.rs`, `generate.rs`, the CLI tests `cli_generate.rs` and
`cli_generate_proton.rs`, the harnesses `validate_samples.rs`,
`validate_unweighting.rs` and `validate_sigma.rs`, and `pixi.toml`.

## Findings

Written and committed before any code edit. Ranked; **fix** marks the ones
this session fixes.

| # | file:line | class | what is wrong | evidence | proposed fix | disposition |
|---|---|---|---|---|---|---|
| F1 | `generate/source.rs:99,119,597,616`; `hadronic.rs:3607`; `proton.rs:4229`; `validate_samples.rs:800,806`; `validate_sigma.rs:2821,2829` | hand-kept list that should be derived | The uniforms an accepted point's labels take are counted three times by hand: `FIXED_BEAM_EVENT_DRAWS = 4` / `PROTON_EVENT_DRAWS = 5` (and a bare `4` in two harnesses), the `select_event(.., u: [f64; 4])` / `[f64; 5]` signatures, and the literal-index rebuild `[u[0], u[1], u[2], u[3]]` in four `prepare` closures. A count one short panics on a worker (`u[3]` out of bounds); one too long burns a draw silently. | `grep -n 'EVENT_DRAWS\|\[u\[0\], u\[1\]' vibegraph-lib/src/generate/source.rs vibegraph-lib/tests/validate_s*.rs` shows the constants and four rebuilds; `select_event` signatures quoted above | Make the count a const parameter of `BlockedPass<T, const D: usize>` and hand `prepare` a `[f64; D]`, so `select_event`'s own array type fixes `D`; the two constants and the index rebuilds go. | **fixed** |
| F2 | `unweight/blocks.rs:141`; `unweight.rs:767`; `validate_samples.rs:842`; `validate_unweighting.rs:507` | dead or duplicated code | The sample cross section `W·Σw/T` is written four times: `Unweighter::sigma_from_events`, `BlockedPass::sigma_from_events` (re-implementing the first over other statistics, its own doc says so), and inline in two harnesses, whose copies divide by `used.trials` with no zero guard (`0/0 = NaN` on a seed that accepts nothing). | `grep -rn 'event_weight_sum /\|event_weight_sum, self' vibegraph-lib/src/unweight* vibegraph-lib/tests/validate_{samples,unweighting}.rs` lists the four: `blocks.rs:144`, `unweight.rs:768`, `validate_samples.rs:842`, `validate_unweighting.rs:507` | One `Unweighter` method over a given `UnweightStats`, with the shipped pass's association `(W·Σw)/T` so the generated σ line keeps its bits; the other three call it. | **fixed** |
| F3 | `validate_samples.rs:2798` (`every_demoted_row_is_a_gated_row`) | vacuous or tautological test | The guard says a demotion list entry that "silently demotes nothing" fails it, but it checks only that the key names a row. A `KS_INFO_ROWS` row whose `mode` is `"gate"` demotes nothing (its KS columns run at `row.mode`), and a `SCALE_INFO_ROWS` row whose `mode` is `"info"` demotes nothing either; both pass the guard. | Mutation M8 below: `ee_to_wpwm` back to `mode: "gate"` passes the guard | Assert each `KS_INFO_ROWS` row is `info` and each `SCALE_INFO_ROWS` row is `gate`. | **fixed** |
| F4 | `unweight/blocks.rs:542-543` (`a_pass_that_cannot_accept_gives_up`) | vacuous or tautological test | The second `next` after giving up is asserted `None`, which holds even with the `exhausted` latch deleted: the counter is already past the budget, so the next block also returns `None`, but it consumes a block and moves the statistics. The test cannot see that. | Mutation M7 below | Also assert the statistics are unchanged by the second call. | **fixed** |
| F5 | `validate_samples.rs:138-139` | comment that breaks AGENTS.md | "against `3.6e-6` and `0` for the two rows that used to disagree": history framing, naming neither row; the readings are an old bank's (`research/kb/history/notes/25-validation-layering-plan.md:667`). The change rewrote this paragraph and kept the clause. | quoted lines | Drop the clause. | **fixed** |
| F6 | `unweight.rs:775-779` | comment that breaks AGENTS.md (doc does not match body) | `Unweighter::trial`'s doc says "three uniforms' worth of the caller's stream are consumed per trial"; the body it now delegates to consumes `2 + channel_grid_ndim(j)`, as `trial_with`'s own doc says. | `sed -n 771,800p vibegraph-lib/src/unweight.rs` against `trial_with`'s doc | Say `trial` is `trial_with` on the pass's own scratch, statistics and per-channel trailing streams, and point at its draw count. | **fixed** |
| F7 | `unweight/blocks.rs:129` | dead or duplicated code (public item with no user) | `BlockedPass::unweighter()` has no caller anywhere. | `grep -rn '\.unweighter()' vibegraph-lib vibegraph-cli` returns nothing | Delete. | **fixed** |
| F8 | `unweight/blocks.rs:45` | public item that should be private | `pub const BLOCK_TRIALS` is read only inside `blocks.rs`. | `grep -rn BLOCK_TRIALS --include=*.rs . \| grep -v unweight/blocks.rs` returns nothing | Make it private. | **fixed** |
| F9 | `unweight/blocks.rs:183-186,199-203` | (behaviour/doc mismatch, not a listed class) | `stats()` is documented as "every trial up to the one that produced the last point handed out", but the two give-up branches leave it differently: a too-late acceptance leaves the last event's statistics, an exhausted block leaves every consumed block's, past the last point. Neither is the serial pass's count of the `max_trials` failing trials. | quoted lines | Give the give-up a single defined statistic; it moves what a short-sample report prints, so it is not a hygiene edit. | **Found** |
| F10 | `unweight.rs:780-877` | dead or duplicated code | Production generates through `BlockedPass` only; the serial `Unweighter::trial` / `next_event` and the `stats` / `scale_draw` state they keep now serve only tests (`unweight.rs` tests, `hadronic.rs:5933`, `validate_lhef.rs:531`, `validate_samples.rs:2712`, `decay_chain_ladder.rs:208`). Two accept/reject passes are kept in step. | `grep -rn '\.trial(\|\.next_event(integ' --include=*.rs vibegraph-lib vibegraph-cli` | Move the tests onto `BlockedPass` (or a one-thread wrapper of it) and delete the serial pass; crosses six files and the census probe that compares the two passes on purpose. | **Found** |
| F11 | `unweight/blocks.rs:349-359,378-406` | vacuous or tautological test | Found by mutation M1, after the table above was committed. `the_output_is_independent_of_the_thread_count` claims its 3000 events span "several batches" at four threads; they span about six blocks, and one batch is 8 blocks at two threads and 16 at four, so no pool size past one ever runs a second batch. `the_blocks_tile_one_serial_sequence` stops at 2500 events, inside the first batch of the default pool. A batch that starts at the wrong block passes every unit test. | M1 (`self.next_block += BLOCKS_PER_THREAD as u64`) survives all six unit tests | Take events until the trials span three batches: of the widest pool in the thread-count test, and of the running pool in the tiling test. | **fixed** |

Suspected in the brief and not reproduced:

- *`trial` and `trial_with` overlapping beyond the intended delegation*: `trial`
  is a twelve-line wrapper that moves the pass's scratch, statistics and
  trailing streams into `trial_with` and back; no logic is duplicated. Only its
  doc is stale (F6).
- *The block size 1024 as an unexplained literal*: `BLOCK_TRIALS` carries a
  seven-line doc giving the two bounds it sits between (`blocks.rs:37-45`). Its
  visibility is the finding (F8).

## Identity oracle

Chosen before the first edit. Main 8b3af5ab is built in a detached worktree
(`/home/user/wt/lz8-main`, own target) and this branch in its own, both
`cargo build --profile release-debug -p vibegraph --bin vibegraph`. Each card
is integrated once with main's binary; both binaries then generate from that
same grid at `-j 1` and `-j 4`, seed `20260728`:

- fixed beams: `e+ e- > mu+ mu-` at 45.6 GeV a beam, `--neval 20000 --niter 4`,
  1000 events;
- proton: `p p > l+ l- j QCD=2 QED=2` on the banked `pp_to_llj_fixed` run card
  and the NNPDF set, `--fixed-budget --neval 100000 --niter 6`, 3000 events.

Identity holds when all four LHE files of a card are `cmp`-equal to main's
`-j 1` file and the `sampling:`, `overweight:` and `from the sample` lines
`generate` prints are equal, timestamps stripped. The driver is
`oracle.sh integrate|compare` in the session scratchpad; its output is below.

## What changed

- **F1, draws derived.** `BlockedPass<T, const D: usize>`: `prepare` now takes
  `[f64; D]`, and `new` no longer takes a count. The count comes from
  `hadronic::SELECT_EVENT_DRAWS` (4) and `proton::SELECT_EVENT_DRAWS` (5),
  which are now the array lengths in the two `select_event` signatures, so the
  pass type, the signature and the draw cannot drift apart. The constants in
  `generate/source.rs` are gone, and so are the literal-index rebuilds in it and
  in two harnesses: their closures pass `u` straight through, and `D` is
  inferred.
- **F2, one sample-σ formula.** `Unweighter::sigma_of_events(&stats)` computes
  `W·Σw/T`, and `0` before the first trial. It uses the shipped pass's
  association, `(W·Σw)/T`, so the σ line `generate` prints keeps its bits.
  `Unweighter::sigma_from_events`, `BlockedPass::sigma_from_events` and the
  harness copies in `validate_samples` and `validate_unweighting` all call it.
  The serial `Unweighter::sigma_from_events` moves from `W·(Σw/T)` to
  `(W·Σw)/T`, about one ulp; only tests read it. The harness readings move by
  about one ulp too, because `GEV2_TO_PB` now multiplies last.
- **F3.** `every_demoted_row_is_a_gated_row` now asserts each `KS_INFO_ROWS` row
  runs at `info` and each `SCALE_INFO_ROWS` row at `gate`.
- **F4.** `a_pass_that_cannot_accept_gives_up` asserts that a second call after
  giving up runs no trial.
- **F5, F6.** The history clause is dropped from `P_FLOOR`'s doc.
  `Unweighter::trial`'s doc now names what the stream supplies and points at
  `trial_with` for the count.
- **F7, F8.** `BlockedPass::unweighter()` is deleted. `BLOCK_TRIALS` is now
  private.
- **F11.** `the_output_is_independent_of_the_thread_count` takes events until
  the trials span `SPAN_TRIALS`, three batches of a four-thread pool (49 152
  trials), not 3000 events. `the_blocks_tile_one_serial_sequence` replays
  until three batches of the running pool, not 2500 events.
  `a_restart_replays_the_pass` takes five blocks' worth of trials.

Tests added or deleted: none. The suite's count is unchanged.

## Oracles

**Identity, against main 8b3af5ab.** `oracle.sh integrate` integrates both
cards with main's binary. Its output:

```
ee integrate rc=0
pp integrate rc=0
2810bd31d9a16e1d0ace02c8abc9c6df34193b3939a392f9484dd1e75beae1c6  .../ee/out/grid.bin.zst
9802be3eec9295cfd2cf9f059dfa8eac6ad92450ea8a4d833cace51ec55292e3  .../pp/out/grid.bin.zst
```

`oracle.sh compare` was run on this branch's fixed tree, with both binaries
built `--profile release-debug`. Its output:

```
ee LHE main-j1 == main-j4        ee stats main-j1 == main-j4 (3 lines)
ee LHE main-j1 == branch-j1      ee stats main-j1 == branch-j1 (3 lines)
ee LHE main-j1 == branch-j4      ee stats main-j1 == branch-j4 (3 lines)
9141b933bb774f74adc0e69dd090236558eb3992145b7439e27844c8342ab511  (all four ee files)
 INFO sampling: 1000 events from 1000 accepted points in 2422 trials (efficiency 4.1288e-1)
 INFO overweight: rate 1.858e-2, cross-section share 5.339e-2, largest w/w_max 2.826
 INFO σ:        1966.949064 ± 62.355101 pb from the sample vs 2027.272305 ± 1.982966 pb from the integration (-2.976%, pull -0.97)
pp LHE main-j1 == main-j4        pp stats main-j1 == main-j4 (3 lines)
pp LHE main-j1 == branch-j1      pp stats main-j1 == branch-j1 (3 lines)
pp LHE main-j1 == branch-j4      pp stats main-j1 == branch-j4 (3 lines)
9feb0a4dd573f57884f4842850958e0da68f8b0bc94447a7abd24c9d1fb01a31  (all four pp files)
 INFO sampling: 3000 events from 3000 accepted points in 34138 trials (efficiency 8.7879e-2)
 INFO overweight: rate 1.611e-3, cross-section share 2.814e-2, largest w/w_max 4.931
 INFO σ:        436.488340 ± 8.025039 pb from the sample vs 423.169837 ± 0.905184 pb from the integration (+3.147%, pull +1.65)
```

The ee card's 2422 trials fit inside one batch at any `-j` (three blocks), so
that card checks identity on the first batch only. The pp card's 34 138 trials
are 34 blocks, nine batches at `-j 1` and three at `-j 4`, so it is the card
that crosses batch boundaries.

**Mutations.** `mutate.sh` (session scratchpad) applies each `perl`
substitution, runs the named tests under the lock, logs the result and restores
the file. Unit runs use `cargo test -p vibegraph-lib --lib unweight::blocks`;
the baseline was 6 passed. Logs are `/home/user/wt/logs/lz8-mut-*.log`.

| id | mutation | old tests | new tests |
|---|---|---|---|
| M1 | `run_batch`: `self.next_block += BLOCKS_PER_THREAD as u64` (the next batch overlaps the last) | **survived** all six unit tests; caught by `cli_generate::the_sample_does_not_depend_on_the_thread_count` | caught by `the_output_is_independent_of_the_thread_count` and `the_blocks_tile_one_serial_sequence` |
| M2 | `stride = 1 + widest_grid + D` (blocks overlap draws) | caught: tile, restart, merged statistics, gives-up, integral (the `debug_assert` stride bound fires) | unchanged |
| M3 | trailing offset `b * scale_ndim` instead of `first * scale_ndim` | caught: tile | unchanged |
| M4 | `absorb` drops `excess_sum` | caught: merged statistics | unchanged |
| M4b | `absorb` overwrites `ratio_max` | caught: merged statistics | unchanged |
| M5 | `restart` keeps `consumed` | caught: restart | unchanged |
| M6 | `trial_with` adds `1.0` instead of the event weight | caught: integral | unchanged |
| M7 | the `exhausted` latch deleted | **survived** | caught: gives-up |
| M7b | the block-end give-up disabled | hangs (600 s timeout): a failure, but one that shows as a hang | unchanged |
| M8 | `ee_to_wpwm` back to `mode: "gate"` (its `KS_INFO_ROWS` listing then demotes nothing) | **survived** `every_demoted_row_is_a_gated_row` | caught |
| M9 | fixed-beam `prepare` passes `[u[0], u[0], u[2], u[3]]` (helicity drawn off the subprocess uniform) | **survived** `cli_generate::the_sample_does_not_depend_on_the_thread_count` | not addressed: see below |

The M8 runs used
`cargo test -p vibegraph-lib --features extended-validation --test validate_samples every_demoted_row_is_a_gated_row`.
The old guard was restored into the file for its run.

## Gates

Each was run on the fixed tree under `flock /home/user/wt/heavy.lock`, with this worktree's `CARGO_TARGET_DIR`:

- `cargo fmt --all --check`: `FMT_OK`.
- `cargo clippy --workspace --all-targets -- -D warnings`: `CLIPPY_RC=0`.
- The same with `--features vibegraph/extended-validation,vibegraph-lib/extended-validation`:
  `CLIPPY_EV_RC=0`. This build compiles the changed `validate_sigma` probe.
- `verify-hidden.sh /home/user/wt/sprintl-hyg-generate lz8`, the hermetic
  `cargo test --workspace --no-fail-fast` with the three data paths moved
  aside: `TEST_RC=0`, `lines=79 passed=2352 failed=0`. The three paths were
  restored afterwards (`ls` confirms them).
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: rc 0. Its
  14 warnings are all in files this change does not touch.
- `pixi run --skip-deps kb-lint`: `1394 concepts, 480 backlog items; 0 errors,
  7 warnings`. None of the warnings is in this report.
- Extended validation, `--profile release-debug --features extended-validation`:
  - `validate_samples every_demoted_row_is_a_gated_row` (the tightened guard):
    1 passed.
  - `validate_unweighting`, whose pass and σ formula changed: 1 passed in
    15.5 s. The last row reads `vs VEGAS -> pull +0.91, rel +0.274%`.
  - `cli_generate_proton a_hadronic_sample_does_not_depend_on_the_thread_count`
    (the proton identity test): 1 passed.

  The full `samples` gate was not run here. Its `generate` changed only in
  the closure's signature and the σ call, and CI's banked job runs it.

## What each test cannot detect

- **The `-j` byte-identity tests** (`the_output_is_independent_of_the_thread_count`,
  `cli_generate::the_sample_does_not_depend_on_the_thread_count`,
  `cli_generate_proton::a_hadronic_sample_does_not_depend_on_the_thread_count`)
  see only differences between pool sizes. A pass that is deterministic but
  wrong is invisible to them. Examples: blocks overlapping their draws (M2),
  trailing streams reused across blocks (M3), a mislabelled draw (M9), a bias
  in `trial_with` (M6). At one thread and on a one-core host they compare
  nothing. The CLI ee test sees batch boundaries only because its 20 000
  events cross them. That count is not derived from the batch width, so a
  higher acceptance or a larger `BLOCKS_PER_THREAD` would quietly take it back
  inside one batch.
- **`the_blocks_tile_one_serial_sequence`** checks the stream layout against an
  independent replay. That replay restates the layout (stride `2 + 1 + D`,
  block offsets, a trailing stream read straight through) rather than deriving
  it, so a change made to both together passes. On a one-thread pool it sees
  one block per batch.
- **`the_merged_statistics_are_the_serial_ones`** cannot see a heaviest-trial
  merge that keeps the right ratios on the wrong trials: it compares ratios,
  not channels or points.
- **`the_sample_reproduces_the_integral`** cannot see an error with the same
  expectation, such as reading `ratio_sum` in place of `event_weight_sum`, or a
  skipped or duplicated block. Its channel-share bound adds `2e-3` (about two
  standard errors) to `5σ`.
- **`a_pass_that_cannot_accept_gives_up`** pins that the pass gives up and
  stays given up. It does not pin which statistic the give-up leaves (F9), and a
  disabled give-up shows as a hang, not as a failure (M7b).
- **`every_demoted_row_is_a_gated_row`** cannot see a row whose manifest mode
  disagrees with `ROWS`. That correspondence is hand-kept, and no test here
  reads the manifest's `samples` modes.
- **No hermetic test** sees a mislabelled draw in a `prepare` closure (M9): the
  labels it feeds are drawn consistently at every `-j`. Only the banked
  `samples` gates' helicity and flow χ² columns can, statistically.

## Long-run requests

None.

## Found

- **F9: the give-up statistic.** `BlockedPass::next`'s two give-up branches
  leave `stats()` differently (`blocks.rs:183-186` and `199-203`). A
  too-late acceptance leaves the last event's statistics; an exhausted block
  leaves every consumed block's, past the last point. Neither is the
  documented "up to the last point handed out", and neither is the serial
  pass's count of the failing trials. What a short-sample report prints
  therefore depends on which branch fired. To close: choose one definition
  (the trials consumed up to the give-up, at block granularity, is the
  cheapest), apply it to both branches, and pin it in
  `a_pass_that_cannot_accept_gives_up`.
- **F10: the serial accept/reject pass is test-only.** `Unweighter::trial`,
  `next_event` and the `stats` / `scale_draw` state are no longer reached from
  production. Their users are the `unweight.rs` tests, `hadronic.rs:5933`,
  `validate_lhef.rs:531`, `validate_samples.rs:2712` (the census probe, which
  compares the two passes on purpose) and `decay_chain_ladder.rs:208`. To
  close: move the non-probe users onto `BlockedPass` and delete the rest, or
  record that the serial pass is kept as the probe's reference.
- **The CLI identity test's batch coverage is incidental.**
  `cli_generate::the_sample_does_not_depend_on_the_thread_count` crosses batch
  boundaries only because 20 000 ee events take about 48 000 trials at
  efficiency 0.41: three batches at `-j 4` (16 384 trials a batch), but less
  than one at the `-j 16` it also runs (65 536). To close: derive the event
  count from the batch width, or assert from the `sampling:` line it already
  parses that the trials span at least two batches at every `-j` it compares.
- **An unused method on a release build.** `NodeAnalysis::resolve_mom`
  (`helas/eval/analysis.rs:208`) warns `never used` in
  `cargo build --profile release-debug -p vibegraph`, while clippy
  `--all-targets` is clean, so it has only test users. This is outside this
  change and belongs to the evaluator's owner.

## Method

- Reading the diff against the two-way questions (who calls this, what bounds
  this) gave F1, F2, F5 to F8 and F10. That was about 60% of the time, and the
  greps quoted in the findings settled each one.
- Mutation batches gave F4 and F11 and confirmed F3. F11, the strongest
  finding, came only from running M1: the test's own doc claimed the coverage
  it lacked.
- Leads that did not hold up: the brief's `trial` / `trial_with` overlap and
  the unexplained 1024 (see above). A suspected off-by-one between the two
  give-up comparisons (`>` against `>=`) is consistent once read: both
  refuse after `max_trials` trials without an acceptance.

## Brief corrections

- The `trial` / `trial_with` overlap and the unexplained 1024 were not
  reproduced; see Findings.
- `verify-hidden.sh` runs cargo itself without the heavy lock, so it was
  wrapped in `flock` from outside.
- A background command's default 30-minute limit killed the first mutation
  batch mid-mutation and left `unweight.rs` mutated. It was restored from
  `HEAD` before anything else ran. Drivers for this container need an explicit
  long `timeout`.
