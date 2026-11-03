---
type: Sprint Record
title: "The run card picks the PDF set and member"
description: "lhaid resolves set and member through pdfsets.index over four tiers; built-in pdlabels are refused on proton beams; artifact v14 records per-file SHA-256; pp_to_ll rebanked on lhapdf/247000 (+0.008 %); refdata-11 pinned."
closed: 2026-10-26
status: draft
tags: [pdf, lhapdf, run-card, artifact, cvmfs, refdata, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: d1, resource: "sessions/D1-report.md", title: "D1 report"}
  - {id: d1b, resource: "sessions/D1b-report.md", title: "D1b report"}
  - {id: d1c, resource: "sessions/D1c-report.md", title: "D1c report"}
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: f2, resource: "sessions/F2-report.md", title: "F2 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/60", title: "nsmith-/vibegraph#60"}
---
One PR ([nsmith-/vibegraph#60](https://github.com/nsmith-/vibegraph/pull/60),
merged 2026-10-26, the last of sprint D) closing `pdf-set-not-taken-from-run-card`
and pinning reference bundle refdata-11. Sessions: [D1](sessions/D1-report.md),
[D1b](sessions/D1b-report.md) and [D1c](sessions/D1c-report.md) (design, Opus),
[F1](sessions/F1-report.md) and [F2](sessions/F2-report.md) (feature-dev, Opus).
The design ([integrate and generate split](../../pipeline/integrate-generate-split.md),
[run inputs in the artifact](../../pipeline/run-inputs-in-the-artifact.md)) split
the work into H1 (gridpack generate, #69) and H2 (this PR). The M3 rebanked two
rows and cut the bundle.

## What was established

- **The user's principle and choices.** Information a step introduces lives in
  that step's artifact. C1–C12 (2026-10-21): per-file SHA-256 of what is read;
  every built-in `pdlabel` refused on proton beams; `LHAPDF_DATA_PATH` (else
  `LHAPATH`) replaces `VIBEGRAPH_PDF_DIR`; no `validation/pdf` fallback; refuse on
  a checksum mismatch.
- **The CVMFS layout, measured on the CERN host.** `…/lhapdfsets/current` holds
  each set unpacked and as `.tar.gz`, plus `pdfsets.index`; 325300 is 44.9 MB,
  247000 27.6 MB and identical to the pinned copy. One `.info` has
  `SetIndex: 325300 ` with a trailing space; the parser trims it, pinned by a
  test.
- **What landed.** `integrate` takes no `--pdf-set`/`--pdf-dir`; `pdlabel =
  lhapdf` with `lhaid` resolves set and member (lhaid − SetIndex) through
  `pdfsets.index`, looking in `$LHAPDF_DATA_PATH`, CVMFS, the cache, then a
  consented download. Artifact v14 records lhaid, set, member and a SHA-256 per
  parsed file, and `generate` refuses a set whose files differ. Generated LHE
  files carry `<vibegraphInputs>` (set, member, tier, every input digest including
  the grid's). The default card is `lhapdf`/247000. *Manager-verified (F1):*
  version 14; `pdf::select`, the built-in-label refusal and `cli_pdf_cache` pass.
- **A CI failure with two causes.** MadGraph writes fixed-energy cards with
  `none = pdlabel1/2` and `230000 = lhaid` but no `pdlabel` line, so a test helper
  reading `pdlabel`/`lhaid` directly asked for set 230000; it now follows
  `RunCard::pdf_choice`. Behind it, the D1c correction was wrong: `pp_to_ll` and
  `pp_to_ll_qcd0` are banked proton `nn23lo1` runs in the αs and scales gates.
  F2 stopped and reported; the user chose to re-card and rebank them, and to add
  `--no-fail-fast` to the banked job (`validate.sh`), since cargo had stopped at
  `validate_alphas` and hidden `validate_scales`' failure. `uu_tt` needs no
  rebank: every row reading its card overrides `lpp = 0`.
- **PDFSUP.** On fixed-energy cards it follows MadGraph's `get_pdf_id` of the
  written label, or of the template's `nn23lo1` (247000) without one
  (`banner.py:3824-3853`).
- **The rebank** (M3; quoted from #60). Both rows re-carded to `lhapdf`/247000:
  σ 1686.48 pb against 1686.34 before (+0.008 %, inside MadGraph's 3.3 pb error);
  `AQCDUP` within budget on 10000 events per row; `amplitude_oracle` 49 passed.
  `couplings/pp_to_ll_qcd0.json` was regenerated: the card writes
  `aS = 1.300028e-01` for `1.300000e-01`, and only GC_10, GC_11 (×√ratio) and
  GC_12 (×ratio) move (*manager-verified*). No other banked proton number moved
  under the new default.
- **refdata-11** (sha256 `637cb81f…acb6a`, 15874 files) adds the ten rows
  sprint D banked; they lose `bundled = false`. Against cut 10 nothing is removed
  and only the two re-carded rows' 30 files differ. The M3's port also taught
  `tests/common/leshouche.rs` Fortran continuation lines and named the new runs
  in the αs and scales inventories. A full `validate` on the merged tree and on a
  clean `git archive` export gave the same cells (211 ✅, 5 ⚠️, 13 ⏳) (M3,
  asserted).
- **One unexplained SIGABRT** in `validate_hadronic` on the export's first run,
  with no panic message; the rerun was clean. A probe ran the binary 22 times
  on Linux under the banked flags without an abort, with run-to-run identical
  numbers. It stays recorded as unexplained; the M3 was asked for the macOS crash
  report.

## Backlog

- Deleted: `pdf-set-not-taken-from-run-card`.
- Filed during the sprint: `generate-rereads-integrate-inputs` (D1b Found 1),
  closed by #69.
- Drafted for filing at close-out: `pdf-download-held-in-memory-under-a-cap`
  (D1 Found 2), `stale-pdf-index-prompts-for-the-wrong-set` (F1 Found 2, F2
  Found 1; needs-user), `kb-concepts-stale-after-sprint-d` (D1 Found 1, D1b
  Found 5, F2 Found 2), `build-sh-has-no-row-selection` (F2 Found 3),
  `refdata-fetch-fails-on-cloud-hosts` (F2 Found 4),
  `validate-hadronic-sigabrt-unexplained` and `validate-sh-keeps-no-crash-evidence`
  (#60 comments and the probe).
- Edited, not a new item: `cli-has-no-param-card-option` drops the obsolete
  "`generate` refuses a different one" (D1b Found 2).
- Deleted at closeout: `validate-sh-stops-at-first-failed-suite`. This PR's
  `--no-fail-fast` closes it. The manager's local banked run on the refdata-11
  tree (`validate.sh`, 2026-10-26) had one failing suite (a validation-report
  unit test, fixed in `7f43544`). Every later suite still ran, the script exited
  101, and the collator rendered 216 measured cells.
- Not filed:
  - D1 Found 3 (CVMFS layout unverified): measured, above;
  - D1 Found 4 (`ParamCard` drops unknown entries): already
    `input-cards-lack-field-classification`;
  - D1 Found 5 (`pdfsup` ids): fixed by F2's PDFSUP rule;
  - D1b Found 3, 4 (driver in the CLI; dead version guards): done by #69;
  - D1c Found 1 (moved pins): measured by F2, none moved;
  - D1c Found 2, 3 (`uu_tt` rebank; the log's misstatement): wrong, and
    corrected in the log;
  - F1 Found 1 (banked gates not run): run by F2 on refdata-10;
  - F1 Found 3, 4 ("unpacks to" wording; interim `--pdf-*` flags): fixed by F2.

## Process notes

The design took three passes (D1, D1b after the user's gridpack principle, D1c
with the answers), and the D1c correction about `nn23lo1` was wrong in both
directions. A session that stopped on a false design claim (F2) instead of
working around it is what exposed that. Merge order was the user's: #67, #72,
#74, then this PR pinning the bundle that carries their rows.
