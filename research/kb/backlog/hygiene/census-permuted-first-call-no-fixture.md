---
type: Backlog Item
title: The permuted-first-call census script has no committed regression fixture
description: census_permuted_first_call.py now fails loudly on missing or malformed MadGraph files, but nothing in CI keeps it that way.
area: hygiene
state: open
priority: low
closes_when: A small synthetic SubProcesses fixture (good, no-PERMS, dangling mapconfig) is committed and a CI step runs the script's perms mode against it, asserting the good output and the non-zero exits.
blocked_by: []
opened: 2026-10-15
tags: [h1, validation-tooling, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: hy1, resource: "../../sprints/sigma-h1-ickkw0-census/sessions/HY1-report.md", title: "sigma-h1-ickkw0-census HY1 report, Found"}
---
HY1 built five synthetic fixtures under which the original
`validation/madgraph/census_permuted_first_call.py` exited 0 with a vacuous
answer. The fixed script exits 1 on each. The fixtures were not committed, and
the repository has no Python test harness, so a later edit could make the
census silent again. The refdata bundle carries no `symperms.inc`, so `perms`
cannot be exercised against it either.
