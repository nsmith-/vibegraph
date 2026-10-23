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

## 2026-10-23
* **M3: 20-seed MadEvent references (item 1) done** (PR #47 comments, 14:34 UTC).
  SCATTER is **confirmed on both rows**, with look-elsewhere p < 1e-300 (uux)
  and 4.6e-150 (bbx). Every flagged channel carries 0.0% of σ. The records are
  in the M3's local refdata-11 candidate bundle; they are not pushed.

  | row | reported (20) | equal-weight (20) | EW − reported |
  |---|---|---|---|
  | uux | 7.175795e-7 ± 5.84e-10 | 7.180603e-7 ± 7.14e-10 | +0.067% ± 0.080% |
  | bbx | 1.686455e-6 ± 1.31e-9 | 1.687758e-6 ± 1.52e-9 | +0.077% ± 0.051% |

  - The manager's reading against vibegraph's 20-seed rungs:
    - uux, sprint tree: 600k +0.29σ and 1.2M +0.67σ against reported.
    - bbx, main tree (`784582d` + main, B1), 600k at 1.689996e-6 ± 6.94e-10:
      +2.39σ against reported, +1.34σ against equal-weight.
  - Per D1's third amendment the cells stay `info` against MadEvent, since the
    references' own channels scatter.
