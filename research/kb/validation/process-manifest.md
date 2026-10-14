---
type: Design
title: "validation/manifest.toml: the per-process source of truth"
description: "One committed file names every reference process, its script and per-category tier and mode with rationale; generators, gates and the collator read it and must match its rows."
status: draft
tags: [validation, manifest, layers, report, reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: n25-41, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L198-L235", title: "Note 25 §4.1, the scripts and their rationale headers"}
  - {id: n25-43, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L243-L253", title: "Note 25 §4.3, gating mechanisms"}
  - {id: n25-51, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L269-L282", title: "Note 25 §5.1, the manifest"}
  - {id: n25-book, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L720-L739", title: "Note 25 §10, bookkeeping the sweep turned up"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml"}
  - {id: manifest-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/common/manifest.rs", title: "vibegraph-lib/tests/common/manifest.rs"}
---
# `validation/manifest.toml`: the per-process source of truth

A process is added to the validation suite once, in
`validation/manifest.toml`, rather than in the five or six places a reference
used to touch (scripts, amplitude builders, reference JSON, Rust plan
tables)[^n25-51]. The file's header documents its own schema; this concept says
what the file is for and what holds it honest.

## What it holds

- **`[refdata]`**: the pin of the fetched reference bundle (`version`, `archive`,
  `url`, `sha256`, `size_bytes`, `unpacks_to`, `published`) and one comment per
  bundle cut saying what changed. Read the current version from the file; prose
  that quotes one goes stale with every cut ([refdata-bundle](refdata-bundle.md)).
- **`[[process]]` rows**, one per reference process: `key`, `process`, `script`
  (the `.mg5` under `validation/madgraph/scripts/`), `class` (single- or
  multi-channel), `n_final`, `rationale`, `artifacts`, optionally `mg_amplitude`
  (the process card, PDGs, RAMBO grid and seed `gen_amplitude.py` compiles
  `MATRIX1` against, present only on the row that owns the generation), `model`
  and `restrict` for a non-SM row, `status = "planned"` for a row whose reference
  run does not exist yet, and `bundled = false` for a row banked locally but not
  yet in the pinned bundle.
- **`[process.categories]`**: one cell per category (`diagrams`, `amplitudes`,
  `integrals`, `samples`) with a `tier` (`hermetic`, `banked`, `long`, `blocked`,
  `covered-by`, `uncovered`), a `mode` (`gate` or `info`) on measured tiers, and a
  curated `note`; `blocked` carries a `blocker`, `covered-by` the covering `rows`,
  and an `amplitudes` cell may record `factorized`.
- **`[[standalone]]` entries** for gates that are not per-process cells: `key`,
  `layer`, `targets`, `inputs`, `rationale`, and for a non-Rust driver the pixi
  `task`, its `environment` and the `row` it writes. Two are hermetic:
  `smeftsim-model` (`tests/smeftsim.rs`) and `toy-models` (`tests/toy_models.rs`)
  run the vendored SMEFTsim and the authored toy UFO models against committed
  inputs only, so a drifted model fails the default `cargo test`.
- **The seed policy** for MadEvent references, in the header
  ([madevent-reference-seed-policy](madevent-reference-seed-policy.md)).

## Rules the file encodes

- **A tier is where the gate is registered**, not the weakest layer it could run
  in. Layer membership is decided by registration alone: `required-features =
  ["extended-validation"]` for banked integration tests, `#[ignore]` plus a pixi
  task for the oracle layer, and `#[cfg(feature = "extended-validation")]` only for
  the handful of `--lib` unit tests that need banked inputs[^n25-43]. The layers
  are [validation-layers](validation-layers.md).
- **The banked layer takes no runtime skips.** A missing input fails naming
  itself through `vibegraph::validation::require`; a row not mentioned as
  unbundled is not exempt from anything.
- **`rationale` says why this exact process**, including what it is not used for;
  the full set of reasons is [reference-row-rationale](reference-row-rationale.md).
- **A cell note states current truth.** When a blocker lands, the cells naming
  it are re-worded to what actually blocks them, measured (a `kt-clustering`
  blocker that had landed became `mg-internal-pdf` on eight cells, and a "refuses
  to load" note became the gate's measurement). The collator prefers the curated
  note over a measurement's own, so a stale string a gate writes does not reach
  the report but should still be removed at its source[^n25-book].

## Who reads it, and what holds them to it

| reader | what it asserts against the manifest |
|---|---|
| reference generators (`build.sh`, `gen_amplitude.py`, the extractors) | which processes to build and which tables to write |
| `vibegraph-lib/tests/common/manifest.rs` | sets the gates draw from: `unbundled_rows()`, `hermetic_diagram_rows()` (which `diagrams.json` must cover exactly), `hermetic_amplitude_rows()`, each row's model and restrict card |
| `tests/amplitude_oracle.rs` | the `every_hermetic_amplitudes_row_is_covered` trial: one committed amplitude table for every row whose `amplitudes` cell is hermetic and none for any other, a table for every `mg_amplitude` declaration, and the tables without a declaration exactly `BORROWED_TABLES` (`uux_to_mumu`, read from `pp_to_ll_qcd0`'s module), so a deleted table or a row promoted without one fails |
| per-file inventories | each sweeping gate declares every banked run it meets (`validate_scales`' classes, `validate_alphas`' `SCALUP_IS_THE_RENORMALISATION_SCALE` and `GRID_ALPHA_S_RUNS`), so a new row appears as a failure until classified ([scale-replay-gate](scale-replay-gate.md)) |
| `tests/smeftsim.rs` | `GATED_ROWS` equal to the manifest's gated SMEFTsim rows; each table's `process` equal to `mg_amplitude.process` |
| `tests/toy_models.rs` | `GATED_ROWS` equal to the manifest's gated toy-model rows; each row's process equal to its banked table's |
| the collator | the rendered row set and cell set: "the measured cells are exactly the cells the manifest declares" ([validation-report](validation-report.md)) |
| `validation/madgraph/README.md` | an index over the manifest, not a hand-kept process list |

The collator's assertion is the manifest-level analogue of refusing silent
skips: a row that vanished, or a cell measured that the manifest does not
declare, fails rather than renders.

## Caveats

- `bundled = false` and `status = "planned"` are transient: they exist between a
  locally banked run and the next bundle, so a test must not depend on some row
  carrying them ([oracle-blind-spots-and-non-vacuity](oracle-blind-spots-and-non-vacuity.md)).
- The `blocked` tier has no cell using it
  ([manifest-blocked-tier-unused](../backlog/hygiene/manifest-blocked-tier-unused.md)).
- A row whose reference is a committed summary rather than a banked run (decay
  widths, decay-chain events) is deliberately not a `[[process]]` row: as one, its
  run would land in `output/` where the sweeping gates expect an inventory class
  for it ([refdata-banking-procedure](refdata-banking-procedure.md)).
- One compiled amplitude module can serve several rows (`pp_to_ll_qcd0` generates
  the table `uux_to_mumu` also reads); the generator must build a probe for every
  row it emits, not only the owners.

[^n25-51]: Note 25 §4.2 and §5.1.
[^n25-43]: Note 25 §4.3 and the manifest header's "Layers".
[^n25-book]: Note 25 §10, "Bookkeeping the sweep turned up"; note 28 Z.4 for the re-worded blockers.
