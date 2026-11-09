---
type: Backlog Item
title: "The re-carded ladder's source comments quote readings older than the code"
description: "pp_to_llj and pp_to_ll_scalefact2's recorded ladder readings in validate_hadronic.rs RECARDED_ROWS do not reproduce at their pinned seeds, while pp_to_bb's do."
area: hygiene
state: open
priority: low
closes_when: "The RECARDED_ROWS comments and pp_to_llj's ladder_note are re-read from a current run, and the budget justification they carry is rechecked."
blocked_by: []
opened: 2026-11-09
tags: [comments, ladders, hygiene]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: iz2, resource: "../../sprints/v02-last-refusals/sessions/I-Z2-report.md", title: "I-Z2 report"}
---
On 2b26c109, pp_to_llj reads +0.25/+0.26/+0.14/+0.19% (χ²/dof
0.45/0.23/1.12/1.07). The comment says +0.28/+0.20/+0.22/+0.21%,
χ²/dof 1.32/0.49/0.49/1.43.

pp_to_ll_scalefact2 reads +0.03/+0.01/+0.02/+0.04%, against the comment's
−0.02/+0.00/+0.03/+0.05%.

The ladder is deterministic, because pp_to_bb reproduces exactly, so the
comments predate later code. pp_to_llj's `ladder_note` reaches the report
cell, so the report repeats the stale numbers.
