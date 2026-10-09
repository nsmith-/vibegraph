---
type: Session Brief
title: "F-G2: sampling, scale, PDF and event test fixes"
description: "Fix R-G2's 14 triaged findings in the σ, scale, PDF and event test targets and the manifest notes, and close five claimed stale-comment and probe items."
status: draft
agent: validation-dev (Opus)
depends_on: [triage]
closes: [validation-test-comments-stale, validate-scales-module-doc-stale, validate-hadronic-calibration-comments-superseded, manifest-notes-describe-superseded-state, jj-banked-orderings-eta-uses-wrong-components]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-G2.2, .3, .4, .5, .6, .7, .8, .9, .14, .15, .19, .20, .21, .22 (`sessions/R-G2-report.md`).

**Claimed items closed here:** validation-test-comments-stale, validate-scales-module-doc-stale, validate-hadronic-calibration-comments-superseded, manifest-notes-describe-superseded-state, jj-banked-orderings-eta-uses-wrong-components.

**Notes:**
- **R-G2.2 and R-G2.5 tighten σ gates** to the recorded census readings (`research/kb/validation/seed-headroom-census-2026-09.md`, note 36a). Quote each reading at its constant. Run each tightened row's gate, which is long, so run detached and one at a time.
- **R-G2.6 to R-G2.8 fix sites the filed items miss.** `llj-gate-comments-quote-pre-floor-ladders` stays open; only its report strings change here.
- **jj-banked-orderings-eta-uses-wrong-components:** the item asks for the counts to be re-recorded. Run the probe on the banked events.
- **Do not change how seeds are combined** (R-G2.1 is the user's open call).
- **Banked gates:** `validate_sigma`, `validate_hadronic`, `validate_scales`, `validate_alphas`, `validate_vegas`, `rambo_flat_mc`, `scales_run_cards` and `alphas_reference_grid`, as touched.
