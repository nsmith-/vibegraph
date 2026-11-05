---
type: Backlog Item
title: "Couplings a param card sets to exactly zero keep their diagrams"
description: "Pruning runs only from the restriction card, as in MadGraph; a param card zeroing ymb leaves 184 dead b-Yukawa diagrams in b b > H H b b, costing evaluation and a group split."
area: performance
state: open
priority: low
closes_when: "Either vertices whose couplings evaluate to exactly zero at the bound card are dropped before compilation, with the diagram census reading MadGraph's counts through a documented exemption and σ unchanged bit for bit, or the decision to keep MadGraph's behaviour is recorded."
blocked_by: []
opened: 2026-11-05
tags: [restriction, param-card, diagrams, performance, flavour-groups]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: hg0, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G0-report.md", title: "H-G0 report, Found 2"}
---
`ParsedModel::apply_restriction` prunes the vertices whose couplings the
restriction card fixes to zero, as MadGraph's `restrict_*.dat` does. It never
prunes from the param card bound at run time.

The four CMS Singlet-model VBF gridpacks were generated on a restriction with
the b mass and Yukawa non-zero, and their param card sets `MB = ymb = 0`. So
`b b > H H b b` keeps 184 diagrams of exactly zero value: 192 against 8 for
`d d > H H d d`. MadGraph's own output has the same 192 (H-G0). σ is exact
either way. The costs are:
- about 24× the diagram work on that subprocess;
- the `d d` / `b b` pair cannot share a flavour group, because the dead diagrams
  give its flows no fingerprint counterpart. Each is evaluated separately.

Pruning on exact zeros of the bound card would change no weight, but it would
move per-card diagram counts away from MadGraph's, so the census would need to
exempt it.
