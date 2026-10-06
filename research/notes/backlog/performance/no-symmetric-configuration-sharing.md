---
type: Backlog Item
title: No counterpart to MadEvent's symmetric-configuration sharing
description: MadEvent integrates permutation-related configurations once (SYMCONF/PERMS); merging by map identity covers part of that redundancy, the rest is unmeasured.
area: performance
state: open
priority: low
closes_when: The redundancy left after map-identity merging is measured on a matched mixed row, and a permutation-sharing layer is either built or rejected on that number.
blocked_by: []
opened: 2026-09-28
tags: [mlm, channels, symconf, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L966-L969", title: "TODO.md entry T088"}
---
MadEvent's `SYMCONF` / `PERMS(MAPCONFIG)` integrate permutation-related
configurations once and permute the point. The integrand carries no such layer.
Channel merging by map identity removes part of the same redundancy (364 → 43
densities on `pp_to_ll_0j2j_mlm`). Whether the remainder justifies a second
layer is unmeasured.

First step: count how many of the merged channels are permutation images of each
other on the matched mixed rows, and what their share of the per-point cost is.

Detail: [note 41 §4, D2 diagnosis and "F-B Landed"](../../41-mlm-feature-sprint-plan.md).
