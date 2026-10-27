---
type: Sprint Record
title: "generate runs from the integrate artifact plus generate-time inputs"
description: "generate <ARTIFACT> --seed S takes no proc card, run card or PDF option; the run card splits by stage into the artifact (version 13); the model is checked by digest; LHAPDF_DATA_PATH replaces VIBEGRAPH_PDF_DIR; event files byte-identical to the baseline."
closed: 2026-10-22
status: draft
tags: [gridpack-generate, artifact, run-card, generate, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: f2, resource: "sessions/F2-report.md", title: "F2 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: design, resource: "../../pipeline/integrate-generate-split.md", title: "Integrate and generate split by data dependency"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/69", title: "nsmith-/vibegraph#69"}
---
One PR ([nsmith-/vibegraph#69](https://github.com/nsmith-/vibegraph/pull/69),
merged 2026-10-22) closing `generate-rereads-integrate-inputs`, step H1 of
[integrate and generate split by data dependency](../../pipeline/integrate-generate-split.md)
with the user's choices N1–N12 of 2026-10-21. The item was filed when the
sprint opened, from the pdf-from-run-card D1b report. Sessions:
[F1](sessions/F1-report.md) and [F2](sessions/F2-report.md) (feature-dev, Opus),
[HY1](sessions/HY1-report.md) (hygiene-dev, Opus). V1 was folded into CI's banked
layer, since no PDF set was local.

## What was established

- **The principle (user).** The information a workflow step introduces is
  recorded in that step's artifact and never requested again.
- **The run card splits by stage.** Every `FIELD_CLASSES` row has a `Stage`, and
  a test asserts one for every `PARAM_DEFAULTS` name; 14 fields are generate-time
  and `RunCard::split` hands them to `GenerateSettings`.
- **Seeds (N4).** At `integrate`: `--seed`, else a non-zero `iseed`, else the
  default (`iseed = 0` means unset; a negative one is refused). At `generate`,
  `--seed` is required.
- **Artifact version 13.** It records the proc card verbatim and both settings
  halves; versions below 13 are refused, and the v3–v12 readers and version
  guards are deleted. *Manager-verified:* the v13 floor; runcard lib tests (44)
  and `cli_generate` (12) pass.
- **The command line.** `vibegraph generate <ARTIFACT> --seed S`; a proc card,
  `--run-card`, `--pdf-set` or `--pdf-dir` is an argument error. The recorded
  proc card must re-parse to the recorded process, and the model is checked by
  digest, the interned SM included. The generation driver moved into
  `vibegraph-lib` (`vibegraph::generate::generate`), which never takes a
  `RunCard`. *Manager-verified:* `generate --help` shows `<ARTIFACT>` with
  `--seed` required; `cli_generate` (16), `cli_pdf_cache` (7), `cli_ufo_model`
  (2) pass.
- **PDF location (C7, C8).** `LHAPDF_DATA_PATH`, split on `:`, else `LHAPATH`,
  as LHAPDF 6.5.3 reads them; `VIBEGRAPH_PDF_DIR` is retired with one warning,
  and the cwd-relative `validation/pdf` fallback is gone.
- **Equivalence.** Old and new builds wrote byte-identical event files and
  identical σ/Γ on e+e- → μμ (three strategy and normalisation modes),
  e+e- → uūg and t → b e+ ν (F1, asserted). The proton generate and reweight
  paths ran only in CI's banked layer, which passed on F1 (`a8dcb3e`) and F2
  (`c4acf0a`) (*manager-verified* from CI). All 21 F2 mutations were caught
  (F2, asserted).

## Backlog

- Deleted: `generate-rereads-integrate-inputs`.
- Drafted for filing at close-out: `demo-gifs-show-the-old-command-line`
  (F2 Found 1), `pdf-tiers-need-a-home-directory` (F2 Found 3),
  `model-changed-message-wrong-for-interned-sm` (HY1 Found 1),
  `library-generate-accepts-zero-events` (HY1 Found 2),
  `interned-model-predicate-written-three-times` (HY1 Found 3),
  `absent-run-escapes-spelled-per-gate` (HY1 Found 4),
  `cli-tests-define-pdf-dir-per-file` (HY1 Found 5).
- Proposed edits, not new items (HY1 Found 6): stale sites in
  `lhef-idwtup-minus4-buffers-whole-sample`,
  `stochastic-rounding-refuses-mixed-multiplicity`, `no-lazy-event-iterator`,
  `lib-tests-hand-roll-temp-dirs` and `rustdoc-warnings-ungated`.
- Not filed:
  - F1 Found 1 and F2 Found 6 (the proton path unexercised locally): CI's
    banked layer ran it on both sessions' commits;
  - F1 Found 2, 3 (library messages naming CLI flags; the `nevents` message):
    fixed by F2;
  - F2 Found 2 (`cli_onshell_veto` passing `--pdf-dir`): moot since the
    PDF-from-run-card PR removed the flag and migrated the tests;
  - F2 Found 4, 5 (the recorded member untested; the checksum test): delivered
    by that PR (`pdf::select` tests a member-1 lhaid and the checksum refusal);
  - HY1 Found 7 (the warn-once guard untested): unobservable until a command
    resolves two sets; nothing to do before that change.

## Process notes

The design and its decision log were written on the pdf-from-run-card branch
(D1, D1b, D1c) and arrived here; the PR split (H1 here, H2 on #60) was D1b's
proposal. HY1 corrected its brief: the CLI package is `vibegraph`, not
`vibegraph-cli`.
