# sigma-uux-wbwb-veto log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 landed** (`199b359`): the gap is not the veto, the patch or the
  windows. MadEvent under-integrates the top-free continuum, about 94 % of σ:
  - top-free `u u~ > w+ b w- b~ / t t~`: MadEvent 0.045176 ± 0.000173 (one
    seed) against vibegraph 0.045889 ± 0.000035 (five seeds);
  - at fixed μ, a non-adaptive third opinion reads 0.061029 ± 0.000123,
    vibegraph 0.060942 and MadEvent 0.059867;
  - equal-weight recombination moves MadEvent up 0.5–1.2 %, so part of the
    gap is the iteration bias found by `sigma-2to4-offset`;
  - the "drifts down with budget" claim is not supported.
  Machine-confirmed by the manager: the commit; the third-opinion mean
  recomputed from its five seed lines; vibegraph's top-free seeds; MadEvent's
  top-free line.
  Open: MadEvent's side rests on one seed per configuration, and the decisive
  region test was not run. V2 is queued behind the host's current validate
  run.
