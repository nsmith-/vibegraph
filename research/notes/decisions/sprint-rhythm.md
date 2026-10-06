---
type: Design Decision
title: Sprints cycle feature → validation → performance
description: "A feature lands behind the MadGraph validation net, a validation pass hardens the net around what it exposed, and a performance pass optimizes against the hardened gate."
decided_by: human:nsmith-
status: draft
tags: [process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L3-L6", title: "TODO.md working rhythm"}
---

Sprints cycle **feature → validation → performance**. A feature lands behind
the MadGraph validation net; a validation pass then hardens the net around
what the feature exposed; a performance pass optimizes against the hardened
gate.

As of the `mlm` close-out (2026-10-03) the next directions named were NLO and
the performance backlog.
