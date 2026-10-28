---
type: Backlog Item
title: "MadGraph's pp_to_bb_iht bank scatters on a channel carrying no cross section"
description: "P1_gg_bbx/G1 in the 20-seed pp_to_bb_iht bank reads seed χ²/dof 555 on 19 dof while carrying 0.0 % of σ; the 5-seed record did not show it. Cause unknown."
area: validation
state: open
priority: low
closes_when: "The scatter of P1_gg_bbx/G1 is explained (from its per-seed results.dat values and the channel's cuts) and the explanation is in the row's integrals note."
blocked_by: []
opened: 2026-10-26
tags: [madevent-reference, ht-cuts, seed-record, scatter]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: pr74, resource: "https://github.com/nsmith-/vibegraph/pull/74#issuecomment-6370452561", title: "nsmith-/vibegraph#74, the 20 + 20 ladder comment"}
---
The 20 + 20 budget ladder for `pp_to_bb_iht` (`p p > b b~`, `ihtmin 200`,
`ihtmax 500`) read "stays at 1×" (+0.40σ), and the 20-seed 1× record is the
row's reference. Its SCATTER line read CONFIRMED on one channel:
`P1_gg_bbx/G1` at seed χ²/dof 555.00 on 19 dof, 0.0 % of σ
([#74 comment](https://github.com/nsmith-/vibegraph/pull/74#issuecomment-6370452561)).
The 5-seed record's worst channel was `P1_qq_bbx/G1` at 0.20.

The channel does not move σ at the quoted precision, and the row's note records
the flag. Nobody has looked at why a channel with no weight scatters: a channel
whose integral is zero to the printed digit and whose quoted error is tiny can
read an enormous χ² from rounding alone, or the HT window may leave it a sliver
of phase space MadEvent samples badly. The banked per-seed values settle which.
