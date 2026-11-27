---
type: Backlog Item
title: "Cards 6 and 7 integrate with iteration χ²/dof from 1e7 to 2e11, a confidently low seed, and 15% of points zeroed"
description: "On the dim6top cards, iteration χ²/dof reaches 1e7–2e11 even where σ agrees with MG. Card 6 has a seed at 0.902 quoting 2.0%, and one chain zeroes 15% of its points as degenerate."
area: validation
state: open
priority: medium
closes_when: "The source of the iteration disagreement on cards 6 and 7 is localised (phase-space map, collinear b~ region, degenerate-point rejection or grid adaptation). After a fix or reformulation, iteration χ²/dof is O(1) on the affected subprocesses. Card 6's 5-seed spread on the fixed build is consistent with its quoted errors."
blocked_by: []
opened: 2026-11-26
tags: [vegas, convergence, phase-space, degenerate-points, collinear, dim6top, confidently-wrong]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Sprint L's L-D1 probe and `vg-validator`'s card 6 control found three
symptoms. They are probably one cause.

- **Huge iteration χ²/dof.** vibegraph's `b b~` and `u u~ > t t~ l+ l-` runs at
  card 7's cuts show iteration χ²/dof from 1e7 to 2e11. That includes runs where
  σ agrees with MadGraph: `b b~` at ctXI = 0 reads 1.06, but unconverged at
  χ²/dof 1.4e7.
- **A confidently low seed.** In card 6's 5-seed ctXI = 0 control on 40e87aa8,
  seed 2 read 0.902 with a quoted 2.0% error, the smallest of the five. The set's
  χ²/dof is 3.39.
- **Zeroed points.** On `g u > e+ ve b b~ d mu+ mu-` at ptj = 0.1, 176361 of
  1143312 points were zeroed as degenerate, against 1 in 2.0M on the
  same-flavour set.

Card 6's σ is collinear-dominated: 51% of it is below pT(b~) = 5 GeV, and the
only regulator is ptj = 0.1. Until this is understood, a 5-seed agreement on
these cards is necessary but not sufficient.
