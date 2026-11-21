---
type: Backlog Item
title: "WH HToAATo4B reads +0.05% and +0.11% above MadGraph after the repeated-decay fix"
description: "After #150, the WH HToAATo4B card reads vg/MG 1.00052 (M-15) and 1.00110 (M-40) at 5 seeds each, +0.08% ± 0.04% combined; five seeds cannot calibrate it, and the predicted interference is below 1e-8."
area: validation
state: open
priority: low
closes_when: "20 or more seeds per side at M-15 and M-40, with a MadEvent budget ladder, show the ratio consistent with 1, or the offset is attributed (scale, PDF or width handling) and fixed or registered as a deviation."
blocked_by: []
opened: 2026-11-17
tags: [sigma, decay-chains, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-D1 session (its report, `sessions/K-D1-report.md` in the
sprint folder). The pairing interference that separates our |A+B|²/4 from
MadGraph's |A|²/2 is (Γ/M)²-suppressed, under 1e-8 here, so it cannot explain
the offset. The VBFH and TTH HToAATo4B cards, and the other masses, are unrun.
