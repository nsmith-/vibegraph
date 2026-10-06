---
type: Backlog Item
title: The matched Pythia comparison has no decided tolerance
description: The MLM matched-through-Pythia comparison agrees within 1σ but stays info; no tolerance is decided and its driver writes no collator row.
area: validation
state: needs-user
priority: medium
closes_when: A tolerance for the per-@N acceptances and the jet-rate χ² is decided, the driver writes a row the collator renders, and the cell is flipped to gate.
blocked_by: []
opened: 2026-09-29
tags: [mlm, pythia, gate-promotion, collator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L255-L261", title: "TODO.md entry T008"}
---
`pixi run -e pythia validate-mlm-pythia` (`validation/pythia/mlm_match.py`) on
`pp_to_ll_0j2j_mlm` (2026-10-01): MadEvent's 21 files against vibegraph's 20,
10000 events each, Pythia seeds 20261201–10, `setMad = off`, qCut 30. Acceptance
pulls `@0` −0.80, `@1` −0.95, `@2` −0.98 at a per-side resolution of 0.3 %
(`@1`) and 0.5 % (`@2`); merged σ 685.77 ± 0.68 against 685.98 ± 0.80 pb;
jet-rate χ² 24.5/26, 14.4/23, 20.7/20.

The cell is `info`. Promoting it needs (a) a decided tolerance, which is the
user's call, and (b) a driver that writes a row the collator renders.
MadEvent's own A-vs-A split reads d12 χ² 35.6/22 (p 0.034), so a jet-rate gate
needs that null beside it. The comparison sees `<scales>`: removing it moves
`@1` by +25σ.

Detail: [note 41 M5 and C](../../41-mlm-feature-sprint-plan.md).
