---
type: Session Report
title: "F1 report: event_norm = sum and unity in the written events"
description: "generate honours event_norm: sum writes IDWTUP -3 and weights summing to sigma, unity writes IDWTUP -3 and weights of one, average is unchanged; pinned to MadGraph's source and to five banked MadEvent init blocks written under sum."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "sprintd-event-norm-sum, the F1 commit"}
---
## Implemented

- `RunCard::event_norm()` / `runcard::EventNorm {Average, Sum, Unity}`; the field is
  reclassified `Consumed`. A value outside MadGraph's `allowed` list is refused at
  the parse (`RunCardError::UnsupportedEventNorm`); matching is case-insensitive and
  the stored spelling is canonicalised, as MadGraph's `ConfigFile` does.
- `EmitPlan::event_norm`; `Buffer` writes: average `IDWTUP -4`, weights with mean σ
  (as before); sum `IDWTUP -3`, the same weights divided by the event count (sum σ);
  unity `IDWTUP -3`, every `XWGTUP` 1, `XMAXUP` 1. `XSECUP`/`XERRUP` are σ in all
  three; the integration and the events are untouched. `UnweightStrategy::weight_strategy`
  now takes the norm.
- `StochasticRounding` (IDWTUP +3) refuses sum/unity, in the CLI before the scan and
  in the emitter.
- Artifact agreement: the run card stays part of the artifact's exact fingerprint, so
  `generate` with a different `event_norm` than the grid was integrated with is
  refused like any other differing card parameter (test pinned). Not exempted, since
  the setting is cheap to re-integrate and the policy is "no case-by-case harmless
  differences".

## MadGraph citations (pinned b7687064)

- `madgraph/various/banner.py:4298` `event_norm` allowed `['sum','average','unity']`,
  `sys_default='sum'`; `banner.py:5619` (LO card default `average`); case-insensitive
  match `banner.py:1285-1298`.
- `madgraph/various/lhe_parser.py:1225-1239` (`MultiEventFile.unweight`): sum ->
  strategy 3, `new_wgt = sum(across)/event_target`; average -> strategy 4,
  `new_wgt = sum(across)`; `'unit'` -> strategy 3, `new_wgt = 1`.
- `lhe_parser.py:1009-1093` `define_init_banner`: `lha_stra = -1*abs(strategy)`, each
  process line's `XMAXUP` = `wgt`; `lhe_parser.py:519-527` keeps the sign of the
  current strategy, so sum and unit are written `-3`, average `-4`.
- `lhe_parser.py:466-469, 571-580`: every kept event gets the constant written weight
  `copysign(new_wgt)`, i.e. MadGraph flattens the overweight tail.
- Banked MadGraph evidence (committed, hermetic):
  `validation/madgraph/decay_chain_events_reference.json`, rows `pp_ttx_lep_dyn`,
  `ttx_nested`, `zz_ee`, `dy_two_procs`, `veto_chain`: run cards omit `event_norm`
  (so `sum`), all have `IDWTUP -3` and `XMAXUP = sum(XSECUP)/10000` (e.g. 5.892250e+00
  -> 5.892250e-04; `dy_two_procs` (17.215+627.37)/1e4 = 6.44585e-02 on both lines).

## Deviations / findings about MadGraph

- MadGraph writes one constant weight to every event; this writer keeps the
  overweight tail (weights proportional to the average-mode weights), so a single
  event's `XWGTUP` is not comparable, only the sum/mean (tested against the banked
  `XMAXUP`). Under unity the tail is flattened as in MadGraph (weight 1).
- `unity` is spelled `'unity'` in the allowed list but compared as `'unit'` in
  `lhe_parser.py:1232` (and `madevent_interface.py:4627`): reading the source,
  `MultiEventFile.unweight` with `event_norm = unity` leaves `strategy`/`new_wgt`
  unbound. So no MadGraph sample under `unity` is expected to exist; the unity
  behaviour is a code-read of the `'unit'` branch only.

## Tests and mutation evidence

New: `lhef::emit::tests::{event_norm_sets_the_idwtup_and_the_weights,
a_sum_file_carries_madevents_idwtup_and_event_weight,
stochastic_rounding_refuses_an_event_norm_it_cannot_write}`,
`runcard::tests::event_norm_is_parsed_against_madgraphs_allowed_list`,
`cli_generate::{event_norm_changes_the_written_weights_and_nothing_else,
an_event_norm_that_did_not_train_the_grid_is_refused,
stochastic_rounding_refuses_an_event_norm_off_average}`; the refused inventory test
is 21 -> 20.
The CLI test shows (three integrations of one card plus the event_norm line) equal
artifact sigma and bit-identical event momenta across the three settings, only the
weights differ.

Mutations (each reverted and touched; tail of the failing runs):

- M1 sum scaled like average: `event_norm_sets_the_idwtup_and_the_weights` FAILED,
  `a_sum_file_carries_madevents_idwtup_and_event_weight` FAILED.
- M2 sum/unity declare -4: same two FAILED.
- M3 unity keeps the weights: `event_norm_sets_the_idwtup_and_the_weights` FAILED.
- M4 emitter accepts rounding+sum: `stochastic_rounding_refuses_an_event_norm_it_cannot_write` FAILED.
- M5 CLI plan ignores the card: `event_norm_changes_the_written_weights_and_nothing_else` FAILED.
- M6 CLI pre-draw refusal removed: `stochastic_rounding_refuses_an_event_norm_off_average` FAILED
  (the first version of that test survived M6, as the emitter's own refusal also names
  event_norm; the test now pins the CLI message).
- M7 parse admits any string: `event_norm_is_parsed_against_madgraphs_allowed_list` FAILED.

Blind spots: files are read by this crate's parser, so a convention shared by writer
and reader is invisible; the banked MadEvent evidence is `<init>` lines only (no
per-event `XWGTUP`); multi-part (`@N` multiplicity) normalisation under sum follows
the same per-part scale but is not separately tested.

## Gate (run with the three CARGO_* disk flags set)

- `cargo fmt --all --check` -> rc=0
- `cargo clippy --workspace --all-targets -- -D warnings` -> `Finished ... in 1m 52s`, no warnings
- `cargo test --workspace` -> every suite `ok`, none failed: vibegraph-lib lib
  `1013 passed; 0 failed; 12 ignored`, cli_generate `13 passed`, cli main `106 passed`,
  the other suites ok (log: enormJ-F1-gate.log).
- `python3 scripts/kb.py lint` -> `830 concepts, 291 backlog items; 0 errors, 4 warnings` (the 4 are pre-existing).
- `validate-lhef` (banked-data feature gate) not run here (no refdata); the reader/parser
  is untouched by this change.

## Banking request (for the per-event half and for unity)

The item is not deleted: the rule for `sum` is pinned by five banked init blocks
(IDWTUP and the written weight), but no banked `<event>` `XWGTUP` exists, and `unity`
has no MadGraph sample. Request: one cheap row (e.g. `e+ e- > mu+ mu-`, `ee500` card,
`nevents = 1000`) run twice, once with `event_norm = sum` and once `event_norm = average`
in the run card; extract the `<init>` block and the first 5 `XWGTUP` values of each into
a JSON beside `decay_chain_events_reference.json`; the test extending
`a_sum_file_carries_madevents_idwtup_and_event_weight` would read it and assert the
per-event constant `XWGTUP == XMAXUP == sigma/N` (sum) and `XWGTUP == sigma` (average,
IDWTUP -4). A `unity` run is expected to fail in MadGraph (see above); banking that
failure would document it.

## Found

- MadGraph `lhe_parser.py:1232` compares `'unit'` while the card admits `'unity'`; an
  `event_norm = unity` run likely crashes in `MultiEventFile.unweight`. Evidence: source
  read only; close by running it once and recording the outcome.
- `mg_run_card()` writes `event_norm` into the `<MGRunCard>` header block, whereas
  MadGraph's banner marks it `include=False` (`banner.py:4298`): the header carries a
  line MadGraph's does not. Close by skipping non-included parameters there.
