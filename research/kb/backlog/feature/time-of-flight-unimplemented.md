---
type: Backlog Item
title: "time_of_flight other than the off value is refused"
description: "CMS cards 120 and 121 set time_of_flight = 0, which vibegraph refuses by name; they integrate only with the line stripped. MadGraph writes a decay-length vertex displacement into the LHE file."
area: feature
state: open
priority: low
closes_when: "time_of_flight ≥ 0 writes MadGraph's per-particle vertex lifetime into the event record (with the LHE field checked against a MadGraph sample), or the off-value reading is documented as the only one supported in release-scope."
blocked_by: []
opened: 2026-11-21
tags: [run-card, lhef, cms-gridpacks, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-21}
sources:
  - {id: cern, resource: "../../sprints/v022-smeft-and-refusals/log.md", title: "Sprint K log, CERN targeted check on 400c88f9"}
---
This was found in the CERN targeted check: the refusal is unchanged from
v0.2.1, and σ agrees with the line stripped. It is an event-record feature
only: it does not change σ.
