---
type: Session Report
title: "L-R1 report: the model-parameter report and machine-readable artifact provenance"
description: "vibegraph parameters and parameters.json / param_card.dat beside diagrams.json, with the record shape the widths report builds on; artifact refusals carry a stable reason and integrate/inspect results carry enumeration_version."
status: draft
tags: [cli, model, param-card, ldjson, artifact, refusal, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-22}
---
Branch `sprintl-reports` off main b2434a0. The two closed items' files are
deleted in the final commit.

## What changed

**Parameter record** (`vibegraph-lib/src/ufo/report.rs`, `vibegraph-cli/src/parameters.rs`).
`describe_parameters(model, sets)` and `default_param_card(model)` are library
functions; every decision is read from the model's own state (restriction
zero/one/merged sets, the binder's value checks, `reweight::check_movable`),
so a reader holds no copy of the rules.

Surface: a model-only subcommand `vibegraph parameters PROC_CARD [--out DIR]`
(reads `import model`; a `generate` line is optional and adds the process
view), and `enumerate` writes the same `parameters.json` and `param_card.dat`
beside `diagrams.json`. Why both: simshop already runs `enumerate` for its
check and gets the process view free; the model-only entry serves a form that
has no process yet, and needs no enumeration. `enumerate` now refuses to
overwrite either file without `--force`, and its ldjson result gains
`parameters_path` and `param_card_path`.

Record shape (documented in `docs/src/cli/overview.md`, "Reporting a model's
parameters"): top-level `schema_version`, `version`, `model`, `process`,
`entering`, `parameters`. Per parameter: `name`, `block`, `code`, `default`,
`class` (`free`, `zero`, `one`, `merged` with `into` and `factor`, `internal`),
`particles` (pdg, name, role, `external_in_process`), `constraints` (`finite`,
`minimum`, `not_auto`, `fixed`, `tied_to`, `leads_group`), `reweight`
(`verdict`, `usable_as_coupling`, `polynomial` with `status` and
`max_degree`), `enters`. A later field (a computed width and its method) is
an added field on the record.

Brief deviations and choices:
- `internal` is a class of the extra rows for the internal mass or width a
  `MASS` or `DECAY` entry addresses (SM `MW`), not of externals; the brief's
  "class ... internal" reads that way.
- `enters` has two values, `couplings` and `masses_widths`; `entering` is the
  `couplings` list the brief asked for. A parameter locked to zero is never
  listed (its vertices were pruned).
- The polynomial entry is per parameter alone: status plus the highest power in
  any subprocess's amplitude.
- **Brief error:** the "vibegraph version that wrote" the artifact is not in
  the artifact (format 24 records `format_version` and `enumeration_version`
  only). Recording it is a shape change that bumps the format and refuses every
  existing artifact, so it was not done. `integrate`'s result carries
  `vibegraph_version` (its own, exact); `inspect` reports `null` and the
  documentation says why. See Found.

**Provenance.** `integrate` and `inspect` results carry `enumeration_version`.
An `artifact` refusal carries `reason`, tagged by `kind`: `enumeration_version`,
`format_version`, `model_digest`, `diagram_set_digest`, `collider_energy`,
`process_reparsed`, `unreadable`, `other`. Every `artifact` refusal has one
(`other` when no specific cause is mapped); other kinds have none. Added
fields under `schema_version` 1; no new kind.

## Oracles

Commands were run in `/home/user/wt/sprintl-reports` with
`CARGO_TARGET_DIR=$PWD/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

| oracle | command | result |
|---|---|---|
| library report tests (7) | `cargo test -p vibegraph-lib --lib ufo::report` | 7 passed |
| CLI parameters tests (4) | `cargo test -p vibegraph --test cli_parameters` | 4 passed |
| provenance tests (5) | `cargo test -p vibegraph --test cli_artifact_provenance` | 5 passed |
| ldjson schema tests | `cargo test -p vibegraph --test cli_output_ldjson` | 10 passed |
| CLI reference chapter | `cargo test -p vibegraph --test cli_reference_docs` | 1 passed |
| workspace suite | `cargo test -j 2 --workspace --no-fail-fast` | rc=0, no failures |
| lints | `cargo fmt --all --check`; `cargo clippy -j 2 --workspace --all-targets -- -D warnings` | both clean (rc=0) |
| bundle | `pixi run --skip-deps kb-lint` | 0 errors, 5 pre-existing warnings |

What each pins, and what it cannot detect:
- `default_card_matches_the_restriction_card`: every external's default equals
  the restriction card's value (MadGraph's one/zero spellings mapped), the
  written card reads back value for value, binding it moves and ignores
  nothing, and every evaluated parameter equals `EvaluatedModel::from_model`
  bit for bit. Blind to a wrong *restriction card* (it compares the model to
  its own card) and to the card's formatting for a consumer other than this
  reader; `{:e}` is shortest round-trip, not MadGraph's `%e`.
- `verdicts_agree_with_the_reweight_resolver`: for every SMEFTsim external,
  the verdict equals what `reweight::resolve` reaches for a move of that
  parameter alone to 2.7. Blind to `external_mass`, which needs a process (the
  CLI test pins `MT` in `e+ e- > t t~`).
- `the_entering_list_matches_a_numeric_perturbation`: for `e+ e- > t t~ NP<=1`
  the listed parameters equal the set that moves some vertex coupling when
  moved alone and every parameter is re-evaluated from scratch (independent of
  the reverse-dependency map). Blind to a parameter whose coefficient cancels
  at the default point, which the symbolic list keeps and the numeric check
  drops; none occurs in this process, since the sets are equal. The CLI test
  pins the 20 names exactly.
- Class tests: SMEFTsim `massless` gives `zero` (`MC`), `free` (`MT`) and the
  DECAY width (`WT`); no shipped restriction produces `one` or `merged`, so a
  test builds a restriction (symlinked model copy, `cW` set to `cG`'s value,
  `cH` to 1) and pins `merged` with `into`/`factor`, `leads_group`, `one`, and
  `fixed`. Blind to MadGraph's own merge detection beyond the loader's.
- Refusal reasons: edited bytes of a real artifact (decompress, patch,
  recompress): enumeration version 2 (`found` 2, `expected` 3) refused by
  `inspect` and `generate`; format version 23; an edited model digest and an
  edited diagram-set digest; a non-zstd file (`unreadable`); a usage refusal has
  no `reason`. The collider-energy reason is checked only at the error
  conversion (`generate.rs` unit test), because the check runs only at proton
  beams with a parton-density set; the end-to-end route is not exercised
  here.
- No σ line can move: the change adds report-only code and added fields; no
  amplitude, colour, coupling, enumeration, sampler or phase-space code is
  touched. `check_movable` is made `pub(crate)` and unchanged. Not shown by a
  run of `pixi run validate`, which is CI's banked job and was not run here.

## Found

- **Writer version is not in the artifact.** Evidence: `IntegrateArtifact`
  (format 24) has `format_version` and `enumeration_version` but no release
  string, so `inspect` cannot report the writer. Closing it needs a field in the
  next format bump (it refuses existing artifacts, so pair it with another
  shape change) and then `vibegraph_version` in `inspect`.
- **Collider-energy reason has no end-to-end test.** Needs a proton artifact
  and the NNPDF set; a validation-host CLI test could patch `sqrt_s_had` and
  expect `reason.kind == "collider_energy"`.
- **`enumerate --reweight-card`** (the item's low-priority part) is not done.
