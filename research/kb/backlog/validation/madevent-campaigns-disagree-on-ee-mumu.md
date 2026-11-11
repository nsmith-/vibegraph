---
type: Backlog Item
title: "Two MadEvent campaigns on e+ e- > mu+ mu- disagree by 4 sigma"
description: "Five fresh-directory MadEvent seeds give 0.41950 +- 0.00008 pb on the grammar card where the committed grammar reference reads 0.41982; the grammar sigma gate reads the higher campaign."
area: validation
state: open
priority: low
closes_when: "The cause of the offset is named (iteration weighting, survey, or a card difference), the committed reference is regenerated or annotated with its measured campaign spread, and the grammar sigma tolerance is reconciled with it."
blocked_by: []
opened: 2026-11-10
tags: [madevent, sigma, reference, statistics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares (found there)"}
  - {id: bias, resource: "../../validation/madevent-iteration-weighting-bias.md", title: "MadEvent's iteration combination reads low"}
---
The configuration-share oracle (nsmith-/vibegraph#124) generated five
MadEvent 3.7.1 seeds of `e+ e- > mu+ mu-` at √s = 500 GeV, one fresh process
directory each: 0.41950 ± 0.00008 pb, seed χ²/dof 0.39. The committed
`grammar_sigma_reference.json` gives 0.41982 for the same process and card,
4σ away, and vibegraph's 0.41978 matches the latter. The share gate reads
fractions of each side's own σ, so it is unaffected; the grammar σ gate is
not. [Iteration weighting](../../validation/madevent-iteration-weighting-bias.md)
is documented for 2 → 3 and 2 → 4 rows; this is a 2 → 2 row.
