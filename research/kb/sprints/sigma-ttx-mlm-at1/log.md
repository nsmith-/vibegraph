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
* **V2 landed** (`d1be31a`): root cause found on this side. The matched
  integrand clusters in the lab frame (`per_group_sum` hands `term_at` the
  lab momenta), but MadEvent clusters in the partonic CM. `cluster.f`'s
  beam tie-break reads the sign of p_z, which flips at large x. Clustering
  in the CM makes every point agree with MadEvent factor by factor, and is
  worth −1.05 % on gg and −0.09 % on gq (paired), about 57 % of the `@1`
  excess. A residual of +0.57 % on gg and +0.87 % on gq remains. V1's Found 4
  (`Q2FACT(1)` in gq) is not a defect.
  Machine-confirmed by the manager:
  - recomputed from the raw comparison tables: in the lab frame 3330/20000
    (our points) and 3135/20000 (MadEvent's) differ, in the CM frame 0/20000
    in both (max |r−1| 2.4e-10);
  - `proton.rs` passes `lab` to `term_at`;
  - the `ScaleEvent` doc claims the lab frame suffices;
  - the tie-break is at `cluster.f:629` at the pin.
* **Next**: F1 (feature-dev): CM clustering in the integrand; then V3
  (re-measure the clustered rows and attack the residual).
