---
type: Backlog Item
title: "The <MGRunCard> header now writes pdgs_for_merging_cut's default list"
description: "MadEvent replaces pdgs_for_merging_cut per process; vibegraph writes default_setup's list, which the ignored banner oracle may flag on rows whose card leaves it unset."
area: validation
state: open
priority: low
closes_when: "validate_mlm_dumps::mg_run_card_matches_madevents_banner runs against the matched runs, and either passes or the field is written as MadEvent does."
blocked_by: []
opened: 2026-10-10
tags: [lhef, run-card, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-f-report, resource: "../../sprints/hygiene/sessions/F-F-report.md", title: "F-F report (hygiene sprint)"}
---
F-F Found 4–5 (list payloads also compare as raw text; enforcement over list fields needs list normalisation).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-F](../../sprints/hygiene/sessions/F-F-report.md).
