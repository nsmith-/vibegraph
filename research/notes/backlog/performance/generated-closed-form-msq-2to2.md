---
type: Backlog Item
title: No generated closed-form |M|² for small processes
description: "Finite-field reconstruction now makes a closed-form |M|² for 2 → 2 (and the llj subprocesses) cheap to obtain; nobody has measured it against the evaluator."
area: performance
state: open
priority: low
closes_when: "A reconstructed closed form for at least one 2 → 2 and one llj subprocess is generated, gated against eval_m2, and timed against it on one host; adopted or recorded as not worth it."
blocked_by: []
opened: 2026-10-05
tags: [finite-field, trace-form, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: note41, resource: "../../41-completeness-trace-msq-feasibility.md", title: "Note 41 (completeness/trace-form feasibility), §9–§10"}
---
[Note 41](../../41-completeness-trace-msq-feasibility.md) found that, for
2 → 2, "a generated closed form is now cheap to obtain: reconstruct" the
trace-form |M|² over finite fields. Its §10 full-|M|² results ran 4–5×
faster than `eval_m2` on the llj subprocesses. Neither follow-up was filed:
generate the closed form as code for the small processes, check it against
`eval_m2`, and decide from a timing whether it is worth a code path.
Helicity sampling, the other lever note 41 names, is the
[nhel1-run-cards-refused](../feature/nhel1-run-cards-refused.md) item.
