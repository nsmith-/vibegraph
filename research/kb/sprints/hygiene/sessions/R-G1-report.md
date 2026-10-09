---
type: Session Report
title: "R-G1 report: amplitude, model and diagram test targets"
description: "27 findings: colour oracles that pass on zero trials, a known-disagreement arm that swallows any panic, self-comparing diagram counts, unchecked table coverage, hermetic tests registered as banked, and duplicated test plumbing."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Mutations were reasoned
from the code, not run.

**Backlog check:** about 18 filed items cite cluster files. The claimed
`smeftsim-vendored-checksum-not-hermetic` covers too few sites (R-G1.5).

## Findings

Paths are under `vibegraph-lib/` unless stated.

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-G1.1 | 2 | `tests/color_cf_oracle.rs:620-641`; `tests/color_flow_tags_oracle.rs:266-271` | Both banked colour oracles exit green with zero trials when `output/` is missing, which breaks the no-runtime-skip rule, and drop two hermetic controls. Directory walks swallow `read_dir` errors. | fix here (`require`; controls unconditional) | checked |
| R-G1.2 | 2 | `tests/standalone_jamps.rs:~224` | The known-disagreement arm accepts any panic (a missing table, an assert) as "disagrees as listed", discarding the payload. | fix here | checked |
| R-G1.3 | 2 | `tests/color_cf.rs:236-255` | The diagram-count asserts read back the size passed in, and `LeadingColorFlows::of` silently drops out-of-range diagrams. | fix here (pass the real count) | checked |
| R-G1.4 | 2 | `tests/amplitude_oracle.rs:881,1510-1535`; `tests/coupling_oracle.rs:279-290` | No two-way check of the amplitude tables against the manifest. Deleting `ee_to_ee.json` stays green. `uux_to_mumu` has no `mg_amplitude`. | fix here (coverage trial) | checked |
| R-G1.5 | 2 | `Cargo.toml:211-217`; `tests/smeftsim.rs`; `tests/toy_models.rs` | Both targets have only committed inputs but are banked, so their gates never run in CI. | file (widen the claimed item; it changes layer registration) | checked |
| R-G1.6 | 2 | `tests/validate_scale_couplings.rs:33-72` | Silently falls back to the default param card without `output/`. `PROCESSES` lists 14 of 48 tables. | fix here (read the committed table's card) | checked |
| R-G1.7 | 1, 2 | `tests/color_cf_oracle.rs:420-446` | `COLOUR_ENFORCED_INFO_ROWS` holds a row that is now a gate and is checked in one direction only. | fix here | checked |
| R-G1.8 | 2 | `tests/validate_helas.rs:66`; `tests/helas_kernel_composition.rs:177` | Tolerances of 3e-6 and 1e-4 on what should be exact arithmetic, likely a parameter-provenance mismatch. | file (needs a re-measurement) | checked; cause suspected |
| R-G1.9 | 2 | `tests/ufo.rs:60-74` | The ignored `test_load_taudecay` returns early on exactly its ignore reason. | fix here | checked |
| R-G1.10 | 1 | `tests/amplitude_oracle.rs:32,68-72` | The module doc lists pruned-vs-unpruned `AMP2` as asserted; it is only measured. | fix here | checked |
| R-G1.11 | 1 | `tests/validate_madgraph_diagrams.rs:70`; `tests/amplitude_oracle.rs:320,787,870`; `tests/common/mod.rs:145` | Comments say SMEFTsim rows don't load. | fix here | checked |
| R-G1.12 | 1 | `tests/helas_kernel_composition.rs:98-100` | The TODO awaits `/ H` filtering, which exists. | fix here | checked |
| R-G1.13 | 2 | `tests/decay_chain_census.rs:146-221` | The decay-chain identical-particle factor is banked but only printed, against a test-local function. | file | checked |
| R-G1.14 | 2 | `proc_grammar_oracle`, `schannel_census`, `decay_chain_census`, `polarization_census` | Census oracles accept a refusal for any reason (58 cases), and two lack a non-empty guard. | fix here (docs, guards); file (class matching). Same as R-C.1 | checked |
| R-G1.15 | 2 | `tests/smeftsim.rs:172-217,405-421` | Counts asserted as "matches_madgraph" (1985 …) are not banked anywhere. | file (bank, or rename as a change detector) | suspected |
| R-G1.16 | 3 | `src/helas/color/colorize.rs:91-106`; `src/ufo/mod.rs:545` | `ColorBasis.elements` is `pub`, and its length strides the private CF matrix. `param_values` is the same as R-C.11. | file | checked |
| R-G1.17 | 4 | four sites | "Manifest mode for a row" is looked up four times. | file | checked |
| R-G1.18 | 4 | the two colour oracles | `run_trial`, `trial_name`, header cleanup and the directory walk are duplicated. | file | checked |
| R-G1.19 | 4 | five sites | Row-to-model resolution is implemented five times. | file | checked |
| R-G1.20 | 4 | about 8 test sites plus `rooting_soundness.rs` | Banked-reference readers (param card, momenta, diagram counts) are duplicated, some verbatim. | file (`common::banked`) | checked |
| R-G1.21 | 4, 1 | `smeftsim.rs:620`, `toy_models.rs:128` | Re-gate the `diagrams.json` counts `validate_madgraph_diagrams` already gates. | file | suspected |
| R-G1.22 | 4 | four census files | The census skeleton has no shared home. | file | checked |
| R-G1.23 | 1 | `tests/amplitude_oracle.rs:872-1504` | `measure` is 630 lines. | fix here | checked |
| R-G1.24 | 1, 4 | `tests/finite_field_msq.rs` | A 3091-line study with hand-written CRT and rational reconstruction; four gates. | file | suspected |
| R-G1.25 | 1 | `tests/color_cf_oracle.rs:844-864` | A doc attached to the wrong item. | fix here | checked |
| R-G1.26 | 1 | `smeftsim.rs`, `color_cf_oracle.rs` | History narration. | fix here | checked |
| R-G1.27 | 1, 4 | `validate_helas.rs`, `helas_kernel_composition.rs` | The e⁺e⁻→μ⁺μ⁻ fixture is duplicated. | file, or fix here in `tests/` | checked |

**Also seen:**
- censuses transcribed with no banked artifact;
- no tie of standalone tables to tests;
- a tautological event count;
- `zip` truncation;
- loose `ufo.rs` tolerances;
- `generate_with(..).remove(0)` on a possibly empty set;
- bench closures possibly dead-code-eliminated.

**Cross-cluster patterns:**
- `catch_unwind` discarding payloads;
- per-file banked readers;
- hermetic inputs registered as banked (audit every `required-features`
  target's inputs against `git ls-files`);
- "what the code produced" literals under oracle names;
- one-way allowlists.

## Method

| step | share | produced |
|---|---|---|
| backlog check | 10% | — |
| non-vacuity: every `return`, `exists()`, `catch_unwind` and tolerance tested against what it hides | 55% | R-G1.1–4, R-G1.6–7, R-G1.9, R-G1.13–15 |
| reuse greps plus diffing bodies | 15% | R-G1.17–22, R-G1.27 |
| outlines and narration grep | 10% | R-G1.10–12, R-G1.23–26 |
| visibility | 5% | R-G1.16 |
| benches | 5% | — |

**Fastest technique:** grepping for `exists()|return;|read_dir|catch_unwind`.

**Dead ends:**
- `finite_field_msq`'s returns are inside closures;
- `gluon_parke_taylor`'s counts are derivable;
- `polarization_census`'s list is two-way;
- the bit-equality asserts are same-platform.

## Found

- **Production draws configurations from the helicity-pruned `AMP2`,** and only
  the unpruned one is compared with MadGraph. The gap is reported, not bounded
  (39.5% on `gg_to_ttx` when introduced).
- **`LeadingColorFlows::of` silently drops out-of-range contributions.**

## Brief corrections

- **The cluster is about 12.8k lines,** not 15k.
- **`git branch` is empty on a detached checkout.**
- **The claimed smeftsim item should widen** to both targets.

## Manager check (2026-10-09)

Worktree clean.
- **R-G1.1:** the empty-set `libtest_mimic::run(&args, vec![])` exit is present.
- **R-G1.2:** the any-panic arm is at `standalone_jamps.rs:224` (the report
  said ~204-212).
- **R-G1.3:** `n_diagrams()` derives from the table size it was given.
