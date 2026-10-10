---
type: Session Report
title: "F-F report: library I/O fixes"
description: "Both claimed items closed and seven R-F library findings fixed in 5 commits: a named unknown-SM-variant error, the test-only UFO fetch deleted, cache paths built once, Buffer::emit split, artifact versions named and write-checked, run-card Opaque defaults matched to banner.py."
status: draft
tags: [hygiene, fix, io, runcard, artifact, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 4bdecb6, resource: "https://github.com/nsmith-/vibegraph/commit/4bdecb6", title: "refuse an unknown SM restrict variant by name"}
  - {id: 7a69498, resource: "https://github.com/nsmith-/vibegraph/commit/7a69498", title: "drop the test-only UFO fetch path, build entry dirs once"}
  - {id: f5525b9, resource: "https://github.com/nsmith-/vibegraph/commit/f5525b9", title: "split Buffer::emit and share its error formulas"}
  - {id: 478e96d, resource: "https://github.com/nsmith-/vibegraph/commit/478e96d", title: "name each current-schema version by its own constant"}
  - {id: 212df29, resource: "https://github.com/nsmith-/vibegraph/commit/212df29", title: "store MadGraph's list and dict defaults as its card spells them"}
---
The dev agent's report, condensed by the manager. Mutation logs are in the
scratchpad under `ff/`: six mutations in one batched build, all failing.

## Fixed

| finding or item | commit | change | evidence |
|---|---|---|---|
| R-F.9 | 4bdecb6 | `UfoError::UnknownSmRestrict { variant, known }` lists every valid variant; the test checks the variant and message | Restoring the old IO error fails |
| R-F.10 | 4bdecb6 | The `config.rs` TODO becomes a statement of behaviour | — |
| R-F.11 (lib) | 7a69498 | History narration removed in `validation.rs` and `cache/` | — |
| R-F.12 | 7a69498 | `cache_ufo_model` (test- and doc-only) and `StoreError::UfoParse` deleted with their two tests; cache docs say UFO models are resolved, never fetched | — |
| R-F.13 | 7a69498 | `AssetKind::entry_dir` and `store::staging_dir` build each path once | — |
| R-F.14 | f5525b9 | `Buffer::emit` split into three helpers; `share_variance` and `estimate_error_from_sums` replace the duplicated formulas; the exact-duplicate `f == 1.0` branch removed | Banked `validate_lhef` passes |
| artifact-reader-arm-names-format-version, R-F.20 | 478e96d | `MAP_CHOICES_VERSION = 9` replaces the literal 9; the reader arm names each version's constant; `write_to_path` refuses unreadable versions | The item's hazard (`FORMAT_VERSION = 12` with the old arm) fails the new test; disabling the write check fails |
| runcard-opaque-defaults-unverified | 212df29 | `Def::O` stores MadGraph's card-writer spelling of each default. `defaults_match_banner_py_dump` now compares every Opaque payload; the mismatch set is asserted empty over 14 opaque parameters | Reverting `mxx_only_part_antipart` to `""` fails three tests |

## Decision on R-F.12 (human:nsmith-, 2026-10-10)

**Keep the deletion of `cache_ufo_model` and record it.** The draft concept
`tooling/asset-resolution.md` had recorded keeping it as plumbing ready for a
real UFO source. FeynRules has no name-to-URL index, so nothing could call it.

If a user interface to fetch a model by URL is added later, restore it from
7a69498's parent. The design to restore: fetch through `Fetch`, extract to
staging, pin the model's `model_digest` (not the archive hash) so a
repackaged model pins identically and an unparseable archive is never
published, then atomic publish. Close-out records this in
`asset-resolution.md`.

## Gate (agent, at 212df29)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1354 passed, 16 ignored (+3 tests, −2
  tests, 2 renamed).
- **`cargo doc`:** 8 lib and 1 bin warnings, unchanged.
- **Banked:** `validate_lhef` 3, `cli_generate_proton` 5 (2 ignored),
  `cli_pdf_cache_fetched` 2.

## Found

1. **`tooling/asset-resolution.md:141-144` contradicts the deletion.** It is
   updated at close-out, per the decision above.
2. **The CLI stale-artifact tests stamp old version numbers onto
   current-schema bytes.** They decode only because bincode ignores trailing
   bytes. Build real old-schema fixtures, then tighten the write check to
   `>= max(MAP_CHOICES_VERSION, version_for(channels))`. File it.
3. **Run-card parser bug:** `split_line` uses `split_once('=')`, where MadGraph
   reads `rsplit('=', 1)` (`banner.py:2902`). So the banked cards'
   `systematics_arguments` line (`['--mur=0.5,1,2', …] = systematics_arguments`)
   is silently skipped, and a typo in that name would not be caught. A
   one-line fix with a test. File it.
4. **`<MGRunCard>` now writes three more lines.** `pdgs_for_merging_cut` is
   `default_setup`'s list, which MadEvent replaces per process
   (`banner.py:4782`). The ignored oracle
   `validate_mlm_dumps::mg_run_card_matches_madevents_banner` could flag it on
   rows whose card leaves it unset. Run it when matched runs are available.
5. **Opaque list payloads compare as raw text.** Enforcement over list fields
   needs list-level normalisation.
6. **Stale kb sites:** `run-card/run-card-parser-and-defaults.md:68-69` and
   `field-classification.md:144,158`.

## Brief corrections

- **`closes_when`'s "under parse_value normalisation"** doesn't fit, because
  `parse_value` doesn't normalise Python reprs.
- **70 banked cards carry the `mxx` line,** not 35.
- **R-F.20 has no single right fix** (Found 2).

## Manager check (2026-10-10)

- **Commits and trailers:** five commits, `Assisted-by` only.
- **Gate re-run at 212df29:** fmt passes; clippy exits 0 in both
  configurations; `cargo test --workspace` gives 35 suites, 1354 passed,
  0 failed, 16 ignored; the banked `validate_lhef` passes 3.
