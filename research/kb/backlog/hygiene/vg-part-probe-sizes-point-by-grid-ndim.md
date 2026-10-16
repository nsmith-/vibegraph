---
type: Backlog Item
title: "probe_pta_windows_against_madgraph panics: vg_part sizes the point by the grid"
description: "validate_samples.rs's vg_part builds a point of channel_grid_ndim() coordinates, but event_in_channel needs point_ndim(), which adds the scale-draw uniforms; the ignored probe panics."
area: hygiene
state: open
priority: low
closes_when: "vg_part sizes its point by point_ndim() and fills the scale-draw uniforms, and probe_pta_windows_against_madgraph runs to completion."
blocked_by: []
opened: 2026-10-16
tags: [testing, probes, ee-to-mumua]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
Found by V1 in `sprints/sigma-ee-mumua`. `vg_part` in
`vibegraph-lib/tests/validate_samples.rs` builds `x` with
`channel_grid_ndim()` (5 on `ee_to_mumua`); `event_in_channel` asserts a point
of `point_ndim()` (6: five map coordinates and one scale-draw uniform,
`hadronic.rs:2827`). The `#[ignore]`d `probe_pta_windows_against_madgraph`
therefore panics before measuring anything. V1's scratch copy sized `x` by
`point_ndim()` and ran.
