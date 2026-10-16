# sigma-2to4-offset log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 landed** (`ec4cf0c`): the offset belongs to MadEvent. It combines
  iterations with weights mean²/variance (`dsample.f:1995-1997`). Re-combined
  with equal weights, it lands on vibegraph and on an adaptation-free
  estimator. The item's "tail surplus" was read off MadEvent's event sample,
  which under-populates the tail; the sliced σ shows none. Manager-checked:
  the commit; the `dsample.f` lines at the pin; MadEvent 0.527760 ± 0.000170
  (60 seeds) and vibegraph 0.528663 ± 0.000052 (10), recomputed from the raw
  logs. No banking needed. The optional request for equal-weight references
  goes to `sigma-reference-scatter`'s D1.
* **Close-out**: no code landed, so no hygiene session. The item is deleted,
  the measurement recorded, and three Found items filed.
* **Approval relayed** (2026-10-15, by the planner session, as the user's
  decision): close on attribution to MadEvent, as proposed. Correcting
  references for it is decided in `sigma-reference-scatter`'s D1 (option A).
