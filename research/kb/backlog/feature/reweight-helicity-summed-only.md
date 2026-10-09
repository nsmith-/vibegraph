---
type: Backlog Item
title: Reweighting is helicity-summed only
description: generate writes the helicity-summed ratio; MadGraph's default per-event-helicity ratio (change helicity True) is not available.
area: feature
state: open
priority: low
closes_when: A per-run option (e.g. --reweight-helicity summed|event) writes the per-helicity ratio, and the gen_reweight_oracle.py rows with change helicity True agree event by event.
blocked_by: []
opened: 2026-10-05
tags: [reweight, helicity, spinup, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L668-L730", title: "TODO.md entry T059"}
---
`generate --reweight-card` writes `|M_hyp|² / |M_card|²` summed over
helicities. MadGraph's default (`change helicity True`, which the card parser
refuses today) is `|M_hyp(λ)|² / |M_card(λ)|²` in the helicity configuration
the event records in `SPINUP`. Both are unbiased for the reweighted sigma; the
per-helicity ratio keeps the written helicities consistent with the
hypothesis, which matters when the sample is showered or decayed with spin
correlations, at the cost of larger weight variance where a hypothesis moves a
helicity amplitude the card nearly zeroes.

Needs: the selected helicity carried to the reweighter (it is written, not
kept per event); a refusal or fallback for events whose recorded helicity has
`|M_card(λ)|² = 0`; on the polynomial path, the Gram contraction restricted to
one helicity combination.
