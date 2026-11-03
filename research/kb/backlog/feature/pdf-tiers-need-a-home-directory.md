---
type: Backlog Item
title: "A PDF lookup is refused without a home directory, even when the set is on the data path"
description: "pdf_tiers computes the cache root before any tier is read, so a worker with no $HOME and no $VIBEGRAPH_HOME is refused although its set is on $LHAPDF_DATA_PATH."
area: feature
state: open
priority: medium
closes_when: "A run with the set on $LHAPDF_DATA_PATH and neither $HOME nor $VIBEGRAPH_HOME set finds the set, and the cache root is required only when the cache or download tier is reached; a CLI test pins it."
blocked_by: []
opened: 2026-10-26
tags: [pdf, lhapdf, cache, grid-worker, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f2, resource: "../../sprints/gridpack-generate/sessions/F2-report.md", title: "gridpack-generate F2 report, Found 3"}
---
`pdf_tiers` (`vibegraph-cli/src/assets.rs`) calls `cache_root()` first and
refuses when it is `None`, before `Tiers::from_env` reads
`$LHAPDF_DATA_PATH`. Its message even advises putting the set on the data path,
which does not help. A grid worker that runs `generate` from a gridpack with the
set on `$LHAPDF_DATA_PATH` and no home directory is the case the gridpack design
serves ([F2](../../sprints/gridpack-generate/sessions/F2-report.md); the function
then was `resolve_pdf_set_dir`, removed by the PDF-from-run-card PR, and the
order survived the move).
