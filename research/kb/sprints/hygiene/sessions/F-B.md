---
type: Session Brief
title: "F-B: Lorentz, colour and wavefunction fixes"
description: "Fix R-B's 24 triaged findings in helas/repr, helas/color and the helas root files: blind oracles, squared-norm tolerances, dead test-only abstractions and a hand-written rational arithmetic."
status: draft
agent: validation-dev (Opus)
depends_on: [triage]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-B.1–15, R-B.17–25 (`sessions/R-B-report.md`); R-B Found 1 (document `ColorTensor::conj` of `f` as MadGraph-faithful and unreachable); R-G1 Found 2 (`LeadingColorFlows::of` must refuse, by assert, a contribution outside `n_diagrams`).

**Claimed items closed here:** none.

**Notes:**
- R-B.10 replaces hand-written arithmetic with num-rational's `CheckedMul`/`CheckedAdd`. The overflow tripwire test must still fail on overflow.
- R-B.8's corrected `(j_L,j_R)` table: derive it, and state the reading used. If it can't be made consistent, drop the column, and say so.
- Run the banked `color_cf_oracle` and `color_flow_tags_oracle`.
