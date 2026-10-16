---
type: Backlog Item
title: "87 concepts cite our own code at a pinned commit instead of by path"
description: "workflow/knowledge-bundle says our own code is cited by path, but 176 citations in 87 concepts pin vibegraph-lib at 787070e, which a shallow clone cannot resolve."
area: hygiene
state: open
priority: low
closes_when: "Every own-code citation is by path, or the bundle rule is changed to allow pins and kb-lint enforces whichever is chosen."
blocked_by: []
opened: 2026-10-16
tags: [kb, citations]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: r1, resource: "../../sprints/amplitude-signs/sessions/R1-report.md", title: "R1 report, Found"}
---
R1 Found 9. Count: `grep -rhoE 'blob/787070e[0-9a-f]*/vibegraph-[a-z]+/' research/kb`. The R1 review replaced only the stale or branch-commit pins in its ten concepts.
