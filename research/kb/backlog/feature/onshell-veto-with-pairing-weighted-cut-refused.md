---
type: Backlog Item
title: "A '$' veto on a card whose cut_decays = F exemption varies across pairings or subprocesses is refused"
description: "v0.2.1 refuses the combination because each pairing's weight would be formed from unvetoed amplitudes; two CMS Wto2Q-3Jets HT cards hit it."
area: feature
state: open
priority: medium
closes_when: "The pairing weights of a '$'-vetoed, pairing-weighted card are formed from the vetoed amplitudes (or the refusal is shown to be the only consistent choice), and the two Wto2Q-3Jets HT cards integrate or stay refused with that reason recorded."
blocked_by: []
opened: 2026-11-12
tags: [cuts, decay-chains, onshell-veto, identical-particles, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-12}
sources:
  - {id: close, resource: "../../sprints/v021-fixes/closeout.md", title: "Sprint J record, the campaign"}
  - {id: concept, resource: "../../process/identical-particles-across-decays.md", title: "Identical particles across decays"}
---
The pairing-weighted cut (#131) multiplies each pairing's filter Θ_p by
w_p = |M_p|²/Σ_q |M_q|². Under a `$` veto, the term being multiplied is the
vetoed |M|², while the weights would be formed from amplitudes the veto does
not reach. v0.2.1 therefore refuses this combination by name
(`OnShellVetoError::PairingWeighted`): it suggests `cut_decays = T`, or
dropping the `$` restriction.

The CMS campaign on the v0.2.1 release candidate (`v021rc-ff20017-200-pc`) hit
this refusal on two cards, Wto2Q-3Jets HT-400to800 and HT-800to1500
(subprocess `u u > u d~ u d`). Both previously timed out, so no `ok` was lost,
but these cards are now out of reach.

What would resolve it: form each pairing's weight from the same vetoed
amplitude the integrand uses. That needs the veto applied per pairing subset.
Then pin the result with a σ row against MadGraph on a small vetoed,
identical-particle card.
