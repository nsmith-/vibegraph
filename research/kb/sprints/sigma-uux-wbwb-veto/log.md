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
* **V2 landed** (`bcaff1c`): the continuum deficit is confirmed, −2.0 % over
  five fresh seeds (8σ). V1's both-W-forward region is falsified: MadEvent
  integrates that slice correctly on its own, and is −1.46 % on the
  complement. The whole deficit sits in channels G7–G9. In G8 and G9, iterations
  that catch large weights alternate with ones that miss, and the combination
  favours the misses. G7's grid never samples the region. Equal-weight
  recombination recovers about 40 %. Machine-confirmed by the manager: the
  commit; the five fresh seeds' mean, 0.044956, from the raw result lines.
* **Close-out**: documented as a MadEvent defect with a measured cause, as
  the item's closing condition allows. The row stays reported and is not
  gated. The manifest note, the `cli_onshell_veto` doc comment and three
  concepts no longer say "unexplained" or "moves down with budget".
  `cargo fmt --check` is clean and the manifest parses. The item is deleted
  and three Found items filed. V1's "keep `G*/results.dat` in banked runs" is
  covered by `sigma-reference-scatter`'s D1.
