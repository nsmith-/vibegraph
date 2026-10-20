---
type: Backlog Item
title: "Written fixed-beam events: MadGraph's source boosts to the lab, a concept says it writes the CM frame"
description: "unwgt.f boosts every written event into the lab frame, lpp = 0 included; fixed-beam-kinematics.md says MadGraph writes the CM frame, from banked p3 r3 events; vibegraph writes the CM frame."
area: validation
state: open
priority: medium
closes_when: "A banked MadGraph fixed-beam event file with unequal beam momenta is read (its beam records and Σpz settle lab or CM), and either the concept is corrected and vibegraph's written frame made to match, or the source reading is shown not to apply, by a test on the banked file."
blocked_by: []
opened: 2026-10-20
tags: [lhef, fixed-beam, frames, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f2, resource: "../../sprints/cut-refusal/sessions/F2-report.md", title: "cut-refusal F2 report, Found 1"}
---
`Template/LO/SubProcesses/unwgt.f:633-717` (b7687064) boosts every written
event into the lab frame, the `lpp = 0` branch included (:670-697, boost at
:703-706). [Fixed-beam kinematics](../../phase-space/fixed-beam-kinematics.md)
says MadGraph writes the CM frame, citing banked p3 r3 events with Σpz = 0,
where lab and CM differ by y = 5.386e-3. vibegraph writes the CM frame
(`EBMUP = [ebeam1, ebeam2]`). The banked event files were not in the
worktree that found this. Reading one beam record settles it: 250/250 means
lab, 248.70/251.30 means CM.
