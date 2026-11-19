---
type: Session Brief
title: "K-H1: hygiene pass on the model-loading"
description: "Review sprint K's merged the model-loading PRs (#141, #144, #145) for maintainability, test non-vacuity, visibility and reusable abstractions; write findings first, then fix them with mutation evidence, σ bit for bit."
status: draft
agent: hygiene-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Hygiene pass of [sprint K](../sprint.md) (plan step 4), on branch `sprintk-hyg-model` off
main b3bc5399. It covers these merged PRs. Read each squash commit's diff
(`git show <sha>`) and its session report under `sessions/`:

- #141 `54ded749`: negative UFO coupling orders and the gluon-free WEIGHTED trial (K-O1).
- #144 `09490121`: restriction cards as MadGraph applies them (K-R1).
- #145 `f65d1bb7`: UFO value forms and import model options (K-U1).

A sibling hygiene pass runs in parallel on 2 cores; use `CARGO_BUILD_JOBS=2`.
Stay inside the files these PRs touched, plus their direct callers and tests.
Do not edit code another cluster owns (model loading: #141/#144/#145;
diagrams/colour/grouping: #146/#147/#150/#153; run card/CLI/batch:
#143/#152/#156; integration: #142/#155) except where a finding needs it, and
then say so.

## Scope

1. **Findings first.** Before editing anything, write the findings section of
   your report and commit it. For each finding give the file:line, what is
   wrong, the class (dead or duplicated code, a vacuous or tautological test,
   a public item that should be private, a hand-kept list that should be
   derived, a missing reusable abstraction, a comment that violates AGENTS.md's
   comment rules, a panic or unwrap on a fallible path), and the fix you
   propose. Rank them, and mark which ones you will fix here.
2. **Test non-vacuity.** For every test these PRs added, make one mutation of
   the code it guards that it should catch, run it, and record whether it
   failed. Any test that survives its mutation is a finding: strengthen it and
   show it now fails under the same mutation.
3. **Fix** the findings you marked. Behaviour is unchanged: σ and grid sha256
   are bit for bit against main on two banked cards, `dy13_default` (hadronic) and `pp_to_llj_mlm` (MLM), plus `enumerate` output identical on one SMEFTsim restricted process from K-R1's or K-O1's report
   (`integrate --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`, main
   built in a separate target dir and deleted afterwards).
4. **Physics or performance findings** (a wrong convention, a missing oracle,
   a slow path) are not yours to fix: list them under Found.

No seed sweeps, MadGraph runs or local `validate`: CI's banked job is the
banked gate. The gates are fmt, clippy `-D warnings`, `cargo test --workspace`
and kb-lint.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-hyg-model`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-hyg-model`). If you
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
checkpoints (`git push -u origin sprintk-hyg-model`). Prefix logs `kh1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-hyg-model/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-hyg-model-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-H1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
