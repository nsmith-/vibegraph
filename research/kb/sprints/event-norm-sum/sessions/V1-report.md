---
type: Session Report
title: "V1 report: event_norm weights pinned per event against MadEvent"
description: "A hermetic test reads the banked MadEvent sum and average samples event by event against Buffer's weight rule; MadGraph's crash under event_norm = unity is recorded as an observed result."
status: draft
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-21}
---
## Done

- New test `lhef::emit::tests::event_norm_weights_match_madevents_per_event`
  reads `validation/madgraph/event_norm_reference.json`. For `sum` and `average` it
  asserts the banked `<init>` IDWTUP (-3, -4), the banked XMAXUP relation
  (σ/N, σ), that all 1000 banked XWGTUP are constant, and then feeds the banked σ
  and N through `Buffer`:
  - no tail (equal generator weights): our IDWTUP, XSECUP, XMAXUP and all 1000
    XWGTUP equal MadEvent's;
  - tail (one event in ten at four times the generator weight): MadGraph flattens
    the tail, this writer keeps it, so only the rule's consequences are asserted:
    the weights' sum (`sum`) or mean (`average`) is MadEvent's σ, the bulk is
    constant and below MadEvent's constant, the tail is constant above it, and the
    tail/bulk ratio is the generator ratio 4.
  The blind spot is in the test's doc comment: one process, one part, one process
  entry, so a multi-part or multi-process error is not seen (that is
  `each_part_is_normalised_to_its_integration_under_every_event_norm`), and a
  rescaling that keeps the sum or mean but moves weight between events is seen only
  through the tail's ordering and ratio.
- `EventNorm::Unity` doc (`vibegraph-lib/src/runcard.rs`) and a row in
  `research/kb/validation/madgraph-defects.md` record the M3 observed result:
  `UnboundLocalError` on `new_wgt` at `lhe_parser.py:1238` in
  `MultiEventFile.unweight`, no event file written. Not posted upstream.

## Gates

- `cargo fmt --all --check` -> `fmt rc=0`
- `cargo clippy --workspace --all-targets -- -D warnings` -> `Finished dev profile ... clippy rc=0`
- `cargo test --workspace` -> 40 `test result` lines, sum 1547 passed, 0 failed;
  `test lhef::emit::tests::event_norm_weights_match_madevents_per_event ... ok`
- `python3 scripts/kb.py lint` -> `834 concepts, 291 backlog items; 0 errors, 4 warnings`
  (the four warnings are existing `state: blocked` backlog items without `blocked_by`)

## Mutations (each reverted)

Run as `cargo test -p vibegraph-lib --lib event_norm_weights_match`.

- Sum scaled like average (`events_per_weight` = `n` under `sums_to_sigma` in
  `tally_parts`): `event_norm_weights_match_madevents_per_event` FAILED,
  `assertion failed: (file.init.processes[0].xmax / banked_xmax - 1.0).abs() < 1e-5`.
- Average scaled like sum (`events_per_weight` = 1.0 otherwise): same test FAILED,
  same assertion.

## Brief corrections

- The brief says the branch is at `9743133`; HEAD was `efe2ce1` (the assignment's
  expected value), which has `9743133` as an ancestor. No effect.

## Found

- The `.agents`/CI path cannot tell which of the per-event or XMAXUP assertions
  fires first in a mutation; both mutations stop at the XMAXUP assertion, so the
  per-event loop itself has no recorded failing mutation. A mutation that keeps
  XMAXUP but moves weight between events would be needed to exercise it
  (`Buffer` has none today to mutate to).
- `cargo test --workspace` on this host takes about 20 minutes with sibling builds
  running; no defect, only a planning note.
