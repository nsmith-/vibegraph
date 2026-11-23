---
type: Backlog Item
title: "ldjson progress records carry no group index and no run-global position"
description: "On multi-multiplicity cards the per-group alpha_survey progress {stage, done, total=6} restarts once per group, and nothing in the record is run-global, so consumers read it as the survey restarting."
area: feature
state: open
priority: medium
closes_when: "Every per-group progress stage carries group, groups, label and channels; a run-global phase/phases from the phase records never decreases; both are documented as extras under schema_version 1; a hermetic test on p p > e+ e- plus add process p p > e+ e- j (sm-no_b_mass) shows groups 1 of 2 and 2 of 2 with a non-decreasing global position, and a single-group run shows 1 of 1."
blocked_by: []
opened: 2026-11-23
tags: [cli, ldjson, progress, output]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
A downstream consumer reports this on a DY + 0–3 jets card (@1–@4). The
`alpha_survey` progress record is `{stage, done, total=6}` and runs once per
outgoing-leg group, so readers see "1..6 of 6" restart several times and take
it for the survey restarting. No field in the record is run-global.

The `phase` records (`integrate --output-ldjson`, see `docs/src/cli/overview.md`,
"Phase timing") already give the run's phase sequence. The fix reuses them
for the global position instead of defining a second notion of phase.
