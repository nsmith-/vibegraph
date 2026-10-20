---
type: Backlog Item
title: "The collider energy is taken as ebeam1 + ebeam2, wrong for unequal beams"
description: "integrate and generate set the hadronic √s to ebeam1 + ebeam2; for head-on massless beams √s = 2√(ebeam1·ebeam2), so any asymmetric-beam run is at the wrong energy."
area: validation
state: open
priority: medium
closes_when: "√s for unequal beams is 2√(ebeam1·ebeam2) (or MadGraph's own convention, cited), or unequal beams are refused, by a test; and the cut-refusal ceiling's doc follows."
blocked_by: []
opened: 2026-10-20
tags: [beams, kinematics, runcard, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/cut-refusal/sessions/HY1-report.md", title: "cut-refusal HY1 report, Found 2"}
---
`sqrt_s_had = rc.ebeam1 + rc.ebeam2` (`vibegraph-cli/src/integrate.rs` ~:931,
`vibegraph-cli/src/generate.rs` ~:1713, and the doc at `vibegraph-lib/src/runcard.rs` ~:188).
For head-on massless beams s = 4·ebeam1·ebeam2, so the sum equals √s only when
the beams are equal. Every validated row uses equal beams, so no gate sees it.
Unchecked: what MadGraph does for unequal beams on each `lpp` path (the boost of
the partonic frame matters as well as √s).

Found by the [cut-refusal hygiene review](../../sprints/cut-refusal/sessions/HY1-report.md).
The no-op pT ceiling there uses the same sum and is only looser, so it is sound.
