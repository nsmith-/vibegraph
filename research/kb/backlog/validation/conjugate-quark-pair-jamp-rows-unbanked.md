---
type: Backlog Item
title: "Multi-flow quark conjugate pairs have no committed per-flow JAMP row against MadGraph standalone"
description: "q q > q g q and q q > g g q q have no gen_standalone_jamps.py row, so per-flow JAMP order and value are not pinned for either member of a conjugate pair that #100 groups under π."
area: validation
state: open
priority: low
closes_when: "gen_standalone_jamps.py has rows for u d > d g u, u~ d~ > d~ g u~, u u > g g u u and u~ u~ > g g u~ u~, registered in standalone_jamps.rs, and the gate passes on all four."
blocked_by: []
opened: 2026-11-07
tags: [colour, standalone, jamps, conjugate-pairs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hg1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md", title: "H-G1 report, Found 5"}
---
Found by H-G1. The per-flow JAMP comparison against MadGraph standalone has no
row for a multi-flow quark pair: `q q > q g q` and `q q > g g q q` are absent
from `gen_standalone_jamps.py`. H-G1 banked tables for `u d > d g u` and
`u~ d~ > d~ g u~` in scratch only. Rows for those two and for `u u > g g u u` /
`u~ u~ > g g u~ u~` would pin per-flow JAMP order and value for both members of
the conjugate pairs that #100 joins under the fingerprint permutation.
Per-graph comparison of multi-flow rows is the separate
[standalone-amps-single-flow-only](standalone-amps-single-flow-only.md).
