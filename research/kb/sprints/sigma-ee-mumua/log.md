# sigma-ee-mumua log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 landed** (`17713e7`): the offset belongs to MadEvent. The banked σ is
  1.00 % below a non-adaptive fixed-density estimate (1.00806e-1 ±
  1.1e-5), and vibegraph agrees with that estimate to 0.1 % in total, in both
  radiative-return windows and in the named 2D cell. The cause is MadEvent's Z
  radiative-return channels G6 and G8. They carry rare large weights that the
  mean²/variance iteration combination suppresses. At the bank's 10k events
  the equal-weight recombination closes about 40 % of the gap; at 100k it
  closes all of it. "Our η(γ) at fixed m(μμ) differs" is falsified.
  Machine-confirmed by the manager: the commit; the estimator's per-seed
  totals in `ee-fdB.log` (1.0075–1.0087e-1); the bank's 0.099801; that
  `gen_pta_windows.sh` has the `mll-371` stage.
* **Blocked on a design decision:** the corrected reference V1 proposes is
  the per-seed equal-weight recombination, and nothing produces that yet. How
  MadEvent references treat the iteration bias is decided once, in
  `sigma-reference-scatter`'s D1 (it also governs `sigma-2to4-offset` and
  `sigma-uux-wbwb-veto`). The banking request is posted once D1 is approved.
