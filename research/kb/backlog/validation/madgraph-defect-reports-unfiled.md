---
type: Backlog Item
title: MadGraph defect reports are drafted but not filed upstream
description: Report drafts for the MadGraph defects still present in 3.8.1 are ready, each with a MadGraph-only reproducer; the user files them.
area: validation
state: needs-user
priority: low
closes_when: Each report in madgraph-upstream-report-drafts whose defect is still present upstream is filed, and its link recorded in that concept.
blocked_by: []
opened: 2026-09-29
tags: [mlm, madgraph-defect, upstream-report, h1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L229-L241", title: "TODO.md entry T005"}
---
The drafts are in
[madgraph-upstream-report-drafts](../../validation/madgraph-upstream-report-drafts.md).
Each was checked against MadGraph 3.8.1, the latest release, and has a
reproducer under `validation/madgraph/repro/` that needs only a stock MadGraph.
Ready to file:

- H1, the grouped first `setclscales` call on the unpermuted point;
- `reweight.f:1138`, `scalefact` skipped with one fixed μF;
- `setcuts.f`, the duplicate `iforest(2)` test;
- `cuts.f`, the `NMASSLESS` precedence (`ktdurham`/`ptlund`);
- `addmothers.f:115`, the stale loop index (code quality; unreachable);
- `banner.py`, `RivetCard.setWeightName`;
- `rewgt`, the stale `ipdgcl` of a non-jet parton.

The `AQCDUP` π truncation and `rambo.py` drafts are still in note 07's appendix
and are still present in 3.8.1. The ALOHA `P1D` (`FFV2P1D_1`) defect is fixed
in 3.8.1 (PR #428); do not file it.

The CDATA / `setMad` interplay is tracked separately as
[pythia-setmad-drops-cdata-run-card](pythia-setmad-drops-cdata-run-card.md).

Candidate added 2026-10-15: MadEvent's default survey under-integrates by
0.5–1 % on some processes, and `hard_survey` is silently ignored when `mmjj`
exceeds 1 % of the beam-energy sum
([default survey bias](../../validation/madevent-default-survey-bias.md)).
The same candidate covers the difficult-mode `run_config.inc` substitution,
whose pattern does not match its own output, so a reused process directory
keeps its first survey rate. Related: 3.8.1's `keep_grid_for_refine = False`
moves grouped σ by about 2 %
([report drafts, "Found while checking"](../../validation/madgraph-upstream-report-drafts.md#found-while-checking)).
