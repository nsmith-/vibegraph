---
type: Session Brief
title: "L: lessons for the hygiene agent"
description: "Turn the review reports, triage outcomes and fix-session results into a draft methodology concept on running a hygiene review, the design input for the hygiene agent."
status: draft
agent: claude (Opus)
depends_on: [F-A, F-B, F-C, F-C2, F-D, F-E, F-F, F-CLI, F-G1, F-G2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
**Inputs:** every `sessions/*-report.md`, the triage entries in
[log.md](../log.md), and the fix sessions' mutation results.

**Deliverable:** one concept, `research/kb/workflow/hygiene-review.md`
(`type: Procedure`, `status: draft`). It is the only bundle file this
session writes. It answers, with numbers from the reports:

- **Yield per point.** For each of the four points, the findings reported,
  checked, fixed, filed and rejected.
- **What worked.** Which techniques produced checked findings, and which
  produced leads that failed.
- **Calibration.** Reviewer confidence against the triage outcome: how often
  *checked* findings held up, and how often *suspected* ones did.
- **Vacuity proposals.** How many of the proposed mutations the fix sessions
  confirmed.
- **Scope for the agent.** What a hygiene session on a single PR's diff
  should check, what only a whole-codebase pass can see, and a proposed
  checklist and report shape for the agent definition.
- **Protocol corrections** collected from the reports' "Brief corrections".

**Gate:** `pixi run kb-lint` (or `python scripts/kb.py lint`) passes.
