---
type: Backlog Item
title: The MadEvent default-survey bias has no upstream report
description: The other MadGraph reports are filed or declined; the default-survey under-integration candidate, and the related keep_grid_for_refine change, are not yet drafted for upstream.
area: validation
state: needs-user
priority: low
closes_when: The default-survey bias (with the keep_grid_for_refine change) is drafted in a research/upstream/madgraph-* folder and filed upstream, or declined by the user, with the outcome recorded in madgraph-upstream-report-drafts.
blocked_by: []
opened: 2026-09-29
tags: [mlm, madgraph-defect, upstream-report, h1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L229-L241", title: "TODO.md entry T005"}
---
Six reports were filed on 2026-10-19 as mg5amcnlo/mg5amcnlo #453–#458.
[madgraph-upstream-report-drafts](../../validation/madgraph-upstream-report-drafts.md)
maps each to its issue. The user declined to file three:
- the unreachable `addmothers` one;
- for now, the low-priority `AQCDUP` π truncation and `rambo.py` overflow
  warning. Their drafts stay ready in their folders.

What remains is a candidate added 2026-10-15. MadEvent's default survey
under-integrates by 0.5–1 % on some processes, and `hard_survey` is silently
ignored when `mmjj` exceeds 1 % of the beam-energy sum
([default survey bias](../../validation/madevent-default-survey-bias.md)).

The same candidate covers the difficult-mode `run_config.inc` substitution.
Its pattern does not match its own output, so a reused process directory keeps
its first survey rate.

Related: `keep_grid_for_refine = False`, in both 3.8.1 and 3.5.17, moves grouped
σ by about 2 %
([report drafts, "Found while checking"](../../validation/madgraph-upstream-report-drafts.md#found-while-checking)).

Upstream asked on #452 for one problem per issue, stating whether 3.5.17 and
3.8.1 are affected.

The CDATA / `setMad` interplay is tracked separately as
[pythia-setmad-drops-cdata-run-card](pythia-setmad-drops-cdata-run-card.md).
