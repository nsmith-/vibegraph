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
