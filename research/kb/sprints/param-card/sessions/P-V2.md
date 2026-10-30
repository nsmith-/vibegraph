---
type: Session Brief
title: "P-V2: gate the banked moved-parameter rows on refdata-12"
description: "Declare ee_to_tth_ymt250 in the alpha_s and scales inventories, unplan the two moved-parameter σ rows, and run their gate and the banked targets on refdata-12."
status: draft
agent: validation-dev (Sonnet)
depends_on: [P-V1]
closes: [cli-has-no-param-card-option]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card` (PR #82), at the
commit that adds this brief. refdata-12 is pinned (9c92bb94) and unpacked
(hard links; never write into it). Pixi is not installed. Read
`P-V1-report.md` and the M3's result on #82 (comment 6499635262).

1. **Declare `ee_to_tth_ymt250`** in the run inventories of `validate_alphas`
   and `validate_scales`, beside `ee_to_ttx`, as S-V1 measured on #83.
   - With the row declared, those tests pass (2/2 and 10/10). Without it,
     the inventory check stops them.
   - Make this **one commit touching only those inventories**. The manager
     cherry-picks it onto #83.
2. **Unplan the two σ rows** (`pp_to_ll_mz95`, `ee_to_tth_ymt250`) in
   `validation/manifest.toml`, and fill their measured cells from the gate
   run here.
   - Run `validate_param_card_sigma`. Report both `GATE` lines and compare
     them with the M3's (pulls +0.06 and +0.14).
3. **Run the banked targets**, each as its own detached, resumable
   invocation:
   - `validate_param_card_sigma`
   - `validate_alphas`
   - `validate_scales`
   - `reweight_mg_oracle`
   - `cli_reweight_proton`
   - `cli_generate_proton`
   - `cli_integrate`
   - `validate_lhef`
   - `coupling_oracle`
   - `amplitude_oracle`

   Then run the report collator, if it builds, to confirm that the measured
   cells match the manifest.
4. **Fix the two request-doc slips** the M3 found:
   - the coupling-fusion check runs under `pixi run -e madgraph python -I`,
     not bare `python3`;
   - the `# MT` grep must be case-insensitive.

   Fix them where the request text lives in the repository (the README or
   the report).

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint`, with 0 errors
- the targets above

Write and commit `sessions/P-V2-report.md`, with a **Found** section.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push,
and never amend a pushed commit. Do not edit backlog items.

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
