---
type: Session Report
title: "F-G1 report: amplitude, model and diagram test fixes"
description: "Both claimed items closed and all R-G1 findings in scope fixed in 11 commits: colour oracles fail on a missing bundle, the amplitude tables are checked against the manifest, configuration-amplitude phases and sign patterns are pinned, and smeftsim/toy_models run in the hermetic layer."
status: draft
tags: [hygiene, fix, tests, amplitudes, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 2f94cbc, resource: "https://github.com/nsmith-/vibegraph/commit/2f94cbc", title: "fail the colour oracles on a missing reference bundle"}
  - {id: b9417d1, resource: "https://github.com/nsmith-/vibegraph/commit/b9417d1", title: "keep the known-disagreement arm from absorbing panics"}
  - {id: 1a449f4, resource: "https://github.com/nsmith-/vibegraph/commit/1a449f4", title: "check leading-flow tables against the enumerated diagram count"}
  - {id: 93e9209, resource: "https://github.com/nsmith-/vibegraph/commit/93e9209", title: "check the committed tables against the manifest both ways"}
  - {id: 447e178, resource: "https://github.com/nsmith-/vibegraph/commit/447e178", title: "bind each process under its banked card, over every SM table"}
  - {id: af33a29, resource: "https://github.com/nsmith-/vibegraph/commit/af33a29", title: "state the any-refusal blind spot, guard against an empty census"}
  - {id: d770e7a, resource: "https://github.com/nsmith-/vibegraph/commit/d770e7a", title: "dead skip, / h filtering, stale comments"}
  - {id: ac0fabe, resource: "https://github.com/nsmith-/vibegraph/commit/ac0fabe", title: "move smeftsim and toy_models into the hermetic layer"}
  - {id: c74064b, resource: "https://github.com/nsmith-/vibegraph/commit/c74064b", title: "split measure into its stages"}
  - {id: c6fda8f, resource: "https://github.com/nsmith-/vibegraph/commit/c6fda8f", title: "pin configuration-amplitude phases and sign patterns"}
  - {id: 0401a9a, resource: "https://github.com/nsmith-/vibegraph/commit/0401a9a", title: "needless borrows"}
---
The dev agent's report, condensed by the manager. "Before" runs used
binaries built at `29ddd41`. Input mutations were temporary renames or edits,
then restored.

## Fixed

| finding or item | commit | change | before → after |
|---|---|---|---|
| R-G1.1 | 2f94cbc | The colour oracles fail through `require` on a missing or empty `output/`; the hermetic controls still run; directory walks panic on read errors | With `output/` renamed: "0 passed, ok" before; a failing `reference-bundle` trial or panic after |
| R-G1.7, .25 | 2f94cbc | `COLOUR_ENFORCED_INFO_ROWS` deleted (its row is gated now); `normalise_group`'s doc on the function | — |
| R-G1.2 | b9417d1 | `standalone_jamps`: `measure` returns deviations and `check` judges them; `catch_unwind` gone | With the table removed: "informational, ok" before; panic, FAILED after |
| R-G1.3 | 1a449f4 | `LeadingColorFlows::of` gets the enumerated diagram count, which is asserted against the expected table | An extra expected row: passed before; fails after |
| R-G1.4 | 93e9209 | Coverage trial: tables equal the hermetic amplitudes cells; every `mg_amplitude` row has a table; borrowed tables (`uux_to_mumu` ← `pp_to_ll_qcd0`) are checked against the row's script process; `coupling_oracle` requires its table | Removing `ee_to_ee.json` and relabelling `uux_to_mumu` both pass before and fail after |
| R-G1.6 | 447e178 | `validate_scale_couplings` binds each SM table's own banked card at its points (26 processes, was 14); an empty card fails; non-SM scope explained | Without `output/`: passed on default cards before; an emptied card fails after |
| R-G1.10, .11 | 93e9209, d770e7a | Docs: pruned AMP2 is measured, not asserted; SMEFTsim rows do load | — |
| R-G1.9 | d770e7a | The dead early-return guard deleted (its string never matched, so the test was never silently green) | Same failure before and after |
| R-G1.12 | d770e7a | `e+ e- > mu+ mu- / h`, asserting 2 diagrams; Yukawa override gone | Dropping `/ h` fails |
| R-G1.14 (docs, guards) | af33a29 | Census docs state the any-refusal blind spot; non-empty guards added | Emptied cases: passed before; fails after |
| R-G1.23 | c74064b, 0401a9a | `measure` 630 → 174 lines in four stages | Per-row summary byte-identical on all 48 tables |
| R-G1.26 | ac0fabe | Narration rewritten | — |
| config-amp-phase-and-sign-unpinned | c6fda8f | \|Im(k/G)\| < `LINEAR_REL_TOL` per configuration (suite worst 3.0e-14); `CONFIG_AMP_SIGNS` banks 33 per-process patterns, checked both ways | Rotating configuration amplitude 0 by i fails 43 rows; negating it fails the 32 gated listed rows, where every earlier check passed |
| smeftsim-vendored-checksum-not-hermetic (widened) | ac0fabe | `smeftsim` and `toy_models` lose `required-features`; two hermetic `[[standalone]]` manifest rows added | With `output/`, `validation/pdf/` and `mg5amcnlo` hidden, a plain build passes 13/13 and 4/4; all 49 input files are tracked |

## Beyond the brief

- **`helas::eval::op_census` is compiled without `extended-validation`.**
  The doc-hidden module and its two functions were ungated (no signature
  change), because both moved targets use it. It falls within the user's
  decision to move them.
- **The sign pattern is ±1/|c|, not ±1.** The colour coefficient enters, for
  example +0.5 on `gg_to_h_cp*`; the check divides by \|c\|.

## Not reproduced

- **R-G1.9 in part:** the guard never matched.
- **R-G1.1's "drops two controls"** applies to `color_cf_oracle` only.

## Gate (agent)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1391 passed, 16 ignored. That is +18: 13
  `smeftsim` and 4 `toy_models` tests now hermetic, and 1 coverage trial.
- **`cargo doc`:** no new warnings.
- **Banked:** `color_cf_oracle` 97, `color_flow_tags_oracle` 163,
  `validate_scale_couplings` 1, `ufo` 4.
- **Hermetic, run under the banked profile:** `amplitude_oracle` 49,
  `coupling_oracle` 48, `standalone_jamps` 10.

## Found

1. **`validate_scale_couplings` now reads only committed inputs** but is still
   registered banked. Move it to the hermetic layer.
2. **No test covers the scale path on non-SM rows.** The SMEFTsim rows take
   the reference path, and the toy models have no `aS`.
3. **`CONFIG_AMP_SIGNS` can't see the 16 tables without a per-diagram
   comparison,** including every multi-flow row. A per-flow analogue is
   needed.
4. **Stale kb sites:**
   - `model/smeftsim-topu3l.md:42`
   - `model/restriction-semantics.md:17`
   - `model/coupling-orders.md:19`
   - `validation/process-manifest.md:77` (the coverage trial, `BORROWED_TABLES`)
5. **`test_load_taudecay` always fails** (FFCT2 is a custom Fortran
   operator). File it as a known loader gap.

## Manager check (2026-10-10)

- **Commits and trailers:** 11 commits, `Assisted-by` only. The diff is
  22 files, +891/−344.
- **Registration and manifest:** `smeftsim` and `toy_models` carry no
  `required-features`. The manifest diff adds the two hermetic standalone rows
  and corrects `scale-couplings`' inputs.
- **Gate re-run:** fmt, both clippy configurations, the hermetic suite,
  `color_cf_oracle`, and the collator run against the new manifest. The
  results are in `log.md`.
