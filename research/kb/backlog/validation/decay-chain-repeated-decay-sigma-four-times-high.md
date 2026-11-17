---
type: Backlog Item
title: "A decay chain that decays two identical resonances reads σ 4× MadGraph's"
description: "p p > w+ h01, (h01 > a01 a01, a01 > b b~, a01 > b b~) gives vg/MG = 3.99–4.02 on 4 CMS cards: likely a missing 1/(2!·2!) for b b~ b b~, or each a01 decay spec applied to both a01."
area: validation
state: open
priority: high
closes_when: "The factor of 4 is attributed: per-diagram amplitudes and the symmetry factor are compared with MadGraph standalone on the repro, and the cause is named. It is fixed so the 4 cards match MadGraph by the multi-seed criterion, with a test that fails before the fix. The library's decay-chain symmetry factor is asserted against MadGraph's identical_particle_factor on this card and on a repeated-decay census case."
blocked_by: []
opened: 2026-11-17
tags: [decay-chains, identical-particles, symmetry-factor, sigma, silent-wrong-answer, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from stage 2 of its 1009-gridpack survey (5 seeds,
v0.2.1, no card adaptation).

The cards are WH, VBFH and TTH with `HToAATo4B`: gridpacks 88, 89, 448 and 452,
at M-15/40/45. The process is
`p p > w+ h01 , (h01 > a01 a01, a01 > b b~, a01 > b b~)`.

| Cards | σ_vg / σ_MG |
|---|---|
| 4 cards at M-15/40/45 | 3.993, 3.992, 4.022, 4.013 |

The ratio is 4 at every mass point, which suggests a combinatorial factor
rather than a dynamical one. Two readings:
- the 1/(2!·2!) for the two identical b and two identical b̄ in the final state
  is missing;
- each of the two `a01 > b b~` specifications is applied to both a01, giving a
  doubled assignment.

This is a silent wrong answer. It is the concrete failure behind
[`decay-chain-identical-factor-unasserted`](decay-chain-identical-factor-unasserted.md),
which notes that the library's factor is never compared with MadGraph's.
