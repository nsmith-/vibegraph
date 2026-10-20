---
type: Session Report
title: "F1 report: unimplemented cuts refused before enumeration, no-op maxima let through"
description: "detect_unimplemented is public and runs in load_run_card ahead of enumeration for integrate and generate; single-object pT maxima at or above half the collider energy read as the default."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "0576ca2", title: "F1 commit on sprintd-cut-refusal"}
---
## Implemented

- `vibegraph-lib/src/cuts.rs`: `detect_unimplemented` is `pub` and is the single
  implementation; `Cuts::compile_with` still calls it, so library callers are
  covered. New `SINGLE_PT_MAXIMA` (`missetmax`, `ptj1max`..`ptj4max`,
  `ptl1max`..`ptl4max`): a value `>= (ebeam1 + ebeam2) / 2` reads as the default.
- `vibegraph-cli/src/integrate.rs`: `load_run_card` calls `detect_unimplemented`
  and returns its message unprefixed. `load_run_card` is the only run-card entry
  point of the CLI: `integrate` and `generate` (decay and fixed-energy paths
  included) call it, and `check` takes no run card. It runs ahead of
  enumeration, flavour-group decomposition and cut compilation.
- Threshold justification: in the partonic rest frame a single object's momentum
  balances the rest of the final state, so `|p| <= E_obj` and `|p| <= E_rest`
  give `|p| <= sqrt(s_hat)/2`; pT is invariant under the longitudinal boost and
  `s_hat <= s`. The same argument covers the missing-ET of any subset of the
  final state. Applies to the nine names above only. The HT-sum maxima
  (`htjmax`, `ihtmax`, `ht2max`..`ht4max`) are bounded by `sqrt(s)` in the same
  way, but MadGraph's definition of which objects they sum is not pinned in this
  tree, so they stay refused. Minima, `xpt*` and the rest stay refused.
- Edge: `value >= ceiling` passes (a bound exactly at the ceiling excludes a
  measure-zero set); `ceiling - 0.1` is refused.
- Deleted both backlog items; removed the dangling links in
  `research/kb/backlog/feature/ht-cuts-unimplemented.md` (paragraph removed) and
  `research/kb/validation/cms-gridpack-survey-2026-10.md` (link text kept as
  plain words).

## Tests

- `vibegraph-lib` `cuts::tests`: `detection_needs_no_legs`,
  `unbindable_pt_maxima_read_as_default` (1e5 at 13 TeV, all nine names, through
  `detect_unimplemented` and `Cuts::compile`), `bindable_pt_maxima_are_refused`
  (500), `pt_maximum_threshold_is_half_the_collider_energy` (edge, and
  asymmetric beams 100+50), `unbindable_value_does_not_excuse_other_cuts`.
- `vibegraph-cli/tests/cli_unimplemented_cuts.rs` drives the real binary:
  integrate names cut and value; integrate on `e+ e- > mu+ mu- QED=0` fails in
  enumeration without the cut (control) and with `30 = ptj1min` fails with the
  cut message and no "enumerate" text; unbindable `1e5 = missetmax` integrates
  successfully while `30 = missetmax` at 91.2 GeV is refused; generate refuses
  on a real artifact.
- "Before" is proven by the order of two failures on a process that cannot
  enumerate. It cannot see elapsed time, nor distinguish the check running
  before model loading from after it (it runs after the model loads, before
  enumeration).

## Mutation evidence

- Early call removed from `load_run_card` (`$S/cutB-F1-mut1.log`):
  `integrate_refuses_before_enumeration` FAILED and
  `generate_refuses_an_unimplemented_cut_naming_it_and_its_value` FAILED;
  `integrate_refuses_an_unimplemented_cut_naming_it_and_its_value` stayed green
  (compile-time refusal on this process carries no flavour-group prefix, so that
  test pins message content, not order).
- Threshold disabled (`if false && ...`) (`$S/cutB-F1-mut2.log`):
  `integrate_lets_an_unbindable_maximum_through_and_refuses_a_bindable_one`
  FAILED; lib `pt_maximum_threshold_is_half_the_collider_energy` and
  `unbindable_pt_maxima_read_as_default` FAILED.
  (`$S` = the scratchpad, files prefixed `cutB-F1-`.)

## Gate

- `cargo fmt --all --check`: rc=0.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile`, no diagnostics.
- `cargo test --workspace`: every `test result` line `ok`, 0 failed (lib 1014 passed, 10 ignored; `cli_unimplemented_cuts` 4 passed; `cli_hard_errors` 9 passed).
- `python3 scripts/kb.py lint`: `822 concepts, 288 backlog items; 0 errors, 4 warnings` (the warnings are pre-existing `state: blocked` without `blocked_by`).
- MG net (`validate`) not run: the change touches no amplitude or evaluation code.

## Brief corrections

- `generate` reads the artifact before the run card, so its refusal comes after a
  (cheap) artifact read; nothing expensive precedes it.
- pixi is absent; commands were run directly.

## Found

- HT-sum maxima (`htjmax`, `ihtmax`, `ht2max`..`ht4max`) at or above `sqrt(s)`
  are still refused as active; extending the no-op rule needs MadGraph's
  `cuts.f` definition of the summed objects (to be read from the pinned
  `mg5amcnlo` submodule) and a test.
- `generate` reads the artifact before loading the run card, so a bad run card
  is reported after an artifact read and parse; moving the run-card load first
  would make the refusal uniform (low priority, cheap).
