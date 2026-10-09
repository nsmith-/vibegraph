---
type: Design Decision
title: "Hygiene sprint scope: the fresh review plus the localised hygiene items"
description: "The sprint claims the filed hygiene items that already name their sites; needs-user items, MadGraph re-runs and long re-measurements stay out."
decided: 2026-10-09
decided_by: human:nsmith-
status: draft
tags: [hygiene, scope, sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "../log.md", title: "Hygiene sprint log, 2026-10-09: the user's answers in the planning session"}
---
Chosen over "fresh review only" and "everything runnable".

**In:** the stale-comment lists, the latent bugs (`asin`/`acos`,
`make_anti`, the SDE = 1 configuration weights, the dead reweight guard, the
artifact version arm), and small tooling and CI fixes. Each already names its
sites, so it adds work to a fix session but no investigation. The full list
is in [sprint.md](../sprint.md), "Scope".

**Out:**
- `needs-user` items, and items blocked on one;
- items that need a MadGraph re-extraction or re-bank;
- items that need a long seed re-measurement, which would break the
  one-deliverable rule for a fix session;
- licence research;
- the evaluator's f64 coefficient migration, which is a refactor (see
  [D3](D3-fix-small-file-large.md)).
