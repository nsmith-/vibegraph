---
type: Session Brief
title: "T-V2: budget study and the CMS VBF/EWK ladder; a committed σ-level spine test"
description: "In a throwaway merge of #81 and #82, measure the seed spread at two budgets, then run the 5-seed ladder on the three CMS cards with their shipped param cards; on #81 itself, commit a σ-level test that the no-spine defect fails."
status: draft
agent: validation-dev (Opus)
depends_on: [T-V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
## Two worktrees, two purposes

- **`/home/user/wt/TV2`**, branch `scratch-t-v2`: #81 (a09a7806) merged with
  #82 (da56ab77), so `integrate --param-card` exists. Use it for
  **measurement only**. Never commit there beyond what already exists, and
  never push it. Build and run the CMS ladder from it. Reference data is
  hard-linked in.
- **`/home/user/wt/T`**, branch `sprinte-spine-fallback` (PR #81): commit the
  test (§3) and your report here. Don't push.

Read `T-F1-report.md` and `T-V1-report.md` first. Pixi is not installed.

## 1. Inputs

The three CMS cards are on branch `vbf-dy-spine-inputs` (4cc4011b), under
`validation/cms_gridpacks/inputs/<gridpack>/`. Extract them into your
scratchpad with `git show`; don't merge that branch. Each directory holds:
`proc_card.dat`, `run_card.dat` (the resolved card MadGraph ran with),
`param_card.dat`, `customizecards.dat` and the bundled
`restrict_ckm_no_b_mass.dat`, plus a README with MadGraph's σ:
- DYto2L-2Jets-EWK_Bin-MLL-50: 13.12 ± 0.031 pb
- VBFto2L_Bin-MLL-50: 7.574 ± 0.030 pb
- VBFto2L_Bin-MLL-4to50: 117.6 ± 0.44 pb

The model is `loop_sm` with that restriction. Build a model directory in the
scratchpad: a copy of `research/refs/mg5amcnlo/models/loop_sm` plus the
gridpack's restrict file. Never write into the submodule. Point
`--ufo-dir` at it.

Pass the shipped `param_card.dat` to `integrate --param-card`, and report the
moved parameters the artifact records. The cards use PDF 325300: run with
`VIBEGRAPH_FETCH_CONSENT=1` once, to fetch it into the cache. That is fine
for measurement runs and never in tests.

## 2. Budget study, then the ladder (AGENTS.md "Samplers gate statistically")

- **Wall time.** The CERN host took 3–19 min per run at 4 threads. Measure
  one run per card first. Then plan a campaign that fits in about 10 hours
  of wall time on this 4-core machine, and write the plan into your report
  before launching it. Priority order: the budget study on DY MLL-50, then
  the ladder on all three cards.
- **Budget study.** On DY MLL-50, run 20 seeds at each of two rungs (the
  rung you expect to gate at, and 4× it). Read the measured seed spread
  against the quoted errors at each rung. The rung comparison must be read
  against that measured spread, not a few-seed scatter.
- **Ladder.** Run 5 seeds per rung on all three cards, rungs × 4 apart,
  until the 5-seed median stops moving within the measured spread. Report
  the unweighted mean, median, spread, χ²/dof of the seeds, and σ/MG at each
  rung. Watch for the failure signature: failures that migrate between seeds
  as the budget grows.
- **Per event.** On one card, histogram the generated events' μF near 2 GeV,
  to show the floor edge, as T-V1 did.
- **Restart safety.** The container has restarted twice this sprint. Detach
  every campaign, write one result line per run as it finishes, and make the
  driver skip runs already recorded.

**Pass** (the item's `closes_when`): on all three cards, the 5-seed medians
have stopped moving, and they agree with MadGraph with seed χ²/dof near 1.

If they don't agree, report:
- the size of the residual;
- whether it is consistent across cards;
- the per-event checks you would run next.

Don't tune anything to make them agree.

## 3. A committed σ-level spine test (on `/home/user/wt/T`)

The structural test already catches a withheld spine. Turn T-F1's `u c` probe
into a committed σ-level test:
- process `u c > e+ e- u c QCD=0`, `ptj = ptl = 0`, an `mmjj` cut, at a budget
  that runs in CI's extended layer in a few minutes;
- ≥5 seeds, asserting seed χ²/dof inside its band, plus a relative spread
  bound measured with headroom.

Show it fails with the spine withheld (T-F1's mutation M1), and give its
runtime. Put it where the `extended-validation` skill says such a gate
belongs.

## Gate (on `/home/user/wt/T` for the committed test)

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- the new test under its feature flag
- `python3 scripts/kb.py lint` (0 errors)

Write and commit `sessions/T-V2-report.md` on `/home/user/wt/T`. It must
contain the campaign plan, the tables, the commands, and **Found**. Correct
this brief where it is wrong.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push.
Do not edit backlog items. Commit checkpoints as you go.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based or `pgrep`-based watcher shells; draft your
  notes and report between completion notifications instead of idle-waiting.
  Never `pkill -f`; kill only PIDs you started.
- **No foreground command over ~8 minutes.** Detach longer runs.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
