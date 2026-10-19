---
type: Backlog Item
title: Two older MadGraph defect drafts are not filed upstream
description: Six MadGraph reports are filed (#453–#458); the AQCDUP π truncation and rambo.py drafts in note 07 are still unfiled and not in the one-issue form.
area: validation
state: needs-user
priority: low
closes_when: The AQCDUP and rambo.py defects are each filed upstream (or declined by the user), with the outcome recorded in madgraph-upstream-report-drafts.
blocked_by: []
opened: 2026-09-29
tags: [mlm, madgraph-defect, upstream-report, h1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L229-L241", title: "TODO.md entry T005"}
---
Six reports were filed on 2026-10-19 as mg5amcnlo/mg5amcnlo #453–#458, and
[madgraph-upstream-report-drafts](../../validation/madgraph-upstream-report-drafts.md)
maps each folder to its issue. The user chose not to file the unreachable
`addmothers` report.

Still unfiled: the `AQCDUP` π truncation (`unwgt.f:768-769` in 3.8.1,
`:719-720` in 3.5.17) and the `rambo.py:218` overflow-warning test. Both are
present in 3.8.1 and 3.5.17 by reading. Their drafts are still note 07's older
form. Each needs a `research/upstream/madgraph-*` folder in the one-issue form
that upstream asked for on #452: one problem per issue, stating whether 3.5.17
and 3.8.1 are affected.

The CDATA / `setMad` interplay is tracked separately as
[pythia-setmad-drops-cdata-run-card](pythia-setmad-drops-cdata-run-card.md).

Candidate added 2026-10-15: MadEvent's default survey under-integrates by
0.5–1 % on some processes, and `hard_survey` is silently ignored when `mmjj`
exceeds 1 % of the beam-energy sum
([default survey bias](../../validation/madevent-default-survey-bias.md)).
The same candidate covers the difficult-mode `run_config.inc` substitution,
whose pattern does not match its own output, so a reused process directory
keeps its first survey rate. Related: `keep_grid_for_refine = False`, in both
3.8.1 and 3.5.17, moves grouped σ by about 2 %
([report drafts, "Found while checking"](../../validation/madgraph-upstream-report-drafts.md#found-while-checking)).
