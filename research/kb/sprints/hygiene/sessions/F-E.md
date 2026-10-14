---
type: Session Brief
title: "F-E: hadronic, PDF and scale fixes"
description: "Fix R-E's 9 triaged findings in proton, hadronic, pdf and coupling, and close the SDE_strategy/tmin configuration-weights item together with R-E.1's duplicated predicate."
status: draft
agent: feature-dev (Opus)
depends_on: [triage]
closes: [configuration-weights-wrong-at-sde1-with-tmin]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-E.1, .6, .7, .8 (unit value tests for closed-form choices 1, 2 and 5 only), .9, .11, .12, .13, .14 (`sessions/R-E-report.md`); R-G2 Found 1 (the `event_scales` doc, `hadronic.rs` ~:2229).

**Claimed items closed here:** configuration-weights-wrong-at-sde1-with-tmin.

**Notes:**
- **R-E.1 and the claimed item land on one shared predicate.** The unit test must assert on the production path, and must fail on R-E.1's mutation.
- **R-E.11:** check the `kin_functions.f` citation against `research/refs/mg5amcnlo` at the pin.
- **R-E.9:** the closed-form σ needs γ+Z interference at 500 GeV. Show the formula's derivation or source in the test doc.
- **Banked gates:** `validate_scales`, `validate_hadronic` (the scale and SDE rows), and `validate_sigma` on the SDE rows.
