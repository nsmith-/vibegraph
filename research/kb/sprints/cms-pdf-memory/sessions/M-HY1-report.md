---
type: Session Report
title: "M-HY1 report: hygiene review of #86"
description: "fetch_cms.sh made bash 3.2 compatible; the table oracle refuses a partial tree and does not skip unreadable entries; history-framed comments rewritten; the bytes guard fails on any single kept table; the oracle catches every non-equivalent mutant tried."
status: draft
tags: [hygiene, report, pdf, memory, sprint-f]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-04}
---
The session returned its report as a message and wrote no file. The manager
transcribed it here, abridged, with the findings and evidence unchanged.
Worktree `/home/user/wt/M`. Commits 5d3f68f (`fetch_cms.sh`) and 4158f98
(the oracle and comments). Scratch logs are in the manager's scratchpad,
under `M-HY1/`.

## Findings (written before any edit)

| id | site | finding | disposition |
|---|---|---|---|
| F1 | `validation/pdf/fetch_cms.sh` | `mapfile` and `declare -A` need bash 4, but macOS's system bash is 3.2 (`build_amplitude.sh:39` avoids `mapfile` for that reason) | fixed, 5d3f68f |
| F2 | `table_oracle.rs` | the only guard was `from_tree > 0`, so a partly fetched tree passed (a 61-member tree printed "ok") | fixed, 4158f98: a tree with fewer members than CMS's 880 is refused |
| F3 | `table_oracle.rs` `members_on_disk` | `e.ok()?` skipped a failing entry, and a bare `unwrap` | fixed; no mutation reaches it |
| F4 | oracle header, `pdf_member_bytes.rs` | history framing | fixed |
| F5 | `pdf/mod.rs`, `interp.rs` docs | over-long lines | fixed |
| F6 | `geomspace` | five copies, two of them new | Found |
| F7 | oracle scaffolding | `select_band` and others copied from production | left: the copy is what lets the seam points test production's `select_band` independently |
| F8 | `XCell`, `QCell`, accessors | nothing widened | no finding |
| F9 | `pdf_member_bytes` isolation | the only test in its binary; stable, 8.11–8.17 B/knot | left; the file's doc says so |
| F10 | the 10 B/knot bound | see the mutations | holds |

## Mutations

**Bytes guard** (baseline 8.17 B/knot synthetic, 8.11–8.12 NNPDF23):

| kept on top of the knots | B/knot | test |
|---|---|---|
| a second grid copy | 16.17 | failed |
| a coefficient table | 39.85 | failed |
| half a grid in f64 | 12.17 | failed |
| an f32 grid copy | 12.17 | failed |

The blind spot is under about 1.8 B/knot extra.

**Table oracle.** Unmutated, 0 of 413,847,296 values differ. The mutants:

| mutant | values differing |
|---|---|
| `ddx_h` one-sided branch off | 634,189 |
| `ddx_l` one-sided branch off | 1,057,428 |
| wrong `dlogx_lo` | 85,586,324 |
| a re-associated `a` | 14,009,235 |
| `select_band` `<` changed to `<=` | 10,196,650 |
| Q² slope term × (1+ε) | 1,308,424 |

The `ix_hi`/`ix_lo` edge clamps survive. They are equivalent: the one-sided
branch never reads the clamped values.

**F1 identity.** Four sets fetched with the bash-3.2 script are identical to
`/home/user/cms-pdfs` (`diff -r -x .done`), and so is `pdfsets.index`. The
offline rerun exits 0. A missing member dies with exit 1.

## Gates

- `cargo fmt --all --check`: clean.
- clippy, default and `--all-features`: clean.
- `cargo test --workspace`: 0 failed, 1805 passed.
- `cargo doc`: 8 warnings, the same count as before the change, none in
  `pdf`.
- `pdf::table_oracle` over the 880-member tree plus NNPDF23: 0 differ.

## Found

1. `geomspace` copied five times (F6). Filed as `geomspace-copied-five-times`.
2. `pdf-lookup-f64-only` cited stale `interp.rs` lines and "coefficients".
   The manager updated it.
3. `pdfsets.index` is unpinned. Already filed (`fetch-downloads-do-not-retry`).
4. `a_loaded_member_holds_about_one_f64_per_knot` skips its real-member half
   when NNPDF23 is absent. That is acceptable for a hermetic test, and it
   prints a line.
