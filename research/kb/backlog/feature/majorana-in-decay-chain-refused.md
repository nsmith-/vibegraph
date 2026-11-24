---
type: Backlog Item
title: "A decay chain whose diagrams carry a Majorana fermion is refused, even when the stitched line is a boson"
description: "All 7 HNL gridpack cards (p p > w, (w > l hnl)) are refused at enumerate: chain.rs refuses any glued diagram that violates fermion flow, though the stitched W carries none."
area: feature
state: open
priority: medium
closes_when: "A decay chain whose stitched propagators are all bosons enumerates when its core or decays carry a Majorana fermion. Its per-diagram, per-helicity amplitudes are checked against MadGraph on one HNL chain card. Chains that stitch a fermion line stay refused unless that case is validated too. The HNL cards 84, 85 and 438–442 then run through gpdb with a 5-seed σ."
blocked_by: []
opened: 2026-11-24
tags: [majorana, decay-chain, cms, hnl, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
`diagrams::chain` glues each core diagram to each decay diagram. It refuses as
soon as the glued diagram `violates_fermion_flow` (`chain.rs`, about line 509):

> in u d~ > e+ N1, a decay-chain diagram carries a Majorana fermion or a
> fermion-number-violating vertex; its lines are oriented per core and per
> decay, and orienting a line across the stitched propagator is not validated
> against MadGraph

The test is coarse. It fires whenever a Majorana appears anywhere in the glued
graph. But the stated risk is orienting a fermion line *across* the stitched
propagator. In the HNL cards that propagator is the W. No fermion line crosses
it, and the Majorana `N1` sits entirely inside the decay `w > l hnl`, as an
external leg.

Found by the sprint L validator on PR 11's head ba190428, once the param-card
ruling let the HNL cards bind. All seven `SM_HeavyN_Gen3Mass_NLO` gridpack cards
(84, 85, 438–442) are decay chains: `generate p p > w, (w > l hnl)`, plus
`w j` and `w j j`. Enumerate refuses all of them in 0.1 s. The param card now
binds on the real model, with 6 YUKAWA and 4 CKMBLOCK entries warned and
ignored. The Majorana chain refusal is these cards' remaining blocker.

Narrowing the refusal to chains that stitch a fermion line is the smaller
step. Each glued graph's fermion flow is then fixed by its core and its decays
separately. It still needs the bit-exact check against MadGraph before it is
trusted (`research/kb/validation/bit-exact-amplitude-debugging.md`).
