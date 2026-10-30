---
type: Session Report
title: "S-HY1 report: hygiene review of #83"
description: "Hygiene review of the systematics change: findings written before any edit, then fixes with mutation evidence and byte-unchanged proofs."
status: draft
tags: [systematics, hygiene, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics` (PR #83), from
`70150daa`. Implements the [S-HY1 brief](S-HY1.md). Scope: `git diff
origin/main...HEAD` without `research/kb/sprints/` and the refdata pin. Logs
are in the session's scratchpad (`S-HY1-*`).

## Findings

Written before the first edit. Point: 1 maintainability, 2 non-vacuity,
3 visibility, 4 abstractions. Confidence: *checked* (the evidence settles
it) or *suspected*.

| id | pt | where | evidence | mutation the test misses | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 1, 4 | `systematics/format.rs:93` `wgt_text`; `lhef/write.rs:708` `rwgt_block`; `systematics/plan.rs:682` | `wgt_text` is `c_exponential(v, 7, true, 0)` line for line (`write.rs:38-55`: `{value:+.precision$e}`, `split_once('e')`, `{:02}` exponent). Production writes `<wgt>` with `c_exponential`; the gates (`validate_systematics::printed`, `cli_generate_proton`) spell with `wgt_text`, so they measure a copy, not the writer. `Writer::weights` spells `XWGTUP` a third way, `format!("{xwgtup:+.7e}")` | — | checked | fix: one spelling, `wgt_text`, used by `rwgt_block` and `Writer::weights`; identity probe |
| H2 | 1 | `tests/validate_systematics.rs:15` | module doc: "What this cannot see: … the MLM emission and chain terms (no banked MLM weights)", while lines 21-27 and `madgraphs_default_arguments_on_the_banked_mlm_runs` gate them on `run_01_syst_default` | — | checked | fix (doc) |
| H3 | 1 | `generate.rs:1611`, `systematics/header.rs:353` | `/// Refusal table (O3): …` (O3 is an open choice of the S-D1 report: design line 285, "open choice O3 in the S-D1 report"); `/// … so it is not the nominal (quirk J).` (a design-table label). AGENTS.md: comments never reference planning docs | — | checked | fix |
| H4 | 2 | `tests/validate_systematics.rs:615`, `:751` | the detection checks loop over the *measured* `r.controls`, not the requested list; a run that evaluated no control has an empty map and passes both | `measure_text`'s control loop skipped (`controls.iter().take(0)`) | checked (run below) | fix: each requested control looked up, absent is a failure |
| H5 | 2 | `tests/validate_systematics.rs:477-487`, `module_gate` `:728-741` | the `--alps` checks are `alps.iter().all(…)` over the weights classed `"alps"`; with none classed so, `all` on an empty list is `true` and every event counts as both moved and still | `class()` never returns `"alps"` | checked (run below) | fix: the number of `alps`-class weights must be the request's `--alps` factors off 1, counted from the request |
| H6 | 2 | `tests/validate_mlm_records.rs:850`, `:819`, `:955` | `head_census` prints "head census not run" and returns when no `leshouche.inc` is found, and `read_dir` errors become an empty list (`unwrap_or_default()`), so the pinned 0/0/219/31 and the mirror-order gate can be skipped green. refdata-12 carries `leshouche.inc` for every directory of all four rows (2, 2, 5, 7) | `idup_tables` reads a misspelt file name | checked (run below) | fix: `validation::require` |
| H7 | 1 | `tests/validate_mlm_records.rs:54` | "What this cannot see: The beam order: it is an input here, never a result." `head_census` gates the beam order against `leshouche.inc` (S-V2); the replay's chain heads and their `x` are also read off MadEvent's record, which the doc does not list | — | checked | fix (doc) |
| H8 | 1 | `tests/validate_systematics.rs:223`, `:987` | `our_block(writer)` is `block_of(writer.header_lines())` body for body | — | checked | fix: one function; byte-identical test output |
| H9 | 4 | `tests/validate_systematics.rs:114-135,186-190`; `tests/validate_mlm_records.rs:114-133,673-677` | `root`, `banked_text`, `banner_card` identical in the two targets | — | checked | fix: `tests/common`; byte-identical test output |
| H10 | 4 | `tests/validate_mlm_records.rs:138`, `tests/validate_mlm_dumps.rs:1730` | `within_one_unit` identical in both | — | checked | Found: the dump target cannot run here, so its output cannot be shown unchanged |
| H11 | 4 | `vibegraph-cli/tests/cli_generate_proton.rs:725-753`, `:1484-1504` | the `<wgt>` line parse and the one-unit comparison written twice in the file | — | checked | fix: one helper (both tests are banked; output shown unchanged) |
| H12 | 3 | `systematics/plan.rs:73` `MemberEval`, `:655` `Writer::raw_weights`; `systematics/request.rs:600` `split_card_list`; `systematics/weight.rs:83` `dynamical_scale` | `grep -rn` finds no user outside the defining file (census command below) | — | checked | fix: private |
| H13 | 3 | `systematics/header.rs` (`SlotId`, `SetHeader`, `Arg`, `expand`, `banner_lines`), `systematics/format.rs` (`py_float`, `fortran_e`, `fortran_i`), `request.rs` (`from_sys_fields`, `from_card_arguments`, `DynScale::code`, `DynScale::label`), `pdf/grid.rs` `MemberMeta` | every user is in the library crate (census below) | — | checked | fix: `pub(crate)` |
| H14 | 2 | `systematics/weight.rs:276`, `:281` | `a_scale_with_no_reading_is_an_error` asserts `is_err()`; with the `radicand < 0` guard removed the tachyon gives `Some(NaN)`, which the finite check also refuses, so the guard is invisible | delete `if radicand < 0.0 { return None; }` | checked (run below) | fix: assert each message |
| H15 | 1 | `tests/validate_mlm_dumps.rs:1178` `replay_row` (549 lines, was 345), `:1749` `compare_mgrwt` (161, new) | function census (scratch `S-HY1-fnlen.py`) | — | checked | Found: the dump target cannot run here (no dumps), so a split has no identity oracle |
| H16 | 1 | `tests/validate_mlm_records.rs:499` `compare_event` (172), `request.rs:366` `from_arguments` (132), `tests/validate_systematics.rs:443` `measure_text` (109), `:1146` (117), `cli generate::run` (144, was 110) | same census | — | checked | leave: `from_arguments` is one `match` arm per option; `compare_event` is the candidate loop of one event; splitting either moves state into a struct without making the rule clearer. Reported for the record |
| H17 | 4 | `systematics/plan.rs:386-395`, `pdf/select.rs` `member_index` | the same `u32::try_from(id − set_index).filter(< num)` | — | checked | leave: four lines, two different error types |
| H18 | 3 | `runcard/classes.rs:32` | `IgnoredBenign(#[cfg_attr(not(test), allow(dead_code))] &'static str)` added: removing the systematics warning removed the payload's only production reader | — | checked | leave: the same attribute already sits on `Consumed` (`:29`, on `main`); the payload is documentation the tests read. The pattern is pre-existing (Found) |
| H19 | 2 | `tests/validate_systematics.rs:852` | `written_head_shift` skips an unparsable record (`.ok()??`) | — | checked | leave: `module_gate` parses every event of the same file first with `expect`, so no parse failure reaches it |
| H20 | 2 | `runcard.rs` `a_refused_sys_pdf_fails_at_integrate` | matches `SystematicsRequest(_)`, any inner variant | — | checked | leave: each inner variant is pinned by its own `request.rs` test; this one pins that the card reaches it |
| H21 | 1 | `generate/source.rs:626`, `:441` | a record that cannot be filled (`LoRecord::fill(…).ok()`, `?`) ends the source, which `draw_all` reports as `Exhausted { wanted, drawn }`: the density error is lost and the message says the trial budget ran out | — | checked | Found: the source's `Option` contract is shared by every failure there (`record_scales(…).ok()?` is the same shape on `main`) |

### Census commands

- Visibility: every `pub` item the diff adds to `vibegraph-lib/src`, with the
  files outside its own that name it
  (`git diff -U0 origin/main...HEAD -- 'vibegraph-lib/src/**.rs' | … | grep -rlw <name>`),
  and every new `pub fn` method by call syntax (`grep -rnE "(\.|::)<name>\("`).
  `MemberEval`, `split_card_list`, `dynamical_scale`, `MemberMeta` returned no
  file outside their own; `raw_weights` is called only by `Writer::weights`;
  the H13 items only from `systematics/`, `runcard.rs` and `generate.rs`.
- Comments: `git diff -U0 origin/main...HEAD -- '*.rs' | grep '^+' | grep -E
  'S-[DFV][0-9]|sprint|quirk|\(O[0-9]\)|§|design'`: the two H3 sites, two
  test comments that say "the design" as a phrase (left), and the `H1`
  deviation name in `validate_mlm_dumps` and `validate_mlm_records`, which is
  the registered defect id (`madgraph-defects.md`), not a plan item (left).
- Backlog: every item naming a reviewed path tail (`grep -rlE
  'systematics/(plan|record|…)\.rs|/runcard\.rs|pdf/grid\.rs|lhef/(emit|write)\.rs|validate_systematics|…'`):
  20 items. One is closed by the code: `runcard-value-with-equals-dropped`
  (`split_line` uses `rsplit_once('=')`, pinned by
  `a_value_holding_an_equals_sign_reads_whole` on the banked line): Found.
  None of the others names a site this change gets wrong.

## Fixed

Commits `24d637c5` (library), `c2ad1f67` (the two banked lib targets),
`94746dd8` (the CLI round trips).

| finding | commit | what changed | evidence |
|---|---|---|---|
| H1 | `24d637c5` | `wgt_text` is `c_exponential(v, 7, true, 0)`, the event writer's call; `rwgt_block` and `Writer::weights` spell through it | identity probe below; the banked round trips print the same counts before and after |
| H3 | `24d637c5` | the two test comments say what they pin, without the design's labels | — |
| H12, H13 | `24d637c5` | 4 items private, 14 narrowed to `pub(crate)` (and `c_exponential` widened from private to `pub(crate)` for H1) | every target builds under both feature sets; `cargo doc` gate below |
| H14 | `24d637c5` | each refusal asserted by its message, `HalfHt` included | mutation M-H14 |
| H2, H4, H5, H8, H9 | `c2ad1f67` | `validate_systematics`: the doc's blind spots; each requested control looked up, a missing one a failure; the `alps`-class weight count checked against the request's factors off one (and `--alps` varied alone); one block writer; the event-file readers moved to `tests/common/banked_lhe.rs` | M-H4, M-H5; output byte-identical |
| H6, H7, H9 | `c2ad1f67` | `validate_mlm_records`: the head census required through `validation::require`, `read_dir` loud, one `process_directories`; the doc names the replay's inputs | M-H6; output byte-identical |
| H11 | `94746dd8` | `cli_generate_proton`: `written_weights` and `units_apart` | output identical |

### Byte-unchanged proofs

- **H1, the spelling.** A temporary test (removed before `24d637c5`)
  compared the new `wgt_text` with the old body, inlined, over 16 edge values
  (±0, the subnormal extremes, `f64::MAX`/`MIN`, ±∞, NaN, carries such as
  `9.99999995`) and 6 000 000 xorshift values (raw bit patterns and a dense
  linear range), and the parse `Writer::weights` takes of each against the
  old `format!("{v:+.7e}")`:
  `S-HY1 probe: 6000016 values identical, 5998541 parse to identical bits`
  (the 1 475 others are the non-finite bit patterns), then
  `test result: ok. 121 passed; 0 failed` over `systematics::`,
  `lhef::write` and `pdf::`. `rwgt_block` now calls `wgt_text`, which is the
  call it made before.
- **H8, H9 (the two lib targets).** Both run with `--nocapture
  --test-threads=1` under `release-debug` before any test edit
  (`S-HY1-base-banked.log`) and after the fixes (the gate's logs); diffed
  with the `finished in` lines removed: IDENTITY_LIB.
- **H11.** `cli_generate_proton`'s two round trips on the committed test
  file (restored afterwards, `S-HY1-cli-before.log`) and on the new one:
  `round trip: 400 events, 58000 weights, 57994 exact (99.9897%)` and
  `matched round trip: 400 events, 58800 weights, 58797 exact (99.9949%);
  --alps moved 696 weights on the 348 events …` before, IDENTITY_CLI after.

## Mutations

`S-HY1-mut/run.py` (scratchpad): one exact replacement, the target, the
restore (checked by content, then `git diff --stat`). "Old" is the test as
committed at `db54af68`, "new" the fixed test. The old runs used the
library of `24d637c5`'s working tree, which the probe shows identical in
behaviour.

| id | mutation | target | old test | new test |
|---|---|---|---|---|
| M-H4 | `measure_text` evaluates no control: `for &(control, name) in controls.iter().take(0)` | `validate_systematics` (`every_banked_weight…`, `madgraphs_default_arguments…`) | `ok. 2 passed` | `FAILED. 0 passed; 2 failed`: `pp_to_ll: control 'muR <-> muF' was not evaluated` (every run, every control), `pp_to_llj_mlm: control 'MLM terms dropped' was not evaluated` |
| M-H5 | `class()` never returns `"alps"`: `} else if v.alps != 1.0 && false {` | `validate_systematics::madgraphs_default_arguments…` | `ok. 1 passed` | `FAILED. 0 passed; 1 failed`: `pp_to_llj_mlm: 0 weights vary --alps alone, the request's 2 factors off one (varied alone: true)`, the same on the other three runs |
| M-H6 | `idup_tables` reads `leshouche.incX` | `validate_mlm_records` | `ok. 1 passed` (the census printed "not run") | `FAILED. 0 passed; 1 failed`: `` `the_mgrwt_record_is_madevents_on_every_banked_mlm_event` needs the process directories' leshouche.inc (validation/madgraph/output/pp_to_llj_mlm/SubProcesses) `` |
| M-H14 | `ht_scale` without `if radicand < 0.0 { return None; }` | `vibegraph-lib --lib systematics::weight` | `ok. 3 passed` | `FAILED. 2 passed; 1 failed`: `dynamical scale choice 2 (HT) is NaN on this event, not a finite positive scale` |

Sites no mutation reaches: H2, H3, H7 (docs and comments), H12/H13
(visibility, which the compiler checks), H8, H9, H11 (refactors, shown
identical instead). The pinned head counts were checked independently
instead of by mutation: a Python count over the banked files (both
`run_01` and `run_01_syst_default`) gives `pp_to_ll_0j2j_mlm` 19 events
with 31 heads (12 events with both heads stale) and `pp_to_ttx_0j1j_mlm`
219 events with 219 heads, which are the two targets' pins (19/219 events in
`validate_systematics`, 31/219 heads in `validate_mlm_records`). The two
pins count different things on the same records and agree.

## Routed

None. No finding moves a computed number, a gate's threshold or a
convention; none rests on speed.

## Gate

GATE_BLOCK

## Method

- **Read the brief's priority list in order, then the tests.** About 35% of
  the session: `systematics/` file by file, `runcard.rs`, `pdf/grid.rs`'s
  metadata section, the LHEF writer, and the four test files' helpers and
  gates.
- **Grep a new definition across the workspace** (point 1, 4): H1 (a
  formatter the writer already had under another name), H8, H9, H10, H11,
  H17. The most productive technique here, as in the hygiene sprint.
- **Ask what each gate's loop iterates over** (point 2): H4 and H5 came from
  asking, of each `for … in` over a result map and each `all()`, what it
  does on an empty input. H6 from the `exists()`/`read_dir`/`return` grep.
  All three were confirmed by running the mutation on the old test.
- **Visibility census** by a script over the diff's new `pub` items and
  their callers (point 3): 18 items narrowed. About 10%.
- **Comment grep** for plan labels: two sites (H3); the `H1` defect id was
  judged a registry name, not a plan label.
- **Function-length census** (`S-HY1-fnlen.py`): seven over 100 lines; none
  fixed (H15 Found, H16 left with reasons).
- **Independent recount of the pinned counts** (Python over the banked
  files): both targets' pins reproduced.
- **Leads that did not hold up.** The replay's "beam labels" field in
  `validate_mlm_records` looked tautological (the labels are read off the
  record), but it still pins that `LoRecord::fill` carries the labels
  through, so it was documented (H7) rather than removed. The record-fill
  `?` in `generate/source.rs` looked like a silently dropped event; it ends
  the run, so it is a misleading message (H21), not a biased sample. The
  `split_card_list` / `shlex` / `py_float` code was checked as a possible
  hand-written standard primitive: each is a port of a MadGraph or CPython
  rule with no crate equivalent.

## Found

- **FHY1. `within_one_unit` twice** (`tests/validate_mlm_records.rs:125`,
  `tests/validate_mlm_dumps.rs:1730`, identical). Not moved here: the dump
  target cannot run without the M3's dumps, so its output cannot be shown
  unchanged. Closes with the function in `tests/common` and the dump target
  re-run on the M3.
- **FHY2. `validate_mlm_dumps::replay_row` is 549 lines** (345 on `main`)
  and `compare_mgrwt` 161, both grown by this PR. A split needs the dump run
  as its identity oracle (the printed field tables byte-identical). Closes
  with the split and that run.
- **FHY3. A record that cannot be filled ends `generate` as "exhausted".**
  `generate/source.rs:441` discards `LoRecord::fill`'s error (`.ok()`), and
  `:626`'s `?` ends the source, which `lhef/emit.rs:517` reports as
  `EmitError::Exhausted { wanted, drawn }`: the density failure reads as a
  trial budget run out. The same shape already carries the scale
  prescription's refusals (`record_scales(…).ok()?`, on `main`). Closes with
  an `EventSource` that can return its refusal, and a test that a failing
  density names itself.
- **FHY4. `cfg_attr(not(test), allow(dead_code))` on `FieldClass` payloads**
  (`runcard/classes.rs:29`, `:32`). `Consumed`'s predates this PR;
  `IgnoredBenign`'s was added when the systematics warning, its payload's
  only production reader, was removed. The rationale strings are read only
  by tests. Closes by gating the payloads behind `cfg(any(test, doc))`
  accessors or by making the strings production-read (the
  `runcard-rationale-readers-unchecked` item touches the same strings).
- **FHY5. The backlog item `runcard-value-with-equals-dropped` is closed by
  this PR.** `split_line` uses `rsplit_once('=')` (`runcard.rs`), pinned by
  `runcard::tests::a_value_holding_an_equals_sign_reads_whole` on the banked
  line; the item's `closes_when` is met. Its file can be deleted with this
  PR's close-out.
- **FHY6. The S-V2 report's evidence "the replay of `xqcut_only` passes every
  field on 10000/10000, the labels included"** is not evidence for the
  mirror rule: the replay reads the labels off MadEvent's record (H7). The
  head census (`10000/10000` against `leshouche.inc`, and MR2) is the
  evidence, and it holds. A one-line correction for the sprint record.

## Brief corrections

- The brief lists `cli_systematics` among the banked targets to re-run; it
  is a hermetic target (no `required-features`), so `cargo test --workspace`
  runs it.
- The brief's clippy line names `--features vibegraph/extended-validation`;
  the gate here also ran `vibegraph-lib/extended-validation` (as S-F2, S-V1
  and S-V2 did) and `--all-features` (the agent definition's line).
