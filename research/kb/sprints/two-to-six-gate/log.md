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
