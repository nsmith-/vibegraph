---
type: Backlog Item
title: Seeded beam-plus-RAMBO probe points are written three times
description: "probe_bridge's points copies AmplitudeEvaluator::generic_probe_points (private, compile.rs) and the same ±z-beam plus seeded rambo_massive shape appears in reweight/engine.rs."
area: hygiene
state: open
priority: low
closes_when: "One generator, with seed and scales as arguments, serves compile.rs, the probe bridge and reweight/engine.rs."
blocked_by: []
opened: 2026-10-15
tags: [duplication, test-helpers, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: ph, resource: "../../sprints/amplitude-signs/sessions/P-H-report.md", title: "P-H report, Found"}
---
Sites: `helas/eval/run/probe_bridge.rs` (`points`), `helas/eval/compile.rs`
(`generic_probe_points`), `reweight/engine.rs` (~:962). Unifying them edits
non-test evaluator code, which the hygiene review of the bridge was barred from.
Also `const NC: i64 = 3` is declared privately in both `helas/eval/lower.rs`
and `helas/color/colorize.rs`; one export from the colour module would serve both.
