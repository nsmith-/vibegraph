---
type: Session Report
title: "V1b report: V1's dead-code allows replaced, doc links restored"
description: "The 144 dead_code allows became cfg gating (127) or deletions (17); 119 intra-doc links restored; rustdoc builds document private items, with private_intra_doc_links allowed workspace-wide."
status: draft
tags: [hygiene, visibility, docs, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: 22f1955, resource: "https://github.com/nsmith-/vibegraph/commit/22f1955", title: "refactor(lib): drop the test-only helas re-exports"}
  - {id: 0802fe6, resource: "https://github.com/nsmith-/vibegraph/commit/0802fe6", title: "refactor(lib): replace the dead-code allows with cfg gating or deletion"}
  - {id: 371f854, resource: "https://github.com/nsmith-/vibegraph/commit/371f854", title: "docs: restore the intra-doc links and document private items"}
---
The dev agent's report, condensed by the manager. Everything except the
manager check is the agent's own claim.

## Result

Three commits on `hygiene-v1` after `8ed73ec`: `22f1955`, `0802fe6` and
`371f854`. The diff is 73 files, +308 / −468.

`allow(dead_code)` in `vibegraph-lib/src`, counted by `git grep -c`:

| rev | all | bare | as `cfg_attr` | `allow(unused_imports)` |
|---|---|---|---|---|
| `4bdd921` (before V1) | 12 | 4 | 8 | 0 |
| `8ed73ec` (after V1) | 156 | 148 | 8 | 3 |
| `371f854` (after V1b) | 22 | 4 | 18 | 0 |

The four bare allows that remain predate V1.

Disposition of the 144:

| disposition | n |
|---|---|
| `#[cfg(test)]` | 83 |
| `#[cfg(any(test, doc))]` (test-only, but a production doc links it) | 22 |
| deleted, with the docs naming them reworded | 17 |
| `cfg_attr(not(test), allow(dead_code))` (fields production writes and only tests read) | 10 |
| feature gating in four forms (`extended-validation` with or without `test`/`doc`) | 10 |
| `cfg(test)`, narrowed from V1's `any(test, feature)` | 2 |

The test-only re-exports are removed, and the tests import from the defining
modules.

**Doc links.**
- 119 links were restored. Nine were re-pathed and two retargeted; one false
  "used downstream" link was dropped.
- `docs-api` and `scripts/build-docs.sh` pass `--document-private-items`.
- The root `Cargo.toml` sets `[workspace.lints.rustdoc]
  private_intra_doc_links = "allow"`, with a comment saying why. Each restored
  link from a public item to a demoted one raises that lint even under
  `--document-private-items`.
- The book's API-link check reports `checked 111 missing=0`. V1 had broken
  three book links (`Op`, `Ast`, `fvixxx`) that its gate did not check; they
  are fixed.

## Method (lesson data)

1. **The dead-code warning pattern decided each disposition.** It was read
   from four `cargo check --message-format=json` builds: lib, lib with all
   features, test profile, and test profile with all features.
2. **`cfg(doc)`:** test-only items that a production doc links need
   `cfg(any(test, doc))`. Such items are invisible to check and clippy, so
   only a doc build catches a later break in one of them.
3. **Splitting hunks with `git apply --cached --unidiff-zero`** misplaced
   insertions in one intermediate commit. That commit was rebuilt with
   `commit-tree`; the final tree matches the gated tree byte for byte.

## Manager check (2026-10-09)

Re-run in `/home/user/wt/hygiene-v1` at `371f854`:

- **Commits and trailers:** three commits, `Assisted-by` only.
- **Allow counts:** reproduced (22 in total, 4 bare, 0 `unused_imports`).
- **Doc builds:** the lint setting and both `--document-private-items` builds
  are present.
- **Gates:** fmt, both clippy configurations, the hermetic suite and the doc
  build. The results are in the log entry that records the merge.
- **Not re-checked:** that each intermediate commit compiles. That rests on
  the agent's claim.

## Found

1. **Kb concepts that name deleted items:**
   - `process/decay-processes.md:24` (`enumerate_decay`);
   - `amplitudes/repr-layer-geometry-and-axes.md:65,154` (`ColorSinglet`,
     `DiracWf::charge`);
   - `events/generate.md:116` and `hadronic/flavour-groups.md:140`
     (`member_luminosity`);
   - `model/ufo-parsing.md` (propagators "attached per particle").

   Close-out.
2. **16 intra-doc links were already unresolved at `4bdd921`.** A small doc
   session can fix them.
3. **The doc gate should include `build-docs.sh`'s API-link check.** It
   needs mdbook and pixi.
4. **Some production docs state their contracts on test-only APIs:**
   `select_color_flow`, `adapt_parallel_seeded`, and `VegasGrid::new`
   recommending `with_warmup`. Moving each contract onto the production
   function would let those 22 items be plain `cfg(test)`. Candidate fix-here
   findings for the reviews.
5. **Ten production-written, test-read fields stay under `cfg_attr`.**
   Whether to gate their writes is a design call.

## Brief corrections

- **`cfg(test)` alone breaks links from production docs.** It needs
  `cfg(any(test, doc))`.
- **"Must not grow" needed the `private_intra_doc_links` allow** in the root
  `Cargo.toml`, a file outside the brief's list.
- **A field production writes cannot take `#[cfg(test)]`** without gating
  every initializer. Those got `cfg_attr`.
- **Two sites the brief listed as feature-gated** are used only by unit tests.
- **Deleting `enumerate_decay` needed a book reword**
  (`docs/src/guide/03-diagrams.md`).
