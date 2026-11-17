---
type: Backlog Item
title: "The quoted σ error is about 7× the seed scatter on a long tt̄ decay chain"
description: "TTToHcToWA_AToMuMu MA15: five seeds scatter 0.045% while each quotes 0.31–0.34%; seed χ²/dof 0.020 falls below the pass band. Inter-iteration χ²/dof is 6–17 in every run."
area: validation
state: open
priority: medium
closes_when: "The over-quote on this card is traced to its cause in the error combination (for example early unadapted iterations inflating the consistency scaling), and either the combination is fixed so 20 seeds' scatter matches the quoted error, or the over-quote is recorded with its cause; the MA125 sister card stays calibrated."
blocked_by: []
opened: 2026-11-17
tags: [sigma, seeds, error-estimate, vegas, decay-chain, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from registry validation of vibegraph v0.2.1 (the
musl-v3 release binary) on CMS gridpacks via batch-proxy, `--target-rel 0.01`,
`--neval 120000`, `-j 4`. The data is under
`/home/ncsmith/bench-v0.2.0/regval-v021/out/` on the vibegraph host.

The card is gridpack 123, `TTToHcToWA_AToMuMu_MHc160_MA15` (MadGraph 2.9.18),
model `2HDM_UFO-type1_5FS`, `p p > t t~` with the full `t > b h+, h+ > h3 l+
vl, h3 > mu+ mu-` and `t~ > b~ w-, w- > j j` chain, plus five `add process`
permutations. MadGraph's reference is 0.591 ± 0.0012 pb.

| seed | σ (pb) | quoted error | inter-iteration χ²/dof |
|---|---|---|---|
| 1 | 0.592019 | 0.0020 | 6.6 |
| 2 | 0.592423 | 0.0020 | 7.0 |
| 3 | 0.591686 | 0.0018 | 17.2 |
| 4 | 0.592041 | 0.0019 | 6.8 |
| 5 | 0.591907 | 0.0020 | 11.4 |

Every seed stopped at `target_met` after 6 iterations, about 1.15M points. The
mean is right (pull +0.83 against MadGraph). Seed sd is 2.7e-4 (0.045%),
against quoted 0.31–0.34% and consistency-scaled 0.34–0.38%. The seed
χ²/dof of 0.020 is outside the pass rule's [0.052, 3.72] band.

The suspected cause: the iterations disagree within each run, but the final
estimates agree across seeds about 7× better than quoted. That fits early,
unadapted iterations inflating the error through the √χ² widening or the
consistency scaling, while the adapted ones dominate the value. The sister
card, gridpack 122 (MA125), has inter-iteration χ²/dof 80–700, yet its quoted
error matches its seed scatter (scatter χ²/dof 0.82). The over-quote is
therefore card-specific. It is conservative, not a wrong σ, but it costs
iterations and misleads a reader of the error. The user keeps this model out
of the registry for now.

A related item is [`quoted-sigma-error-uncalibrated-on-multijet-rows`](quoted-sigma-error-uncalibrated-on-multijet-rows.md),
which has the same symptom in both directions on multi-jet rows.
