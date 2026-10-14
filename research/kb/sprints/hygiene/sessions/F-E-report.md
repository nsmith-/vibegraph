---
type: Session Report
title: "F-E report: hadronic, PDF and scale fixes"
description: "All nine R-E findings, R-G2 Found 1 and the SDE/tmin item fixed in 4 commits: one SDE_strategy predicate with tmin_for_channel refused (the item's prescribed weight was wrong at MadGraph's own source), value tests for scale choices 1/2/5, a closed-form fixed-beam σ test."
status: draft
tags: [hygiene, fix, hadronic, scales, pdf, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: fd76a96, resource: "https://github.com/nsmith-/vibegraph/commit/fd76a96", title: "one SDE_strategy rule for the configuration draw and the scale source"}
  - {id: efd3366, resource: "https://github.com/nsmith-/vibegraph/commit/efd3366", title: "value tests for closed-form choices 1, 2 and 5; one mg_dot"}
  - {id: 4bf4df6, resource: "https://github.com/nsmith-/vibegraph/commit/4bf4df6", title: "closed-form fixed-beam σ test, per-configuration docs, private subgrids"}
  - {id: 4cbaa72, resource: "https://github.com/nsmith-/vibegraph/commit/4cbaa72", title: "closed-form σ in pb"}
  - {id: genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L1878-L1960", title: "get_channel_cut at the pin"}
---
The dev agent's report, condensed by the manager. Mutation logs are in the
scratchpad (`fe-mutations.log`, `fe-mutations2.log`).

## Fixed

| finding or item | commit | change | evidence |
|---|---|---|---|
| R-E.1, configuration-weights-wrong-at-sde1-with-tmin | fd76a96 | One predicate, `configurations_weighted_by_amp2(card)`: `SDE_strategy == 1` weights by AMP2, otherwise by `channel_cuts`. `tmin_for_channel ≠ -1` is refused at either strategy, including for a deserialised card that skips the parser. Docs and `classes.rs` rationale name the real reader. The test asserts on `compile_configuration_weights` itself | R-E.1's mutation (`!= 0`) fails; disabling the refusal fails |
| R-E.12 | fd76a96 | `channel_input` and `vetoes_point` helpers shared by `point_scales`, `point_history` and `scales` | Banked `validate_hadronic` and `validate_sigma` pass |
| R-E.8 (value tests) | efd3366 | Tests for closed-form choices 1, 2 and 5 on legs where E_T, m_T and p_T differ; the overclaiming justification corrected | Three mutations fail all three tests |
| R-E.11 | efd3366 | One `mg_dot` (from `kt.rs`). The citation is `kin_functions.f:593` at the pin; the deleted copy's `:588` was wrong | — |
| R-E.13 | efd3366 | Plan-referencing comments restated in their own terms | — |
| R-E.9 | 4bf4df6, 4cbaa72 | `fixed_beam_integrand_matches_the_closed_form_2to2`: e⁺e⁻→μ⁺μ⁻ via γ+Z at 500 GeV inside the default cuts, at 5× the run's quoted error, derivation in the test doc | Pull −0.8 on the fixed code; a ×2 flux mutation gives +231σ |
| R-E.6 | 4bf4df6 | "Per-diagram" becomes "one channel per MadGraph configuration" at every listed site and four test docs; genuinely per-diagram sites left | — |
| R-E.7 | 4bf4df6 | `symmetry_weighted_luminosity` → plain `cfg(test)`, its doc on `_rows`; `subprocess_evaluator` deleted | — |
| R-E.14 | 4bf4df6 | `PdfMember::subgrids` private behind a getter; readers updated (`PdfSet::info` has no derived state, so left) | — |
| R-G2 Found 1 | 4bf4df6 | The `event_scales` doc says `channel` is the configuration the clustering reads | — |

**The claimed item's resolution differs from its `closes_when`.** It asked for
`AMP2 × channel_cuts` at `SDE_strategy = 1` with `tmin_for_channel` set. At the
pin, `get_channel_cut` (`genps.f:1878-1960`) returns 1 at `sde_strat = 1` for
configurations with fewer than two t-channel lines. Otherwise it multiplies
`exp((t−tmin)/(t+1))` factors whose `t` is assigned only inside the
`sde_strat.eq.2` branches, so that path reads an unassigned value. It is not
the `SDE_strategy = 2` denominator product that `ChannelSet::channel_cuts`
computes. The prescribed weight would therefore be wrong, and the session
refuses the card instead. The manager checked this against the pinned source
(see the check below).

## Stopped (to file)

- **R-E.8, second half:** refuse closed-form choices 1, 2 and 5 at fixed
  beams until a banked row exists. This is a behaviour change.

## Gate (agent, at 4cbaa72)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1353 passed, 16 ignored (+3 tests).
- **`cargo doc`:** 8 lib warnings and 1 bin warning, all pre-existing.
- **Banked:** `validate_scales` 10, `validate_hadronic` 14, `validate_sigma` 7.
- **`validate_pdf_grid`:** 7 of 20 passed. The other 13 lacked
  `NNPDF31_lo_as_0130` in the worktree; the manager fetched it and re-ran
  them (below).

## Found

1. **The claimed item's `closes_when` should be read as the refusal** (above).
2. **`NNPDF31_lo_as_0130` must be copied into fix worktrees.** It is now
   fetched (`fetch.sh NNPDF31_lo_as_0130`).
3. **`get_channel_cut`'s tmin factor is never applied at `SDE_strategy = 2`
   with `tmin = -1`.** It is believed unreachable, but nothing pins it.
4. **`EventScaleSource::constant` hard-codes
   `amp2_configuration_weights: false`.**
5. **MadGraph itself (manager's reading):** `get_channel_cut` at
   `sde_strat = 1` with `tmin_for_channel ≠ -1` reads an unassigned `t`. This
   is a candidate entry for `validation/madgraph-defects.md`.

## Brief corrections

- **The item's premise is wrong at MadGraph's source** (above).
- **R-E.11's correct citation was the `kt.rs` one,** not the `scales.rs` one.
- **The lib package is `vibegraph-lib`;** its library target is `vibegraph`.

## Manager check (2026-10-10)

- **Commits and trailers:** four commits, `Assisted-by` only.
- **MadGraph source:** read at `research/refs/mg5amcnlo`.
  - `genps.f:1878-1960`: the `t` assignments sit only inside
    `if(sde_strat.eq.2)`.
  - `matrix_madevent_group_v4.inc:218-222`: `AMP2·GET_CHANNEL_CUT` at
    `sde_strat = 1`, and `GET_CHANNEL_CUT` alone at 2.
- **Multigrid set:** fetched (`VIBEGRAPH_FETCH_CONSENT=1 bash validation/pdf/fetch.sh NNPDF31_lo_as_0130`)
  and copied into the worktree. `validate_pdf_grid` then passes all 20.
- **Gate re-run:** fmt, both clippy configurations and the hermetic suite.
  The results are in `log.md`.
