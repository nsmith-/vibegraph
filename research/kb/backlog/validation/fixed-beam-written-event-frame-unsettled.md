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

**Measured on the CERN host (2026-10-21).** Both banked fixed-beam rows with
unequal masses (`p3r3_to_p3r3_toy_epsilon`, `_toy_sextet`: ebeam 250/250,
MP3 = 60, MR3 = 70) write the CM frame: first event E = 248.696 / 251.296,
pz = ±241.350; Σpz = 0 for all 10000 events. No banked fixed-beam output has
ebeam1 ≠ ebeam2. Traced in `unwgt.f:633-717` and replayed in float64 (not run
through MadGraph): the lab boost is computed (β = −5.3864e-3), then the
`numerical_accuracy` check `pb(0,1).gt.ebeam(1).or.pb(0,2).gt.ebeam(2)` fires
because the boosted beam-2 energy is 250.00000000000006 > 250, one ulp over;
the "old method" fallback uses η = √(xbk1·ebeam1/(xbk2·ebeam2)) = 1, an identity
boost, so the record stays CM. The CM output is therefore an ulp-sensitive
fallback, not a design choice; symmetric massless or equal-mass beams are
unaffected (β = 0). Listed in [MadGraph defects](../../validation/madgraph-defects.md).

**Banking request to settle it:** one more asymmetric-mass fixed-beam MadGraph
run (masses 50/80, say) and a look at which frame its records are in.
vibegraph's CM output matches the banked rows, so no code change follows now.
