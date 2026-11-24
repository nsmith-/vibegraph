---
type: Backlog Item
title: "On g2HDM card 894, h- events' matched clustering and mgrwt records are unchecked against MadEvent"
description: "generate now writes card 894's h- events with their own resonance codes and colours, but their clustering scales and <mgrwt> records have never been compared with MadEvent's."
area: validation
state: open
priority: medium
closes_when: "On card 894, vibegraph's h- events are compared with MadEvent's per event or by distribution: SCALUP, AQCDUP and the <mgrwt> records (the clustering sequence and the αs and PDF reweighting factors). Either they agree within the gate's tolerance or the difference is filed."
blocked_by: []
opened: 2026-11-23
tags: [g2hdm, mlm, clustering, mgrwt, lhef, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lg2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-G2-report.md", title: "L-G2 report: Found F1"}
---
Sprint L's PR 15 made card 894 generate. Its h- events come from a flavour
group whose representative is an h+ subprocess, and the fix gives each member
its own resonance codes and colours. What the clustering reads for those events,
and what the `<mgrwt>` records carry, has not been checked against MadEvent:
the PR's evidence is the event record's intermediates only.
