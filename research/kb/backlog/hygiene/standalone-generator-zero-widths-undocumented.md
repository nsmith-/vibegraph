---
type: Backlog Item
title: "gen_standalone_jamps.py does not document that a row with a massive t-channel fermion needs zero_widths"
description: "Without zero_widths, standalone keeps spacelike top widths that MadEvent zeroes, which leaves about 1e-3 per-diagram phase residuals."
area: hygiene
state: open
priority: low
closes_when: "The generator's docs (or a guard in it) state when zero_widths is required, and a row with a massive t-channel fermion that is banked without it is refused or warned."
blocked_by: []
opened: 2026-11-26
tags: [standalone, test-tooling, widths, docs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
sources:
  - {id: l-d2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-D2-report.md", title: "L-D2 report, Found 4"}
---
Found by sprint L's L-D2. Its first HC and dim6top scratch tables were banked
without `zero_widths`. They showed about 1e-3 per-diagram phase residuals,
which vanished with `zero_widths` on. No committed row is affected.
