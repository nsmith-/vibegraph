---
type: Backlog Item
title: "e+ e- > z z, z > e+ e- reads 0.14% above MadGraph beyond what the per-event attribution explains"
description: "The informational decay-chain row reads +0.25% ± 0.08%; per event the window union gives +0.11% and the interference nothing, leaving +0.14% ± 0.08% unattributed."
area: validation
state: open
priority: low
closes_when: "The zz_ee residual is attributed (a named cause with a per-event or per-diagram oracle) or shown to be MadEvent's scatter by a larger MadEvent seed sweep, and the manifest note says which."
blocked_by: []
opened: 2026-11-12
tags: [decay-chains, identical-particles, madgraph-parity, sigma]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-12}
sources:
  - {id: close, resource: "../../sprints/v021-fixes/closeout.md", title: "Sprint J record"}
  - {id: concept, resource: "../../process/identical-particles-across-decays.md", title: "Identical particles across decays"}
---
The informational `zz_ee` row of `cli_decay_chain.rs` (`e+ e- > z z, z > e+ e-`,
500 GeV) reads **+0.25% ± 0.08%** above MadEvent's five seeds: ten seeds here,
473.13 ± 0.19 ab against 471.96 ± 0.33 ab. That was +0.26% before v0.2.1
corrected the relative sign of the two pairings, so the sign fix did not move it.

On 20000 of this side's events (seed 7, from the seed-11 grid), the per-event
decomposition used for `g g > z u u~, z > u u~`
([identical particles across decays](../../process/identical-particles-across-decays.md))
gives:
- ⟨Σ_q |M_q|²/|M|²⟩ = 0.99996, so the interference is negligible;
- the window union is +0.11% (⟨W⟩ = 0.99893);
- MadGraph's rule read on these events predicts it at +0.11%.

The residual, **+0.14% ± 0.08%**, is 1.7 standard errors. MadEvent's five seeds
have χ²/dof 1.29, and MadEvent's quoted errors are known to understate its
scatter at 0.1% (the `zz_emu_cut` row). Next steps: twenty MadEvent seeds, or
two at 5× the budget, to calibrate the reference; then, if the residual stands,
compare |M_p|² against MadGraph standalone on the single pairing.
