---
type: Session Report
title: "HY1 report: hygiene review of the event_norm weights"
description: "Review of F1's event_norm change: one weight rule and one refusal predicate, stale IDWTUP docs, a multi-part sum test, shared CLI integrations; F1's include=False finding does not reproduce."
status: draft
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
---
## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 4 | `lhef/emit.rs` `Buffer::weight_strategy`, `written_weight`, `tally_parts` | three separate `match` on `EventNorm` encode one rule (IDWTUP, per-part scale, flatten) | checked | fix |
| H2 | 4 | `generate.rs:447` and `emit.rs:834` | the refusal predicate is written twice (`norm == Average` / `!= Average`) | checked | fix |
| H3 | 1 | `emit.rs:9,467,480`, `generate.rs:98`, `docs/src/cli/reference.md:281`, `docs/src/guide/11-unweighting-events.md:92,137` | "Buffer ... IDWTUP = -4" stated unconditionally; now only under `average` | checked | fix |
| H4 | 2 | `emit.rs` tests | no multi-part test under `sum`; mutation `Sum` scale uses the part's own event count / n for several parts survives all F1 tests (single part: part events == n) | suspected -> settle by mutation | fix |
| H5 | 2 | `cli_generate.rs` | `event_norm_changes...` integrates three times, the refusal test a fourth (same `sum` card); average equals the default-card `run()` | checked | fix (shared cached runs) |
| H6 | 3 | `runcard.rs` `EventNorm::parse`, `derive(Default)` | `grep`: `parse` only used in `runcard.rs`; `EventNorm::default()` never called | checked | fix (narrow parse, drop Default) |
| H7 | - | F1 Found 2 / brief: `mg_run_card` writes `event_norm` against `include=False` | `banner.py:3583-3585` (b7687064): `include` selects the Fortran `run_card.inc`: "The entry False will not be written anywhere" refers to that include file, not `run_card.dat` / banner | checked | not reproduced |

## Dispositions

| id | disposition | commit / evidence |
|---|---|---|
| H1 | fixed | `norm_rule(EventNorm) -> NormRule {idwtup, sums_to_sigma, flatten}` in `lhef/emit.rs`, with the MadGraph citations once; `Buffer::weight_strategy`, `written_weight` and `tally_parts` read it. Identity oracle: the emitted text plus `EmitSummary` `Debug` of three norms x (two-part and one-part plans) is byte-identical before and after (`cmp` rc=0, sha256 `cbcaa699...` both, 1 288 451 bytes). |
| H2 | fixed | `StochasticRounding::honours(norm)` is the one predicate; the emitter and `refuse_rounding_off_average` both call it. |
| H3 | fixed | emit.rs module/`Buffer` docs, `Strategy::Buffer` doc, `docs/src/cli/reference.md` (kept in step with the clap help; `cli_reference_docs` passes), guide chapter 11 now say `-4` is the `average` case and describe `sum`/`unity`. |
| H4 | fixed | new `each_part_is_normalised_to_its_integration_under_every_event_norm` (lib test count 1013 -> 1014): unequal part shares (2/3, 1/3), per-process written weights sum to `N sigma_k` (average) / `sigma_k` (sum). |
| H5 | fixed | `run_with_event_norm(norm)`: `average` is `run()`'s own, `sum` and `unity` one cached integration each; the CLI suite integrates twice beyond the default run where it was four times. |
| H6 | fixed | `EventNorm::parse` is `pub(crate)`; `Default` derive dropped (no `EventNorm::default()` call exists). `EventNorm`, `as_str`, `RunCard::event_norm`, `EmitPlan::event_norm` stay `pub`: each has a user in the CLI crate (`grep` in generate.rs). |
| H7 | not reproduced | see below |

### H7: `include=False` is the Fortran include file, not the written card

`banner.py:2866` stores `self.includepath[include].append(name)`; `banner.py:3583-3585`
(`write_include_file`): "The entry True of self.includepath will be written in
run_card.inc. The entry False will not be written anywhere", that is the include
file. MadGraph's own committed cards write the `include=False` parameters:
`validation/madgraph/onshell_ee_run_card.dat:21` `tag_1 = run_tag`, `:74`
`time_of_flight`, `:75` `average = event_norm`. So `mg_run_card()` writing
`event_norm` is not a divergence on that account; nothing changed. (What does differ
is the hidden-parameter policy, which the extended `mg_run_card_matches_madevents_banner`
already reports as "only in ours"; not touched.) F1's Found 2 is wrong.

## Mutations (each applied temporarily, reverted, file touched)

Driver: `scratchpad/enormJ-HY1-mut.py`; logs `enormJ-HY1-mut.log`, `enormJ-HY1-mut2.log`.

| mutation | old tests | new tests |
|---|---|---|
| A multi-part `sum` scale uses `n` (`sums_to_sigma && parts.len()==1`) | `lhef::emit` 16 passed, 0 failed (survives) | `..._under_every_event_norm` FAILED |
| B `sum` declares `IDWTUP -4` | `event_norm_sets_the_idwtup_and_the_weights`, `a_sum_file_carries_madevents_idwtup_and_event_weight` FAILED | same |
| C `unity` not flattened | `event_norm_sets_the_idwtup_and_the_weights` FAILED | same |
| D `honours` always true | `stochastic_rounding_refuses_an_event_norm_it_cannot_write` FAILED | same |
| E CLI predicate off | `stochastic_rounding_refuses_an_event_norm_off_average` FAILED | same |

Only A changes the verdict of an existing test set (survived before, caught now). The
refactor (H1, H2) keeps B-E caught. The `@N`-multiplicity case is now pinned by
`PartedSource` plans, hermetic and cheap; the full CLI path with `@N` processes is not.

## Gate (disk flags set; log `scratchpad/enormJ-HY1-gate.log`)

- `cargo fmt --all --check` -> rc=0
- `cargo clippy --workspace --all-targets -- -D warnings` -> `Finished dev profile ... in 43.76s`, no warnings
- same with `--all-features` -> `Finished ... in 56.40s`, no warnings
- `cargo test --workspace` -> every suite ok, 0 FAILED; lib `1014 passed; 0 failed; 12 ignored` (+1 = the new test), `cli_generate 13 passed`, cli main `106 passed`, `cli_reference_docs 1 passed`
- `cargo doc -p vibegraph-lib` / `-p vibegraph --no-deps --document-private-items` -> finished; 8 + 1 warnings, none naming emit/runcard/generate items touched here
- `python3 scripts/kb.py lint` -> after `kb.py index` regenerated the sessions index: see the commit; 0 errors
- Banked `validate-lhef` target not run (no reference data in this tree); the parser is untouched and the change here is byte-identical on the oracle above.

## Method

Reading the diff against the brief's four asks (60%); grep of users for visibility
(10%); a byte-level before/after dump as the refactor oracle (10%); mutation batch
(10%); reading `banner.py` and the committed MadGraph cards to settle H7 (10%). The
H7 lead (F1's Found) did not hold up: the cheapest check was a committed MadGraph card
that already writes `include=False` parameters. A first plan to key the "one rule"
on a `WeightStrategy` plus separate scale function was rejected: the three decisions
(IDWTUP, scale denominator, flatten) are independent facets of one rule, so a small struct.

## Found

- F1's own report "Found" 2 should be struck: not reproduced (H7).
- `unity` against MadGraph's `'unit'` compare and the per-event banking request stay as F1 filed them.

## Brief corrections

- "`<MGRunCard>` and `include=False`": the premise is false (H7).
- The CLI never held a repeated `EventNorm` match; the repetition was inside `emit.rs` (three matches) plus the refusal predicate.
