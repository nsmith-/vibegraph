---
type: Backlog Item
title: The live-parity cancellation in fermi_sign is pinned by one process
description: "Dropping the live reversed-parity cancellation from fermi_sign fails one default-suite test and 3 of 270 ignored-sweep re-rootings, all on b b~ > c c~ e+ e- mu+ mu- QCD=0."
area: validation
state: open
priority: low
closes_when: "A default-suite test fails under that mutation on at least two unrelated processes, or the sweep's coverage of reversed bilinears is widened and run in CI."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, sign-convention, non-vacuity, rooting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: bh, resource: "../../sprints/amplitude-signs/sessions/B-H-report.md", title: "B-H report, Found"}
---
The relabelling and renumbering tests are blind to it; `rooting_soundness`'s
`literal_sigma_currents_are_rooting_invariant` is the default-suite witness.
