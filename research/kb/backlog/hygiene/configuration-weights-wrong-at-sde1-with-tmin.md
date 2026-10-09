---
type: Backlog Item
title: Configuration weights would be wrong at SDE_strategy = 1 with tmin_for_channel set
description: "compile_configuration_weights uses channel_cuts alone whenever not (SDE = 1 and tmin_for_channel = -1); MadGraph weights by AMP2·CC at SDE = 1. Unreachable while tmin_for_channel is refused."
area: hygiene
state: open
priority: low
closes_when: "SDE_strategy = 1 with tmin_for_channel != -1 weights configurations by AMP2 × channel_cuts, pinned by a unit test, before or in the change that lifts the tmin_for_channel refusal."
blocked_by: []
opened: 2026-10-09
tags: [sde-strategy, configuration-draw, latent-bug, run-card]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code, resource: "../../../../vibegraph-lib/src/hadronic.rs", title: "hadronic.rs compile_configuration_weights (~:1482-1505)"}
  - {id: classes, resource: "../../../../vibegraph-lib/src/runcard/classes.rs", title: "runcard/classes.rs R_SDE_STRATEGY (~:93-103) and the tmin_for_channel refusal (~:410)"}
---
`compile_configuration_weights` (`vibegraph-lib/src/hadronic.rs` ~:1482-1505)
returns `None` (weight by `AMP2`) only when `SDE_strategy == 1` and
`tmin_for_channel == -1`. Otherwise it returns the channel sets, whose
`channel_cuts` replace the amplitude. That is right at `SDE_strategy = 2`. At
`SDE_strategy = 1` with `tmin_for_channel` set, MadGraph weights configuration
`c` by `AMP2_c × CC_c`, as the run-card class's own rationale says
(`runcard/classes.rs` ~:93-103). The code would drop `AMP2`.

It cannot happen today: `tmin_for_channel` is `IgnoredPhysics`, refused off its
default (`classes.rs` ~:410). It becomes a wrong configuration draw (scales and
colour flows) the moment that refusal is lifted. Found by Phase 2 drafter D12;
confirmed against the code.
