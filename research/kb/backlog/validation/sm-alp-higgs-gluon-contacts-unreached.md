---
type: Backlog Item
title: "SM_alp's ggg-H and gggg-H vertices reach no row, and the ALP UFO cannot be vendored"
description: "No checked process reaches SM_alp's V_13 (g g g H) or V_14 (g g g g H); SM_alp_UFO ships no licence text, so no ALP row can be banked in the repository."
area: validation
state: open
priority: low
closes_when: "V_13 and V_14 are shown to share Lorentz and colour structure with the banked SMEFTsim hggg/hgggg rows (gg_to_hg_chg, gg_to_hgg_chg) by a structure comparison, or are reached by a toy-UFO row; and the SM_alp licence is resolved with its authors or the decision not to vendor is recorded."
blocked_by: []
opened: 2026-11-08
tags: [amplitudes, sign-convention, alp, ufo, licence, oracle-gap]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: ha2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-A2-report.md", title: "H-A2 report, §3: ALP UFO not vendored"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log, 2026-11-07: A, the rule (Found: V_13/V_14 unreached)"}
---
The TCP diagnosis checked SM_alp's ALP–gluon vertices against MadGraph
standalone. It did so on the CERN host, with unbanked tables. Two of the
model's vertices reached no process in it: V_13 (g g g H) and V_14 (g g g g H).
#108's rule gives them −1 and +1 by momentum parity.

The SMEFTsim rows `gg_to_hg_chg` (hggg, −1) and `gg_to_hgg_chg` (hgggg, +1)
pin those signs for O_HG. Whether SM_alp's structures are the same is not
checked (likely, inferred).

The ALP rows cannot be banked. `SM_alp_UFO` (FeynRules 2.3.26, 2017) has no
licence file and no licence text, so it cannot go under `validation/ufo/`.
`gg_to_hg_chgtil` covers the a·GG̃ contact in a model that can be vendored.
[bsm-census-models-not-vendored](bsm-census-models-not-vendored.md) is the
same licence question for four decay-chain models.

The residual on the trial `g g > alp g g` table (6.1e-11) was not localised.
The SMEFTsim analogue's residual was the Higgs width on spacelike lines (H-A2
§3, removed by `zero_widths`). That may hold here too (inferred).
