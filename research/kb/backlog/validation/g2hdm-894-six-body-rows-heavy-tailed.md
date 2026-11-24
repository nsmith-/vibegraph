---
type: Backlog Item
title: "g2HDM card 894's six-body light-jet rows have a heavy weight tail that five seeds cannot calibrate"
description: "On main, one of five 894 seeds reads +4.4% high, all of it in the 6-body lv bb + jet rows; the other four agree to 0.14% while each reports 1.5–2.0%."
area: validation
state: open
priority: medium
closes_when: "The large-weight region in 894's 6-body rows is located (channel, w_max share and variance, seed 4 against seed 1), and either the sampler is changed so 20 seeds give a seed spread consistent with the reported errors, or the 20-seed spread and its tail are recorded as the card's measured uncertainty."
blocked_by: []
opened: 2026-11-24
tags: [g2hdm, sampling, heavy-tail, multichannel, error-calibration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
vg-validator ran g2HDM card 894 (M1000) for five seeds on main 7b36722c
(`--fixed-budget --neval 120000 --niter 3`, about 5850 s each). Four seeds
agree to 0.14% at 0.046014 pb while each reports 1.5–2.0%, so their χ²/dof is
far below 1. Seed 4 reads 0.0480612 ± 3.23%, which is +4.4%. The five-seed
mean is 0.999 of MadGraph, and 0.990 without seed 4.

Seed 4's excess is in the two six-body multiplicities with an extra light jet:

| row | channels | others (pb) | seed 4 (pb) | share of excess |
|---|---|---|---|---|
| ℓν bb̄ + j + b | 412 | 0.005961 (sd 6.5%) | 0.007302 ± 19% | 65% |
| ℓν bb̄ + 2 light | 168 | 0.004108 (sd 7.5%) | 0.004601 ± 9% | 24% |

The 4- and 5-body rows agree within about 1%. The six-body rows hold about 22%
of σ. The signature is a rare large-weight region that most seeds never
sample. The first step needs no new runs: dump each channel's w_max share and
variance in those rows for seed 4 against seed 1.
