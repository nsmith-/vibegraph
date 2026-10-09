---
type: Paper
title: MadWidth
description: "arXiv:1402.1178 (Alwall et al.): automatic tree-level decay widths for any model, and the UFO decays.py extension that ships pre-computed partial widths."
resource: "https://arxiv.org/abs/1402.1178"
status: draft
tags: [widths, decays, ufo, paper, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n01-madwidth, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L562-L601", title: "Note 01, MadWidth summary"}
  - {id: ufo-mod, resource: "vibegraph-lib/src/ufo/mod.rs#L613-L619", title: "UfoModel::width, read from the width parameter"}
---

"Computing decay rates for new physics theories with FeynRules and
MadGraph5_aMC@NLO" (Alwall, Duhr, Fuks, Mattelaer, Öztürk, Shen, 2014)
computes tree-level decay widths of any particle in any model, including
higher-dimensional operators[^n01-madwidth].

## Two parts

1. **FeynRules (Mathematica)** derives analytic two-body partial widths and
   exports them as a UFO `decays.py` of `Decay` objects.
2. **MadGraph (Python)** computes multi-body widths numerically: it generates
   the matrix element of each decay channel and integrates it over the
   `n`-body phase space.

For each particle `P` of mass `M`: enumerate the kinematically allowed final
states from the model's vertices; take two-body widths analytically; for
`n ≥ 3`, generate and integrate; sum over colours and spins and divide by
`2M`.

## The UFO extension

- `Decay(particle, partial_widths)` maps final-state particle tuples to
  partial-width expressions.
- The total width is the sum of the partials.

A UFO can therefore ship with its widths, and MadGraph substitutes them into
`param_card.dat`.

## Relevance to vibegraph

vibegraph does not read `decays.py`. A particle's width is the value of its
width parameter, from the param card's `DECAY` blocks or the model's defaults
(`UfoModel::width`)[^ufo-mod]; it enters only the propagator denominators of
timelike lines ([wavefunctions and propagators](../../amplitudes/wavefunctions-and-propagators.md)).
A `1 → n` process can be integrated as a partial width, in GeV, which is
MadGraph's numerical half of this paper
([decay processes](../../process/decay-processes.md)). How the UFO files are
read is [UFO parsing](../../model/ufo-parsing.md).

[^n01-madwidth]: Note 01, MadWidth summary.
[^ufo-mod]: `vibegraph-lib/src/ufo/mod.rs:613–619`.
