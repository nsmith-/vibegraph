---
type: Backlog Item
title: In-process systematics weights are untimed against MadGraph's systematics module
description: "generate writes MadGraph-format systematics weights in-process (CMS's 880-member request in 0.70 GiB), but no run compares its cost with MadGraph's systematics module on the same events."
area: performance
state: open
priority: medium
closes_when: "A Measurement concept records, on one host in one sitting, the added cost per event of vibegraph's systematics weights and of MadGraph's systematics module on the same events, for a scale-only request and for CMS's runcmsgrid.sh request, at the commit of the MATRIX1 re-measurement."
blocked_by: []
opened: 2026-11-04
tags: [performance, madgraph-comparison, systematics, pdf, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: syst, resource: "../../sprints/systematics/sprint.md", title: "Systematics sprint"}
  - {id: mem, resource: "../../sprints/cms-pdf-memory/closeout.md", title: "CMS PDF memory close-out"}
---
The [systematics sprint](../../sprints/systematics/sprint.md) gated the
weights per event against MadGraph's systematics output, and
[the CMS PDF memory work](../../sprints/cms-pdf-memory/closeout.md) timed our
side alone (CMS's 43-set, 880-member request at about 11 s wall at `-j 4`).
MadGraph's systematics module on the same events was never timed; reweighting
is the only many-weight workflow compared so far
([reweighting-vs-madgraph-stale](reweighting-vs-madgraph-stale.md)).

Measure: generate with and without the request (same grid, seed, events) on
our side; MadGraph's `systematics` on the same LHE file, setup and loop
separated as in the reweighting comparison. Two requests: a 9-point scale
variation and the CMS one.
