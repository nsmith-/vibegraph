---
type: Backlog Item
title: "Code and test doc comments still describe the anchor-era signs and over-claim per-diagram pins"
description: "Five doc comments (propagate_scalar, folded_hel, fermion_line_sign, two test docs) and triple_gluon_sign's unscoped quark–gluon identity are stale against the code since PR #30."
area: hygiene
state: open
priority: low
closes_when: "Each listed doc states what the code and its oracle do now."
blocked_by: []
opened: 2026-10-16
tags: [docs, amplitudes, sign-convention]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: r1, resource: "../../sprints/amplitude-signs/sessions/R1-report.md", title: "R1 report, Found"}
  - {id: r2, resource: "../../sprints/amplitude-signs/sessions/R2-report.md", title: "R2 report, Found"}
---
- `kernel.rs` `propagate_scalar`: points to "scalar-sink vertex roots"; the scalar line's −1 is `propagator_phase_sign`'s.
- `compile.rs` `folded_hel`: "`eval_m2` is its only consumer"; `eval_jamp2`, `eval_amp2`, `eval_hel_m2`, `eval_hel_jamps` read it too.
- `diagram.rs` `Diagram::fermion_line_sign`, and the test doc of `spine_sign_separates_mixed_line_and_crossed_line_propagators`: claim per-diagram `AMP()` pins from rows that bank no `AMP()` (`ud_to_epemud_qcd0`, the 2→6 rows; compared per flow).
- `rooting_soundness.rs` `four_fermion_currents_are_rooting_invariant`: "every four-fermion diagram of the gated rows is a single contact"; `uux_to_ttxg_4f` and `tata_to_ttxa_tensor4f` are not.
- `root_diagram.rs` `triple_gluon_sign`: "−1 per quark–gluon vertex is the same convention" holds only when no other vertex has an odd gluon count (see [odd-gluon-vertex-sign-unmeasured](../validation/odd-gluon-vertex-sign-unmeasured.md)).
