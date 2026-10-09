---
type: Backlog Item
title: 2 → 4 processes read 0.3–0.7 % above MadEvent
description: vibegraph is +0.3–0.7 % high against MadEvent on 2 → 4 processes with identical |M|², matching off, even at fixed beams; which side is wrong is unproven.
area: validation
state: open
priority: high
closes_when: The fixed-beam u u~ > e+ e- g g reproducer agrees with a MadEvent reference whose tail coverage is established, or the offset is attributed to one side and fixed there.
blocked_by: []
opened: 2026-09-28
tags: [sigma, madevent-reference, phase-space, 2to4, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L206-L212", title: "TODO.md entry T002"}
---
The offset survives every piece of MLM switched off:
- `pp` → `e+ e-` + 2 jets, fixed scales, `ickkw = 0`, `xqcut = 0`,
  `ptj = mmjj = 20`: 102.36 against MadEvent's 101.65 ± 0.14 (six runs),
  +0.7 % (`gg` +0.6 %, `llgg` +0.2 %, `gq` +0.7 %, `qq_llqq` +0.9 %).
- **Cheapest reproducer**: fixed beams, √s = 500 GeV, `u u~ > e+ e- g g`, same
  cuts: 0.52894 ± 0.00024 (four seeds at 2M × 8) against 0.52743 ± 0.00040 (six
  runs, four in fresh directories), +0.29 %, 3.2σ. No PDFs, no running scales,
  |M|² identical point by point; vibegraph's split-angle maps agree with one
  another (0.52945 / 0.52934 / 0.52900). ~30 s per MadEvent run, ~75 s per
  vibegraph run.

The surplus sits in the high-ŝ tail (`qq_llgg`: m_ll > 150 +14 %; ŝ > 600 GeV
+9.5 %). MadEvent's dedicated m_ll > 150 run (0.2341 ± 0.0004) sits 11 % above
its inclusive run's tail and 0.7 % below vibegraph's 0.2357 ± 0.0002, and its
runs scatter beyond their quoted errors: MadEvent under-coverage is favoured but
unproven. An acceptance difference in a region MadEvent never populates is the
one class the per-event oracle cannot see, so the test has to be σ in sliced
regions on both sides.

On `pp_to_ll_0j2j_mlm` it is about +0.6 pb of `@2`, carried with no allowance
in that row's gate. Detail: [note 41 D2](../../history/notes/41-mlm-feature-sprint-plan.md) ("The generic offset").
