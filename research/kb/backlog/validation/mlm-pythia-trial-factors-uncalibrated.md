---
type: Backlog Item
title: "The MLM Pythia gate's false-fail rates are bounds, not measurements"
description: "D1's 1.3 % for five |pull| < 3 tests and the per-bin leg's look-elsewhere factor assume independent pulls; the pulls are correlated, so both rates are upper bounds that no toy study has measured."
area: validation
state: open
priority: low
closes_when: "A toy study of the null (resampled events or files under one side) measures the five-pull and per-bin false-fail rates, and D1's figures are restated from it."
blocked_by: []
opened: 2026-10-17
tags: [mlm, pythia, look-elsewhere, calibration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/mlm-pythia-tolerance/sessions/V1-report.md", title: "mlm-pythia-tolerance V1 report, Found 5"}
---
`all` is built from the `@N` acceptances and the merged σ follows `all`, so
the five pulls are correlated, and the normalised-shape pulls share their
normalisation. The gate's stated false-fail rates in
[D1](../../sprints/mlm-pythia-tolerance/decisions/D1-tolerance.md) are
therefore upper bounds, which means the gate is conservative
([V1](../../sprints/mlm-pythia-tolerance/sessions/V1-report.md)). A toy study
of the null's |pull|max distribution would turn them into measurements.
