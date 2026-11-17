---
type: Backlog Item
title: "VBS W/Z + jj with stable bosons reads 3–13% above MadGraph"
description: "Five CMS VBS cards (QED=4 QCD=0, etaj 6.5, mmjj 10, ptj 10) read high by 5 seeds: +9%, +13%, +10%, +3.4% (polarised) and +5% (unpolarised). Not yet diagnosed."
area: validation
state: open
priority: high
closes_when: "The excess is attributed, with per-event or per-diagram comparisons against MadGraph rather than σ alone, on one polarised and one unpolarised card. It is then fixed so the 5 cards match MadGraph by the multi-seed criterion, or registered as a deviation with its predicted size."
blocked_by: []
opened: 2026-11-17
tags: [vbs, sigma, polarization, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from stage 2 of its 1009-gridpack survey (5 seeds,
v0.2.1, no card adaptation).

| Gridpack | Polarised | vg/MG − 1 |
|---|---|---|
| 134 | yes | +9% |
| 135 | yes | +13% |
| 698 | yes | +10% |
| 702 | yes | +3.4% |
| 700 | no | +5% |

All five cards use stable W/Z, `QED=4 QCD=0`, `etaj = 6.5`, `mmjj = 10` and
`ptj = 10`. The unpolarised card is also high, so polarisation is not the whole
story. Candidates to check: the treatment of t-channel photon exchange at low
mmjj, the scale choice, and the Γ = 0 versus finite-width handling of
t-channel bosons.
