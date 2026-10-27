# floor-bound-neyman log

## 2026-10-23

The user asked to start `floor-bound-channels-ignore-budget` with Neyman
allocation. P1 dispatched on `a79753c`; a container restart killed its first
run after 13 Neyman seed-runs, and it resumed with only the missing seeds.

## 2026-10-24

P1 reported (`bb96d79`). Neyman's per-seed sd ratio 1.2M/600k is 0.648 against
ByAlpha's 1.243, but that gain is the point-weighted combine's bias: Neyman −
ByAlpha is −1.71e-9 and −2.41e-9 pb (t −2.40, −2.77), and as plain means the
same iterations read −0.26e-9 and −0.30e-9 with ratio 1.18. Both rules draw
the same points; 551–575 of 615 channels share draws between rungs.

Manager checks: the commit's author and trailer, and that its diff touches
only the probe and the sprint's kb files; the Neyman TSV holds exactly 20
seeds per rung with no duplicates, and its per-rung mean and sd recomputed
from it match the report (1.688286e-6 / 2.712e-9; 1.689281e-6 / 1.758e-9).
The ByAlpha control is B1's main-tree run: `git diff --stat e5a09a8 48d9c1b
-- . ':!research'` is empty (checked); the four bit-identical check seeds are
P1's claim, not rechecked.

## 2026-10-26

The user chose to fix the combine first (relayed by the planner). Filed
`point-weighted-combine-biased-by-earlier-iterations` (claimed here, C1),
`convergence-report-lacks-iteration-split` and
`bbx-drifts-from-madevent-with-budget`; `floor-bound-channels-ignore-budget`
now records Neyman as ruled out and is blocked on the combine.

## 2026-10-26 (C1)

The user approved C1. It reported `fa51739`: `combine_kept` and `pooled_scale`
take the plain mean of a channel's kept iterations whatever counts they drew.
Replayed on P1's traces, the paired Neyman − ByAlpha shift goes t −2.40/−2.77 →
−0.35/−0.22 at no variance cost on the α arm. The α-share weights in the brief
are the plain mean exactly, since `neval` is constant within a run. A 4000-seed
toy test fails on the old combine (−6.6 standard errors) and passes on the new
one (+1.9).

The fixed-budget path is statistically gated, not bit-identical: ByAlpha already
took the count-weighted branch on 219–268 of bbx's 615 channels per seed.
`validate` before and after: 18 enforced cells moved, all pass → pass, the
largest move 2.2e-5 relative. `validate-mlm-sigma`: 8 cells moved, all pass →
pass; `pp_to_ll_0j2j_mlm` @2 132.4486 → 132.6936 pb, pull +1.90 → +2.08.

Neyman's target mode on `p p > l+ l- j` (6 channels, 0.179%, 20 paired seeds)
takes 1.49M evaluations against 1.58M (paired t +2.26), and its σ is the same
before and after the change. No default change, and no D1.

Manager checks: the commit's author and trailer, and that its diff touches only
`budget.rs`, `integrate.rs`, the regenerated CLI reference and the session
files. I ran the budget unit tests, the new one included, on the committed
source; the rest of the gate and every figure is C1's claim.

Close-out: deleted `point-weighted-combine-biased-by-earlier-iterations`.
Updated the two concepts that described the count-weighted combine
(`convergence-stop-rule`, `channel-budget-allocation`; C1 Found 1) and the
`pp_to_ll_0j2j_mlm` manifest note (Found 2). Filed
`fixed-budget-neyman-unproven-under-plain-mean` (Found 2 and 3). Unblocked
`floor-bound-channels-ignore-budget`, which now carries the plain-mean χ²/dof
(Found 4).
