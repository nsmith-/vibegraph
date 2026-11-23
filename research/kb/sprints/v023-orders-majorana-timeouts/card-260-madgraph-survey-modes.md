---
type: Measurement
title: "Card 260 (WBQtoLNuBQ) σ by MadEvent survey mode"
description: "MadGraph 3.7.1 on gridpack 260's cards, 3 seeds each with the default survey and hard_survey = 2, against vibegraph v0.2.2's 5 seeds: the +2.7% excess is MadEvent's default-survey bias."
tags: [sigma, madgraph-parity, survey, b-quarks, cms-gridpacks, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
MadGraph 3.7.1 on 260's own cards, fresh directories, 3 seeds per survey
mode (vg-validator, host-measured):

| MG survey | seed σ (pb) | mean | sd | MG χ²/dof |
|---|---|---|---|---|
| default (min_events 1000) | 0.6583, 0.6775, 0.6201 | 0.6520 | 0.0292 | 206 |
| `hard_survey = 2` (min_events 10000) | 0.7021, 0.7011, 0.6931 | 0.6988 | 0.0049 | 8.0 |

vibegraph v0.2.2 (5 seeds): mean 0.70384, sd 0.0063. vg/MG is
**1.0073 ± 0.0057** (from the standard errors of the means) against
`hard_survey = 2`, 1.080 against the default survey, and 1.027 against the
gridpack's 2.9.13 value 0.6851. The excess is the
[default-survey bias](../../validation/madevent-default-survey-bias.md), the
same class as the VBS finding. With three MadGraph seeds, and MadGraph's own
χ²/dof still 8 at `hard_survey = 2` (its quoted errors are under-stated),
this is *consistent*, not a calibrated pass. This item now covers 188/189 only.
