# sigma-ttx-mlm-at1 log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 landed** (`d5d500a`). The excess is this side's, not the reference's
  and not H1's. Fresh-directory MadEvent reproduces the bank (`@1` −0.55 ±
  0.94 pb), and H1 is +0.03 ± 0.11 pb on `@1`. The excess is matched `gg`
  (+1.6 %) and `gq` (+1.0 %), it mostly vanishes with `pdfwgt = F`, and it
  sits at x_max > 0.3. Machine-confirmed by the manager:
  - the commit and its author;
  - the fresh, patched and `t_strategy = 2` means, recomputed from the raw
    per-run JSON (20, 19 and 10 runs, all 3.7.1);
  - the `Q2FACT(1)` line in the generated `P1_gq_ttxq/auto_dsig1.f`.
  No banking request. V2 is dispatched for the per-point falsifier.
