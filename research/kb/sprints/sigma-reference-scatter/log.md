# sigma-reference-scatter log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **D1 drafted** for the user's decision. It folds in the iteration-weighting
  bias measured by the sibling PRs #22, #26 and #28. Approving it unblocks
  F1 here and the banking requests for #26 (and possibly #28).
* **Approval relayed** (2026-10-15, by the planner session, as the user's
  decision): D1, option A, as proposed. Before any banking request, one
  comment on this PR lists every row to re-bank under A, #26's and #27's rows
  included (5 fresh-directory seeds plus 2 at 4× budget, `G*/results.dat`
  kept), with estimated M3 wall time per row and in total, for the user to
  approve as a batch. D1's `verified` stamp is left for the user to add at
  review.
* **V1 landed** (`144d49e`):
  - fresh directories do not move the llj MLM references (20 seeds each);
  - MadEvent's quoted error is not its seed spread, in either direction;
  - every single-run hadronic bank is iseed 21;
  - the CM-clustering fix (PR #23) moves the llj rows by about −0.007 %, so
    it is not their offset;
  - equal-weight recombination reads this side −0.26 % on `xqcut_only`.
  Machine-confirmed by the manager: the commit, and the three fresh-seed
  means recomputed from `results.txt`.
* **D1 amendment proposed** (validate the equal-weight central value per row
  against an adaptation-free check; a 4× screen with a 20 + 20 ladder on
  failure), with the row inventory and M3 estimates. It goes to the user
  before the banking comment.
* **Approval relayed** (2026-10-15, by the planner session, as the user's
  decision): the D1 amendment, as proposed. The equal-weight value gates only
  rows an adaptation-free check has validated. 5 + 2 × 4× is a screen, with a
  20 + 20 ladder for any row more than 2σ off. The per-row banking list is
  posted on this PR. Its command is the seeded driver F1 lands here, and the
  comment is edited in place when F1 does.
* **Approval relayed** (2026-10-15, by the planner session, as the user's
  decision): the banking batch in PR #25's comment 6196244069, as listed.
  F1 builds to it. When F1 lands, that comment is edited with the exact
  command and output paths. Rows that trip the 2σ screen come back to the
  user before any 20 + 20 ladder.
* **Scope added** (user, relayed 2026-10-15): the M3 timing re-run for
  `madevent-cumulated-time-host-dependence` (claimed on this PR) is step 2 of
  the same banking comment. It produces timings, not references. F1 adds
  `--nb-core` to `time_stages.py` for it.
* **F1 landed** (`2f9d640`, `bd92c97`, `57e9b4a`). It adds:
  - the seeded banking driver, `pixi run -e madgraph bank-seeded-sigma`, with
    row groups;
  - the per-row `seed_record.json`, written by the Rust `seed-record` binary
    and read by `tests/common/seed_record.rs`;
  - the record reader in `validate_sigma` and `validate_hadronic`;
  - `time_stages.py --nb-core`.
  It is inert: with no records present, `validate` reads exactly what it read
  before. Machine-confirmed by the manager: the commits; `report.json` before
  and after identical once timing and host fields are stripped; `build.sh`,
  now sourceable, still runs `build_main` when executed and has no side
  effects when sourced. The banking comment's command is **held**: V4 on
  `sigma-ttx-mlm-at1` found that MadEvent's default survey biases σ by
  0.5–1 % and that `hard_survey = 2` removes it. A cross-check on the
  sibling rows is running, and an amendment (bank with `hard_survey = 2`)
  goes to the user before the comment is made runnable.
* **Approval relayed** (2026-10-16, by the planner session, as the user's
  decision): the second D1 amendment. Bank every row with `hard_survey = 2`,
  record the effective `min_events`/`max_events`, keep heavy-tailed rows
  `info`, and have the driver fail short-event or non-numeric runs. At the
  user's request the amendment now states cases (a) (the uux top-free
  continuum) and (b) (#22's card, where `hard_survey` is inert) explicitly.
  F2 implements the driver side; then the banking comment is made runnable in
  one edit.
