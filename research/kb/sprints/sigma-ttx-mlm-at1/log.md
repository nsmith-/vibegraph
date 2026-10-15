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
* **F1 partial hand-back** (no commit): the fix and the hermetic test are in
  `scratchpad/ttxf1-fix.patch`. Mutation evidence: the lab mutant fails the
  new test and the fix passes it. V2's Found 5 is resolved: 113 of the 133
  points are the same p_z tie-break acting on a top, and 20 are the
  pseudorapidity `dj` of a massive t t̄ pair. The "before" validate is still
  running. F1 is resumed to finish the gate, the "after" validate, and the
  commits. Found (to file): closed-form choice 1 (E_T) is evaluated on lab
  momenta.
* **F1 landed** (`6740954`): the hadronic integrand clusters in the partonic
  CM, the mirrored term at its mirrored argument. A new hermetic test fails on
  the lab-frame mutant and passes on the fix. `validate_scales`' LHE replay
  now clusters in the CM too, and `pp_to_jj`'s nine "tie-break" misses
  vanish: they were this frame bug. The allowance is gone, and the replay
  asserts zero misses. Gates on the fixed tree: fmt and clippy clean; `cargo
  test --workspace` green; `--skip-deps validate` rc 0, "every gate cell
  passed". The matched rows are oracle-layer (⏳), so their σ is measured in
  V3. The commit was meant to be two (fix, then allowance); it is one.
  Machine-confirmed by the manager: the commit; the validate tail (rc 0); the
  `pp_to_jj` line "50000 scale comparisons … 0 needing a channel other than
  the first"; the diff tightening `TIE_BREAK_MISSES` to `missed.is_empty()`.
  F1 wrote no report file; its hand-back is recorded here.
* **V3 landed** (`0468cc6`): on the fix, `@1` moves by −4.465 ± 0.083 pb
  (paired over 10 seeds; V2 predicted −4.5). `@0` and `pp_to_ll_0j2j_mlm` do
  not move, and the ll row still passes. `@1` remains +0.45 % (3.8σ) above
  MadEvent's equal-weight seeds. The iteration bias is all in gg, and after
  it gg is +0.36 % (2.3σ, the generic multi-leg size). gq stays at +0.87 %,
  and the same +0.5–0.7 % is there unmatched and at fixed μ, with MadEvent's
  gq seeds scattering at χ²/dof about 4.7. Machine-confirmed by the manager:
  the commit; the paired shift recomputed from `ttx3-vg-results.txt`.
* **Next:** V4, an adaptation-free estimate of unmatched fixed-μ `g q > t t~
  q` (the test that settled #22 and #28), with a per-point check. Then the
  hygiene session on `6740954`, and close-out.
* **V4 landed** (`ff82833`): the post-fix gq excess belongs to MadEvent.
  - On unmatched, fixed-μ `g q > t t~ q`, an adaptation-free oracle reads
    137.928 ± 0.067, and vibegraph agrees to −0.02 %. MadEvent is −0.55 %,
    wholly in channel G2, with seed χ²/dof 7.1.
  - `hard_survey = 2` removes it (137.887 ± 0.125). Ten times the events,
    splitting the folded configuration and `sde_strategy = 2` do not.
  - On the matched row with `hard_survey = 2` (20 fresh seeds), MadEvent
    reads `@1` 578.239 ± 0.520 against this side's 578.780, a pull of 0.88;
    gg 1.00σ and gq 0.64σ. So V3's gg residual was the same survey effect.
  Machine-confirmed by the manager: the commit; the oracle's 10 seeds
  (137.902) and MadEvent's 10 `hard_survey = 2` seeds (137.887), recomputed
  from the raw result lines.
* **Item status:** attributed. The integrals cell can gate once the
  reference is re-banked with `hard_survey = 2`. That goes into the
  approved batch on PR #25, as an amendment for the user.
