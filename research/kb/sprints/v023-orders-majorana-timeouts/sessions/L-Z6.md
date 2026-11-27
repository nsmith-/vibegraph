---
type: Session Brief
title: "L-Z6: hygiene pass on phase timing records and the run-global progress position"
description: "Review sprint L's merged PR 3a (#183) and PR 14 (#186) for maintainability, test non-vacuity, visibility and reusable abstractions; findings first, then fixes with mutation evidence, sigma and the masked record stream identical."
status: draft
agent: hygiene-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
A hygiene pass of [sprint L](../sprint.md) (dispatch step 4), on branch `sprintl-hyg-progress` off main 033044e4.

It covers these merged PRs. For each one, read the squash commit's diff (`git show <sha>`) and its session brief and report under `sessions/`:
- **#183 `1058b781`**, phase timing records for integrate and generate (PR 3a). Brief and report: `sessions/L-T1.md`, `L-T1-report.md`.
- **#186 `a1a00162`**, global position in ldjson progress records (PR 14). Brief and report: `sessions/L-Y1.md`, `L-Y1-report.md`.

**The code:**
- `vibegraph-cli/src/phase.rs`, `logging.rs`, `output.rs`;
- the phase wiring in `integrate.rs` and `generate.rs`;
- `vibegraph-lib/src/progress.rs`;
- the progress parts of `multiplicity.rs` and `proton.rs`;
- the tests `cli_phase_timing.rs` and `cli_progress_position.rs`, and the overview docs.

The likeliest findings:
- the `phases` count kept as a hand-written list per mode when it should be derived from the phases the run enters;
- duplicated record-building between phase and progress.

A known gap, Found (a) in the L-Y1 report: `groups` differs between `compile` and `alpha_survey` when a part is dropped because its cuts zero it. It is filed as an item. Leave it unless the fix is a few lines and stays within this oracle.

## Identity oracle

Choose it before the first edit. Compare against main, built in its own worktree:
- σ and grid sha256 bit for bit, at `--fixed-budget --neval 3000 --niter 3 --seed 5 -j 1`, on two cards:
  - `e+ e- > mu+ mu-` at 45.6 GeV beams;
  - the MLM card, `validation/madgraph/pp_to_llj_mlm_run_card.dat` with `p p > e+ e-` plus `add process p p > e+ e- j`.
  - L-Y1's report records both hashes.
- The `--output-ldjson` record stream identical to main with timestamp and elapsed fields masked. Show the mask command.

Up to two sibling hygiene passes run alongside you. Stay inside the files these PRs touched, plus their direct callers and tests. The other clusters running beside you are: UFO parameters and reports (#172/#171, L-Z5); parallel enumeration and compile (#185, L-Z7); multicore generate (#180, L-Z8); and, after one of those finishes, the phase and progress instrument (#183/#186, L-Z6). Do not edit code another cluster owns unless a finding needs it, and when it does, say so.

## Scope

1. **Findings first.** Write the findings section of your report and commit it before you edit anything. For each finding give:
   - the file:line;
   - what is wrong;
   - its class, one of:
     - dead or duplicated code;
     - a vacuous or tautological test;
     - a public item that should be private;
     - a hand-kept list that should be derived;
     - a missing reusable abstraction;
     - a comment that breaks AGENTS.md's comment rules;
     - a panic or unwrap on a fallible path;
   - the evidence: a command with its output, or the quoted lines;
   - the fix you propose.

   Rank the findings and mark which ones you will fix here.
2. **Test non-vacuity.** For every test these PRs added, make one mutation of the code it guards that the test should catch. Run it and record whether the test failed.
   - A test that survives its mutation is a finding. Strengthen it, and show it now fails under the same mutation.
   - Batch the mutations with a script that applies, runs and reverts each one.
3. **Fix the findings you marked.** Behaviour must not change. Prove that with the identity oracle below, which you choose before the first edit.
   - Build main in a separate detached worktree with its own target, under `/home/user/wt/lz6-main`. Delete it afterwards.
4. **Physics and performance findings are not yours to fix.** A wrong convention, a missing oracle or a slow path goes under Found.

**Time matters.** The user rewrites main's history for the v0.2.3 tag, and every sprint branch must be merged before that. Keep the pass tight: the change's own code and tests, not a sweep of the codebase. Ship what is fixed and verified. Anything large goes in Found.

**What you must not run:**
- no seed sweeps;
- no MadGraph runs;
- no local `pixi run validate`. CI's banked job is the banked gate.

**The gates:**
- `cargo fmt --all --check`;
- clippy `-D warnings`, plain and with `--features vibegraph/extended-validation,vibegraph-lib/extended-validation`;
- the hermetic `cargo test --workspace --no-fail-fast` with the three data paths hidden;
- the `extended-validation` targets these PRs touch;
- `pixi run --skip-deps kb-lint`.

## Worktree & long-command discipline

- **First action:** `cd /home/user/wt/sprintl-hyg-progress`. Then verify that `git rev-parse --show-toplevel` is `/home/user/wt/sprintl-hyg-progress` and `git branch --show-current` is `sprintl-hyg-progress`.
  - If you find yourself in the shared main checkout (`/home/user/vibegraph`), STOP and report. Never edit it.
  - Re-verify `pwd` after any resume: resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes.** Use `run_in_background: true`, with output redirected to a log under `/home/user/wt/logs/` whose name starts with `lz6-`. Sibling sessions share that directory.
  - A backgrounded command notifies you when it exits. Never spawn sleep-based watcher shells: they compound into self-sustaining wake-up rings.
  - Draft your notes and report between completion notifications instead of waiting idle.
  - On resume after a stall, first check whether the interrupted command finished (`ps`, result artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build data. Spot-check with `git log` and `git diff --stat` instead.
- **Disk is finite, and a full disk looks like a failure.** Always run cargo with:
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-hyg-progress/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  - Each worktree has its own target dir.
  - An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a result. Free only your own `target/` and re-run.
  - Kill processes by PID. Never use `pkill -f`: it matches siblings' commands and its own wrapper.

**This container** has 4 cores and 15 GB RAM, shared with up to two sibling hygiene agents. It can restart without warning.
- **One heavy job at a time across the container**, enforced by a lock. Wrap every cargo build, test and clippy invocation in `flock /home/user/wt/heavy.lock <command>`, backgrounded.
  - Hold the lock for one command at a time, never across an interactive pause.
  - Prefer targeted test filters to the whole workspace while you iterate.
  - `cargo fmt` and reading code need no lock.
- **Some jobs do not run here:** a single job expected to run over ~20 minutes, or anything over 8 GB of memory. Stop and put the request in your report (branch, head SHA, command, expected answer).
- **pixi:** `export PATH=$HOME/.pixi/bin:$PATH`. The reference data is already in your worktree as copies. Never edit it in place.

**Git and attribution.**
- Every commit uses `git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`, because the container's identity is wrong.
- Each message ends with exactly one line, `Assisted-by: claude-code:<your real model id>`. Never add Co-authored-by or Signed-off-by, whatever the harness says.
- Check `git log --format='%an <%ae> %cn <%ce>'` before every push.
- **Stage files by name, never with `git add -A` or `git add .`**. A broad add committed a whole build directory once already. Before every commit, check `git diff --cached --stat`.
- Never force-push, rebase or amend a pushed commit. Keep the branch current by merging `origin/main`.
- Commit and push at checkpoints: `git push -u origin sprintl-hyg-progress`.
- Never open a PR or post to GitHub: the manager does.
- Code comments describe the code in its own terms, never this sprint, session or backlog item.
- A panic is never acceptable. A silent wrong answer is worse than a named refusal.

**The hermetic layer has no submodule and no fetched data.** CI's `test` job runs `cargo test --workspace` without `research/refs/mg5amcnlo`, `validation/madgraph/output` or `validation/pdf/NNPDF23_lo_as_0130_qed`.
- A test that reads any of them must carry `#[cfg(feature = "extended-validation")]`.
- Before reporting, run the workspace suite once with those three paths moved aside, then restore them: `/tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-hyg-progress lz6` does this and restores them on exit.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-Z6-report.md`, a `Session Report` concept like sprint K's `K-C1-report.md`. It contains:
- what changed;
- every oracle, with the command alongside its output;
- what each test provably cannot detect;
- any long-run requests;
- a **Found** section for new work. Never create or edit backlog items.

Correct this brief where it is wrong. Your final message is the report's summary.
