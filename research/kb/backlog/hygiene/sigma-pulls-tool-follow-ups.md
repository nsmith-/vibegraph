---
type: Backlog Item
title: "sigma-pulls copies the seed-record selection and reads its inputs loosely"
description: "sigma-pulls copies the selection around seeds::combine, counts no unmatched cells, has no committed --this-side producer, and pins its output only by a diff."
area: hygiene
state: open
priority: low
closes_when: "SeedRecord gains a selection method that the gates and sigma-pulls share, unmatched cells are counted against an allowlist of controls, the --this-side file has a committed producer, and a golden-output test pins the printed lines."
blocked_by: []
opened: 2026-10-17
tags: [validation-report, sigma-pulls, seed-record, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: hy1, resource: "../../sprints/sigma-pulls-lean-high/sessions/HY1-report.md", title: "sigma-pulls-lean-high HY1 report, Found 1–4"}
---
From [HY1](../../sprints/sigma-pulls-lean-high/sessions/HY1-report.md):

1. **Copied selection.** `sigma-pulls` copies the estimator × part × budget
   selection around `seeds::combine`, and its own record-directory walk.
   `SeedRecord::reference_of` is private, fixes the record's estimator, and
   panics below five seeds. A shared
   `SeedRecord::read(part, estimator, screening)` plus a record lister beside
   `seed_record::read` would serve both. That edits `seed_record.rs`, so it
   waits for PR #43.
2. **Unmatched cells.** These print "not read from a seed record" and exit 0.
   Four controls are expected; any other would shrink the population unseen.
3. **No producer for `--this-side`.** V1 made the file with scratchpad
   scripts.
4. **Unpinned formats.** `EW_SHIFT_RESOLVED` and the printed formats are
   checked only by an output diff.
