---
type: Backlog Item
title: Six test files re-implement reading a banked MadGraph param card
description: "Six oracle test files each join and parse a banked table's param card by hand; tests/common::banked_param_card now does it once."
area: hygiene
state: open
priority: low
closes_when: "Every test that reads a banked amplitude table's param card goes through tests/common (keeping each site's non-empty assertion), and PYTHON_REL_TOL is defined once in tests/common."
blocked_by: []
opened: 2026-10-20
tags: [tests, duplication, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/ckm-no-b-mass/sessions/HY1-report.md", title: "ckm-no-b-mass HY1 report, Found 1"}
---
Sites: `coupling_oracle.rs` (`card_text`), `validate_scale_couplings.rs` ~:73,
`standalone_jamps.rs` ~:486, `polarization_frame.rs` ~:52,
`validate_hadronic.rs` ~:703, `amplitude_oracle.rs` ~:784. The helper
`common::banked_param_card` (and `common::lha_slot`) landed with the
[ckm-no-b-mass sprint](../../sprints/ckm-no-b-mass/sprint.md). `PYTHON_REL_TOL = 1e-13`
is defined in both `coupling_oracle.rs` and `sm_ckm_no_b_mass.rs`.
