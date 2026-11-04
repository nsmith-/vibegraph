# v02-diagnoses log

## 2026-11-03
* **Creation**: part two of sprint G (see v02-refusals for the split). Worktree `/home/user/wt/G2` off main 86b8e5c.
* **Decision (user)**: G5, a runtime warning is the known-limitation fallback for VEGAS under-coverage.
* 2026-11-04: G-V2 checked (91f4d8d, report only, no source change). The deficit does not reproduce on main (30 seeds, −0.07 % and +0.07 %, χ²/dof ≈ 1, against a 1e6 reference and a frozen-grid estimate); it reproduces on the V2-era tree (−2.4 %). Cause: a heavy weight tail starved by unbounded grid adaptation, and a convergence stop ending on calm iterations; removed by #48 and #81.
* **Decision (user, 2026-11-04)**: no production code. `vegas-under-covers-heavily-cut-slice` closes on its "or the default budget covers it" branch; file deleted here. The latent stop bias and the measured Hill-index proposal are filed as `convergence-stop-biased-low-on-heavy-tails`. Found (to file at close-out): the cold-start "accept less than one point in 4" message; a frozen-grid check flag. The χ²/dof overflow is already `vegas-printed-chi2-overflows`.
