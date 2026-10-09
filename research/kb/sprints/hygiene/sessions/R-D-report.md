---
type: Session Report
title: "R-D report: phase space and sampling"
description: "17 findings on vegas, budget, phasespace, cuts, unweight and select: production docs citing test-only APIs, a vacuous parallel-agreement test, a dead parallel scheme, an unselected combination knob, and loose σ gates."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Paths are under
`vibegraph-lib/src/`. Mutations were reasoned from the code, not run.

**Backlog check:** about 37 filed items cite cluster paths. None was
re-reported except where a finding names a site the item lacks.

## Findings

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-D.1 | 3, 1 | `vegas.rs:250,504-529,995,1032`; `budget.rs:413`; `proton.rs:2868`; `channel.rs:17,363,612-641,857-867`; `phasespace/mod.rs:11`; `unweight.rs:570-581` | Production docs define themselves against test-only APIs: `with_warmup`, `adapt_parallel_seeded` (which carries the bit-for-bit contract of production's `adapt_blocks_iteration`), `set_combination`, `sample_channel(_at)`, `Lips2Channel`, `sigma_from_trials`. | fix here (move the contracts onto the production functions, re-point the citations, then plain `cfg(test)`) | checked |
| R-D.2 | 2 | `vegas.rs:1796-1811` | `test_parallel_agrees_with_sequential_statistically` never compares the two arms. A 1% bias in `run_iter_parallel` passes. | fix here (delete with R-D.3) | checked; mutation not run |
| R-D.3 | 1 | `vegas.rs:476-501,601-614,746-822,1198-1201,1764-1811` | The `(iteration, chunk)` parallel scheme, about 150 lines, is `cfg(test)` with no production caller and is kept alive by its own tests. | fix here (remove; kb lines to close-out) | checked |
| R-D.4 | 1 | `budget.rs:623,656-662,56-61,1207`; `vegas.rs:32-45,134` | `IterationCombination::InverseVariance` is never selected: all five production callers pass `default()`. The parameter threads through `integrate_channels`, and `combine_kept` ignores `rule` for channels whose point counts vary. | file (crosses five modules, API change) | checked |
| R-D.5 | 2 | `budget.rs:880-882` | No end-to-end test sees whether the stop reads the pooled-widened error rather than the quoted one. | fix here (one loop test where widening binds) | suspected |
| R-D.6 | 1 | `vegas.rs:233` | The `VegasResult::integral` doc says "weighted combination"; the default is the arithmetic mean. | fix here | checked |
| R-D.7 | 1 | `phasespace/mod.rs:3-59`; `channel.rs:16-17` | Module docs lead with the test-only 2→2 LIPS machinery and omit `DiagramChannel`, the production map. | fix here | checked |
| R-D.8 | 1, 4 | `vegas.rs:833-905` vs `:1050-1148`; variance formula at `:740,821,903,1139` | The seeked chunk kernel is duplicated, and the moments formula appears four times. | fix here (`adapt_parallel_seeded` as a loop of `adapt_blocks_iteration`; a `moments` helper) | checked |
| R-D.9 | 2 | `unweight.rs:799,978-1010,1040`; `vegas.rs:1260-1328` | σ-recovery tests gate at 2%, about 15× the statistical error, on one seed. A 1.5% normalisation bias passes. The VEGAS smoke tests read like gates. | fix here (N× the quoted error, headroom over ≥ 5 seeds) | suspected |
| R-D.10 | 1 | `vegas.rs:1203,1343,1354,1376,1932`; `budget.rs:415,1427`; `diagram_channel.rs:388` | Comments narrate history ("pre-split", "replaces", "used to"). One of two goldens is redundant. | fix here | checked |
| R-D.11 | 1 | `phasespace/maps.rs:91-118`; `multiplicity.rs:84` | `ProcessShape::moving_splits` is computed per diagram and never read. | fix here (remove) | checked |
| R-D.12 | 4 | `rng.rs:30-38` plus five stream constants in other modules | Stream-family disjointness is claimed but pinned by no test. Nothing collides today; the closest gap is 12 835. | file (one registry with a disjointness test) | checked |
| R-D.13 | 1 | `budget.rs:619-1042` | `integrate_channels` is 424 lines; six private helpers are named. | fix here, or file | checked |
| R-D.14 | 4 | `phasespace/channel.rs` | `MultiChannel` and `ScaledMultiChannel` duplicate their constructor checks, normalisation check, density memo sweep and method pairs. | fix here (shared `assert_normalized`); file (single combiner) | checked |
| R-D.15 | 1 | `cuts.rs:440-600` | `Cuts::compile_with` is 161 lines in four independent sections. | fix here | checked |
| R-D.16 | 4 | `select.rs:13-26`; `channel.rs:294`; `stats.rs:442` test | Three categorical draws with different fallbacks, and the `select.rs` doc says "Both draws … defined once here". | fix here (doc); file (unify) | checked |
| R-D.17 | 3 | `diagram_channel.rs:391`; `phasespace/mod.rs:157`; `vegas.rs:1205` | Items `pub` only for integration tests. | leave | checked |

**Also seen:**
- the `DEFAULT_WARMUP_ITERS` doc repeats a value;
- the `multiplicity.rs` inherent `pub` methods duplicate its `ChannelIntegrand` impl;
- `tests/validate_samples.rs:47` links a `pub(crate)` item;
- the `stats.rs` and `select.rs` statistical tests are sound.

**Cross-cluster patterns:**
- production docs citing test-only APIs;
- knob parameters threaded through every caller at `default()`;
- about six Lorentz boost implementations (`lorentz.rs:204`, `hadronic.rs:593`,
  `onshell.rs:582`, `lhef/resonance.rs:427`, `cluster/kt.rs:310`,
  `diagram_channel.rs:1735`);
- hand-rolled pT, rapidity and Δφ (`cuts.rs`, `lhef/observables.rs`,
  `coupling/scales.rs`);
- RNG stream constants spread across modules.

## Method

| step | share | produced |
|---|---|---|
| backlog check | 10% | — |
| `vegas.rs` read whole, every API's callers grepped | 30% | R-D.1–4, 6, 8, 10 |
| `budget.rs`, asking what each assert can see | 20% | R-D.4, R-D.5 |
| function-length awk, history-word grep, stream grep, other files | 25% | R-D.9–16 |
| leads that failed, and the write-up | 15% | — |

**What worked best:** for each `cfg(any(test, doc))` item, grepping which
production doc links it.

**Leads that did not hold up:**
- `diagram_channel.rs` volume tests at 5–6σ, which exact checks cover;
- the `stats.rs` tests;
- a stream collision;
- non-default `MaxRule`, `TauMap` and similar arms, which the CLI can select;
- golden coverage of the default path.

**Coverage gap:** `diagram_channel.rs`'s 2 700 production lines got a targeted
pass only.

## Found

1. **Kb concepts cite drifted `vegas.rs` line ranges** and present
   `adapt_parallel_seeded` as the production contract:
   `phase-space/rng-substreams-and-parallel-determinism.md:16`,
   `vegas-integrator.md:116`, `vegas-grid-weight-tail.md:13`. Close-out, with
   R-D.1 and R-D.3.
2. **`ChannelHistory::combine_kept` silently ignores `rule`** when point
   counts vary. Latent while only `Unweighted` is passed.

## Brief corrections

- **The CLI does not cite `adapt_parallel_seeded`.**
- **`git branch --show-current` is empty on a detached checkout.**
- **The per-name backlog grep is noisy** (`vegas` matched 34 items). Grep path
  tails such as `/vegas.rs`.

## Manager check (2026-10-09)

Worktree clean.
- **R-D.11:** reproduced. `moving_splits` is written at `maps.rs:118` and
  `multiplicity.rs:84` and appears otherwise only in test literals.
- **R-D.2:** reproduced. The test's only asserts are the two `abs() < 0.02`
  lines.
- **R-D.4:** reproduced. The five production `IterationCombination::default()`
  callers are as cited.
