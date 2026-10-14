---
type: Backlog Item
title: "The three ChannelIntegrand implementors duplicate their methods"
description: "ProtonIntegrand, FixedBeamIntegrand and MultiplicitySum copy split_point, adapt_grids(_budget) and the configuration-weight draw; integrate and generate assemble the same integrand twice."
area: hygiene
state: open
priority: medium
closes_when: "One ConfigurationWeights type and provided ChannelIntegrand methods replace the copies, and integrate and generate build their integrands through one builder per beam mode."
blocked_by: []
opened: 2026-10-10
tags: [duplication, hadronic, cli]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-e-report, resource: "../../sprints/hygiene/sessions/R-E-report.md", title: "R-E report (hygiene sprint)"}
  - {id: r-f-report, resource: "../../sprints/hygiene/sessions/R-F-report.md", title: "R-F report (hygiene sprint)"}
  - {id: r-d-report, resource: "../../sprints/hygiene/sessions/R-D-report.md", title: "R-D report (hygiene sprint)"}
---
- Identical bodies: `split_point`, `adapt_grids`, `adapt_grids_budget`, `use_onshell_veto` across `proton.rs`, `hadronic.rs` and `multiplicity.rs`; the `scale_channel` and `select_event` weight blocks share one skeleton (R-E.2).
- `vibegraph-cli` builds the integrand in `integrate.rs` and again in `generate.rs`, and generate must rebuild exactly what integrate trained (R-F.4).
- `MultiChannel` and `ScaledMultiChannel` duplicate their combiner logic (R-D.14, the part not fixed).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-E](../../sprints/hygiene/sessions/R-E-report.md), [R-F](../../sprints/hygiene/sessions/R-F-report.md), [R-D](../../sprints/hygiene/sessions/R-D-report.md).
