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
