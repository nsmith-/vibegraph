---
type: Session Report
title: "F3 report: the look-elsewhere scatter test in seed-record"
description: "seed-record's SCATTER line reads D1's look-elsewhere test: per channel the seed χ² p-value on n − 1 dof, the row flagged when N·p(worst) < 1 %, confirmed at 20 seeds, with the count above χ²/dof 3 kept for information. Schema stays 2; records the running batch writes parse and are rescreened by a new read-only --rescreen. On the kept hard_survey = 2 seeds the Rust test reproduces the manager's fire rates subset for subset (uux 91.9 %, ee_to_mumua 21.2 %, llj 0 %) and ee_to_mumua's all-seed p 1.641e-7 to 7e-12."
status: draft
tags: [madevent-reference, sigma, seeds, refdata, scatter, look-elsewhere, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-16"
  command: "see each section; scripts and logs under the scratchpad's refsf3/ (refsf3-*)"
sources:
  - {id: brief, resource: "F3.md", title: "F3 brief"}
  - {id: d1, resource: "../decisions/D1-reference-error-design.md", title: "D1, point 3 and the third amendment"}
  - {id: f2, resource: "F2-report.md", title: "F2 report"}
  - {id: hy1, resource: "HY1-report.md", title: "HY1 report"}
---
One code commit on `sigma/reference-scatter` (listed under Gate), plus this
report. Nothing was banked, no refdata was committed, and no shell script, card
or `gen_pta_windows.sh` was touched. `helas/` is untouched.

## What was built

**The test** (`vibegraph-lib/tests/common/seed_record.rs`, `SeedRecord::scatter`).
It replaces `heavy_tail`/`HeavyTail`/`HEAVY_TAIL_CHI2_LIMIT`.
- A channel is tested when every seed at the budget carries it with a nonzero
  error. This is unchanged from F2.
- Per tested channel, `combine` gives χ²/dof about the unweighted mean.
  χ² = χ²/dof · (n − 1), and `p_value = chi2_p_value(χ², n − 1)` =
  `puruspe::gammq((n − 1)/2, χ²/2)`.
- `worst` is the channel with the smallest p. `corrected_p` =
  `min(1, channels_tested · p_worst)`, and 1 when no channel is tested.
- `verdict` (`ScatterVerdict`):
  - `within_chance` when `corrected_p ≥ SCATTER_P_LIMIT = 0.01`;
  - otherwise `confirmed` when at least `SCATTER_CONFIRM_SEEDS = 20` seeds sit
    at the budget, and `flagged` below that.
- For information only: `above_info_limit`, the channels above χ²/dof
  `SCATTER_INFO_CHI2_PER_DOF = 3`, and `expected_above_by_chance` = N·Q(dof/2, 3·dof/2).
- Each `ChannelScatter` now carries `chi2`, `chi2_per_dof`, `p_value` and `share`.

**The screen line** (`seed-record.rs`, `scatter_line`). The driver collects it
by its `SCATTER ` prefix, which is unchanged:

```
SCATTER <row>: look-elsewhere p <N·p> over <N> channels tested (worst <ch> at seed chi2/dof <x> on <dof> dof, p <p>, <share>% of sigma) -> {within chance | FLAGGED: confirm with 20 seeds | CONFIRMED}; for information: <k> above chi2/dof 3 [...], <e> expected by chance
```

A p-value of exactly 0 prints `< 1e-16` (see "puruspe floors the tail" below).

**`seed-record --rescreen PATH/seed_record.json`** (new, read-only). It parses
an existing record, runs `check`, recomputes the summary from the seeds with
this code, and prints the same lines `seed-record` prints when it writes. It
writes nothing; the file's sha256 is unchanged after it runs (shown below).
This is how a record the running batch wrote gets the new verdict without a
re-run.

### Schema: stays 2

No stored field changed meaning:
- the per-seed fields, the `by_channel` values the test reads among them, are
  untouched;
- the summary loses `heavy_tail` and gains `scatter`.

`RecordSummary.scatter` is an `Option` with `#[serde(default)]`. A record that
02e2a28 writes has `summary.heavy_tail` and no `scatter`. It parses:
- serde ignores the unknown `heavy_tail`;
- the missing `scatter` reads `None`;
- `read`, `reference`, `part_reference` and the gates never read the summary.

The other direction does not hold. A record written by this code has no
`heavy_tail`, so 02e2a28's reader would refuse it (missing field). Nothing on
the batch host reads records with that code, and a gate reads records with the
code it is built with, so the asymmetry does not reach a gate. It does mean a
bundle assembled from records rewritten by this code needs this reader.

### The batch's records stay readable: shown on F2's dry run

1. **F2's dry-run records are what 02e2a28 writes, byte for byte.** I built
   `seed-record` at 02e2a28 in a throwaway worktree (since removed). I ran it
   over copies of F2's kept files, with the generator, tree, version, budget
   and `hard_survey` read back from F2's records:

   ```
   ee_to_mumu rc=0
   ee_to_mumu: 02e2a28 writes the F2 dry record byte-identically
   pp_to_bb_fixed rc=0
   pp_to_bb_fixed: 02e2a28 writes the F2 dry record byte-identically
   ```
2. **The new code rescreens them** (`refsf3-rescreen-dry.log`). The sha256 is
   taken before and after:

   ```
   f5ecb14a…385a  .../ee_to_mumu/seed_record.json
   SCREEN ee_to_mumu: 2 seed(s) at 4x against the reported mean of 5: pulls [-0.36, -0.47], their mean -0.46 -> within the screen
   SURVEY ee_to_mumu: hard_survey 2 on every card; run_config.inc min/max_events 10000/30000 (rate 2) -> as set
   SCATTER ee_to_mumu: look-elsewhere p 1.00e0 over 2 channels tested (worst P1_ll_ll/G2.1 at seed chi2/dof 0.26 on 4 dof, p 9.05e-1, 99.5% of sigma) -> within chance; for information: 0 above chi2/dof 3 [], 0.03 expected by chance
   rc=0
   f5ecb14a…385a  .../ee_to_mumu/seed_record.json
   5c0ad8a1…57f8  .../pp_to_bb_fixed/seed_record.json
   SCREEN pp_to_bb_fixed: 2 seed(s) at 4x against the reported mean of 5: pulls [-0.70, +0.42], their mean -0.14 -> within the screen
   SURVEY pp_to_bb_fixed: hard_survey 2 on every card; run_config.inc min/max_events 10000/30000 (rate 2) -> as set
   SCATTER pp_to_bb_fixed: look-elsewhere p 1.00e0 over 5 channels tested (worst P1_gg_bbx/G2 at seed chi2/dof 0.47 on 4 dof, p 7.56e-1, 97.2% of sigma) -> within chance; for information: 0 above chi2/dof 3 [], 0.09 expected by chance
   rc=0
   5c0ad8a1…57f8  .../pp_to_bb_fixed/seed_record.json
   ```
   The worst channels and χ²/dof match F2's old `SCATTER` lines (0.26 of 2
   tested, 0.47 of 5 tested).
3. **The new writer on the same kept files.** Everything outside `summary` is
   identical to 02e2a28's record. In the summary, `heavy_tail` is replaced by
   `scatter` and the shared keys are equal:

   ```
   everything outside summary identical: True
   summary keys old-only {'heavy_tail'} new-only {'scatter'} shared equal True
   ```
   (both rows)
4. **The gates read the 02e2a28-format records.** Under Gate, both σ gates run
   with F2's unmodified dry records copied in.

## Tests and mutation evidence

The new and rewritten tests are in `validation-report/src/bin/seed-record.rs`.
Every expected number is a closed form worked in the test's comment (Q(2, x) =
e⁻ˣ(1 + x) on 4 dof), not a value taken from a run.

| test | pins |
|---|---|
| `the_look_elsewhere_p_value_is_the_worst_tail_times_the_channels_tested` | Q(2, 6) = 7e⁻⁶ and Q(1, 1) = e⁻¹ from `chi2_p_value`. On a 6-channel row with one channel at χ² = 12 on 4 dof: worst = that channel, p = 7e⁻⁶ ≈ 0.0174, corrected = 6 · 7e⁻⁶ ≈ 0.104, `within_chance` |
| `many_channels_quoting_their_errors_do_not_flag` | 50 channels at χ² = 4 and one at χ² = 14, so p = 8e⁻⁷ ≈ 0.0073: below 1 % alone and past χ²/dof 3, but corrected 0.36, `within_chance`; the informational count still lists it |
| `one_channel_far_outside_its_quote_flags` | The same 50 with the odd one at χ² = 40: p = 21e⁻²⁰, corrected = 50× that, `flagged` |
| `twenty_seeds_confirm_a_flag` | 10 channels alternating ±1 over 20 seeds, one at ±3 (χ² = 180 on 19 dof): `confirmed` at 20 seeds, `flagged` at 19, `within_chance` with the calm channels alone at 20 |
| `the_flag_falls_below_one_percent_not_above` | One channel at χ² = 13 (p = 7.5e⁻⁶·⁵ ≈ 0.0113, `within_chance`) and at 13.5 (p = 7.75e⁻⁶·⁷⁵ ≈ 0.0091, `flagged`) |
| `the_scatter_line_states_the_corrected_p_the_channels_and_the_verdict` | The whole `SCATTER` line for the flagged 50-channel row; `within chance`, `CONFIRMED`, `< 1e-16` (a p of 0) and `no channel testable` |
| `a_record_written_with_the_heavy_tail_summary_reads_and_rescreens` | A record whose summary carries an 02e2a28-shaped `heavy_tail` block and no `scatter`. It reads through `seed_record::read` (the gates' path) and gives a reference; `--rescreen` gives the new verdict with the seeds unchanged; a record that fails `check` is refused |
| `the_informational_count_names_the_channel_past_chi2_per_dof_3` | F2's test kept for the informational fields: χ² = 130, χ²/dof 32.5, the count above 3, and the chance count 2 · 7e⁻⁶ |
| `the_arguments_refuse_what_they_cannot_read` (extended) | `--rescreen` takes exactly one path |

**Mutations** (`refsf3-mutate.py`, logs `refsf3-mutate.log` and
`refsf3-mutate-m13.log`). Each one replaces one exact string, runs
`cargo test -j 2 -q -p vibegraph-validation-report --bin seed-record` and
restores the file. `sha256sum -c` passed on both sources after each batch.
All 20 were killed:

| id | mutation | killed by |
|---|---|---|
| M1 | drop the ×N (corrected = p_worst) | 6 tests, incl. `many_channels_…`, `the_look_elsewhere_…` |
| M2 | p on χ²/dof instead of χ² | 7, incl. `the_flag_falls_below_…` |
| M3 | p on n dof instead of n − 1 | 6 |
| M4 | worst = largest p | 7 |
| M5 | limit 0.01 → 0.02 | `the_flag_falls_below_one_percent_not_above` |
| M6 | limit 0.01 → 0.005 | `many_channels_…`, `the_flag_falls_below_…` |
| M7 | confirm on > 20 seeds | `twenty_seeds_confirm_a_flag`, `the_scatter_line_…` |
| M8 | confirm at 19 seeds | same two |
| M9 | 20 seeds confirm whatever p reads | `twenty_seeds_confirm_a_flag` |
| M10 | the old rule (any channel above χ²/dof 3 flags) | `many_channels_…`, `the_flag_falls_below_…` |
| M11 | `deny_unknown_fields` on the summary (an old `heavy_tail` refused) | `a_record_written_with_the_heavy_tail_summary_…` |
| M12 | `scatter` required on read | same |
| M13 | `--rescreen` keeps the stored summary | same |
| M14 | `--rescreen` skips `check` | same |
| M15 | `CONFIRMED` printed as the flagged text | `the_scatter_line_…` |
| M16 | uncorrected p printed as the look-elsewhere p | same |
| M17 | p = 0 printed as `0.00e0` | same |
| M18 | chance count on χ² = 3 instead of χ²/dof = 3 | `the_informational_count_…`, `the_scatter_line_…` |
| M19 | a zero-error channel is tested | `a_record_carries_every_seed_…`, `the_informational_count_…` |
| M20 | `--rescreen` accepts extra arguments | `the_arguments_refuse_…` |

M13 and M14 were first logged as "SITE NOT FOUND": the string also matched
`build_record`. They were re-run on a unique site. A mutation that removes only
`#[serde(default)]` would survive, because serde's derive already reads a
missing `Option` field as `None`. M12 removes that behaviour and is killed.

## Measured check on the kept `hard_survey = 2` seeds

`refsf3-measure.py` builds one schema-2 record per row and per subset from the
tarballs in `refscf2/hs2x/keep`:
- `by_channel` from each `P*/G*/results.dat` value and error;
- `reported_pb` from `results.dat`;
- budget 10000.

It runs the Rust binary's `--rescreen` on each record, parses the `SCATTER`
line, and computes the manager's Python p (`confirm.py`'s `lee`,
`f3rule/sf.py`) on the same seeds. The subsets are drawn exactly as `rules.py`
and `confirm.py` draw them: `random.seed(1)`, 2000 × 5, and `random.seed(2)`,
500 × 20, over the same sorted seed list. So the comparison is subset for
subset, not within noise. The full-sample p is also read at full precision by
a scratch binary that includes `seed_record.rs` by `#[path]`, because the
screen line prints three digits. Log: `refsf3-measure.log`. The manager's
scripts were re-run unchanged (`refsf3-mgr-rules.log`, `refsf3-mgr-confirm.log`).

| row | seeds, channels | 5-seed fire rate, Rust / Python / brief | per-subset agreement | 20-seed fire rate, Rust / Python / brief | all-seed corrected p, Rust (full precision) / Python |
|---|---|---|---|---|---|
| `ee_to_mumua` (ee) | 39, 6 | 21.2 % / 21.2 % / 21 % | 2000/2000 | 87.0 % / 87.0 % / 87 % (500/500 agree) | 1.640999765694761e-7 / 1.6409997656837262e-07 (rel 6.7e-12), `CONFIRMED` |
| uux continuum | 20, 10 | 91.9 % / 91.9 % / 92 % | 2000/2000 | 100 % (the one 20-seed set) | **0 (prints `< 1e-16`)** / 4.207e-37, `CONFIRMED` |
| `pp_to_llj_xqcut_only` (llj) | 20, 8 | 0.0 % / 0.0 % / 0 % | 2000/2000 | 0 % | 1 / 1 (worst G2 p 0.402, ×8 capped) |

The all-seed lines as the screen prints them:

```
SCATTER ee: look-elsewhere p 1.64e-7 over 6 channels tested (worst P1_ll_lla/G8 at seed chi2/dof 2.78 on 38 dof, p 2.73e-8, 26.9% of sigma) -> CONFIRMED; for information: 0 above chi2/dof 3 [], 0.00 expected by chance
SCATTER uux: look-elsewhere p < 1e-16 over 10 channels tested (worst P1_qq_wpbwmbx/G9 at seed chi2/dof 12.08 on 19 dof, p < 1e-16, 25.7% of sigma) -> CONFIRMED; for information: 2 above chi2/dof 3 [P1_qq_wpbwmbx/G9 12.08 (25.7% of sigma), P1_qq_wpbwmbx/G8 6.18 (25.9% of sigma)], 0.00 expected by chance
SCATTER llj: look-elsewhere p 1.00e0 over 8 channels tested (worst P1_gq_llq/G2 at seed chi2/dof 1.05 on 19 dof, p 4.02e-1, 13.5% of sigma) -> within chance; for information: 0 above chi2/dof 3 [], 0.00 expected by chance
```

Over all subsets, the largest relative difference between the printed Rust p
and the Python p is 4.9e-3. That is the three-digit print, not the
computation: the full-precision ee value agrees to 7e-12. `ee_to_mumua`
confirms on all 39 seeds, as D1's third amendment states (p = 1.6·10⁻⁷).

### puruspe floors the tail at 0

The uux row is the one place where Rust and Python do not agree numerically.
`puruspe::gammq` 0.4.4 (`gamma.rs:260`) returns exactly 0 once
`x > a + 20(√a + 1)`. For uux's G9, χ² = 229.5 on 19 dof gives x = 114.8,
past the cutoff of 91.2, while the true p is 4.2e-37. I computed the true tail
at the cutoff for every dof from 1 to 199 with `sf.py`. Its maximum is
8.5e-17 at 1 dof, and it is 7.4e-21 at 4 dof. So a 0 means p < 1e-16. Such a
row flags at any channel count below 10¹⁴, and the verdict cannot differ from
an exact tail. The line prints `< 1e-16`, which is true. `chi2_p_value`'s doc
states the cutoff, and the test fixtures stay below it (χ² ≤ 40 on 4 dof)
where they assert a value. I did not add a dependency to print 4e-37 instead
of `< 1e-16` (see Found).

## Gate

Code commit: `8086c97` feat(validation): seed-record reads the look-elsewhere
scatter test. This report is the second commit. Logs: `refsf3-gates.log`,
`refsf3-clippy.log`, `refsf3-test-vr.log`.

| command | tail |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy -j 2 --workspace --all-targets -- -D warnings` | `Finished dev profile … in 3m 01s`, rc 0, 0 warnings |
| `cargo clippy -j 2 -p vibegraph-lib --features extended-validation --test validate_sigma --test validate_hadronic --test seed_combination -- -D warnings` | `Finished`, rc 0 |
| `cargo test -j 2 -p vibegraph-validation-report` | `seed-record`: `38 passed; 0 failed` (31 before: 1 rewritten, 7 new); `validation-report`: `4 passed`; rc 0 |
| `cargo test -j 2 -p vibegraph-lib --test seed_combination` | `4 passed; 0 failed`, rc 0 |
| F2's dry records (02e2a28 format, sha256 `f5ecb14a…`, `5c0ad8a1…`) copied into `validation/madgraph/output/seeded_sigma/`, then `cargo test -j 2 -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --exact sigma_gate_matches_madgraph` | `[ee_to_mumu] MadEvent reference from seeded_sigma/ee_to_mumu/seed_record.json: 5 reported seeds at nevents 2000, 2.024640e3 +- 9.014e-1 pb`; `GATE … pull = +0.40`; `test result: ok. 1 passed` (140.8 s) |
| the same, `--test validate_hadronic -- --exact sigma_bb_fixed_scale_vs_mg` | `[pp_to_bb_fixed] MadEvent reference from seeded_sigma/pp_to_bb_fixed/seed_record.json: 5 reported seeds at nevents 2000, 2.148120e6 +- 4.753e3 pb`; `GATE … pull = -0.01`; `test result: ok. 1 passed` |
| copies removed | `ls validation/madgraph/output/seeded_sigma`: No such file or directory |
| `python3 scripts/kb.py index --check`; `lint` | rc 0; `611 concepts, 206 backlog items; 0 errors, 2 warnings`, both on unrelated performance backlog items |

Both gate readings are identical to F2's and HY1's. `pixi run --skip-deps
validate` was not run: no gate's behaviour changed. The gates read records
through `read`/`reference`/`part_reference`, whose bodies did not change, and
none of them reads the summary. `target/debug` was deleted after the run.

**A stale-binary trap in the mutation driver.** The first
`cargo test -p vibegraph-validation-report` in the gate chain failed
`a_record_written_with_the_heavy_tail_summary_…` on a record with 7 events
written that `--rescreen` did not refuse. That is M14's behaviour, although
the sources were sha-identical to the unmutated files. The driver restores
each file by moving back a copy whose mtime is older than the mutated build's,
so cargo kept the last mutant's test binary. `touch` on both files and a
rerun gave 38 passed. The mutation verdicts are unaffected, because each
mutant write is newer than the previous build. The measurement and the
dry-run rescreen ran before the sweep. Clippy and the release-debug gates
compiled after it from the restored sources: clippy printed `Checking
vibegraph-validation-report`, and `validation-report` clippy was re-run after
the `touch`, rc 0. A mutation driver that restores by `mv` should `touch` the
restored file afterwards.

## Brief corrections

- **"χ² = 12 on 4 dof → 7·e⁻⁶; N = 6 scales it by 6."** This is right if
  "7·e⁻⁶" means 7 × e⁻⁶ ≈ 0.0174 (F2's 1.7 %), not 7 × 10⁻⁶. Then N = 6 gives
  0.104, which is **within chance**. The known-answer test pins it that way.
  I first wrote it expecting a flag, and the test caught the misreading.
- **"The Rust and the Python must agree on the full-sample p-values."** They
  agree for ee (7e-12) and llj (both 1). For uux they cannot: Rust's
  `puruspe` floors at 0, while Python's continued fraction reaches 4e-37. The
  verdicts agree (`CONFIRMED`), and so do all 6500 subset verdicts.
- **"keep `HEAVY_TAIL_CHI2_LIMIT`…" / naming.** The brief names the old
  identifiers as the starting point. They are renamed (`Scatter`, `scatter()`,
  `SCATTER_INFO_CHI2_PER_DOF`), and the record key is `scatter`, not
  `heavy_tail`. Keeping `heavy_tail` with a new `flagged` meaning would have
  been a stored field changing meaning, the case the brief says needs a schema
  bump.
- **"a schema-2 record from that driver must read with the new code and give
  the new verdict."** The gates read such a record unchanged. Nothing in the
  pre-existing code printed a verdict for a record already written: the screen
  came only from writing. `--rescreen` is the smallest addition that gives
  one, and it writes nothing, so a banked file keeps its hash.

## Found

1. **`bank_seeded_sigma.sh`'s header still describes the old rule** (lines
   50–52: "SCATTER the worst per-channel seed chi2/dof at the budget.
   FLAGGED: above 3"). I left it alone because the brief and the running
   batch put the script out of bounds. To close it, after the batch: describe
   the look-elsewhere p, the three verdicts, and `seed-record --rescreen` as
   the way to rescreen a banked row.
2. **Records the batch writes carry a stale `summary.heavy_tail`.** It is the
   old worst > 3 flag. No gate reads it, but `assemble_bundle.sh` ships it,
   and a reader of the JSON could take `"flagged": true` for D1's verdict.
   Two ways to close it:
   - re-run `seed-record` over the kept files with this code before
     assembling (it rewrites `seed_record.json` from the kept seeds);
   - or accept the stale block and use `--rescreen` output (the
     `screen.txt` equivalent) as the verdict of record.

   This is the user's call, and it changes the bundle hash either way.
3. **`puruspe::gammq` returns 0 below about 1e-16.** This affects
   `chi2_p_value` here and `vibegraph-lib/src/stats.rs:274`
   (`chi2_homogeneity`'s p). No decision at a percent-level threshold can
   change, but a printed or recorded p of 0 is a floor, not a value. To close
   it, if an exact deep tail is ever wanted: a crate whose upper incomplete
   gamma does not truncate, pinned by a known-answer test far in the tail
   (for example Q(2, 65) = 66e⁻⁶⁵).
4. **The confirmation step has no driver path.** D1 says a 5-seed flag "is
   confirmed by the same test on 20 seeds". The driver banks 5 + 2, and
   `SEEDS` can be set to 20, but nothing runs the 20 automatically for a
   flagged row. To close it: a documented `SEEDS=<20 iseeds> ROWS=<row>`
   recipe in the driver's usage (a shell change), or a group in the driver
   that re-runs flagged rows.
