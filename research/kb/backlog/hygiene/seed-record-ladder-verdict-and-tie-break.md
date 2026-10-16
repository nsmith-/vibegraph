---
type: Backlog Item
title: "seed-record does not print the ladder verdict, and breaks p-value ties by order"
description: "D1's 20 + 20 ladder verdict is computed by hand from a record; a 20-seed confirmation has no driver recipe; SCATTER's worst channel is chosen by order when several p-values floor to 0."
area: hygiene
state: open
priority: low
closes_when: "seed-record prints the ladder verdict for a 20 + 20 record (measured-spread σ, re-bank at 4x beyond 2σ), the driver documents or offers the 20-seed confirmation run, and SCATTER's worst channel breaks p ties on χ²."
blocked_by: []
opened: 2026-10-16
tags: [seeds, refdata, seed-record, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
Three gaps in `validation-report/src/bin/seed-record.rs` and
`validation/madgraph/bank_seeded_sigma.sh`, found closing
`sprints/sigma-reference-scatter`:

- **Ladder verdict.** D1's fourth amendment decides a 20 + 20 ladder on σ =
  √(s₁ₓ²/n₁ₓ + s₄ₓ²/n₄ₓ) from the measured seed spreads. The `SCREEN` line
  uses the seed-policy error instead (the larger of quoted and spread/√n), so
  the first ladder (`ll_to_qqx_toy_dipole`, −0.87σ) was read by hand.
- **No 20-seed recipe.** A row the look-elsewhere SCATTER rule flags is
  confirmed on 20 seeds; nothing in the driver says how (`SEEDS`, a separate
  `SEEDED_OUT`, which record is banked).
- **Tie-break.** `puruspe::gammq` returns 0 below about 1e-16, so several
  channels can tie at p = 0. The printed worst channel is then the first in
  order: on `uux_to_ccx_emmm_qcd0` it named χ²/dof 34.6 over one at 289.
  The verdict is unaffected.
