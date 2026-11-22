---
type: Backlog Item
title: "σ reads 1–3% high on QCD_bEnriched MLM, WBQtoLNuBQ and TCP ALP MLM cards"
description: "QCD_bEnriched HT500to700 (+2.6%, pull 8.8), WBQtoLNuBQ (+2.7%) and TCP ALP MLM (+1.2–1.9%) read above MadGraph by 5 seeds, unadapted; not diagnosed."
area: validation
state: open
priority: medium
closes_when: "Each card's excess is attributed (MLM merge sequence or scale per event, b-mass or flavour handling, PDF) against MadGraph's own record, then fixed or registered as a deviation with its predicted size."
blocked_by: []
opened: 2026-11-17
tags: [sigma, madgraph-parity, mlm, b-quarks, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from stage 2 of its 1009-gridpack survey (5 seeds,
v0.2.1, no card adaptation):

| Gridpacks | Card | vg/MG − 1 |
|---|---|---|
| 188, 189 | QCD_bEnriched HT500to700, MLM | +2.6%, pull 8.8 |
| 260 | WBQtoLNuBQ | +2.7% |
| 967–969 | TCP ALP, MLM | +1.2–1.9% |

Four of the six involve b quarks in the hard process. A related item is
[`sigma-reads-above-local-madevent-by-a-few-permille`](sigma-reads-above-local-madevent-by-a-few-permille.md).

**Post-release campaign `v022-1009` (v0.2.2, 5 seeds, unadapted, 2026-11-22).**
The excess persists, as it was on v0.2.1:

| Gridpacks | vg/MG | mean pull |
|---|---|---|
| 188, 189 (QCD_bEnriched) | 1.024 | 7.9 |
| 260 (WBQ) | 1.027 | 5.0 |
| 967 / 968 / 969 (TCP) | 1.013 / 1.019 / 1.012 | 5.0 / 5.0 / 3.2 |
