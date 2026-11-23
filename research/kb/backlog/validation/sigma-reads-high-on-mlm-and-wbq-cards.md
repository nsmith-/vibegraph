---
type: Backlog Item
title: "σ reads 2.4% high on the QCD_bEnriched HT500to700 MLM cards"
description: "QCD_bEnriched HT500to700 (188/189) reads +2.4% above its gridpack's MadGraph 2.4.2 by 5 seeds, unadapted; not diagnosed. WBQtoLNuBQ (260) moved out as MadEvent default-survey bias."
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

**TCP ALP (967–969) moved out (sprint L, 2026-11-22).** MadGraph 3.7.1 on
card 967's own cards gives 1.864e-5 ± 4.6e-8 pb, agreeing with the gridpack's
2.9.18 (ratio 1.001), so its +1.2% is vibegraph's. It moves to
[`g2hdm-charged-higgs-high-mass-sigma-reads-high`](g2hdm-charged-higgs-high-mass-sigma-reads-high.md)
(sprint L's PR 12), which owns it from here. This item now covers 188/189 and 260 only.

**First step for 188 and 260.** Their gridpacks are MadGraph 2.4.2 (188) and
2.9.13 (260). A 3.7.1 regeneration on the exact cards is queued on the
validator host before any attribution to vibegraph. 188's reference is the
generation-log value 13220 ± 19.05 pb, as the survey uses it.

**WBQtoLNuBQ (260) moved out (sprint L, 2026-11-23)** as MadEvent
[default-survey bias](../../validation/madevent-default-survey-bias.md):
vg/MG 3.7.1 is 1.0073 ± 0.0057 against `hard_survey = 2` and 1.080 against
the default survey, from three MadGraph seeds per mode, so *consistent*, not
a calibrated pass. The measurement:
[card 260 by MadEvent survey mode](../../sprints/v023-orders-majorana-timeouts/card-260-madgraph-survey-modes.md).
This item now covers 188/189 only.
