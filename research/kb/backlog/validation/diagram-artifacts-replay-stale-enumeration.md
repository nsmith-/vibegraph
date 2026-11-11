---
type: Backlog Item
title: "Diagram and integrate artifacts replay an older build's enumeration silently"
description: "A diagrams.bin.zst banks the Fermi sign, symmetry factor, routing and diagram list; a later build that fixes enumeration or conversion integrates the old physics, and the probe agrees with itself."
area: validation
state: open
priority: high
closes_when: "Both artifacts record an enumeration version; a file recording another is refused by name with the remedy (a test per artifact, failing before the change); and a known-answer test pins the enumeration output of a fixed card set to that version, shown by mutation to fail on an unbumped change to conversion."
blocked_by: []
opened: 2026-11-11
tags: [artifacts, enumeration, diagrams, hard-errors, release]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
sources:
  - {id: ia, resource: "../../pipeline/integrate-artifact.md", title: "Integrate artifact: the amplitude probe's blind spots"}
  - {id: sprint, resource: "../../sprints/v021-fixes/sprint.md", title: "Sprint J plan, PR 2"}
---
`vibegraph enumerate` writes `diagrams.bin.zst`, and `integrate` reads it
instead of enumerating (#121). The file banks what enumeration and the
conversion of feyngraph's diagrams computed: each diagram's Fermi `sign`,
`symmetry_factor`, momentum routing, interaction indices, and the diagram list
itself. The integrate artifact embeds the same sets, and `generate` reads them.

The format version covers a change to the schema, and the model digest a change
to the model. Nothing covers a change to what enumeration or conversion
produces. The amplitude probe recompiles from the banked diagrams, so it
agrees with itself: once a build fixes a sign or a missing diagram, an old
artifact integrates the old physics with no warning
([integrate artifact](../../pipeline/integrate-artifact.md), check 7's blind
spot). v0.2.1 is the first release that lets users create these files.

Resolution (sprint J's plan): record an enumeration version in both
artifacts, refuse a mismatch by name with the remedy (enumerate or integrate
again), and pin the version with a known-answer test over a fixed card set's
`diagram_set_digest`, so an unbumped change to the output fails CI.
Re-enumerating on read was rejected: at high multiplicity it costs minutes.
