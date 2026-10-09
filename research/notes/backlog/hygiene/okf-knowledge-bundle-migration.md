---
type: Backlog Item
title: Reference material is not yet an OKF knowledge bundle
description: research/notes and TODO.md are being migrated to an OKF knowledge bundle under research/kb/; Phase 2 is committed and Phase 3 is in progress per note 42.
area: hygiene
state: open
priority: high
closes_when: Note 42's Phase 4 is complete. The notes have moved to research/kb/history/notes/, external citations point to concept IDs, and the agent briefs and AGENTS.md planning section are updated.
blocked_by: []
opened: 2026-10-05
tags: [okf, knowledge-bundle, backlog, migration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L566-L571", title: "TODO.md entry T046"}
---
The plan is [note 42](../../42-okf-knowledge-bundle-plan.md), with user
decisions in §9; [note 42 §8](../../42-okf-knowledge-bundle-plan.md) records
how each phase went. Phases B, 0 and 1 are done (2026-10-06), the user
approved the taxonomy on 2026-10-09, and Phase 2's draft concepts are committed
(a4536c5, all sixteen groups). What remains:

- **Phase 3**: adversarial verification and a coverage map (in progress;
  checkpoint commits on 2026-10-09).
- **Phase 4**: move the backlog, decisions and archived notes under
  `research/kb/`, re-cite from the (note, §) → concept map, re-point
  `scripts/kb.py` and the docs build, update the briefs, and add the
  `new-sprint` scaffold.

Phase 5 (Attested Computations) is optional and outside this item. The first
sprint after the migration is the hygiene sprint, which also trials the
per-sprint lifecycle (note 42 §4).
