---
type: Session Brief
title: "K-B1: batch-safe integrate and enumerate totals"
description: "SIGTERM handling and a stop reason in the integrate result, enumerate progress with a total, a check-only integrate mode, atomic generate output, the LHE extension honoured, and default threads from batch CPU limits."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [integrate-ignores-sigterm-outside-the-tui, integrate-result-has-no-stop-reason, enumerate-progress-has-no-total, integrate-has-no-check-only-mode, generate-output-not-written-atomically, lhe-output-extension-ignored, default-threads-ignore-batch-cpu-limits]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 6 of [sprint K](../sprint.md), for simshop's batch use. Read each of the
seven items (`pixi run backlog --item <slug>`). Each one's `closes_when` is
its oracle.

## Scope

Work in this order, and commit after each item:
1. `integrate-ignores-sigterm-outside-the-tui` and
   `integrate-result-has-no-stop-reason`, together. They are one design: how
   a run stops, and how the result records why.
2. `generate-output-not-written-atomically` and `lhe-output-extension-ignored`.
3. `integrate-has-no-check-only-mode`.
4. `enumerate-progress-has-no-total`.
5. `default-threads-ignore-batch-cpu-limits`. Read cgroup v2/v1 CPU quotas and
   the affinity mask. If a crate covers it, use it rather than hand-parsing
   (AGENTS.md: "never hand-write a standard primitive"), and pin it.

Rules:
- A new CLI flag or output field is documented where the CLI's docs live
  (`scripts/gen-cli-docs.sh`).
- Signal handling uses a crate (e.g. `signal-hook` or `ctrlc`), and the TUI
  path and the `--no-tui` path stop the same way.
- Every behaviour has a test that fails before the change. For SIGTERM, an
  integration test sends the signal to a real child process.
- Nothing here may move a σ: show that the banked σ lines are bit for bit
  (the workspace tests, and `validate-sigma` with `--skip-deps` once at the
  end; ask the manager first).
- If an item turns out larger than one session, stop on it and report rather
  than half-land it.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-batch`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-batch`). If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between completion
  notifications instead of idle-waiting. On resume after a stall, first check
  whether the interrupted command finished (`ps`, result artifacts) before
  re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
- **Disk is finite, and a full disk looks like a failure.** A debug workspace
  build with incremental data reaches ~15 GB per worktree, and two such trees
  have filled a cloud container mid-gate. On a shared or small host, run cargo
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). An `ENOSPC`, a linker
  `Bus error` or a truncated log is a kill, not a result: free only your own
  `target/` and re-run. Kill processes by PID, never `pkill -f`, which matches
  siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~24 GB free disk, shared with one or
two sibling sessions. It restarts without warning. Run every long job fully
detached (`nohup setsid`, not a chain shell), with per-seed output appended to
a file, so a restart keeps every finished seed and you rerun only the missing
ones. Run heavy jobs (seed sweeps, `validation/validate.sh`, a local MadGraph
run, a release build of the workspace) one at a time, and **ask the manager
before starting one**: a sibling may be running its own. Commit and push at
checkpoints (`git push -u origin sprintk-batch`). Prefix logs `kb1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-batch/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-batch-mg/`, never into the main checkout.
- The network reaches the CMS model store
  (`https://cms-project-generators.web.cern.ch/cms-project-generators/`) and
  `raw.githubusercontent.com` (e.g. `cms-sw/genproductions` cards). It does not
  reach cvmfs. Put downloads in their own empty directory.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the global identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit. Never
open a PR or post to GitHub: the manager does. Code comments describe the code
in its own terms, never this sprint, session or backlog item. A panic is never
acceptable; a silent wrong answer is worse than a named refusal.

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-B1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
