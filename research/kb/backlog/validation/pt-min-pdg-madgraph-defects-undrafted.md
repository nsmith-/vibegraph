---
type: Backlog Item
title: "The two MadGraph per-PDG cut defects H-C1 found have no upstream drafts"
description: "banner.py:4722's light-code guard never fires and setcuts.f:681–683 indexes legs by list index in hash-seed order; both are catalogued in madgraph-defects but not drafted for upstream."
area: validation
state: open
priority: low
closes_when: "Each defect has a research/upstream/madgraph-* draft (one problem per issue, saying whether 3.5.17 and 3.8.1 are affected; the setcuts one with H-C1's three-line patch and reproducer), and filing or declining is recorded in madgraph-upstream-report-drafts."
blocked_by: []
opened: 2026-11-07
tags: [madgraph-defect, upstream-report, cuts, pt-min-pdg]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hc1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C1-report.md", title: "H-C1 report, Found F1 and F2"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: H-C1 done, 'upstream report drafts for the two defects'"}
---
H-C1 found two MadGraph defects in the per-PDG cuts. Both are rows in
[madgraph-defects](../../validation/madgraph-defects.md), marked "not reported
upstream", and vibegraph refuses the cards that reach them:

- `banner.py:4722` compares string keys with integers, so the refusal of a
  per-PDG cut on light quarks, b, charged leptons, gluons and photons never
  fires and MadEvent replaces their class cuts. Fix: `int(pdg) in …`.
- `setcuts.f:681–683` reads `idup(i, …)` with the `pdg_cut` list index where the
  leg index is meant; with ≥3 listed codes it raises the τ floor past the cuts
  by hash-seed chance (−0.67% ± 0.18% on `u u~ > a g g g`). H-C1 has a
  three-line patch (`setcuts_index.patch`) and the reproducer.

Upstream asked (#452) for one problem per issue. Filing is the user's call, as
for [madgraph-defect-reports-unfiled](madgraph-defect-reports-unfiled.md).
