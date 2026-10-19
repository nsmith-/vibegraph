# sigma-calibration-rerecord log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.

## 2026-10-17
* **Hand-off state, statistical-gates hand-off (2026-10-17 ~16:20 UTC)**

Manager session `session_012K5sTjLQ9sV8eQYVSSbfgZ`; planner
`session_014wzXyZJfXdGTAVjvtKqbZM`. Worktrees are under `/home/user/wt/<branch>`,
and the reference data is copied in with the scratchpad's `copydata.sh`
(refdata-10, `NNPDF23_lo_as_0130_qed`, `NNPDF31_lo_as_0130`, SM submodule).
The repo-local git identity is Nicholas Smith <nick.smith@cern.ch>.

**Merged:**
- #42 `sigma-pulls-lean-high` (five-seed fixed-beam σ gate; `sigma-pulls` tool;
  2 → 3 adaptation-free estimate);
- #43 `seed-record-tooling`.

**Merged:** #42, #43, #46, #50, #52 (and #56 on main, `6ddcaf6`, CI green).

**Ready for review:** #44 `refdata10-cell-flips` at `10d278d` (main merged).
New `gg_to_gg_cg` `rel_tol` 0.005 for the user's review.

**Done, closing:**
- **#51** `map-default-guards`: V1 flipped both defaults (soft-all; 1/τ² unless
  a whole-state resonance), banked layer identical tally before/after. Closed
  out (`1fdfcad`); a hygiene-dev session is bringing the phase-space concepts
  to the new defaults, then push, CI, ready. Must land before #45.
- **#48** `vegas-low-acceptance-bins` at `c468b7a` (main merged): P2's K ladder
  (`b3e5f3d`) proposes K = 6 as the default bin-width bound; the user's
  decision is pending via the planner. If approved, the flip commit also adds a
  CLI off-switch, records the grid rule in the artifact, times K = 6 against
  base in one sitting, and runs the banked layer (sequenced with #51) before
  #45. Then close out #48: P1/HY1/P2 Found entries (artifact lacks the grid
  rule; GridRule constructor validation; refine_grid damping order;
  rescale + bound measured only at K = 6; per-row best K; one-sitting timing;
  no off-switch; stale site in `sampler-tunings-predate-pooled-stop`). HY1's
  squared-rescale entry closes: P2 found its premise wrong (`dsample.f`).
  Whichever of #48 and #51 merges second re-runs `scripts/gen-cli-docs.sh`.

**Waiting on the M3** (the user's banking batch; comments on the PRs):
- #42's comment (merged PR, still valid):
  1. a 20 + 20 ladder on `uux`/`ddx_to_epemg`;
  2. 20 seeds on `pp_to_bb`/`pp_to_bb_fixed`;
  3. the `validated_ew` re-record of three rows.
- #47's comment:
  1. the 20-seed SCATTER confirmation of both 2 → 6 references;
  2. the vibegraph 2 → 6 ladder (600k + 1.2M, 20 seeds, both rows, from #47's
     branch at `63cdacd`).

#47 stays a draft until those come back. Its P1 closed the stop-scale item on
measurement; at close-out file P1's Found (`chi2_per_dof` unreadable on wide
multichannel; no per-channel stop record).

**Not started:**
- #49 `vegas-absolute-grid-coords`: after #48, since both touch `vegas.rs`
  binning.
- #45 `sigma-calibration-rerecord`: last. It needs #44 merged, #51's flip landed
  and #48's K decision applied. User decision: the t-channel fix is later, so
  #45 records today's numbers.

  Its brief must also cover:
  - restating `PULL_LIMIT`'s rationale (the gate now asserts the five-seed mean's
    pull);
  - fixing the stale `validate_sigma.rs` module doc ~:72-76 (`gg_to_gg_cg`
    "not integrated");
  - the "34 gated rows" doc, which is 35 after #44;
  - the 20-seed spreads beside calibrations, from #48 P1's Found;
  - `ddx_to_epemg`/`gux_to_epemux` re-measured against refdata-10 for
    `sigma-cells-thin-budget-headroom`;
  - the drift attribution (`probe_gate_row_seed_headroom` at `f85718d^`);
  - the llj ladder rationale in `validate_hadronic.rs`.
  - the manifest's stale `pp_to_llj_fixed` note ("chi2/dof 1.86"; #51 V1 reads
    1.39 under the new defaults, 0.92 over 80 seeds);
  - `pp_to_llj_mlm`'s quoted error ~20–30 % above its 20-seed spread (#48 P2
    Found 4), in the spreads-beside-calibrations table;
  - re-recording under #51's map defaults and #48's K = 6 bound (both
    decided; #48 P3 measured the combined tree, tree `6aecfc6`);
  - its "before" from a banked run of the tree it lands on: #51's V1 banked
    report predates #42's five-seed gates, so it is not a baseline for later
    trees (#48 P3 Found; P3's map-only run at `87e3695` is the nearest).

**User decisions on record:**
- #52 D1 plus its amendment (α 0.5 % per leg);
- #42 D1;
- #51 D1;
- t-channel fix later;
- #47 ladder on the M3;
- 28-channel check skipped;
- #48 K ladder now.

**Safety-net check-in:** 22:32 UTC, over #44 (empty check-in 2 of 3).

## 2026-10-19
* **V1 dispatched**: #44, #51 and #48 are on main (`e3f543f`), so both default
  flips are in. Main merged into this branch at `445aa59`;
  [V1's brief](sessions/V1.md) records today's numbers on that tree.
