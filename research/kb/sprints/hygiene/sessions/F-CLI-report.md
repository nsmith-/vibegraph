---
type: Session Report
title: "F-CLI report: CLI and validation-report fixes"
description: "All 16 R-F CLI/report findings and the no-network item fixed in 9 commits with 32 caught mutations: typed collator vocabularies with a checked manifest schema, a unit test per check-events complaint, refusals that must name their cause, and CLI σ checks bounded by the runs' own errors."
status: draft
tags: [hygiene, fix, cli, validation-report, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 658d465, resource: "https://github.com/nsmith-/vibegraph/commit/658d465", title: "type the collator's closed vocabularies, check the manifest schema"}
  - {id: 56dcb88, resource: "https://github.com/nsmith-/vibegraph/commit/56dcb88", title: "one CliError for every command"}
  - {id: c921b0e, resource: "https://github.com/nsmith-/vibegraph/commit/c921b0e", title: "a unit test for every check-events complaint"}
  - {id: 8a8c4a4, resource: "https://github.com/nsmith-/vibegraph/commit/8a8c4a4", title: "name the file or flag behind an unreadable card or an empty request"}
  - {id: 34388d0, resource: "https://github.com/nsmith-/vibegraph/commit/34388d0", title: "VIBEGRAPH_NO_NETWORK refuses at any value"}
  - {id: 5eaff93, resource: "https://github.com/nsmith-/vibegraph/commit/5eaff93", title: "proton IDBMUP is a constant"}
  - {id: e65a707, resource: "https://github.com/nsmith-/vibegraph/commit/e65a707", title: "split draw_loop"}
  - {id: 55104da, resource: "https://github.com/nsmith-/vibegraph/commit/55104da", title: "a generate refusal must say what it refused"}
  - {id: 296875c, resource: "https://github.com/nsmith-/vibegraph/commit/296875c", title: "bound the CLI's sigma checks by the runs' own errors"}
---
The dev agent's report, condensed by the manager. Mutation logs are in the
scratchpad under `fcli/`. All 32 mutations were caught, each by an assertion
panic.

## Fixed

| finding or item | commit | change | evidence |
|---|---|---|---|
| R-F.23 | 658d465 | Collator `status`, `mode`, `class` and `layer` are serde enums over the writers' exact vocabulary | Old and new collators render byte-identical `report.md`/`report.json` on 157 real rows, and on a copy with two injected fails. Two alias mutations caught |
| R-F.21 | 658d465 | `MANIFEST_SCHEMA = 1` is checked | A schema-2 manifest is refused (the old binary rendered it). Removing the check is caught |
| R-F.22, R-F.11 (report) | 658d465 | The manifest doc states which fields are required and which optional; history phrase removed | — |
| R-F.16 | 56dcb88 | One `CliError` and one `err` | — |
| R-F.1 | c921b0e | check-events checks gathered in `complaints()`, with 8 unit tests damaging a passing fixture | 18 mutations, one per check and exemption, all caught |
| R-F.7 | 8a8c4a4 | Proc and run-card errors name the path; "no events requested" says which source gave 0 | Three message reversions caught |
| R-F.8, R-F.11 (assets) | 8a8c4a4 | Not-found errors say whether a flag or an env var named the directory | Caught |
| no-network-variable-read-two-ways, R-F.17 | 34388d0 | `fetch_common.sh` refuses when the variable is set at all, as the CLI does; the rule is stated in five places; the stream prompt prints `Download::notice()` | A three-value test (1, 0, empty) over both readers fails on the old script rule. Two more mutations caught |
| R-F.18, R-F.24 | 5eaff93 | `PROTON_BEAM_PDG` constant; stale test doc rewritten | — |
| R-F.26 | e65a707 | `draw_loop` 138 → 67 lines | — |
| R-F.3 | 55104da | `expect_refusal` asserts no panic and that stderr names the cause | A panicking refusal is caught. **The test itself had vacuous cases:** clap rejected `--max-truncation -0.01` and `--scan-points -5` as unknown arguments, so the value parsers never ran. They now pass as `--flag=value` |
| R-F.5 | 296875c | The `rel < 1%` OR arm dropped; `delta < 3·combined` | A 0.5% unit shift fails at 5.5σ and 5.8σ, and the old check passed it |
| R-F.25 | 296875c | `\|pull\| < 3` against the committed `ee_to_ttx` reference (0.54866 ± 0.00018) | Fixed pull +1.10; the 0.5% mutation +5.34; 0.552 was inside the old band |
| R-F.6 | 296875c | The replay uses the artifact's banked maps, with a per-channel 4σ check and an efficiency bound: replay error < 2× the rescaled banked error | **σ cannot see a wrong-map replay on single-channel Drell–Yan.** A wrong map passes the σ check but reads 23–29× on the efficiency bound; the fixed code reads 1.00–1.07× |

## Gate (agent, at 296875c)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1373 passed, 16 ignored (+19 tests).
- **`cargo doc`, per package:** `vibegraph` 1 warning (pre-existing),
  `validation-report` 0.
- **Banked CLI targets, all pass:**
  - `cli_integrate` 4
  - `cli_generate_proton` 5
  - `cli_decay_chain` 1
  - `cli_decay_chain_events` 7
  - `cli_reweight_proton` 2
  - `cli_pdf_cache_fetched` 2
  - `validate_samples_proton` 11 (801 s)

## Found

1. **`integrate_refuses_overwrite_without_force` checks only the exit code**
   (`cli_integrate.rs:271`).
2. **Negative `--max-truncation` and `--scan-points` values get clap's
   misleading "unexpected argument" error.** Set `allow_negative_numbers`.
3. **`RunCard::parse_file` names the path only on I/O errors.**
4. **`fixed_energy_nbody_finite_sigma` checks only that σ is finite and
   positive.** `ee_to_tatah` has a reference at ebeam 250.
5. **The efficiency bound's 2× headroom rests on two fixed-seed readings.**
6. **A pre-existing unresolved link at `vibegraph-cli/src/assets.rs:5`.**
7. **Stale kb:** `tooling/network-consent.md:114-118` (the old `=1` rule).

## Brief corrections

- **R-F.6 needed an error-based oracle,** not a tighter σ bound.
- **`cargo doc --workspace` always warns on the lib/bin name collision.**
  Per-package runs are the meaningful check.
- **R-F.3 has 8 call sites, not 9,** and two of the loop cases were vacuous.

## Manager check (2026-10-10)

- **Commits and trailers:** nine commits, `Assisted-by` only. The diff is
  21 files, +1116/−310.
- **Gate re-run:** fmt, both clippy configurations, the hermetic suite and
  `cli_integrate`. The results are in `log.md`.
