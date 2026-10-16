---
type: Backlog Item
title: Library-side colour and banked-table helpers are written several times in helas
description: "The colour-coefficient-to-complex conversion, the banked amplitude-table reader and the MadGraph colour-key renderer each exist in several copies inside helas and its tests."
area: hygiene
state: open
priority: low
closes_when: "One copy each: a colour-coefficient-to-complex helper and a MadGraph-notation colour-key renderer in helas::color, and one lib-side banked amplitude-table reader used by rooting_soundness and triple_gluon."
blocked_by: []
opened: 2026-10-16
tags: [tests, duplication, colour]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: th, resource: "../../sprints/amplitude-signs/sessions/T-H-report.md", title: "T-H report, Found 1–3"}
---
Sites:
- `run/probe_bridge.rs` `colour_coeff` and `eval/triple_gluon.rs` `coeff`;
- `eval/rooting_soundness.rs` (four readers) and `eval/triple_gluon.rs`;
- `eval/triple_gluon.rs` `key_name` (prints `F(...)` where MadGraph writes
  `f(...)`) and `tests/color_cf_oracle.rs` `render_structure`.

[test-banked-readers-duplicated](test-banked-readers-duplicated.md) covers the
`tests/` copies only; a library unit test cannot reach `tests/common`.
