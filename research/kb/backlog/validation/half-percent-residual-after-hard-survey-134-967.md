---
type: Backlog Item
title: "Cards 134 and 967 read about 0.5% above MadGraph even at hard_survey = 2 with equal weighting"
description: "With MadEvent's survey bias removed, VBS 134 reads +0.64% ± 0.25% and ALP → ττ 967 +0.53% ± 0.22% above MG; control 700 closes to 0.03%. vibegraph high, or the reference still low?"
area: validation
state: open
priority: medium
closes_when: "Per-subprocess σ from current main, summed into MadGraph's P-directories, is compared with R1_perP_hs2.txt (134) and R3_perP_967_hs2.txt (967). For 967 the residual is shown uniform or concentrated: if uniform, the decay side (BW window, fixed-width BR) is checked; if concentrated, the group is localised. Each card's residual is explained, or shown under 2σ with 20+ seeds per side. The ALP siblings 968 and 969 are read on the same footing."
blocked_by: []
opened: 2026-11-25
tags: [vbs, alp, sigma, madgraph-parity, survey, cms, residual]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: l-v1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-V1-report.md", title: "L-V1 report: the VBS σ excess is two MadGraph effects; the 134/967 residual and its plan"}
---
This is the residual the [L-V1 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-V1-report.md)
lists under Found. It is left after its two MadGraph effects are removed: the
2.9.18 sampling deficit, and MadEvent's default-survey bias. With MadGraph 3.7.1
at `hard_survey = 2` and the iterations re-combined with equal weights, the
readings are:

| card | process | vg/MG − 1 | MG χ²/dof |
|---|---|---|---|
| 134 | polarised VBS `w-{T} z{0} j j` | +0.64% ± ~0.25% | 0.33 |
| 967 | ALP → ττ, MLM | +0.53% ± 0.22% | 1.80 |
| 700 (control) | unpolarised VBS | +0.03% | |

MadGraph's χ²/dof rules out its error quoting as a shared cause.

The two cards' diagnoses stay separate. 967's 50 groups all decay through
`alp > ta+ ta-` at fixed width 1.0, so a residual uniform across groups points
at the decay: the Breit–Wigner window (`bwcutoff`), or how the BR enters at a
fixed width. 134 has no decay chain. Its cuts are identical to 700's, and 700
closes.

The first step is in L-V1's plan: localise per subprocess. On 967, start with
`g g > alp g g`, which is 52% of σ and about 8 min per MadGraph seed.

Card 967 had been tracked on the g2HDM high-mass σ item since sprint L's PR 12.
That item closes on V1's g2HDM seeds, and L-G1 showed the chirality fix cannot
be 967's cause: 967 has no charged-scalar FFS vertex. Its siblings 968 and 969
(+1.2–1.9% before the survey treatment) belong here too.
