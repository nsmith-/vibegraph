---
type: Backlog Item
title: "MadEvent's boost anchor for an exchanged event with a massive parton is unverified"
description: "write_leshouche picks its anchor by pmass(1) (unwgt.f:642), leg 1 in amplitude order even after the IMIRROR = 2 flip; vibegraph anchors on the beam-1 parton. Code reading only."
area: validation
state: open
priority: low
closes_when: "One mirrored MadEvent event of a g b > … card with a massive b is compared with vibegraph's on its written beam energies and parton momenta, and vibegraph either matches or records the difference as a registered deviation."
blocked_by: []
opened: 2026-11-08
tags: [lhef, mirror, massive-incoming, madevent, events]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: hv2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-V2-report.md", title: "H-V2 report, Found 3"}
  - {id: unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L642", title: "unwgt.f: write_leshouche's pmass(1) test"}
---
#107 writes an exchanged (`IMIRROR = 2`) event as `R q` with the beams swapped.
For an event with a massive incoming parton, `write_leshouche` chooses its boost
anchor by `pmass(1)` (`unwgt.f:642`). That is leg 1's mass in amplitude order,
even after the `p(3,1) < 0` flip. So on an exchanged event it tests the beam-2
parton's mass while anchoring beam 1 (H-V2, from code reading).

vibegraph anchors on the physical beam-1 parton's mass, as before #107. If
H-V2's reading is right, the two differ on exchanged events whose two partons
have different masses (e.g. `g b` with a massive b). This touches the written
event, not σ.

Related: [massive-incoming-partons-ungated-beyond-sigma](massive-incoming-partons-ungated-beyond-sigma.md)
(no per-event oracle for massive incoming partons).
