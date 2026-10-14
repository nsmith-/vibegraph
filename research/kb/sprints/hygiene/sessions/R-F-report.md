---
type: Session Report
title: "R-F report: I/O, run cards, the CLI and the report"
description: "28 findings: check-events and the collator nearly untested, exit-code-only refusal tests, duplicated integrate/generate integrand assembly, loose CLI σ checks, unnamed-file errors, and stringly-typed report state."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Mutations were reasoned
from the code, not run.

**Backlog check:** about 50 hits; none was re-reported. R-F.20 adds sites
to `artifact-reader-arm-names-format-version`.

## Findings

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-F.1 | 2 | `vibegraph-cli/src/check.rs:130-281` | `check-events` makes 12 checks; only momentum balance is tested. | fix here (a unit test per complaint) | checked |
| R-F.2 | 2 | `validation-report/src/` | The collator has zero tests. Flipping the severity sort or dropping the fail bump passes. | file (fixture harness) | checked |
| R-F.3 | 2 | `vibegraph-cli/tests/cli_generate.rs:548-556` | `expect_refusal` (9 callers) checks only a non-zero exit, so a panic passes as a refusal. | fix here (assert stderr names the flag or field) | checked for one; suspected for others |
| R-F.4 | 1, 4 | `integrate.rs:851-1003` vs `generate.rs:851-925,1706-1765` | Integrand assembly is duplicated between `integrate` and `generate`, which must rebuild the same integrand. | file (one builder per beam mode) | checked |
| R-F.5 | 2 | `vibegraph-cli/tests/cli_integrate.rs:95-100` | `pull < 3` OR `rel < 1%`, about 20× MadGraph's error (the same shape as R-G2.2). | fix here | checked; mutation suspected |
| R-F.6 | 2 | `cli_integrate.rs:111-139` | The frozen-grid replay ignores `artifact.maps`. | fix here | suspected |
| R-F.7 | 1 | `main.rs:29-40`; `generate.rs:818`; `integrate.rs:459` | Errors name no file or flag (missing proc card, "no events requested", run-card parse). | fix here | checked |
| R-F.8 | 1 | `vibegraph-cli/src/assets.rs:82-87,192-196` | Flag and env-var sources share one not-found message. | fix here | checked |
| R-F.9 | 1, 2 | `vibegraph-lib/src/config.rs:100-112,224` | An unknown SM restrict variant is reported as an IO error listing no variants; its test checks only `is_err`. | fix here | checked |
| R-F.10 | 1 | `config.rs:8-9` | A TODO contradicts the code (caching exists; model download refused). | fix here | checked |
| R-F.11 | 1 | `validation.rs:19`, `cache/mod.rs:47`, `cache/resolve.rs:9`, `assets.rs:38`, `validation-report/src/main.rs:260` | History narration. | fix here | checked |
| R-F.12 | 1, 3 | `cache/mod.rs:7-13`, `cache/store.rs:9-11,51,79` | Cache docs present UFO fetching as live; `StoreError::UfoParse` is reachable only from test-only code. | fix here | checked |
| R-F.13 | 4, 1 | `cache/{resolve,pinned,store}.rs` | The entry-dir and staging-dir formulas are built four and three times. | fix here | checked |
| R-F.14 | 1 | `lhef/emit.rs:313-365,531-669,825` | A dead `f == 1.0` branch, an inlined helper repeated, a duplicated share-error formula, and a 139-line `Buffer::emit`. | fix here | checked |
| R-F.15 | 4 | `check.rs`, `generate.rs`, `samples.rs`, `observables.rs` | Incoming and outgoing legs are filtered by hand four times. | file (`LheEvent::incoming/outgoing`) | checked |
| R-F.16 | 1 | `vibegraph-cli/src/{integrate,generate,check}.rs` | `IntegrateError` is every command's error type, and `fn err` is defined three times. | fix here (`CliError`) | checked |
| R-F.17 | 1 | `vibegraph-cli/src/network.rs:144-220` | The consent text is spelled twice. | fix here | checked |
| R-F.18 | 1 | `generate.rs:1950-1966` | `hadron_beam_pdg` re-derives the beam policy, with an unreachable arm and contradictory text. | fix here | checked |
| R-F.19 | 3 | `runcard.rs:199-213` | Typed `pub` fields beside a private map holding the same values can desync. | file | suspected |
| R-F.20 | 3 | `artifact.rs:699,702,741-771` | Literal `9` three times (missing sites of the claimed item); `write_to_path` doesn't check `format_version`. | fix here (with the claimed item) | checked |
| R-F.21 | 2 | `validation-report/src/manifest.rs:16` | `Manifest.schema` is parsed and never checked. | fix here | checked |
| R-F.22 | 1 | `manifest.rs:5-7` | The doc says "required, not defaulted"; 11 fields default. | fix here | checked |
| R-F.23 | 4, 1 | `validation-report/src/{cells,main,render,manifest}.rs` | Stringly-typed status, mode, layer and class beside existing serde enums. | fix here | checked |
| R-F.24 | 1 | `vibegraph-cli/tests/cli_first_run.rs:201` | Says proton event generation is not wired. | fix here | checked |
| R-F.25 | 2 | `vibegraph-cli/tests/cli_fixed_energy.rs:102-118` | The CLI fixed-energy σ band is ±45%, the only CLI-path σ check. | fix here | checked; uniqueness suspected |
| R-F.26 | 1 | `vibegraph-cli/src/tui/mod.rs:474-612` | `draw_loop` is 138 lines. | fix here | checked |
| R-F.27 | 4 | `lhef/write.rs`, `validation/samples.rs`, `lhef/parse.rs` | The record-line rule is re-implemented. | file | checked |
| R-F.28 | 1, 4 | `cache/resolve.rs:108-127`; `assets.rs:71-78,166` | An unused `None` branch; `$VIBEGRAPH_UFO_DIR` is read two ways. | file | checked |

**Also seen:**
- `ordering_slot` is duplicated;
- `EventSample` has no length check;
- duplicated decade arithmetic;
- a hard-coded "9 restrict cards" message;
- hand-rolled temp dirs;
- the XMAXUP tolerance has only 2× headroom over rounding.

**Cross-cluster patterns:**
- exit-code-only refusal helpers;
- tests rebuilding the CLI integrand;
- hand-filtered legs;
- hand-rolled temp dirs instead of `tempfile`;
- history narration;
- report-schema constants duplicated with no cross-check.

## Method

| step | share | produced |
|---|---|---|
| backlog check | 10% | — |
| non-vacuity: assert counts, grepping each complaint string against all tests, σ asserts against banked JSON | 35% | R-F.1–3, R-F.5–6, R-F.9, R-F.21, R-F.25 |
| maintainability: length ranking, narration grep, error messages | 35% | R-F.4, R-F.7–8, R-F.10–14, R-F.16–18, R-F.22, R-F.24, R-F.26 |
| visibility | 10% | R-F.19, R-F.20 |
| idiom greps | 10% | R-F.13, R-F.15, R-F.23, R-F.27–28 |

**Dead ends:**
- artifact version arms, which are all round-trip tested;
- the progress field contract, which is pinned;
- `validate_samples_proton`'s returns, which are not soft skips.

## Found

- **About 33 MadGraph line citations in the cluster are unverified**, because
  the submodule was empty. `madgraph-line-citations-predate-pin` lists one.

## Brief corrections

- **`validation-report` is 1.6k lines**; the cluster is about 29.5k.
- **`git branch` is empty on a detached checkout.**
- **The submodule was missing.**
- **The version-arm lead did not hold** for `artifact.rs`.

## Manager check (2026-10-09)

Worktree clean.
- **R-F.2:** `validation-report/src` has 0 `#[test]`.
- **R-F.1:** the only `check-events` complaint asserted in tests is "does not
  balance" (`cli_first_run.rs:280`).
- **R-F.21:** `schema` is a parsed field (`manifest.rs:16`).
