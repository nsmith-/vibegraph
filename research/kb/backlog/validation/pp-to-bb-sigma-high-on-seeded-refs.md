---
type: Backlog Item
title: "pp_to_bb rows read 0.07–0.22 % above their seeded references"
description: "pp_to_bb, pp_to_bb_fixed and pp_to_bb_qcd2 sit above refdata-10 on two independent five-seed sets this side, and MadEvent's iteration weighting does not explain it; a 20-seed reference is requested."
area: validation
state: needs-user
priority: medium
closes_when: "The 20-seed MadEvent records for pp_to_bb and pp_to_bb_fixed (banking request on PR #42) are read, and the offset is either inside the new references' error or attributed to one side, with an adaptation-free hadronic 2 → 2 estimate if needed."
blocked_by: []
opened: 2026-10-17
tags: [sigma, hadronic, bb, madevent-reference, seeds]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/sigma-pulls-lean-high/sessions/V1-report.md", title: "sigma-pulls-lean-high V1 report, step 4 and Found"}
---
On ten seeds this side the three rows pull +2.11, +2.57, +1.93 against the
reported seed mean, and +1.39, +1.59, +1.02 against the equal-weight mean of
the 4× seeds. On five fresh seeds (20261201–05) they read +0.071 %, +0.216 %
and +0.078 % above the reference, which repeats the gate's own five seeds
([V1](../../sprints/sigma-pulls-lean-high/sessions/V1-report.md)). EW − R is
only about +0.015 % on these rows, so MadEvent's iteration weighting does not
explain it. Waiting on the banking request's item 2 (20 seeds at 1× on
`pp_to_bb` and `pp_to_bb_fixed`). If that leaves the offset, the next oracle
is a fixed-density 2 → 2 estimate with a 1/(t − m_b²) density, about one
session.
