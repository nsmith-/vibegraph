---
type: Backlog Item
title: "MadGraph refuses some explicit cards at launch that vibegraph runs"
description: "On a model with the MLM limitation MadGraph refuses ickkw = 1 or dynamical_scale_choice = -1, and on a fix_scale model running scales; vibegraph runs both."
area: feature
state: open
priority: low
closes_when: "The model limitations reach the parsed card and the same combinations are refused by name, with a test per case against MadGraph's check_card_consistency."
blocked_by: []
opened: 2026-11-09
tags: [run-card, refusal, madgraph-parity, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: ir1, resource: "../../sprints/v02-last-refusals/sessions/I-R1-report.md", title: "I-R1 report"}
---
Found by I-R1 (`common_run_interface.py:6549-6570`). The absent-card default
already applies the MLM limitation, and `FixedScaleModel` refuses the fix_scale
default. Explicit cards are not checked.

Also open from I-R1:
- the fix_scale default is refused, not reproduced (`fixed_ren_scale` and
  `fixed_fac_scale` with their hidden-line read-back);
- `create_default_for_process`'s matching rule also runs for decays of mixed
  multiplicity (`z > j j` plus `add process z > j j j`), which
  `decay_default` ignores.
