# two-to-six-gate log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.

## 2026-10-17
* **Banking request** posted on the PR (2026-10-16): the 20-seed SCATTER
  confirmation of both MadEvent references (`SEED_PLAN=confirm20`, from PR #43's
  branch).
* **P1 landed** (`3fee267`): under `pooled_scale` no kept channel-iteration
  holds fewer than 49 accepted points, and the stop factor is ×1.01–1.12 on
  both rows over five seeds at 600k × 8. `stop-scale-inflated-by-few-accepted-iterations`
  closes on the measurement; no fix. Manager-verified by recounting kept
  iterations from the traces (uux 17 370, min 49; bbx 18 450, min 51; none
  at 1–3 accepted points).
* **User decisions** (relayed by the planner): the 2 → 6 ladder (600k and
  1.2M rungs, 20+ seeds, both rows) runs on the M3, after the references'
  20-seed confirmation; its commands go into this PR's banking comment. The
  low seed χ²/dof (0.10–0.66) stays flagged in the record. F1 makes the probe
  runnable that way.

## 2026-10-22
* **M3 results** (PR #47 comments, 04:55 and 14:04 UTC):
  - **The ladder (item 2) is complete**, 20 seeds a rung on both rows. Both rows
    sit within +0.9σ of their 5-seed MadEvent records at both rungs.
    - `uux` shrinks with budget: sd 2.91e-9 → 1.58e-9 pb, Δ +0.26σ.
    - `bbx` **grows**: sd 7.44e-9 → 1.00e-8 pb, and χ²/dof 3.09 at 1.2M.
  - **The SCATTER confirmation (item 1) stopped after one seed** (92 min). The
    user has queued it as a dedicated multi-night M3 run. Until it lands, the
    cells stay `info` per D1's third amendment.
* **B1 dispatched**: the user wants the `bbx` growth diagnosed
  ([brief](sessions/B1.md)).
* **B1 done** ([report](sessions/B1-report.md)). The `bbx` spread does not grow
  with budget. On the M3's tree, single points in floor-bound channel 551 carry
  the excess at both rungs. 551–585 of the 615 channels draw bit-identical
  points at 600k and 1.2M for a given seed. Neutralising 551 gives sd 3.98e-9
  and χ²/dof 1.03 at 1.2M.
  - On current main (with #48 and #51), the 551 and 564 tails are gone: 600k
    reads sd 2.69e-9, χ²/dof 0.79, and 1.2M reads 3.34e-9, χ²/dof 1.44. One
    non-floor-bound tail remains, in channel 613 on seed 55.
  - Proposed fix, untested: Neyman allocation, then a floor that scales with
    neval.
