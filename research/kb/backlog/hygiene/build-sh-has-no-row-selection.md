---
type: Backlog Item
title: "build.sh cannot rebuild one banked process directory"
description: "build.sh rebuilds whatever output directory is absent; it has no ROWS= selection, so a single-row rebank means moving directories aside by hand or pointing SCRIPTS_DIR at a copy."
area: hygiene
state: open
priority: low
closes_when: "ROWS=<row …> pixi run -e madgraph build-diagrams rebuilds exactly the named rows, as bank_seeded_sigma.sh and the gen_* scripts accept."
blocked_by: []
opened: 2026-10-26
tags: [tooling, madgraph, rebank]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f2, resource: "../../sprints/pdf-from-run-card/sessions/F2-report.md", title: "pdf-from-run-card F2 report, Found 3"}
  - {id: pr74, resource: "https://github.com/nsmith-/vibegraph/pull/74#issuecomment-6368130522", title: "nsmith-/vibegraph#74, the M3's HT bank"}
---
`validation/madgraph/build.sh` has no `ROWS` handling (`grep -n ROWS` returns
nothing). Sprint D needed single-row builds three times: the `pp_to_ll` /
`pp_to_ll_qcd0` rebank, and the two HT rows, where a bare `build-diagrams`
started regenerating eight unrelated rows missing from the M3's work area. Each
time the M3 called `build.sh`'s `build_main` with `SCRIPTS_DIR` pointed at the
wanted scripts ([F2](../../sprints/pdf-from-run-card/sessions/F2-report.md),
[#74](https://github.com/nsmith-/vibegraph/pull/74#issuecomment-6368130522)).
