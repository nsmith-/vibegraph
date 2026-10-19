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
* **P2 landed** (report `b3e5f3d`): the K ladder over 20 seeds on three rows;
  K = 6 proposed. Manager-verified: per-arm σ and ε_unw means recomputed from
  `results.jsonl`; the `dsample.f` reading (no power-2 rescale on the refined
  grid) checked against the submodule source.
* **Decision** (user, relayed by the planner): K = 6 becomes the default, with
  a CLI way back to the unbounded grid, the grid rule in the artifact, and a
  one-sitting timing before any variance-per-CPU claim; the banked run is
  sequenced with `map-default-guards` so the calibrations re-record once. P3
  dispatched.
* **P3 landed** (flip `23b8a42`, test fixes `87a3711` and `502d75f`, report
  `7c2d67c`). K = 6 is the default; `--grid-max-bin-width none` restores the
  unbounded grid; the artifact records the rule at format version 12, and an
  unbounded artifact keeps its old version and bytes. One-sitting timing:
  quoted²·CPU K6/unbounded 0.913 ± 0.031 / 0.870 ± 0.015 / 0.978 ± 0.013 on
  `pp_to_llj` / `pp_to_llj_mlm` / `pp_to_jj`. Banked layer on the tree merged
  with `map-default-guards` (tree `6aecfc6`): exit 0, 1924 passed, every gate
  cell passing; against the map flip alone 32 integral and 28 sample cells
  moved, none out of its limit. Manager-verified: both banked runs' exits,
  tallies and empty `problems`; the timing summary; CI green on `502d75f`.
* **CI** on `3b392dd` was red: a feature-gated test pinned the proton artifact
  at version 11. The brief's gate list omitted the feature-gated test binaries,
  which cargo stops running after the first failure.
* **Close-out**: the item is deleted. Filed: refine_grid's damping order,
  rescale + bound row-dependence, a per-row/per-dimension K, GridRule
  constructor validation, the pinned golden that never binds the bound, and the
  local toolchain lagging CI's stable. P1/HY1's artifact-rule entry is closed by
  P3; HY1 F3 (squared rescale power) closed by P2 as a wrong premise. The stale
  site in `sampler-tunings-predate-pooled-stop` and the sampler-kind item's
  version wording are updated. Whichever of this PR and `map-default-guards`
  merges second re-runs `scripts/gen-cli-docs.sh`.
