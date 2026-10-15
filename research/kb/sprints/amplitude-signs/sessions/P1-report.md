---
type: Session Report
title: "P1 report: the per-diagram probes are bridged to the production Program"
description: "A hermetic test pins the per-diagram probes to the production evaluator, as complex values per amplitude, per flow and per helicity combination, on eleven SM processes. Measured agreement is at most 2.4e-14, and three single-path sign mutations fail it. The bridge cannot see anything the two paths share: rooting, anchor-read convention signs, external wavefunctions and kernels."
status: draft
tags: [evaluator, oracle, blind-spot, validation, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: bridge, resource: "https://github.com/nsmith-/vibegraph/commit/23e57f2", title: "test(eval): bridge the per-diagram probes to the production Program"}
---
Closes [per-diagram-probes-bypass-production-runtime](../../../backlog/validation/per-diagram-probes-bypass-production-runtime.md)
by its second arm: a hermetic test that bridges the two paths on processes with
internal fermions.

## What was done

Added `vibegraph-lib/src/helas/eval/run/probe_bridge.rs`, a `#[cfg(test)]`
child module of `run`. Its one test is
`per_diagram_probes_match_the_production_program`. The only other change is one
sentence in the `run_forward_slot` doc comment that points to the bridge. Nothing
in the emitted `Program`, the lowering, the layout or any kernel changed.

The test compiles each process once with `AmplitudeEvaluator::compile`. It
rebuilds the same `DiagramEval`s with `root_amplitudes` (the function `compile`
calls) and evaluates four generic on-shell points per process. The points are
seeded massive RAMBO at two energy scales, beams along ±z. At every helicity
combination it compares, as complex values:

| level | production side | probe side |
|---|---|---|
| per amplitude, hot path | each `(diagram, chain)` configuration amplitude read off the **helicity-expanded** program's `amp_locs` row after `fill_for` (the `eval_m2` / `eval_amp2` arena), × symmetry factor × Fermi sign | `eval_single_diagram` on the same `DiagramEval` |
| per amplitude, base | the same value from `run_config_amps` (the base program, which `amplitude_oracle` compares to MadGraph's `AMP()`) | same |
| per flow, hot path | `eval_hel_jamps` | Σ over the flow's contributions of colour coefficient (Nc = 3, `i` when imaginary) × probe |
| per flow, base | `run_flows`, or `eval_amplitude` for one flow | same |
| \|M\|² | `eval_m2` | the CF contraction of the probe JAMPs |
| probe consistency | `eval_single_diagram` on `compile_diagram_ast`'s diagram, the colour-free rooting the existing probes use | its chain-zero amplitude, required equal to the bit |

Residuals are taken relative to the largest probe amplitude at the same helicity
combination (a diagram scale, not the flow's value, so a cancelling flow is not
lost). The gate is `REL_TOL = 1e-12`.

Non-vacuity is asserted. The configuration-amplitude order must equal
`config_amp_diagrams()`, and the row length must equal the number of
configuration amplitudes. Each process must make at least one flow comparison.
The diagrams must cover each of these features at least once (measured counts in
brackets):

- internal fermion (22);
- crossed (final-final) line with a propagator (17);
- mixed line with a propagator (2);
- contact vertex (3);
- multi-flow process (4);
- both relative Fermi signs in one process (2);
- non-zero colour chain (16).

## Route chosen and why

I took the test route. The probe *values* can be read from the production
program without changing it, through the configuration-amplitude bundle (`Op::Configs`).
Since the `AMP2` work, `lower_flows` has threaded that bundle through the program:
it holds every non-contact `(diagram, chain)` amplitude, readable after a
`fill_arenas` pass.

Routing the probes themselves through `Program` would need two things the
program does not have:

- **A readout for off-shell currents.** `eval_single_diagram_slot` also roots at
  internal currents, and the Z-current tests against MadGraph's `W` arrays read
  them.
- **A readout for contact diagrams.** These carry no configuration, by
  MadGraph's `get_amp2_lines` rule.

Adding either changes the emitted `Program`, which the brief rules out.

**The labelling is the same on both sides.** Both paths evaluate one
`DiagramSet` and one set of `DiagramEval`s. This is exact by construction:
`root_amplitudes` and `compile` call the same `compile_single_diagram` on the
same diagrams. So W1's finding (per-diagram signs move under relabelling legs 0
and 1) moves both paths together, and nothing here depends on it.

**A correction to the item's framing.** The integration-level per-diagram gate
already reads production. `amplitude_oracle` compares `run_config_amps`, a
`fill_arenas` readout of the base program, to MadGraph's `AMP()`. Only the
in-crate probes bypassed production: those in `run.rs` and the temporary dumps
sessions write on `compile_diagram_ast`. The bridge also ties the base program
(the one `amplitude_oracle` reads) to the helicity-expanded program (the one
`eval_m2` runs).

## Gates

Measured residuals, from
`cargo test -p vibegraph-lib --lib probe_bridge -- --nocapture`:

```
process            amp_hel  amp_base  flow_hel flow_base        m2  n_amp n_flow
e+ e- > mu+ mu-      0.0e0     0.0e0     0.0e0     0.0e0     0.0e0    256    128
e+ e- > a a          0.0e0     0.0e0     0.0e0     0.0e0     0.0e0    256    128
e+ e- > w+ w-      3.8e-15   3.8e-15   4.9e-15   4.9e-15   1.0e-14    864    288
w+ w- > e+ e-      4.2e-16   4.2e-16   4.2e-16   4.2e-16     0.0e0    864    288
e- a > e- a          0.0e0     0.0e0     0.0e0     0.0e0     0.0e0    256    128
g g > u u~           0.0e0     0.0e0     0.0e0     0.0e0     0.0e0    384    256
g g > g g          1.2e-14   1.2e-14   2.4e-14   2.4e-14   4.0e-16    384    768
u u > u u            0.0e0     0.0e0     0.0e0     0.0e0     0.0e0    256    256
w+ w- > w+ w-      1.3e-14   1.3e-14   1.3e-14   1.3e-14   2.5e-14   3888    648
z z > t t~           0.0e0     0.0e0   3.3e-16   3.3e-16   3.4e-16    864    288
g g > t t~ g       9.9e-16   9.9e-16   1.1e-15   1.1e-15   4.3e-16   3840   1536
Features { internal_fermion: 22, crossed_line_with_propagator: 17, mixed_line_with_propagator: 2, contact: 3, multi_flow: 4, relative_fermi_sign: 2, nonzero_chain: 16 }
test helas::eval::run::probe_bridge::per_diagram_probes_match_the_production_program ... ok
test result: ok. 1 passed; 0 failed; ...; finished in 1.04s
```

The worst residual is 2.4e-14 (`g g > g g`, per flow), which leaves 40× headroom
under 1e-12. Seven of the eleven processes are bit-identical at every level.
Where they are not, the gap comes from the production path's
common-subexpression elimination and n-ary sums, which re-associate.

Every cargo invocation below ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo fmt --all --check` gave exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` finished
  (`Finished dev profile ... in 1m 44s`) with exit 0.
- `cargo test --workspace` gave exit 0. It ran 38 suites, all `ok`, with
  1396 passed, 0 failed and 16 ignored (summed over the suites' `test result`
  lines).
- `pixi run --skip-deps validate` gave exit 0. It ran 63 suites, with 1781
  passed, 0 failed and 84 ignored. The report says: `64 rows × 4 categories =
  256 cells: 203 measured in the layers this run drove (196 ✅, 7 ⚠️, 13 ⏳,
  40 — / uncovered)` and `every gate cell passed`. The seven ⚠️ cells are the
  manifest's declared informational cells; no production code changed.

## Falsifier mutations

Each mutation was applied by script, run against the bridge, and reverted
(scratchpad `P1-mut/mutate.py`, `P1-mut/run_all.sh`). Afterwards
`git diff --stat` showed only the session's own `run.rs` change.

| mutation | path it breaks | result |
|---|---|---|
| `fill_arenas`, `Instr::PropagateFin`: output × −1 | production only, every diagram with a ket fermion propagator | **fails**: `e+ e- > a a: amplitude (0, [0, 0]) hel [-1, 1, -1, -1]: helicity-expanded program -7.031179099987257e-2+6.038362814076872e-2i vs probe 7.031179099987257e-2-6.038362814076872e-2i (rel 2.00e0)` |
| `lower_flows`: diagram 1's symmetry × Fermi factor × −1 | production JAMPs only; configuration amplitudes carry no such factor | **fails**: `e+ e- > mu+ mu-: flow 0 hel [-1, 1, -1, 1]: helicity-expanded JAMP -5.856819646224324e-2+5.0203984433826156e-2i vs probe sum -5.48320855034758e-2+4.7183955197646336e-2i (rel 6.43e-2)` |
| `lower::lower`: negative-Fermi-sign diagrams × −1 | probe only | **fails**: `e+ e- > mu+ mu-: amplitude (0, [0, 0]) hel [-1, 1, -1, 1]: helicity-expanded program -5.670014098285952e-2+4.8693969815736246e-2i vs probe 5.670014098285952e-2-4.8693969815736246e-2i (rel 2.00e0)` |
| `kernel::propagate_fin_bare`: numerator × −1 | **both** paths (a shared kernel) | passes, as designed: this is the bridge's blind spot |

The second row is why the bridge needs both levels. A sign that production puts
only on a JAMP passes the per-amplitude comparison and is caught only per flow.

## Blind spots

The bridge cannot see:

- **Anything the two paths share.** This covers rooting and the choice of root,
  every convention sign read at `Diagram::anchor` (the class of W1's finding),
  external wavefunctions, the kernels (shown by the fourth mutation) and the
  colour basis with its coefficients. All of these are MadGraph's job, through
  `amplitude_oracle` / `standalone_jamps`. Agreement here says the probes
  describe production; it does not say that production is right.
- **The labelling dependence.** Both sides use one labelling, as the brief asks,
  so a convention that changes under relabelling is invisible.
- **Contact diagrams one at a time.** Contact diagrams have no configuration
  amplitude, so they enter only through the flow sums. A wrong sign on one
  contact chain is caught in every flow that chain reaches. Errors that
  compensate between contact chains within the flows are not caught.
- **Off-shell currents.** The current-rooted probes (`eval_single_diagram_slot`
  returning a non-scalar slot) have no production counterpart and stay
  unbridged.
- **Pruned evaluators.** The bridge compiles unpruned evaluators. The
  helicity-pruned and zero-amplitude-pruned arenas are pinned separately by the
  existing pruning tests and the amplitude gate.
- **The colour coefficient's `i` convention.** The probe sum applies
  `ColorCoeff` the way the production lowering reads it. A convention error in
  `ColorCoeff` itself would hit both sides.
- **Points.** There are four points per process, at two scales. The comparison
  is exact algebra, not statistics, so a missed region would need a kinematic
  coincidence.

## Found

- **The colour-free probe sees chain zero only.** `compile_diagram_ast` roots
  every diagram at chain `[0; n]`. The 16 non-zero-chain amplitudes in this
  process set (the four-gluon contact's other colour structures in `g g > g g`
  and `g g > t t~ g`) cannot be dumped by any probe written on it, and W1's
  probe was written on it. To close it, per-diagram dumps should iterate the
  `root_amplitudes` keys, as the bridge does.
- **Contact diagrams have no production per-amplitude readout.** Because of
  this, neither the bridge nor `amplitude_oracle` can compare one contact
  diagram's amplitude on its own, against the probe or against MadGraph's
  `AMP()` (MadGraph computes an `AMP()` for it; only the `AMP2` channel is
  missing). To close it, add a test-only readout of contact amplitudes. It
  would need a `Program` change, which this session was not allowed to make.
- **`test_whole_amplitude_equals_diagram_sum_eemumu` is now subsumed.** Its
  process is the bridge's first row, compared at finer levels. It can be
  deleted in a hygiene pass.
