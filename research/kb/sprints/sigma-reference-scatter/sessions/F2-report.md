---
type: Session Report
title: "F2 report: hard_survey = 2 on every banked run, the recorded survey, the heavy-tail flag and the run checks"
description: "bank-seeded-sigma sets hard_survey = 2 on every installed card, keeps each run's run_config.inc and event count, and refuses short-event or non-numeric runs (retried once). Seed record schema 2 carries the survey and every channel; seed-record prints SURVEY (INERT under the mmjj condition) and SCATTER (worst channel seed chi2/dof, flagged above 3). Dry run on ee_to_mumu and pp_to_bb_fixed; the flag's power at five seeds measured on the earlier cross-check's seeds."
status: draft
tags: [madevent-reference, sigma, seeds, refdata, banking, hard-survey, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-16, no sibling session running"
  command: "see each section; logs under the scratchpad's refscf2/ (refscf2-*.log)"
sources:
  - {id: brief, resource: "F2.md", title: "F2 brief"}
  - {id: d1, resource: "../decisions/D1-reference-error-design.md", title: "D1 and both amendments"}
  - {id: f1, resource: "F1-report.md", title: "F1 report"}
  - {id: c-driver, resource: "https://github.com/nsmith-/vibegraph/commit/1c4583e", title: "1c4583e feat(validation): bank seeded sigma with hard_survey = 2 and record the survey"}
---
One code commit on `sigma/reference-scatter`, `1c4583e`; this report is the
second. Nothing was banked and no refdata was committed. `helas/` is untouched.

## What was built

**1. `hard_survey = 2` on every banked run.**
- `bank_seeded_sigma.sh` adds `hard_survey=$HARD_SURVEY` to each run's
  `mes_install_card` overrides, so it reaches the copy installed in the run
  directory. The committed `*_run_card.dat` files and the banked
  `output/<row>/Cards/` are not edited. `HARD_SURVEY` defaults to 2 and is
  documented as a diagnostic override, like `NEVENTS`.
- `gen_pta_windows.sh` takes `HARD_SURVEY` (replace or append, as
  `mes_install_card` does), and the driver passes it for `ee_to_mumua`. The pta
  seed stamp includes it.
- The card hash changes, so every seed kept under the old cards re-runs.
- The record states the setting twice: the row's `hard_survey`, and each seed's
  `hard_survey` read from its banner. `seed-record --hard-survey N` is
  required, and a seed whose banner says otherwise is refused. The generator
  text names it too.

**2. The effective survey.**
- `keep_run` keeps `Source/run_config.inc` as the run left it.
- Each seed records `min_events`/`max_events`. MadEvent's
  `pass_in_difficult_integration_mode` (`madevent_interface.py:3575-3594`)
  writes `2500·(2+r)` and `10000·(1+r)` for rate `r`. The template's own values
  (`madevent_run_config.inc`) are 1000/2000.
- `survey_rate` maps a pair back to `r`. The summary's `as_set` is true when
  every seed, at either budget, ran at the rate `hard_survey` asks.
- The screen prints `SURVEY <row>: ... -> as set`, or `INERT ... list the row
  back`.

**3. The heavy-tail flag.**
- Each seed records every channel (`P<dir>/G<tag>`, or `w<k>/P<dir>/G<tag>` in
  a window) with its `results.dat` value, error, equal-weight value and
  iteration count. The record therefore carries what the flag is computed
  from.
- `SeedRecord::heavy_tail`: for each channel present with a nonzero error in
  every seed at the budget, `combine` over the seeds gives its χ²/dof about the
  unweighted mean. The worst is recorded with its share of σ, along with every
  channel above `HEAVY_TAIL_CHI2_LIMIT = 3`. The row is flagged when the worst
  channel exceeds 3.
- It also records the number of channels expected above the limit by chance,
  `n_tested · P(χ²_dof > 3·dof)`, computed with `puruspe::gammq`. This is the
  same crate vibegraph-lib uses, and it is now a dependency of
  `validation-report`.
- The screen prints `SCATTER <row>: ...`. No gate reads the flag, and the
  cell's `info`/`gate` mode is untouched.

**4. Failing bad runs.** `mes_check_run PROCDIR TAG NEVENTS` (in
`madevent_seeds.sh`) refuses a run whose `results.dat` first line is not a
positive finite σ with a finite error, or whose `unweighted_events.lhe.gz`
holds fewer than `NEVENTS` `<event` blocks. It also refuses an unreadable or
truncated gzip. It writes the count to `Events/run_<tag>/vg_events_written.txt`.
- `run_fresh` calls it after `mes_run_seed` inside the existing two-attempt
  loop, so a refused run is retried once in a fresh copy.
- `run_pta` checks all five windows after each `gen_pta_windows.sh` attempt.
- The writer checks again: `events_written.txt` is required, and the run is
  refused when the count is below the banner's `nevents` or σ is not positive
  and finite. `SeedRecord::check` refuses the same in a parsed record, so a gate
  that reads such a record panics rather than falling back.
- `mes_run_seed` itself is unchanged. Its other callers (the decay, grammar,
  MLM and on-shell generators) do not get the check (see Found).

**5. Inert.** `seed_record::read` returns `None` at the same not-found branch
as before, ahead of any parsing. No gate reads the new fields.

The schema went from 1 to 2: the new per-seed fields are required, and a
schema-1 record would be refused. No schema-1 record was ever banked.
`assemble_bundle.sh` already selects `seeded_sigma/<row>/s<digits>/**`, so the
new kept files are carried without a change.

## Gate results

| suite | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | clean |
| clippy (writer) | `cargo clippy -j 2 -p vibegraph-validation-report --all-targets -- -D warnings` | `Finished`, no warnings |
| clippy (gate targets) | `cargo clippy -j 2 -p vibegraph-lib --features extended-validation --test validate_sigma --test validate_hadronic --test seed_combination -- -D warnings` | `rc=0` |
| writer tests | `cargo test -j 2 -p vibegraph-validation-report` | `seed-record`: 25 passed (18 before, 7 new); `validation-report`: 4 passed |
| hermetic (lib, includes `common::seed_record`) | `CARGO_PROFILE_DEV_DEBUG=0 cargo test -j 2 -p vibegraph-lib --test seed_combination` | 4 passed, `rc=0` |
| gates reading schema-2 records | records copied into `validation/madgraph/output/seeded_sigma/`, then `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --exact sigma_gate_matches_madgraph` and `--test validate_hadronic -- --exact sigma_bb_fixed_scale_vs_mg`; copies removed after | both `ok` (see the dry run) |

**`--skip-deps validate` was not run.** The reader's absent-record path did not
change: `read` returns `None` on `NotFound` before it parses anything, and every
gate's fallback is untouched. The changes are to the record's fields and to
`check`, which only a present record reaches. Both σ gates were run with
schema-2 records present, as above. `target/debug` was deleted after the tests.

The new writer tests:
- `run_config_gives_the_survey_mode_the_run_made`
- `a_survey_at_another_rate_than_the_card_asks_reads_inert`
- `a_short_event_file_or_a_non_numeric_sigma_is_refused` (NaN, zero, negative and missing `results.dat`)
- `a_record_refuses_a_short_seed_and_another_hard_survey`
- `a_kept_run_records_every_channel_and_its_survey`
- `the_heavy_tail_flag_names_the_channel_that_scatters_past_its_quote` (χ²/dof 32.5 pinned by hand, plus the chance count `2·7·e⁻⁶`)
- `a_partitioned_seed_names_each_windows_channels`

## Dry run

```
ROWS="ee_to_mumu pp_to_bb_fixed" NEVENTS=2000 SEEDED_OUT=<scratch>/refscf2/dry/refdata/seeded_sigma \
  SEEDED_WORK=<scratch>/refscf2/dry/work pixi run --frozen -e madgraph --skip-deps bank-seeded-sigma
```

Five seeds at 2000 events and two at 8000 per row, the defaults. `rc=0`. No
attempt failed: every log is `s<iseed>.1.log`. The kept files total 632 kB
(`ee_to_mumu`) and 772 kB (`pp_to_bb_fixed`).

**`hard_survey = 2` on the card, in the banner and in `run_config.inc`**
(`pp_to_bb_fixed` s20261101; `ee_to_mumu` reads the same):

```
card_s20261101.dat:158:  2 = hard_survey
banner.txt:304:  2	= hard_survey
Source/run_config.inc:18:      parameter (min_events = 10000 )         !to refine a channel
Source/run_config.inc:20:      parameter (max_events = 30000 )        !to refine a channel
card identical to output/pp_to_bb_fixed/Cards/run_card.dat modulo iseed/nevents/hard_survey
```

**The record's per-seed survey lines and the screen** (the full output is in
`work/<row>/record.log`):

```
[ee_to_mumu] s20261101 survey: hard_survey 2, run_config.inc min_events 10000 max_events 30000, 2000 events written
  ... (all 7 seeds identical; the 4x seeds: 8000 events written)
[ee_to_mumu] reported over 5 seeds at nevents 2000: 2.024640e3 +- 9.014e-1 (quoted 9.014e-1, spread 4.366e-1, chi2/dof 0.26)
[ee_to_mumu] equal-weight over 5 seeds at nevents 2000: 2.024721e3 +- 9.758e-1 (quoted 9.758e-1, spread 6.493e-1, chi2/dof 0.44)
[pp_to_bb_fixed] s20261101 survey: hard_survey 2, run_config.inc min_events 10000 max_events 30000, 2000 events written
  ... (all 7 seeds identical)
[pp_to_bb_fixed] reported over 5 seeds at nevents 2000: 2.148120e6 +- 4.753e3 (quoted 4.753e3, spread 3.210e3, chi2/dof 0.47)
[pp_to_bb_fixed] equal-weight over 5 seeds at nevents 2000: 2.150540e6 +- 7.134e3 (quoted 6.847e3, spread 7.134e3, chi2/dof 1.10)

=== screen (output/seeded_sigma/<row>/seed_record.json carries each) ===
SCREEN ee_to_mumu: 2 seed(s) at 4x against the reported mean of 5: pulls [-0.36, -0.47], their mean -0.46 -> within the screen
SURVEY ee_to_mumu: hard_survey 2 on every card; run_config.inc min/max_events 10000/30000 (rate 2) -> as set
SCATTER ee_to_mumu: worst channel P1_ll_ll/G2.1 at seed chi2/dof 0.26 (4 dof, 99.5% of sigma) of 2 tested; 0 above 3 [], 0.03 expected by chance -> within the limit
SCREEN pp_to_bb_fixed: 2 seed(s) at 4x against the reported mean of 5: pulls [-0.70, +0.42], their mean -0.14 -> within the screen
SURVEY pp_to_bb_fixed: hard_survey 2 on every card; run_config.inc min/max_events 10000/30000 (rate 2) -> as set
SCATTER pp_to_bb_fixed: worst channel P1_gg_bbx/G2 at seed chi2/dof 0.47 (4 dof, 97.2% of sigma) of 5 tested; 0 above 3 [], 0.09 expected by chance -> within the limit
```

`ee_to_mumu` tests 2 channels of 3 because `G2.2` is zero with a zero error.

**The gates read the schema-2 records:**

```
[ee_to_mumu] MadEvent reference from seeded_sigma/ee_to_mumu/seed_record.json: 5 reported seeds at nevents 2000, 2.024640e3 +- 9.014e-1 pb (quoted 9.014e-1, spread 4.366e-1, chi2/dof 0.26)
[ee_to_mumu] GATE vg = 2.025398e3 +- 1.690e0 pb | MG = 2.024640e3 +- 9.014e-1 pb | pull = +0.40 | rel = +3.74e-4 | chi2/dof = 0.50 (30000x6)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 178.35s
[pp_to_bb_fixed] MadEvent reference from seeded_sigma/pp_to_bb_fixed/seed_record.json: 5 reported seeds at nevents 2000, 2.148120e6 +- 4.753e3 pb (quoted 4.753e3, spread 3.210e3, chi2/dof 0.47)
[bb_fixed] GATE vibegraph σ = 2148063.5 ± 1262.2 pb (5 seeds, χ²/dof = 0.69) | MG σ = 2148120.0 ± 4752.8 pb | pull = -0.01 | rel = -0.0000
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 7.54s
```

These are 2000-event diagnostic seeds, not references.

**Cost.** Summed per-seed wall time:
- `ee_to_mumu`: 104 s with `hard_survey = 2`, against 70 s in F1's dry run.
- `pp_to_bb_fixed`: 347 s, against 285 s.

F1's host was shared and these runs are at 2000 events, where the survey's
fixed cost weighs most. D1's ~1.05–1.09× was measured at 10k events.

### The inert case on a real run

One `ee_to_mumu` run from the dry run's template (`refscf2-inert.sh`), card
overrides `mmjj=10;hard_survey=2;nevents=2000`, iseed 20261199. The beams are
45.6 + 45.6 GeV, so 0.01·91.2 = 0.91 < 10.

```
187:  10 = mmjj
188:  2 = hard_survey
2022.9 1.7374 11
--- run_config.inc after the run
18:      parameter (min_events = 7500 )         !to refine a channel
20:      parameter (max_events = 20000 )        !to refine a channel
```

That is rate 1, as `madevent_interface.py:3526` predicts: the `mmjj` branch
takes precedence over `hard_survey`. Feeding that `run_config.inc` into a copy
of the `ee_to_mumu` record (s20261105's replaced) gives:

```
SURVEY ee_to_mumu: hard_survey 2 on every card; run_config.inc min/max_events 7500/20000 (rate 1), 10000/30000 (rate 2) -> INERT: the survey ran at another rate than hard_survey asks (mmjj > 0.01 (ebeam1 + ebeam2) forces rate 1); list the row back
```

### Forced failures

The driver check `mes_check_run` was run on hand-made runs
(`refscf2-fake.log`) and on a short copy of the real run above
(`refscf2-inert.log`):

```
--- 3-event file, card asks 3                                  ACCEPTED (3 events)
--- fake short event file: 3 events, card asks 2000            ... holds 3 events, the run card asked for 2000      REFUSED rc=1
--- empty results.dat (the compile-race outcome)               no cross section on the first line of ...            REFUSED rc=1
--- NaN cross section                                          ... reports nan +- 0.1, not a positive finite ...    REFUSED rc=1
--- truncated event file                                       cannot read the event file ...: Compressed file ended before the end-of-stream marker was reached   REFUSED rc=1
--- no event file                                              cannot read the event file ...: [Errno 2] ...        REFUSED rc=1
--- real run, its own event file                               ACCEPTED: 2000 events
--- real run, first 1500 events only                           ... holds 1500 events, the run card asked for 2000   REFUSED rc=1
```

The writer, `seed-record`, was run on copies of the real `ee_to_mumu` kept
files (`refscf2-forced.log`):

```
B: s20261103/events_written.txt set to 1500  -> seed-record [ee_to_mumu]: .../s20261103: the event file holds 1500 events, the run card asked for 2000   exit=1
C: s20261102 results.dat emptied             -> seed-record [ee_to_mumu]: .../s20261102/SubProcesses/results.dat: empty results.dat              exit=1
D: --hard-survey 0 against seeds that ran 2  -> seed-record [ee_to_mumu]: [ee_to_mumu] seed 20261101 ran hard_survey = 2, the record's runs set 0   exit=1
```

The driver's retry was not exercised on a real failure. No attempt failed, and
I did not add a fault-injection hook.

### How much the heavy-tail flag can see at five seeds

The flag statistic was recomputed (`refscf2-flagpower.py`) on the
`hard_survey = 2` seeds that the second amendment's cross-check kept
(`hs2-keep.tgz`), over 2000 random five-seed subsets:

```
ee:  39 seeds, 6 channels;  all seeds: worst P1_ll_lla/G8   chi2/dof 2.78;  5-seed subsets flagging (>3): 49.5%; median worst 2.98
uux: 20 seeds, 10 channels; all seeds: worst P1_qq_wpbwmbx/G9 chi2/dof 12.08; 5-seed subsets flagging (>3): 98.0%; median worst 12.42
llj: 20 seeds, 8 channels;  all seeds: worst P1_gq_llq/G2   chi2/dof 1.05;  5-seed subsets flagging (>3): 1.3%;  median worst 1.70
```

`ee` is the 10k-event single-run `ee_to_mumua` card, `uux` the top-free
continuum, and `llj` `pp_to_llj_xqcut_only`. The flag separates `uux` (G9, as
D1 says) from `llj`. For `ee_to_mumua`, though, the channel χ²/dof over all 39
seeds is 2.78, below the limit, and five seeds flag it half the time. See the
brief errors below.

## Banking commands for the comment

These are unchanged from F1: `HARD_SURVEY` defaults to 2, and nothing else in
the interface moved. Every kept seed re-runs, because the installed card's hash
changed. Each row now prints three screen lines (`SCREEN`, `SURVEY`,
`SCATTER`), collected in `work/seeded_sigma/screen.txt`.

```
ROWS=@fixed22   JOBS=4 pixi run -e madgraph bank-seeded-sigma   # 30 rows
ROWS=@fixed23   JOBS=4 pixi run -e madgraph bank-seeded-sigma   # 5
ROWS=@fixed24   JOBS=2 pixi run -e madgraph bank-seeded-sigma   # 2
ROWS=@hadronic  JOBS=4 pixi run -e madgraph bank-seeded-sigma   # 10
ROWS=@llj-mlm   JOBS=3 pixi run -e madgraph bank-seeded-sigma
ROWS=@mlm-multi JOBS=2 pixi run -e madgraph bank-seeded-sigma
ROWS=@ee-mumua         pixi run -e madgraph bank-seeded-sigma   # 5 windows x (5 + 2) seeds at 100000 / 400000 events
ROWS=@long      JOBS=2 pixi run -e madgraph bank-seeded-sigma   # 2 -> 6, untimed
bash validation/madgraph/assemble_bundle.sh                     # then pin the new sha256 in validation/manifest.toml
```

The comment should ask for `work/seeded_sigma/screen.txt` back. Any `INERT`
`SURVEY` line and any `FLAGGED` `SCATTER` line is what the second amendment
says the screen lists back. The F1 limitation still holds: `@ee-mumua` needs
the bank host's full `output/ee_to_mumua/SubProcesses/*/configs.inc`.

## Brief and D1 errors

- **"`hard_survey = 2` in the banner and `run_config.inc`."**
  `run_config.inc` does not carry `hard_survey`; it carries its effect
  (`min_events`/`max_events` 10000/30000). That is what is recorded.
- **"Inert" under `mmjj` is partial.** On the `mmjj` branch the survey runs at
  rate 1 (survey points ×3, iterations +2, accuracy /3, run_config 7500/20000) rather
  than rate 2. Separately, `gen_ximprove.py:114` still sets
  `combining_job = 1` from `hard_survey > 1`. So `hard_survey` still changes
  the job layout there. The record reports the rate, which is the
  distinguishing quantity.
- **D1 lists `ee_to_mumua` as a row whose channels stay above χ²/dof 3 under
  `hard_survey = 2`.**
  - On the 10k single-run card's 39 seeds, the worst channel (G8, on its
    `results.dat` value and quote) is at 2.78.
  - The heavy tail D1 measured shows in the equal-weight value and the row's
    seed scatter, not in this per-channel statistic over many seeds.
  - At five seeds the flag fires on that card about half the time.
  - The banked row is the 100k five-window partition, which nobody has
    screened this way yet.
  - So the flag, as specified, is a coin flip for `ee_to_mumua`. The manifest
    decision for that row should not wait on it. D1 already keeps it `info`.
- **The flag has a look-elsewhere rate.**
  - With five seeds (4 dof), a channel whose quote is right exceeds
    χ²/dof 3 with probability 7·e⁻⁶ ≈ 1.7 %.
  - A row with 50 channels flags by chance 58 % of the time, and one with 100
    channels 83 %.
  - The record carries the expected count and each channel's share of σ, so a
    chance flag on a 0.1 % channel can be told apart from a real one, but the
    rule "worst > 3" does not do that by itself.

## Found

- **The run checks cover only `bank_seeded_sigma.sh`.**
  - `mes_run_seed`'s other callers (`gen_decay_widths.sh`,
    `gen_decay_chain_*.sh`, `gen_onshell_veto.sh`, `gen_grammar_sigma.sh`,
    `gen_mlm_references.sh`) still accept a run that exits 0 with an empty
    `results.dat`. `mes_run_seed` then records an empty value with the card
    hash, and the cache reads it back on every later call.
  - To close it, call `mes_check_run` from those generators, or have
    `mes_run_seed` refuse an empty or non-numeric first line. The event-count
    half does not apply to decays.
- **The heavy-tail rule needs a decision before banking.**
  - The evidence is the look-elsewhere rate and the `ee_to_mumua` coin flip
    above.
  - To close it, the user picks a rule: for example, flag only channels above
    some share of σ, or compare the count above the limit with
    `expected_above_by_chance`, or screen with 20 seeds where the five-seed
    flag fires.
  - The record already holds what any of these needs, so no re-bank is
    implied.
- **A reused process directory keeps a stale survey rate.**
  - `pass_in_difficult_integration_mode` substitutes
    `\(min_events = \d+\)`. Its own output, `(min_events = 10000 )`, has a
    space before `)`, so a directory that ran once in that mode never has its
    rate rewritten. A later run at another `hard_survey`, or none, silently
    keeps the first rate.
  - Fresh copies avoid this. The recorded `run_config.inc` would show it.
  - Any generator that reruns one directory across survey settings (the
    `run_pta` template is cloned, so it is safe) would mislabel its survey.
  - Worth a line in the `madevent-default-survey-bias` concept, and possibly
    an upstream note.
- **The cost of `hard_survey = 2` at small budgets is not D1's figure.**
  - At 2000 events the per-seed wall time rose 1.2–1.5× (above).
  - The M3 time estimate in D1's first amendment is at 10k events and is
    probably still right there. The 5 + 2 screens at 2000-event dry runs are
    not.
