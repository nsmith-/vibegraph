---
type: Session Brief
title: "F-D: phase-space and sampling fixes"
description: "Fix R-D's 14 triaged findings in vegas, budget, phasespace, cuts and unweight, and close the sampler-doc and MadGraph-citation items."
status: draft
agent: performance-dev (Opus)
depends_on: [triage]
closes: [sampler-and-phase-space-doc-comments-stale, madgraph-line-citations-predate-pin]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-D.1, .2 with .3 (delete the test-only `(iteration, chunk)` scheme and its tests), .5, .6, .7, .8, .9, .10, .11, .13, .14 (a shared `assert_normalized` only), .15, .16 (the doc only) (`sessions/R-D-report.md`).

**Claimed items closed here:** sampler-and-phase-space-doc-comments-stale, madgraph-line-citations-predate-pin.

**Notes:**
- **Bit-for-bit:** R-D.8 refactors the production `adapt_blocks_iteration`. The VEGAS goldens and `test_adapt_parallel_seeded_is_the_sequential_adapt` must stay bit-identical.
- **R-D.13 (`integrate_channels`, 424 lines):** fix it if time allows. Otherwise report it as stopped, and it will be filed.
- **madgraph-line-citations-predate-pin:** check each citation against `research/refs/mg5amcnlo` at the pin. Its test-file sites (`validate_scales.rs`, `validate_alphas.rs`, `validate_sigma.rs`) are in scope here.
- **Banked gates:** `validate_vegas`, `validate_unweighting`, and `validate_sigma` on the rows R-D.9's tightenings touch.
