---
type: Sprint Record
title: "CMS's 880-member systematics request under 2 GB"
description: "One item closed in PR #86: each PDF member is held as its knots with the x cubic built on use, bit-identical to the old tables; CMS's 880-member request peaks at 0.70 GiB instead of 3.6 GB, its weights still exact, no slower end to end."
closed: 2026-11-04
status: draft
tags: [pdf, memory, cms, systematics, sprint, sprint-record, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: d1, resource: "decisions/D1-lossless-knots-first.md", title: "The user's D1–D4"}
  - {id: pr86, resource: "https://github.com/nsmith-/vibegraph/pull/86", title: "PR #86 (merged)"}
---
Part of sprint F (CMS production readiness), alongside
[grid-worker-fixes](../grid-worker-fixes/closeout.md).

## What was banked

- **Where the memory went.** The heap held 4.06 MB per member, from three
  copies of the data. The precomputed x-cubic coefficients were 66 %. A
  second copy of the grid was 17 %. The parsed grid itself was 17 %. Parse
  transients and allocator overhead were small: 14 MB at `-j 1`, 63 MB at
  `-j 4` ([M-P1](sessions/M-P1-report.md)).
- **The layout.** `LogBicubic` owns the member's subgrids. It builds `[a, b,
  c, d]` at the point, with the same expressions in the same order as the
  tables did.
- **Bit-identical.** `pdf::table_oracle` carries the old algorithm and
  compares `to_bits` across the CMS tree, NNPDF23 and three synthetic
  members: 0 of 413 847 296 values differ. Its mutants are all detected,
  except two equivalent ones ([M-HY1](sessions/M-HY1-report.md)).
- **The item's numbers:**
  - peak RSS of at most 0.701 GiB at `-j 1` and `-j 4`
    ([M-V1](sessions/M-V1-report.md));
  - CMS weights exact on all four banked runs (9.26 M each);
  - wall time 0.92–0.98× the old build. One reading is 1.83× slower, but
    readings are about 3 s of a 24 s run.
- **Guards.** A default test holds a member to at most 10 B per knot. It
  fails on any layout that keeps even one of the dropped tables.
  `fetch_cms.sh` and the `fetch-cms-pdfs` and `validate-cms-pdfs` tasks put
  the CMS gate within reach of the long tier.

## Decisions

The user's [D1–D4](decisions/D1-lossless-knots-first.md), each the
recommended option:
- lossless knots first;
- up to 2× time;
- aim below 1.5 GB;
- a CI bytes-per-member guard.

Neither of the alternatives (set at a time, `f32`) was needed.

## Lessons

- **Attribute before building.** The plan had five `f64` per knot; there
  were six, because the grid was stored twice. The S-V1 timings the plan
  quoted were taken under contention: they showed PDF readings as dominant,
  and they are not. M-P1's attribution step found both before any code
  changed.
- **A bit-level oracle needs to live in the tree.** M-P1's exactness was
  first shown with a scratch harness and 3.1 GB of dumps. M-V1 made it a
  test that refuses a partial tree, and M-HY1 found that its first version
  would have accepted one.

## Filed from the sessions

- `pdf-grid-oracle-blind-to-the-seam-rule`
- `fetch-downloads-do-not-retry`
- `pdf-reading-builds-cubic-per-flavour`
- `geomspace-copied-five-times`

`pdf-lookup-f64-only` was updated for the new layout.
