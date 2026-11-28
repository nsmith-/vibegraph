---
type: Backlog Item
title: "A run does not compare its α-survey estimate with its trained σ per multiplicity, so a large drop goes unflagged"
description: "On CMS card 252 the untrained α-survey reads γ+4j at 1.25 ± 0.49 of MadGraph over 5 seeds and every trained result at 0.12–0.71; the run logs the survey mean but never sets it against σ."
area: feature
state: open
priority: medium
closes_when: "integrate records each multiplicity's α-survey estimate beside its trained σ, warns when they differ by more than their combined errors, and a test pins the warning on a constructed integrand whose trained grids miss a region the survey sampled."
blocked_by: []
opened: 2026-11-28
tags: [vegas, multichannel, survey, confidently-wrong, observability, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-28}
---
Found by `vg-validator` on card 252 (GJets-4Jets, 1068 channels) while
localising `four-jet-photon-cards-confidently-low-at-campaign-budget`.

The α-survey samples each multiplicity's channel mixture on untrained grids
before training and logs a per-multiplicity survey mean (the "N outgoing legs
… survey mean" line). That estimate is unbiased but heavy-tailed. On 252's
γ+4j it reads 0.654, 0.178, 2.786, 1.924 and 0.733 of MadGraph over 5 seeds,
while the trained σ reads 0.21–0.56 at niter 6 and lower at niter 3.

A trained grid that has adapted away from a region the survey sampled reports a
low σ with a small error, which is the confidently-wrong signature. Setting the
two estimates side by side is a check the run already has the numbers for.
