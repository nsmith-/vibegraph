# sigma-h1-ickkw0-census log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 interim** (`0954868`): census of 89 banked banners, the vectorised path,
  and a phased plan (B: read permutations; C0: zero-variance screen; C1:
  statistical runs). Spot-checked by the manager: commit and author, the
  `update_scale_coupling_vec` call site, the 89-run census file. Phase B held
  until a compute slot frees (load 7 on 4 cores).
* **Amendment** (brief error reported by V1): the item's criterion
  "`dynamical_scale_choice = -1` or matching" is wrong in both directions.
  `xqcut > 0` at `ickkw = 0` also clusters, and `-1` with all three scales
  fixed does not. The census uses the corrected criterion. The item text is
  corrected at close-out.
