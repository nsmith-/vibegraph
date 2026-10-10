---
type: Session Brief
title: "F-A: evaluator fixes"
description: "Fix R-A's 16 triaged findings in helas/eval and close evaluator-doc-comments-stale, without changing the emitted program or any kernel body."
status: draft
agent: performance-dev (Opus)
depends_on: [triage]
closes: [evaluator-doc-comments-stale]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-A.1, .2 (the docs only), .4, .5, .6, .7, .8, .9, .10, .12, .13, .14, .15, .16, .19, .20 (`sessions/R-A-report.md`).

**Claimed items closed here:** evaluator-doc-comments-stale.

**Hot path:** this module is the evaluator. No change may alter the lowered `Program` for any process, or any kernel body.
- R-A.7–R-A.10 are compile-time refactors.
- Show that the emitted program is unchanged before and after, for the hermetic processes and at least one banked multi-flow row. Use a debug dump of the instruction stream, or the op census, whichever exists; say which.
- Run `amplitude_oracle` (hermetic) and the banked `color_cf_oracle`.
- The R-A.12 contract move touches link sites in `hadronic.rs`, `proton.rs`, `flow_tags.rs` and the book. Those edits are in scope.
