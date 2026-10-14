---
type: Session Report
title: "R-E report: hadronic integration, PDFs and scales"
description: "15 findings on proton, hadronic, pdf and coupling: a duplicated SDE_strategy predicate whose unit test pins the unread copy, triplicated integrand methods, module-split boundaries, and untested scale choices."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. The protocol caps findings
at 30; R-E reported 15. Paths are under `vibegraph-lib/src/`. Confidence is
the reviewer's own.

**Backlog check:** 13 filed items touch the cluster, and none was
re-reported.

## Findings

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-E.1 | 2, 1 | `hadronic.rs:349,384,1486,3882-3935` | The `SDE_strategy` rule is computed twice. Production reads only `compile_configuration_weights`; `weights_configurations_by_amp2` has only test and doc readers, and the unit test pins that copy. Mutating `:1486` (`!= 0`) passes it. The field doc at `:172-182` and `runcard/classes.rs:70,177` are stale or name the wrong reader. | fix here (shared predicate; test the production path; fix docs). It lands with configuration-weights-wrong-at-sde1-with-tmin | checked |
| R-E.2 | 4 | `proton.rs` / `hadronic.rs` / `multiplicity.rs` | `split_point`, `adapt_grids(_budget)` and `use_onshell_veto` are duplicated verbatim; `probe_scale`, `scale_channel` and the `select_event` weight block share a skeleton. | file (`ConfigurationWeights` type; provided `ChannelIntegrand` methods) | checked |
| R-E.3 | 1 | `hadronic.rs` (4.7k lines) | It is shared scale and subprocess plumbing plus the fixed-beam integrand. Proposed split: `hadronic/{scale_source, subprocess, initial_state, fixed_beam}.rs`. | file | checked |
| R-E.4 | 1 | `proton.rs` (6.5k lines) | The seam at `:1089` divides flavour grouping from `ProtonIntegrand`. `derive_flavor_groups` (216 lines) splits along its three stages. | file | checked |
| R-E.5 | 1, 4 | `hadronic.rs` / `budget.rs` | An import cycle: the channel-combination helpers live in `hadronic`. | file, with R-E.3 | checked |
| R-E.6 | 1 | `hadronic.rs:9,1553,2384,2452,2519`; `proton.rs:72,1221` | Docs say "per-diagram" channels; production builds one per MadGraph configuration. Not in the claimed stale-comment items. | fix here | checked |
| R-E.7 | 3 | `proton.rs:442`, `hadronic.rs:2846` | Retarget the docs linking `symmetry_weighted_luminosity` to `_rows`, then plain `cfg(test)`. Delete `subprocess_evaluator` and reword `EventSelection::subprocess`. | fix here | checked |
| R-E.8 | 2 | `coupling/scales.rs:653-680` | Fixed-beam closed-form choices 1, 2 and 5 are honoured, but only 3 has a banked row, and only 3 and 4 have value tests. The justification at `:1054` overclaims. | fix here (value tests); file (refuse 1, 2 and 5 at fixed beams until a row exists) | checked |
| R-E.9 | 2 | `hadronic.rs:3320-3349` | `fixed_beam_integrand_finite_positive_2to2` asserts only σ > 0. Doubling `prefactor()` passes it. | fix here (closed-form σ, or delete) | checked (other catchers suspected) |
| R-E.10 | 2 | `hadronic.rs:3372-3433` | `probe_from_diagram_volume` is `#[ignore]`d and asserts nothing, yet it is the only volume check on `from_diagram` channels. | file (move to `phasespace` as an asserting gate) | suspected |
| R-E.11 | 1 | `coupling/scales.rs:852`, `cluster/kt.rs:209` | `mg_dot` is transcribed twice, citing different `kin_functions.f` lines. | fix here (one copy; citation checked at the pin) | checked |
| R-E.12 | 1 | `hadronic.rs:431-438/512-517, 462-468/521-526` | `point_scales` and `point_history` duplicate the refusal routing and the group lookup. | fix here | checked |
| R-E.13 | 1 | `coupling/alphas.rs:558`, `coupling/scales.rs:1391` | Comments cite "the sprint's named trap" and "note 22", against the AGENTS.md comment guidelines. | fix here | checked |
| R-E.14 | 3 | `pdf/mod.rs:141` (and `:84`) | `PdfMember::subgrids` is `pub` but `interp` is precomputed from it; a write desynchronises them. | fix here (private field plus getter) | checked |
| R-E.15 | backlog | two claimed items | The `proton.rs` and `configs.rs` line citations in `sampler-and-phase-space-doc-comments-stale` and `validation-test-comments-stale` have drifted. | fix here, in the items, by the fix session that closes them | checked |

**Also seen:**
- `Dynamic` is built once and could be inlined (`scales.rs:686`).
- The cubic-Hermite basis is duplicated (`pdf/alphas.rs:179`, `pdf/interp.rs:603`).
- The `proton.rs:1813` doc mis-describes the probe.
- A missing doc paragraph break at `hadronic.rs:2060`.
- The kT, setclscales and rewgt Fortran transcriptions are long but should stay 1:1.

**Cross-cluster patterns:**
- sibling integrands duplicating methods;
- a rule computed twice, with the test pinning the unread copy (check every run-card rationale string's named reader);
- diagnostics living in the wrong module;
- "per-diagram" wording left from before channels merged per configuration.

## Method

The share of the session each step took:

| step | share | produced |
|---|---|---|
| backlog check | 10% | — |
| structure outline and boundary-crossing imports | 15% | R-E.3–5 |
| `cfg` inventory | 15% | R-E.7 |
| run-card branch tracing, grepping accessor readers workspace-wide | 25% | R-E.1, R-E.8 |
| reading same-name method pairs | 15% | R-E.2, R-E.11 |
| assertion and tolerance sweeps, plan-citation regex | 15% | R-E.9, R-E.10, R-E.13 |

**What worked:** grepping an accessor's readers across the whole workspace.

**Little yield:** length censuses and tolerance regexes.

**Dead ends:**
- an assertion-less-test awk census was too loose;
- a suspected nonexistent `unreachable!` exists;
- loose-looking bounds carry measured justifications;
- `f64::from(1e-99f32)` is a documented transcription.

## Found

- **The run-card rationale strings name reader functions,** and nothing checks
  that those readers exist or are live (R-E.1).

## Brief corrections

- **`git branch --show-current` is empty on a detached checkout.** Use
  `git rev-parse --short HEAD`.
- **`pixi` is absent.**
- **Only two gated helpers here are doc-linked,** not "several".
- **The `SDE_strategy` defect is a duplicated predicate,** not an unreachable arm.
- **The `mg5amcnlo` submodule is empty in reviewer worktrees,** so Fortran
  citations could not be checked. (Manager: fix sessions get the submodule
  copied in.)

## Manager check (2026-10-09)

Worktree clean after the review.
- **R-E.1:** reproduced. `weights_configurations_by_amp2` is read only at
  `hadronic.rs:3921-3929` (tests), `tests/validate_hadronic.rs:2742` and doc
  strings. `compile_configuration_weights` tests the condition itself at
  `:1486`.
- **R-E.11:** both `mg_dot` copies exist at the cited lines.
- **R-E.8:** `CLOSED_FORM_RUNS` is the only closed-form run list in
  `validate_scales.rs`.
