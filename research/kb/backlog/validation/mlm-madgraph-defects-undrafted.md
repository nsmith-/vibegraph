---
type: Backlog Item
title: Six MadGraph defects from the MLM study have no report draft
description: The six MadGraph defects note 41 §1.5 found are not yet drafted as upstream reports in note 07's appendix.
area: validation
state: open
priority: low
closes_when: Note 07's appendix holds a report draft for each of note 41 §1.5's six defects, each with location, mechanism, effect on a weight and, where one exists, a reproducer.
blocked_by: []
opened: 2026-09-28
tags: [mlm, madgraph-defect, upstream-report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L229-L241", title: "TODO.md entry T005"}
---
[Note 41 §1.5](../../history/notes/41-mlm-feature-sprint-plan.md) lists six defects in the pinned MadGraph 3.7.1 tree
(`b7687064`), none drafted in [note 07](../../history/notes/07-mg5-code-quality.md):

| Where | Defect | Effect |
|---|---|---|
| `reweight.f:1138` | `.not.fixed_fac_scale1.or.fixed_fac_scale2` precedence | changes a weight with exactly one fixed μF; refused in this crate |
| `setcuts.f:939-942` | duplicate `iforest(2)` test | grids only |
| `cuts.f:565` | `ktdurham` `.and.`/`.or.` precedence | CKKW-L, out of scope here |
| `addmothers.f:115` | compares `igraphs(1)` to a stale loop index | measured unreachable on every MLM row |
| `banner.py:1706` | `setWeightName` raises when `ickkw ≠ 0` | Python systematics only |
| `rewgt` | reads the previous event's final-state `ipdgcl` | empty on every MLM row by census |

Drafting is agent work; filing them is the user's (see
[madgraph-defect-reports-unfiled](madgraph-defect-reports-unfiled.md)).
