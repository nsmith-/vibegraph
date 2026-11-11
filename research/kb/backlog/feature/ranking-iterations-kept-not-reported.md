---
type: Backlog Item
title: "chi2/dof reads null after a 3-iteration integration and nothing says why"
description: "A --niter 3 integration keeps fewer than two iterations, so chi2_per_dof is null in the artifact, inspect and the ranking JSON; the kept-iteration count is not reported anywhere."
area: feature
state: open
priority: low
closes_when: "The integrate artifact records how many iterations entered the result, inspect and the ranking JSON report it beside chi2_per_dof, and the rule for which iterations are kept is stated in the integrate concept."
blocked_by: []
opened: 2026-11-10
tags: [integrate, inspect, ranking, vegas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr126, resource: "https://github.com/nsmith-/vibegraph/pull/126", title: "inspect --ranking (found there)"}
---
Found while building `inspect --ranking` (nsmith-/vibegraph#126): both
`g g > g g g` and `e+ e- > u u~ g` at `--niter 3` report `chi2_per_dof: null`,
meaning fewer than two iterations entered the result. That is presumably the
warm-up iterations being dropped, but neither `inspect` nor the ranking says
so, and a status page showing "χ²/dof: null" for a 3-iteration run reads as
a bug. Confirm the keeping rule, then bank and report the kept-iteration
count.
