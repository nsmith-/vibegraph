---
type: Sprint Record
title: "v0.2 CLI output that misleads"
description: "Sprint G part three (#90): integrate prints σ to its error's digits, χ²/dof as n/a below two iterations and the factorization-scale choice instead of μF = 0; the artifact (format 19) records both absences; the survey parser reads the new lines. Three items closed."
closed: 2026-11-04
status: draft
tags: [cli, output, artifact, sprint-g, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: report, resource: "sessions/G-HY3-report.md", title: "G-HY3 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/90", title: "nsmith-/vibegraph#90"}
---
One session, [G-HY3](sessions/G-HY3-report.md), on #90, stacked on #88.

## What landed

- σ and its error print to the error's second significant figure
  (`value_and_error`): a 9e-6 pb σ reads `8.6812e-6 ± 2.3e-9`.
- `VegasResult::chi2_per_dof` is `Option<f64>`; fewer than two entering
  iterations print `n/a` and record `None`.
- `FactorizationScale` names a constant, a per-beam or a dynamical scale (with
  its choice). A fixed μF with running μR used to record 0 and now records the
  fixed value.
- Artifact format 19; 18 refused.
- The survey parser reads the new lines, and the per-multiplicity line it had
  silently stopped matching.

Evidence: 13 of 13 Rust and 2 of 2 parser mutations killed; workspace 1897
passed; banked `cli_integrate` and `cli_generate_proton` green; CI green on
05cf757, `banked` included.

Closed: `sigma-printed-to-fixed-decimals`,
`iteration-chi2-printed-as-zero-with-one-result`,
`dynamical-scale-summary-prints-mu-f-zero`.

## Filed

`cli-output-followups` (the stale format floor in
`artifact-format-versioning.md`, the report collator's χ²/dof default, the
survey's single-tag parts and missing μF column).
