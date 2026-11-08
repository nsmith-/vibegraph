---
type: Backlog Item
title: "Splitting groups by MeV-scale quark masses thins the per-group budget: VBF_HH scatters at neval 1e4"
description: "#95's split_by_external_masses gives VBF_HH 92 groups in 52 parts; at neval 1e4 its seed spread is 16% against ~10% quoted errors (one seed 1.38× MG). At 1e5 it reads 1.011 ± 0.008."
area: performance
state: open
priority: medium
closes_when: "Groups that differ only in negligible external masses share a grid or are sampled as one, or the budget is allocated per merged channel, with a warning when the split multiplies the channel count; VBF_HH then reads σ/MG with χ²/dof ≈ 1 over 20 seeds at both 1e4 and 1e5, and the forward-jet direct-term tail is re-measured on the same seeds."
blocked_by: []
opened: 2026-11-08
tags: [flavour-groups, massive-incoming, sampling, budget, cms, vbf-hh]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: hv2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-V2-report.md", title: "H-V2 report, Found 1"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log, 2026-11-07: the VBF_HH root cause and the refined mechanism"}
  - {id: closeout, resource: "../../sprints/cms-gridpack-coverage/closeout.md", title: "Sprint H close-out draft: the closing rerun"}
---
#107 fixed VBF_HH's mean: the mirror term now follows MadEvent's IMIRROR = 2.
Its spread at the campaign budget is still too wide:

| run | σ/MG | spread |
|---|---|---|
| H-V2, 10 seeds, 1e4 | 1.002 ± 0.051 | χ²/dof 2.54; seeds 6–10 at 0.86–0.91 |
| closing rerun, 5 seeds, 1e4 | 1.116 | sd 0.158; per seed 1.089, 1.119, 0.985, 1.008, 1.381 |
| H-V2, 10 seeds, 1e5 | 1.011 ± 0.008 | χ²/dof 0.67 |

The mechanism (H-V2): MeV-scale light-quark masses make `split_by_external_masses`
build 52 parts. Each channel takes the 512-point floor, so parts carrying 0.1%
of σ spend budget the heavy t-channel region needs. With u/d/s/c set to zero
the groups merge and VBF_HH reads 1.023 ± 0.009 at 1e5. That run predates
#107, so its mean is not comparable.

The CERN host's proposed directions are a design call:
- (a) share a grid, or merge for sampling, among groups that differ only in
  negligible external masses;
- (b) allocate the budget per merged channel;
- (c) at least warn when the split multiplies the channel count.

Also to re-measure: a forward-jet direct-term tail (η 6.2, 64% of one part on one
seed) seen before the mirror fix. Per the seed policy, few-seed spreads
understate heavy tails, hence 20 seeds per rung.
