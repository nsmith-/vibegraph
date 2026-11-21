---
type: Backlog Item
title: "A function that a model's function_library.py leaves undefined is evaluated with the SM definition"
description: "MadGraph raises NameError when a model's expressions call a function its function_library.py does not define; vibegraph evaluates the SM definition instead."
area: feature
state: open
priority: low
closes_when: "A call to a function the model's own library does not define is refused by name, or the fallback is documented with the models that rely on it (toy models without a library), with a test."
blocked_by: []
opened: 2026-11-17
tags: [ufo, refusal, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-U1 session
(its report, `sessions/K-U1-report.md` in the sprint folder).
Models that ship no library (the toy models use `complexconjugate`) need a
decision first.
