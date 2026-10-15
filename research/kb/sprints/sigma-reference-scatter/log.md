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
