# vegas-low-acceptance-bins log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.

## 2026-10-17
* **P1 landed** (`1ed69b7`, `97a5927`, report `064a7b3`): three opt-in grid
  rules, default byte-identical; the bound K=4 is the one clear win.
  Manager-verified: ε_unw and σ means for llj, mlm and jj recomputed from the
  300-run records.
* **HY1 landed** (`1b2d6fc`, `a7727d4`, report `8e716d8`): five test gaps
  closed; default bit-identical by a library digest and two CLI artifact hashes.
  Manager re-ran the rescale-cap mutation: caught.
* **User decision** (relayed by the planner): run the K ladder now (P2). If a K
  wins, its default flip and banked run land before the calibration re-record
  (PR #45), sequenced with PR #51's map-default flip.
