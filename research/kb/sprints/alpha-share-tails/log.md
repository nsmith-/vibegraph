# alpha-share-tails log

## 2026-10-28

The user asked to start the α-share tail study after #79 merged (`f5fb3e6`).
Worktree `/home/user/wt/alpha-share-tails` created off main. Unlike earlier
sprints, the reference data is fetched from the pinned bundle
(`pixi run fetch-refdata`) rather than copied from the main checkout, whose
copy predates it (scaling-floor log).

## 2026-10-28 to 2026-10-29

A container restart stopped T1 once; it was resumed and reran only seed 121
at 1.2M. T1 reported (`0b3f2c1` probe, `5260d03` report). The tails sit on the
half of the H → ZZ* peak no channel maps (85% of the recorded tail's summed
square at 1.2M, 46% at 600k; seed 55's point in 613 is one) and on Z → 4f
peaks whose channels the α survey drove to 1e-11–1e-5 (11% / 31%). A
symmetric split and an α floor of 1e-4 together cut the recorded tail to 6% /
25%, checked arithmetically on the dumps; T1 projects an sd ratio of 0.91, not
1/√2, and states a falsifier.

Manager checks: commits authored by the repo identity with the trailer; the
diff adds only the `#[cfg(test)]` probe module, its one `mod` line and the
report. The ranking, tail indices and arithmetic are T1's claims.

## 2026-10-29

The user chose to implement both fixes. F1 brief written: the symmetric split
by default, the α floor selectable, the bbx ladder in three arms against T1's
falsifier, and the validate pair for the split's gate impact.

## 2026-10-30

F1 reported (`91c3e6d` change, `4cfe188` report). Each resonant daughter pair
is drawn in either order with probability ½ inside its channel (39 bbx
channels, uux 288/305/322, `ee_to_mumu_tata_qcd0` channel 16 and variants), and
the α floor is selectable. bbx, 20 paired seeds, sd ratio 600k → 1.2M: control
1.227, split 0.784 (χ²/dof 1.18 at 1.2M), split + floor 0.708 (0.66). T1's
falsifier: (b) and (c) hold on both arms; (a) holds for the population T1 named
and fails only on H points with both Z pairs off shell; (d) is within ±1.8 of
the difference's error. Split vs base moved 2 banked cells, both still pass;
the forced-on floor moved 47, all still pass.

Manager checks: commits by the repo identity with the trailer; the floor
arm's TSV holds 20 seeds per rung, and its per-rung mean and sd recomputed from
it match the report (600k 1.691503e-6 / 1.696e-9; 1.2M 1.691141e-6 /
1.201e-9). The split arm recomputed earlier matched as well. The bit-identity,
density tests and validate pairs are F1's claims.

A separate read of MadGraph 3.7.1 and this row's MadEvent output found that
MadEvent splits each H → ZZ* config into one channel per on/off-shell
assignment (`madevent_symmetry.f` conflicting-resonance logic; `cut_bw` in
`myamp.f:186-200`, both lines checked), so the one-sided coverage was a
deviation from MadEvent. The notes are kept beside the session reports
(`sessions/madevent-hzz-coverage.md`).

The user chose the α floor on by default (D1) and to file the both-off H
component rather than add it here. F2 dispatched for the default flip.

## 2026-10-31

A container restart stopped F2 once; it was resumed and moved its working
files to `/home/user/scratch/ast-F2/` when the session scratchpad was retired.
F2 reported (`567c02c` merge of main, `fea0dfc` change, `59e10b5` report). The
1e-4 α floor is the default (`--alpha-floor off` restores the survey's α), and
it returns the α vector untouched when nothing is under it, pinned by two tests
that fail by 1 ulp with the early return removed. Against F1's split-only
tree, the banked validate pair moved 6 cells (`gg_to_gg`, `gg_to_gg_cg`,
`uux_to_uux` integrals and samples), all pass → pass; F1's 41 ulp moves are
gone. The MLM σ gate moved only `pp_to_ttx_0j1j_mlm`, pass → pass.

Manager checks: commits by the repo identity with the trailer, tree clean.
The validate and MLM pairs are F2's claims.

Setup note: main's #82 needs refdata-12, which this worktree did not have; F2
added refdata-12's 487 new members to it. Later worktrees should fetch the
pinned bundle current at their base.

Close-out: deleted `bbx-ladder-excess-in-alpha-share-tails` (the ladder now
falls 0.708 over 20 paired seeds with the default). Unblocked
`two-to-six-integrals-info-on-scattering-references`, and added T1's
per-point record to `two-to-six-tail-point-unlocated`. Filed
`h-to-zz-both-off-region-unmapped` (F1 Found 1),
`alpha-survey-differs-between-builds` (T1 Found 4),
`long-tier-rows-unmeasured-under-alpha-floor` (F2 Found 1),
`uux-2to6-low-seed-at-1200k` (F1 Found 3) and
`ee-wpwm-samples-gate-thin-headroom` (F1 Found 4). Updated the multichannel
(α frozen → floored then frozen), resonance-and-pole-maps (either-order pair
draw, MadEvent's split) and seed-sweeps concepts.
