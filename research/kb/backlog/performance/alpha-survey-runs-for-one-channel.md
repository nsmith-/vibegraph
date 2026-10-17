---
type: Backlog Item
title: "The α survey runs for a single channel"
description: "A one-channel run surveys 6 × 40000 points to learn α = [1.0], more evaluations than the integration itself on h > b b~."
area: performance
state: open
priority: low
closes_when: "A run with one channel skips the α survey, with every seeded golden of a multi-channel run bit-identical and the one-channel runs' change recorded."
blocked_by: []
opened: 2026-10-17
tags: [vegas, multichannel, alpha-survey, decay-width]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: p1, resource: "../../sprints/vegas-constant-integrand/sessions/P1-report.md", title: "vegas-constant-integrand P1 report, Found 1"}
---
On `h > b b~` the log reads `alpha_survey: surveying 1 channels over 6 ×
40000 points`. That is 240 000 evaluations spent learning `α = [1.0]`,
against 120 000 for the integration
([P1](../../sprints/vegas-constant-integrand/sessions/P1-report.md), Found 1).
Skipping `adapt_alphas` when there is one channel
(`FixedBeamIntegrand::use_multichannel` and its proton analogue) removes the
waste, but it changes the random sequence of every single-channel run, so it
needs its own bit-identity check on the multi-channel goldens.
