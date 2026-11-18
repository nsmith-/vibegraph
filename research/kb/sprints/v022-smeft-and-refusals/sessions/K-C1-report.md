---
type: Session Report
title: "K-C1 report: run-card off sentinels, the resolved-card report and the classification table"
description: "Cited off-sentinel rules for the rank, missing-energy and HT parameters, the vibegraph run-card command reporting a resolved card and a generated per-release table, and the bit-for-bit and campaign-card checks."
status: draft
tags: [run-card, cuts, cli, refusal, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-18}
---
Branch `sprintk-runcard` off main 0340a89. The four closed items' files are
deleted in the final commit (three links in other pages were repaired by it).

## What changed

**Off sentinels** (`vibegraph-lib/src/cuts.rs`). `unimplemented_cut_refusal` now
holds the per-parameter rule `detect_unimplemented` loops over. Pinned mg5amcnlo
(`research/refs/mg5amcnlo`, `Template/LO/SubProcesses/`), one citation set per
rule:

| parameters | inert when | guard |
|---|---|---|
| `ptj1min`–`ptj4min` | ≤ 0 | `cuts.f:817-820` requires a jet only at `ptjNmin.gt.0`; `cuts.f:855` fails a jet only at `ptjet(i).lt.ptjmin4(i)`; `setcuts.f:541-562` folds each into `max(…, 0d0)` |
| `ptl1min`–`ptl4min` | ≤ 0 | `cuts.f:920` and `:934-937` test `ptlNmin.gt.0`; `cuts.f:963` fails a lepton only at `ptlepton(i).lt.ptlmin4(i)`; `setcuts.f:646-666` clamps with `0d0` |
| `misset` | ≤ 0 | `cuts.f:371` fails at `pt(miss).lt.misset`; `setcuts.f:776` reports it only at `.gt.0` |
| `ptheavy` | ≤ 0 | `cuts.f:390` and `setcuts.f:781` guard `.gt.0d0` |
| `ht2max`–`ht4max` | < 0, or ≥ `ebeam1 + ebeam2` | `cuts.f:883-884`: `htjmax4(i).ge.0d0.and.htj.gt.htjmax4(i)`; `setcuts.f:502-504` only copies them |

The template's off value for `ht2max`–`ht4max` is `-1` (the LO default), and
the older templates write 100000. 100000 is not "off" by the guard: it is a cut
no collider reaches. The sum of the `n` hardest jets' pT is at most their
energy, at most `sqrt(ŝ)`, at most `ebeam1 + ebeam2`, so a bound at or above
that sum excludes nothing (13 TeV: 13000; the bound is `ebeam1 + ebeam2`, not
half of it as for the single-object maxima). `ht2max = 0` through 12999.9 at
6500 GeV beams stays refused by name. Values `ht2min`–`ht4min`, the rank
maxima and any positive minimum are unchanged: refused by name.

**Resolved-card report and table**
(`vibegraph-lib/src/runcard/report.rs`, `vibegraph-cli/src/run_card.rs`,
`vibegraph run-card`). One implementation serves both items.
- `vibegraph run-card RUN_CARD [--proc-card PROC]` reports every key: MadGraph
  default, the card's value and line, the resolved value, `moved_by_card`,
  `moved_by_resolution`, class, status, note and the refusal `integrate` emits
  for that key; the card's first run-card refusal (`parse`, `cuts` or `split`);
  `unknown_keys` (unknown names, malformed values, and `$`-led template
  placeholders the parser skips); `warnings`; and, given a process,
  `process_defaults` (the difference between the process's default card and the
  LO template, with `overridden_by_card`). A refused card is reported, not
  failed. With only `--proc-card` the report is of the default card.
- `vibegraph run-card --table` prints the table of every known parameter
  (`honoured`/`ignored`/`refused`, `refused_values`, a one-line `note`),
  generated from `FIELD_CLASSES`, `PARAM_DEFAULTS` and the cut lists.
- Shipping: command output only. The binary is distributed bare, so a release
  asset could drift from the binary a consumer holds; the table carries
  `version`. A release asset can be produced by redirecting the command if a
  consumer wants one.
- `matching::resolve` now also returns the notices it logs; `RunCard`
  resolution has a variant that skips the `IgnoredPhysics` refusal so a report
  can read the rest of a card that carries one.
- `serde_json` moved from the lib's dev-dependencies to its dependencies.
- Docs: `docs/src/cli/overview.md` (new section), `reference.md` regenerated,
  `scripts/gen-cli-docs.sh` and `cli_reference_docs.rs` list the command.

## Oracles

Gate commands and results are in the final section. Bit for bit, main
0340a89 (built in a separate target, deleted afterwards) against this branch,
`integrate --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`, default
cuts, `LHAPDF_DATA_PATH=validation/pdf`:

| card | σ main | σ branch | grid.bin.zst sha256 (main = branch) |
|---|---|---|---|
| `config_share_ee_mumu_run_card.dat`, `e+ e- > mu+ mu-` (fixed beams) | 419.72 ± 0.75 fb | 419.72 ± 0.75 fb | 934120b117f2e527… |
| `dy13_default_run_card.dat`, `p p > e+ e-` (hadronic) | 935 ± 11 pb | 935 ± 11 pb | b5321668ddc818289629… |
| `pp_to_llj_mlm_run_card.dat`, `p p > e+ e- @0; add process p p > e+ e- j @1` (MLM) | 932 ± 12 pb | 932 ± 12 pb | 1c274c427d46a9ff288086f3… |

`cmp` of the three grid files: identical.

Campaign cards: no cvmfs, so one family card each was rebuilt from
`cms-sw/genproductions` `bin/MadGraph5_aMCatNLO/cards/examples/wplustest_4f_LO/wplustest_4f_LO_run_card.dat`
(raw.githubusercontent.com) by writing the old template's off value over the
line, process `p p > w+ j`:
- A: `ht2max`, `ht3max`, `ht4max` = 100000;
- B: `misset`, `ptheavy`, `ptl1min`–`ptl4min` = -1;
- C: `ptj1min`–`ptj4min` = -1.

Main refuses each (`ht2max`, `misset`, `ptj1min` by name); this branch's
`integrate --check` prints `check ok: p p > w+ j` for each, and `run-card`
reports `refusal: null`. A short integrate of the unedited template and of A, B
and C at seed 3 (`--neval 5000 --niter 3`) gives σ = 44.30 ± 0.44 nb for all
four, the same value, as an inert value must. These are reconstructions of the
families, not the 31 campaign cards; the real older templates differ from the
current one in other lines.

## What each test cannot detect

- The per-rule tests (`cuts.rs`) check that the compiled cuts equal the default
  filter for the cited values and refuse the values that would cut. They
  cannot see a guard in `cuts.f` that I did not read: the citations are the
  evidence. Two residuals are deliberate and named in the code comment.
  `setcuts.f:675` squares `misset` into the lower limit on ŝ of a process with
  a charged lepton, so `misset = -1` makes MadGraph exclude ŝ < 1 GeV² there,
  a region every default lepton-pT or pair-mass cut already empties and
  vibegraph does not exclude. And `smin` for `ht2max`–`ht4max` is not read.
- The report test compares the report with `RunCard::parse` and
  `detect_unimplemented` over the 21 committed validation cards, so a shared
  parser error is invisible; `integrate --check` parity for refused cards
  (CLI test) covers the run-card stages only, not refusals that need the
  process.
- The table test fails when a name in a cut list or in the value-refusal list
  has no entry, or a field class has none; `VALUE_REFUSALS` (the consumed
  fields that refuse some values) is hand-kept, and a new `RunCardError` for a
  consumed field is not caught by any test.
- The bit-for-bit rows cover default cuts only; sentinel inertness is shown by
  equal σ at one seed on one process, not by a sweep.

## Gates

With `CARGO_TARGET_DIR` in the worktree and the lean cargo variables:

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean (one
  `len_zero` lint in the new CLI test, fixed, rerun clean).
- `cargo test --workspace`: 66 test binaries ok, none failed (log
  `kc1-test.log`); `cli_run_card` rerun after the lint fix: 7 passed.
- `pixi run --skip-deps kb-lint`: 0 errors, 5 pre-existing warnings.
- The `banked` job is CI's; `validate.sh` was not run.

## Found

- **Consumed-field value refusals are hand-listed.** `VALUE_REFUSALS` in
  `runcard/report.rs` names which consumed parameters refuse some values; a new
  refusal for a consumed field does not fail any test. Closing it: attach the
  refusal text to the field class (or to the error variant) so the table reads
  it.
- **The parser silently skips `$`-led lines** such as
  `$DEFAULT_PDF_SETS = lhaid`; `integrate` runs at the default `lhaid`
  without saying so. The report lists them as `placeholder`; the run does not.
  Closing it: an `integrate` warning naming the skipped parameter.
