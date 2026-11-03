---
type: Backlog Item
title: uux_to_ccx_emmm_qcd0 had a low seed at 1.2M before the pair-order fix
description: "In F1's 5-seed uux control, seed 11 at 1.2M read 7.153e-7 ± 9.0e-10, 3.6e-9 below the other four (χ²/dof 4.09); under the symmetric pair draw it read 7.180e-7. Five seeds cannot credit the fix."
area: validation
state: open
priority: low
closes_when: "Seed 11's recorded top points are decomposed with probe_2to6_tail_points (VG_TAIL_DIR=uux_to_ccx_emmm_qcd0) on the control build, and a 20-seed uux ladder at 600k and 1.2M under the default says whether its spread now falls with budget."
blocked_by: []
opened: 2026-10-31
tags: [two-to-six, heavy-tail, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
sources:
  - {id: f1, resource: "../../sprints/alpha-share-tails/sessions/F1-report.md", title: "alpha-share-tails F1 report, uux and Found 3"}
---
uux has the same three ZH channels with the same pair order (288, 305, 322)
at α 2–4× bbx's (T1 Found 3). F1's per-point traces for this run were kept in
a session scratchpad that has since been retired, so the decomposition needs
a rerun on the control build.
