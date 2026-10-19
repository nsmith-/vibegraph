# map-default-guards log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.
* **V0 landed** (`b23e246`, report `235604d`): at the tip both guards pass
  under the flipped defaults; the items' readings were from `main@02e8b25`.
  Manager-verified from the run logs: the τ-flipped jj gate's min p 5.576e-2
  and `test result: ok`; the soft-all quintet 1.96 and the default's 2.25
  (which matches the current reading quoted in
  `sigma-seed-calibrations-drifted`).

## 2026-10-17
* **D1 proposed** (flip on today's statistics), to the user through the
  planner. It replaces the planner's proposed statistics, whose premise V0
  found stale.
* **Approval** (user, relayed by the planner): D1 as written. No statistic
  changes; both defaults flip; V1 re-runs the banked layer under both flips and
  lands before the calibration re-record (PR #45). The jj composition excess
  and V0's Found 1 are filed at close-out.
* **V1 landed** (`fba61d4` flip, `0618bf7` probes, report `6205921`). Both
  defaults flip: soft-all for every process; 1/τ² unless a whole-state
  resonance keeps `Log`. Manager-verified from the logs: the banked layer
  passes on `fba61d4` and on `c7843b0` with identical tallies (203 measured:
  197 ✅, 6 ⚠️, 13 ⏳) and the collator's "measured cells are the declared
  cells"; hermetic 1478 passed, 0 failed; the 80-seed `pp_to_llj_fixed`
  calibrations (0 of 16 quintets above 4 under either map).
* **Brief error** (V1): scope item 4 asked for kb concept edits that the
  boundaries forbade; V1 edited `docs/` only. The manager's close-out brings
  the concepts to the new defaults.
* **Close-out**: both items are deleted. Filed: the jj composition excess (D1
  point 3), the seed-reuse audit (D1 point 4), the per-process τ rule, the
  shipped llj_dyn arm's calibration, the MLM and 2→6 rows under the flip, and
  `gu_to_epemu`'s KS null. V1's stale `pp_to_llj_fixed` manifest note goes to
  `sigma-calibration-rerecord`. V0's stale dijet-samples doc is already fixed.
  Whichever of this PR and `vegas-low-acceptance-bins` merges second re-runs
  `scripts/gen-cli-docs.sh`.
