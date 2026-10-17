# vegas-constant-integrand log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.
* **P1 landed** (`79eacd9`, report `0ecbbb0`). Manager re-ran `decay_widths`
  (5 passed) and `cli_decay` (3 passed, 124 s) on the commit. The gate change
  P1 flagged was reviewed and accepted: `t_wb` and `z_ee` now integrate
  exactly, so their seed-scatter window has nothing to read, and they are held
  to the exact width at 1e-6, about 300× tighter than their old noise; no
  window moved. `pixi run --skip-deps validate` was not run; the case for
  inertness is structural (every scattering integrand is pinned
  non-constant) plus 11 of 11 bit-identical non-constant comparisons.
