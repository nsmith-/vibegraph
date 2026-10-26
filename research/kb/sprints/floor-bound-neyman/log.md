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
