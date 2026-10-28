---
type: Backlog Item
title: The run summary prints μF = 0 GeV for a per-event (dynamical) scale
description: "integrate prints 'μF = 0 GeV' when the factorization scale is chosen per event (e.g. dynamical_scale_choice = -1), because recorded_mu_f uses 0 for 'no single number'."
area: hygiene
state: open
priority: low
closes_when: "The integrate summary names the scale choice (e.g. 'μF dynamical, choice -1') instead of printing 0 GeV whenever no constant scale applies, the artifact field records the absence of a constant scale distinctly from 0, and a CLI test pins both."
blocked_by: []
opened: 2026-10-28
tags: [cli, output, scales, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
sources:
  - {id: pr71-sweep, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6438701885", title: "PR #71: VBF/EWK Drell-Yan sweep, Found"}
  - {id: integrate, resource: "../../../../vibegraph-cli/src/integrate.rs", title: "vibegraph-cli/src/integrate.rs recorded_mu_f (~:527) and the '√s: … μF = {} GeV' summary line (~:658)"}
---
`recorded_mu_f` (`vibegraph-cli/src/integrate.rs` ~:527) returns the run card's
constant factorization scale. It returns `0` when the scale is chosen per event,
or differs between beams, since no single number describes the run. The summary
line (~:658) then prints `μF = 0 GeV`, which the CERN host's CMS sweep (#71)
saw on cards with `dynamical_scale_choice = -1`.

Zero is a sentinel in an `f64` field that a reader takes as a value. Print the
scale choice instead, and give the artifact field an explicit "no constant
scale" state (e.g. `Option<f64>`), checking the artifact's version rules if
that changes its format.
