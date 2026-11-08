---
type: Backlog Item
title: "Record-order colour-flow frequency checks cannot see a wrong pairing inside the CF automorphism group"
description: "Relabelling events by a CF automorphism leaves the record-order flow χ² at p 0.37–0.49, while pT-ranked and ΔR-split readings reject it; the pp_to_jj ICOLUP cell is the blind kind."
area: validation
state: open
priority: medium
closes_when: "The samples gate's colour cell reads flows kinematically resolved (flow × pT rank of identical particles, or × nearest-parton category), its floor is re-measured, and a mutation relabelling events by a CF automorphism fails it."
blocked_by: []
opened: 2026-11-07
tags: [colour, samples-gate, oracle-blind-spot, events]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hg1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md", title: "H-G1 report, Found 4"}
---
Found by H-G1 while checking #100's colour pairing under π. The 12-flow
`u u > g g u u` CF matrix has 16 automorphisms, and only one labels a group
member's events right. Relabelling the member's events by the automorphism M2
induces leaves the record-order flow-frequency χ² at p 0.368 (12 flows) and
0.494 (2→3). The pT-ranked and ΔR-split readings reject it at χ² 377.7/11 and
310.9/3.

The `pp_to_jj` samples cell's `ICOLUP` χ² is a record-order frequency comparison
of this kind, so it cannot see a wrong pairing within the automorphism group.
H-G1's readings were scratch (`H-G1-colour-patterns.py --ptrank` / `--split`).
