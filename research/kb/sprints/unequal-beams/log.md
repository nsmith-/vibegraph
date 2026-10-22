# unequal-beams log

## 2026-10-21
* **Creation**: Sprint folder created.

## 2026-10-21
* **F1 checked** (ffc8d65, e9b19dd): the manager re-ran the three new `proton::tests` (pass), confirmed `vegas.rs`, `budget.rs` and `phasespace/` untouched, and read the boost (`β = (x1 − r·x2)/(x1 + r·x2)`, `r = E2/E1`, `proton.rs` ~:2060), which reduces exactly at equal beams.
* **V1 checked** (9067088): the manager re-ran the new run-card guard test and `cli_hard_errors` (15) from the committed tree. V1's permission layer refused `git stash`, `git fetch`, `git merge` and `git show` on `origin/main`, so the main merge that CI needs (main's #49 `edge_probe.rs` passes an `f64` to `ProtonIntegrand::new`) is not done; put to the user, not worked around.
* **Correction from V1:** MadGraph's massive √stot equals 2√(E1E2) exactly at equal beams (stot = 4E²); at 6500 × 4000 it differs by −1.0e-9. The ≈2e-8 residual is the lab boost's rapidity, present at equal beams too.
- 2026-10-21: main merged after #59 landed (49d6802; edge_probe passes ProtonBeams::equal; full workspace green). V2 dispatched to prepare the banking row.
- 2026-10-21: V2 checked (1e142c3). The card differs only in the two header lines and ebeam2. scales_run_cards and validation-report tests pass. V2 Found 5 is stale: edge_probe was fixed in 49d6802. Banking request posted.
- 2026-10-22: V3 diagnosis (57475b1): vibegraph is correct. MadEvent cuts the mirrored ordering at the direct ordering's cm_rap when E1 != E2 (dsample.f passcuts before select_grouping; CM_RAP flipped afterwards; the DSIG passcuts commented out). Emulating that gives 717.58 +- 0.31 against MG 717.25. Awaiting the user's policy decision and the M3 eta check.
