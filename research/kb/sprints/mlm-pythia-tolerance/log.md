# mlm-pythia-tolerance log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.
* **D1 proposed** to the user through the planner.
* **Amendment relayed** (planner, as the user's decision): items 1, 3 and 4
  approved; item 2 gains a per-bin pull leg under look-elsewhere. The manager
  chose α = 0.5 % per leg (option (a)). Implementation proceeds unless today's
  B-vs-A or null max-pull already fails the rule.

## 2026-10-17
* **V1 landed** (`df3bbf9`, report `706fe1b`). Today's B vs A and MadEvent
  null both pass the amended rule, so the gate is implemented and the
  `mlm-pythia` standalone row is `mode = "gate"`. Manager-verified: the 16
  driver tests, and the cached recorded run (`verdict: PASS`, with the D1
  numbers above). Today's numbers are written into D1.
* **HY1 landed** (`61a5af1`, report `6c58ea3`): 14 Python and 5 Rust tests;
  the row records the Pythia build. Manager-verified: 30 tests; the cached
  run's verdict unchanged; the `n_bins/2` mutation caught.
* **Close-out**: the item is deleted, three items are filed, and the gate and
  report concepts describe the gate.
