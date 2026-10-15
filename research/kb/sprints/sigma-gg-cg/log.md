# sigma-gg-cg log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 landed** (`5e68122`): the offset belongs to the reference. The bank is
  one MadEvent draw (iseed 21, which reproduces bit for bit), +2.2e-3 above
  MadEvent's own fresh seeds at ten times its budget. This side, integrated by
  deterministic quadrature, agrees with those seeds to −6e-6. |M|² agrees
  pointwise to 2.2e-15 over the αs range, so the brief's coupling premise was
  wrong. The iteration bias is negligible here, apart from one spiky seed in
  ten. Machine-confirmed by the manager: the commit; the banked banner's
  `21 = iseed`; the seed means recomputed from `me_results.txt` (bank budget
  270654.0 ± 61.3 over 20 seeds, 10× budget 270652.5 ± 23.0 over 8); the
  quadrature channels summing to 270650.9.
* **Close-out**: closed by attribution. The cell stays `info` until the row is
  re-banked as seeds, which belongs to
  `reference-sigma-seed-scatter-untested` (PR #25) together with every other
  single-draw σ reference. The item is deleted, the concepts and the manifest
  note are updated, and two Found items are filed.
* **Approval relayed** (2026-10-15, by the planner session, as the user's
  decision): close on attribution to MadEvent, as proposed. Correcting
  references for it is decided in `sigma-reference-scatter`'s D1 (option A).
