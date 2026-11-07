---
type: Backlog Item
title: "A run does not record which near-degenerate flavour groups it kept apart"
description: "Since #103 a pair separating above GROUP_REL_TOL starts its own group with nothing in the log or ldjson naming the pair or its separation, so the CERN host could not report the CKM splits."
area: feature
state: open
priority: low
closes_when: "A run records, for each flavour-group split decided by |M|² separation, the two subprocesses and their best-probe separation (in the log, and in the ldjson result or a debug object), with a test on a card that splits."
blocked_by: []
opened: 2026-11-07
tags: [flavour-groups, logging, ldjson, ckm, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: #103 and the closing campaign"}
  - {id: hn1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-N1-report.md", title: "H-N1 Found 5: the CERN host should record each split"}
---
H-N1 asked the closing campaign to record, for the survey's CKM cards, the pair
and separation each now splits on, because 3.88e-8 was unattributed. The CERN
host reports that the 3.88e-8 card (089) was not in the campaign's sample, and
that the split is not logged anywhere: a run does not say that a near-degenerate
pair was kept apart.

Recording it would have served
[ckm-flavour-groups-near-degenerate-refused](../validation/ckm-flavour-groups-near-degenerate-refused.md)
directly, and is the first thing to read on any CKM card whose σ disagrees with
MadGraph after a split.
