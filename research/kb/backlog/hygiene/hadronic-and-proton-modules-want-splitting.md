---
type: Backlog Item
title: "hadronic.rs and proton.rs want splitting, and budget depends on hadronic in a cycle"
description: "hadronic.rs (4.7k lines) is shared scale and subprocess plumbing plus the fixed-beam integrand; proton.rs (6.5k) is flavour grouping plus ProtonIntegrand; budget and hadronic import each other."
area: hygiene
state: open
priority: low
closes_when: "hadronic and proton are split along the boundaries R-E.3/R-E.4 propose (or better ones), and the channel-combination helpers move out of hadronic so budget no longer imports it."
blocked_by: []
opened: 2026-10-10
tags: [module-boundaries, hadronic, proton]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-e-report, resource: "../../sprints/hygiene/sessions/R-E-report.md", title: "R-E report (hygiene sprint)"}
---
Proposed boundaries: `hadronic/{scale_source, subprocess, initial_state, fixed_beam}.rs`; `proton/{groups, integrand}.rs` split at the `ProtonIntegrand` seam. `combine_channels`, `ChannelIntegration`, `channel_share`, `CHANNEL_STREAM_BASE` and `VEGAS_NBINS` belong in `budget.rs` (R-E.3, R-E.4, R-E.5).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-E](../../sprints/hygiene/sessions/R-E-report.md).
