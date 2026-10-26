---
type: Backlog Item
title: "Two docs predate the CKM rows they describe"
description: "sm_ckm_no_b_mass.rs says no coupling table is banked for the CKM sector, and MG_VALIDATED_PROCESSES' doc says it holds every subprocess the amplitude gate covers (19 of 51)."
area: hygiene
state: open
priority: low
closes_when: "sm_ckm_no_b_mass.rs's 'What composition cannot see' names the banked ckm_no_b_mass coupling table, and MG_VALIDATED_PROCESSES' doc says which gate rows it holds and why (default restriction only)."
blocked_by: []
opened: 2026-10-26
tags: [docs, tests, ckm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v3, resource: "../../sprints/ckm-no-b-mass/sessions/V3-report.md", title: "ckm-no-b-mass V3 report, Found 5 and 6"}
---
- `vibegraph-lib/tests/sm_ckm_no_b_mass.rs` module doc, "The CKM sector against
  MadGraph": "No coupling table is banked at `ckm`". Since the bank,
  `validation/madgraph/couplings/ubx_to_epve_ckm_no_b_mass.json` holds MadGraph's
  whole `coupling_dict` at this card.
- `vibegraph-lib/src/helas/eval/compile.rs` above `MG_VALIDATED_PROCESSES`: "It
  is every concrete subprocess that gate covers". The list has 19 entries against
  the gate's 51 rows, and it compiles on the default model, so it cannot hold
  rows at other restrictions, where `V_ub = 0` prunes the CKM vertex
  ([V3](../../sprints/ckm-no-b-mass/sessions/V3-report.md)).
