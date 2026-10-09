---
type: Backlog Item
title: The three llj MLM rows all read high against MadEvent
description: pp_to_llj_xqcut_only, _mlm and _mlm_alps2 all sit +0.03 to +0.23 % above MadEvent; their references come from one shared directory per row.
area: validation
state: open
priority: medium
closes_when: The llj references are regenerated from independent MadEvent directories and the shared offset is attributed either to the reference or to this side.
blocked_by: []
opened: 2026-09-30
tags: [mlm, sigma, llj, madevent-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L213-L219", title: "TODO.md entry T003"}
---
`validate-mlm-sigma` on `refdata-9` (2026-10-03), each row gating
(|pull| < 3 and seed χ²/dof in band):

| row | vibegraph (pb) | MadEvent (pb) | rel | pull |
|---|---|---|---|---|
| `pp_to_llj_xqcut_only` | 212.608 ± 0.126 (5 seeds) | 212.549 ± 0.229 | +0.03 % | +0.23 |
| `pp_to_llj_mlm` | 268.433 ± 0.119 (10) | 268.169 ± 0.284 | +0.10 % | +0.86 |
| `pp_to_llj_mlm_alps2` | 241.255 ± 0.154 (5) | 240.710 ± 0.252 | +0.23 % | +1.85 |

Each passes alone; all three on one side is the finding. The MadEvent
references are M0's: the samples run plus nine seeds in one shared directory,
whose quoted error the seed policy takes. On `pp_to_ll_0j2j_mlm` independent
directories moved `@2` and explained about half its offset. Regenerating these
three from one fresh directory per seed (as `pp_to_ll_0j2j_mlm`'s reference
now is) separates the reference's share from this side's.

Detail: [note 41 §4](../../history/notes/41-mlm-feature-sprint-plan.md) (M2 dump gates, F-B, Z2).
