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

**Ready for review** (CI green, all subscribed):

| PR | branch | note |
|---|---|---|
| #44 | refdata10-cell-flips | `origin/main` (#42) merged locally as `5a349d2`, not pushed yet; merge #43's main too, run `every_gated_row_is_read_on_five_seeds`, push, then wait for CI. New `gg_to_gg_cg` `rel_tol` 0.005 for the user's review. |
| #46 | ee-ee-nsz-low-budget | probe only; no hygiene pass (stated on the PR) |
| #50 | vegas-constant-integrand | `cli_decay` gate change for the user's review |
| #52 | mlm-pythia-tolerance | gate live; the run cache is in that worktree's `target/mlm-pythia/` |

**Running** (logs in the scratchpad, by prefix):
- **#51 V1** `mapguards-V1-*`: flips the τ and soft-all defaults (D1 approved) and
  re-runs the banked layer. Next: check it, close out (file the jj composition
  excess and the seed-reuse audit), CI, ready. Must land before #45.
- **#48 P2** `vegasacc-P2-*`: the K ladder for the bin-width bound (user: run now).
  Next: if a K wins, propose its default flip to the user, then flip it, run the
  banked layer, and land it before #45, sequenced with #51. Then close out #48,
  filing P1's and HY1's Found entries (artifact lacks the grid rule; GridRule
  constructor validation; refine_grid damping order; squared-rescale power;
  stale site in `sampler-tunings-predate-pooled-stop`).

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

**User decisions on record:**
- #52 D1 plus its amendment (α 0.5 % per leg);
- #42 D1;
- #51 D1;
- t-channel fix later;
- #47 ladder on the M3;
- 28-channel check skipped;
- #48 K ladder now.

**Safety-net check-in:** 18:30 UTC, over #44, #46, #50, #52.
