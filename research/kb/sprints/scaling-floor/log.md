# scaling-floor log

## 2026-10-27

The user asked to start the scaling floor after #75 merged (`283b53b`).
Worktree `/home/user/wt/scaling-floor` created off main with the reference
data copied in; S1 dispatched.

## 2026-10-27 to 2026-10-28

Two container restarts killed S1's ladder runs. Each time the agent was resumed
and its resumable driver reran only the missing seeds; every seed-run appears
once per TSV.

S1 reported (`30a8b3b` code, `93367d0` report). `ChannelFloor::ScalesWithBudget`
(`max(512, ⌈512·neval/600 000⌉)` accepted points) equals today's floor at and
below 600k, and doubles every floor-held channel's points at 1.2M. It does not
fix the ladder: sd ratio 1.2M/600k 1.245 against 1.220 for today's floor,
χ²/dof 1.70 against 1.43, at 1.48× the points per iteration. The floor-held
channels carry about 2% of the quoted variance (0.4% scaled); seed 55's tail
point in α-share channel 613 drives the excess. The falsifier fails per seed
(551's variance falls on 14 of 20 seeds, geometric mean ×0.39). A default switch
moved 0 of 217 banked cells, since every gate runs at 600k or below. The agent
dropped the queued 800k rung at my request: the variance decomposition settled
it without the run.

Manager checks: commits authored by the repo identity with the trailer; the
fixed/scaled TSVs hold 20 seeds per rung with no duplicates, and their per-rung
mean and sd recomputed from them match the report (fixed 600k 1.690032e-6 /
2.737e-9, 1.2M 1.691703e-6 / 3.339e-9; scaled 1.2M 1.691528e-6 / 3.409e-9).
The bit-identity checks and the validate pair are the agent's claims.

Setup error (mine): the reference data copied into the worktree came from the
main checkout, whose `validation/madgraph/output` predates main's pinned
bundle. Both sides of the validate pair failed the same 14 tests on missing
inputs (5 ❌ cells), so those cells were not measured by it; CI's banked job
fetches the pinned bundle.

## 2026-10-28

The user chose to record the result and drop the code. The rule
(`budget.rs`, `hadronic.rs`) and the probe's `VG_2TO6_FLOOR` knob are
reverted to main; the probe keeps S1's one-seed list (`11,`). Deleted
`floor-bound-channels-ignore-budget` (its `closes_when` allows the record of
why the fix fails). Filed `bbx-ladder-excess-in-alpha-share-tails` (S1 Found
1–2), which now blocks `two-to-six-integrals-info-on-scattering-references`;
Found 3 is a line in the seed-sweeps concept, and Found 4 lapses with the code.
