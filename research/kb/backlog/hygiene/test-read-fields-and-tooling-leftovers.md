---
type: Backlog Item
title: "Ten production-written test-read fields and two small tooling nits remain"
description: "Ten fields are written in production and read only by tests under cfg_attr; gen_higgs_window.sh hard-codes mg_version; host_info.py trips ruff PLW1510."
area: hygiene
state: open
priority: low
closes_when: "Each field's write is gated or the field earns a production reader; the version is read from MGMEVersion.txt; subprocess.run calls pass check=."
blocked_by: []
opened: 2026-10-10
tags: [dead-code, tooling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: v1b-report, resource: "../../sprints/hygiene/sessions/V1b-report.md", title: "V1b report (hygiene sprint)"}
  - {id: t1-report, resource: "../../sprints/hygiene/sessions/T1-report.md", title: "T1 report (hygiene sprint)"}
---
V1b Found 5; T1 Found 2–3.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [V1b](../../sprints/hygiene/sessions/V1b-report.md), [T1](../../sprints/hygiene/sessions/T1-report.md).
