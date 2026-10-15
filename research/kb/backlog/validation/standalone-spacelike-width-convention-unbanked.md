---
type: Backlog Item
title: Standalone tables zero every width, so the spacelike-width convention is unbanked
description: "Standalone rows with a massive t-channel line compare at zero width; MadEvent's pattern (zero only on spacelike lines) was matched by a throwaway diagnostic, not a banked table."
area: validation
state: open
priority: low
closes_when: "gen_standalone_jamps.py can bank a row at MadEvent's width pattern (the width argument zeroed only on propagators of spacelike lines), and the t t~ + jet rows are banked and gated that way."
blocked_by: []
opened: 2026-10-15
tags: [standalone-jamps, widths, oracle-gap, ttx]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: g1, resource: "../../sprints/amplitude-signs/sessions/G1-report.md", title: "G1 report, Found 1 and the width diagnostic"}
---
`zero_widths = True` makes a row comparable where MadEvent zeroes a spacelike
line's width and `output standalone` does not. It also removes the timelike
widths that MadEvent keeps. The per-flow gate can see the difference: on the
`t t~ + jet` rows, all-WT standalone disagrees with this crate at ~1e-3, and
standalone patched to MadEvent's `P1_*` pattern agrees at ≤ 6.9e-16
([G1](../../sprints/amplitude-signs/sessions/G1-report.md)). That patch was a
throwaway; a `madevent_widths` option banking those tables would pin the
convention permanently.
