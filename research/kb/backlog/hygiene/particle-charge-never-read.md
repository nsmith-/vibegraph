---
type: Backlog Item
title: "Particle::charge is parsed but nothing in production reads it"
description: "#145's parameter-valued charge is parsed and now pinned by a lib test, but no production code reads Particle::charge, so a wrong value cannot show in any σ or amplitude."
area: hygiene
state: open
priority: low
closes_when: "Either a consumer reads Particle::charge (e.g. a charge-conservation check on each vertex, as MadGraph's check_charge), with a test a wrong charge fails, or the field is documented as informational."
blocked_by: []
opened: 2026-11-19
tags: [ufo, dead-data]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-H1 (hygiene pass on #141, #144 and #145, PR #157), Found 2.
