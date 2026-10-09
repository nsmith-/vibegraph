---
type: Backlog Item
title: e+ e- > e+ e- $$ z under-samples a region below the plateau budget
description: The twenty-seed σ mean of the $$ z Bhabha row climbs 0.5% with budget before plateauing; the missed region and the CLI default budget's reach are unmeasured.
area: validation
state: open
priority: high
closes_when: The under-sampled region is identified, and the CLI's default budget is shown to reach the plateau on every one of ≥20 seeds (or the sampler is fixed so it does).
blocked_by: []
opened: 2026-09-27
tags: [process-grammar, s-channel-restriction, vegas, budget-ladder, bhabha]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L280-L288", title: "TODO.md entry T012"}
---
`e+ e- > e+ e- $$ z` (`ee_to_ee_nsz`) is the only row of the grammar σ
ladder (`probe_grammar_seed_sweep`, twenty seeds per rung) whose mean moves
with budget:

| budget | ×¼ (10k×6) | ×1 (40k×6) | ×4 (160k×6) | ×16 (640k×6) |
|---|---|---|---|---|
| mean (pb) | 156.71 ± 0.17 | 157.25 ± 0.09 | 157.55 ± 0.05 | 157.54 ± 0.02 |
| seed χ²/dof | 2.23 | 1.80 | 1.22 | 1.53 |

A region is missed until the budget covers it. The seeded gate
(`the_grammar_rows_match_madevents_seeds` in `validate_sigma`) runs on the
plateau at 160 000 × 6, so it does not see this. MadEvent's own seeds have the
same low tail: one of five at 156.48 on this row, and two of five 0.8% and
1.3% low on unrestricted Bhabha (seed χ²/dof 20 and 93).

Unmeasured: which region it is, and whether `vibegraph integrate` at its
default budget reaches the plateau on every seed. Start with a per-channel
split of the low-rung seeds against the plateau. Detail:
[note 38 §8.5](../../history/notes/38-process-grammar-sprint-plan.md).
