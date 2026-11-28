---
type: Backlog Item
title: "4-jet and γ + jets cards read 5–30% low at the campaign budget, with errors too small, on seeds that miss a region"
description: "Cards 252, 993 and 232 read 0.91, 0.96 and 0.70–0.76 of MadGraph with too-small errors. Their 4× neval rungs spent only ~1.15× the points (512-point channel floor), so no real budget test exists."
area: validation
state: open
priority: medium
closes_when: "The region these seeds miss is localised (channel, cut boundary or phase-space map). Sampling is fixed so that card 252 reads within 2σ of MadGraph at the campaign budget over 20 seeds with seed χ²/dof near 1, or the campaign budget for this family is raised with a 20-seed ladder showing the chosen rung converged."
blocked_by: []
opened: 2026-11-27
tags: [vegas, multichannel, confidently-wrong, multi-jet, photon, cms, budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-27}
---
Found by `vg-validator`'s v0.2.3 pre-tag stage B and its follow-ups on the fix
build 2431b587. None of these cards ran in v0.2.2.

| card | rung | seeds | unweighted mean vg/MG | spread |
|---|---|---|---|---|
| 252 GJets-4Jets dRGJ-0p2 HT40-70 | 1× | 4, plus 1 refused | 0.907 ± 0.016 | 0.032 |
| 252 | 4× neval (≈1.15× points) | 5 | 1.012 ± 0.027 | 0.061 |
| 993 GJets_DR-0p4 HT-100to200 | 1× | 5 | 0.958 ± 0.060 | 0.135 |
| 232 GJ-4Jets HT-600to1000 PTG-100to200 | 1× | 4, plus 1 refused | 0.696 ± 0.053 | 0.105 |
| 232 | 4× neval (≈1.16× points) | 3, plus 2 refused | 0.759 ± 0.034 | 0.059 |

Every rung ran `--fixed-budget --niter 3`, so σ comes from the third iteration
alone. With 1068 grids, about 1030 channels sit at the 512-point floor per
iteration, so 4× `--neval` moved the last iteration from 2.24M to 2.50M points
(232) and 2.25M to 2.58M (252): ×1.12–1.14 in the iteration σ comes from,
×1.16–1.18 in total evaluations (`vg-validator`, from the job logs). The 1× →
"4×" shifts on 252 (+0.105) and 232 (+0.063, 1.0σ) are therefore seed scatter
on heavy-tailed rows, not convergence. A real budget step raises the iteration
count, or neval well above 512 × channels (about 550k here).

On stage B's one seed each, about 20 independent 4-jet rows (GJ-4Jets,
GJets-4Jets, WBtoLNuB-4Jets, DYBto2LB-4Jets, GJets_HT/DR) averaged about 0.93.

On 252 at 4× neval, the two seeds with a small quoted error read 0.951 (1.2%) and
0.961 (1.6%), and the seeds that read above 1 carry 7–11% errors. That is the
missed-region signature.

Card 232 differs: its four seeds (0.651, 0.696, 0.842, 0.596; errors 9–13%)
agree on a deficit of about 30% with χ²/dof ≈ 1.7 about their mean, and seed 5
is refused (5.1 effective points). Only the highest seed carries the largest
error. Five seeds cannot tell persistent undersampling from a bias here; a real 4×
budget rung (more iterations) decides it (mean moves toward 1: coverage; stays
near 0.7: a defect). At 4× neval its three surviving seeds read 0.808, 0.777
and 0.693, with seeds 1 and 5 refused (4.6 and 7.5 effective points).
Seed 1 reproduces stage B's value on 54cecfac (0.6507).

Five seeds per rung cannot calibrate the 0.907 → 1.012 shift on a row this
heavy-tailed, so 20 seeds per rung are needed. These cards use
`sde_strategy = 1`, so the MadGraph channel-weight difference
(`scales-pdf/channel-cut-madgraph-versions`) does not apply.

## Per-channel reading of card 232

`vg-validator` read the per-channel σ, error and neval of all 1068 channels from
the result records of 7 completed 232 seeds (4 at 1×, 3 at neval 480000, build
2431b587), taking each channel's effective N as (σ/δσ)².

- 1054–1059 channels sit at the 512-point floor at 1× (1034–1040 at 480000).
  Together they carry 0.26–0.50 of MadGraph per seed, at a median effective N
  of 3–8 each, many at 1.
- By final state, per seed: γ+2j 0.045–0.052 (stable), γ+3j 0.218–0.286,
  γ+4j (945 channels) 0.311–0.519. The two highest seeds are the two whose γ+4j
  sum is about 0.5.
- The well-sampled channels are stable: γ+2j (3,0,0) reads 0.0195 ± 0.0015 at
  effective N ≈ 925, and the top γ+3j channels have effective N ≈ 117.
- The channels with the widest seed spread are all floor channels with
  effective N 1–5, e.g. (5,9,383) at 0.004–0.090 of MadGraph.
- The mean over the 7 seeds of the summed channel σ is 0.723.

That fits about 1000 barely sampled channels, each unbiased in expectation but
with a median far below its mean. A likely upstream cause is the α-survey: its
points per iteration are clamped to 40000 (`survey_points` in
`vibegraph-cli/src/integrate.rs`), about 37 per channel here, too few to rank
1068 channels, so almost all of them get the floor. A bias in the γ+4j channels
themselves is not ruled out. The gridpack banks only MadGraph's total
(101.95 pb), so a per-multiplicity MadGraph survey (@0–@3 separately) is the
oracle that would separate the two.
