---
type: Backlog Item
title: "Gaps G-HY2 left: the proton NoUsableWeight path untested, three tests inverting the log map at tau_max = 1, and the tau rule settled for fixed beams"
description: "The proton path's NoUsableWeight is reached by no test and its scan block is duplicated; three proton tests assume tau_max = 1; ProcessShape::of picks a tau map for fixed-beam runs that draw no tau."
area: hygiene
state: open
priority: low
closes_when: "A banked cli_generate_proton case with --scan-points 1 (or one shared scan helper) pins the proton refusal; the three tests use tau_range/log_tau; fixed-beam runs record no tau choice, each with a test."
blocked_by: []
opened: 2026-11-05
tags: [phase-space, tests, hygiene, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
G-HY2 Found 2–4 ([report](../../sprints/v02-diagnoses/sessions/G-HY2-report.md)): `generate.rs` `generate_sample`/`generate_proton_sample`; `proton.rs` ~6286, ~6391; `hadronic.rs:2105`.
