---
type: Backlog Item
title: "The decay-chain census cannot pin BSM survey cards, because their UFOs are not vendored"
description: "The census's Rust side reads only the built-in SM, so TTToHcToWA, TpTp, VLLD and VLLVLL have no committed census row; pinning them needs their UFOs vendored after a licence check."
area: validation
state: open
priority: low
closes_when: "The licences of the four models are checked, the permitted ones are vendored with a model path per census case, and their census rows are committed; or the decision not to vendor is recorded."
blocked_by: []
opened: 2026-11-07
tags: [census, decay-chains, ufo, bsm, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hd3, resource: "../../sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "H-D3 report, Found: the BSM census needs vendored models"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: #93 Found triage, item 5"}
---
Found by H-D3 (#93 Found 5; the manager's triage files it). The decay-chain
census's Rust side reads only the built-in SM. Pinning TTToHcToWA, TpTp, VLLD
and VLLVLL in the committed census takes vendoring their UFOs, after a licence
check, and a model path per case. Until then the survey's BSM decay-chain cards
are checked only by runs, not by a census against MadGraph's matrix elements.
