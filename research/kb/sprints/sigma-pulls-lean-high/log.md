# sigma-pulls-lean-high log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.
* **V1 landed** (`7e92ac2`): the lean is attributed. Manager-verified by
  re-running the committed `sigma-pulls` on V1's recorded runs (R +0.615 over
  48 rows; five seeds: R +0.475, EW +0.285, EW4x +0.110); its 7 tests pass.
* **Banking request** posted on the PR: a 20 + 20 ladder on
  `uux/ddx_to_epemg`, and 20 seeds on `pp_to_bb` and `pp_to_bb_fixed`.
* **HY1 landed** (`9cc7296`): four silent-skip paths made loud; 20 tests.
  Manager-verified: output byte-identical on V1's inputs, the `MATCH_REL`
  mutation caught.

## 2026-10-17
* **Approval** (user, relayed by the planner): D1's three points. F1 (five
  seeds on the fixed-beam σ rows) and V2 (the adaptation-free 2 → 3 estimate)
  are added to this PR; the item closes on V1's attribution at close-out.
