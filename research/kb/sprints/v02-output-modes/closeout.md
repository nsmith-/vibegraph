---
type: Sprint Record
title: "v0.2 output modes: plain, rich and ldjson, and vibegraph inspect"
description: "Sprint G part four (#99): one output mode per run, a versioned --output-ldjson stream with ten error kinds, per-multiplicity totals in artifact format 20, vibegraph inspect, generate's final object, and the survey reading ldjson. One item closed, two filed."
closed: 2026-11-05
status: draft
tags: [cli, output, ldjson, tui, artifact, cms-survey, sprint-g, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview, decisions G10–G12"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/99", title: "nsmith-/vibegraph#99"}
---
Two sessions on #99, stacked on #90.

## What landed

- **G-F6** ([report](sessions/G-F6-report.md)): plain, rich and ldjson modes
  chosen once per run; `--output-ldjson` (conflicts with `--tui`; refused by
  `check-events`); `schema_version` 1 on every line, kinds `log`, `progress`,
  `result`, `refusal`; ten error kinds; artifact format 20 recording
  per-multiplicity totals; `vibegraph inspect`; `generate`'s final object with
  the sample σ and its pull; the survey runs `--output-ldjson` and still reads
  old text. 9 schema tests failed on the base; 10/10 mutations killed.
- **G-HY5** ([report](sessions/G-HY5-report.md)): the TUI shows a decay's
  running width in eV (it labelled Γ × 3.894e8 as pb); the survey stores the
  refusal's error kind (database schema 3); only the card `integrate` reads is
  announced at INFO; a declined systematics download is `network`; a decay's
  result object tested. 15/15 mutations as expected.

Gate on the merged tree: workspace 1935 passed; banked `cli_integrate`,
`cli_generate_proton`; survey 31 tests; lints clean.

Closed: `integrate-output-not-machine-readable`.

## Filed

`ldjson-error-kinds-too-coarse` and `ldjson-stream-gaps`.
