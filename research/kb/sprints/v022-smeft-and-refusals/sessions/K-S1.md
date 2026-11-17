---
type: Session Brief
title: "K-S1: the coloured-scalar seagull colour structure"
description: "Compile the squark-squark-gluon-gluon contact colour T(a,-1,i)*T(b,j,-1), match MadGraph per colour flow on g g > t1 t1~, and integrate an SMS-StopStop card."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [coloured-scalar-seagull-colour-unsupported]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 8 of [sprint K](../sprint.md). Read the item
(`pixi run backlog --item coloured-scalar-seagull-colour-unsupported`), then
`research/kb/validation/bit-exact-amplitude-debugging.md` and the colour
concepts it links. AGENTS.md's physics-validation rules are binding here:
validate at the finest linear level, per flow.

## Scope

1. **Reproduce** the refusal on `g g > t1 t1~` in the MSSM
   (`research/refs/mg5amcnlo/models/MSSM_SLHA2`, the copy in your worktree).
2. **Implement** the product of two generators sharing a summed index,
   `T(a,-1,i)*T(b,j,-1)`, in the colour compiler. Write it in general terms (a
   chain of generators between a 3 and a 3̄ slot), not as a special case of
   this one string, if that stays small. Say what the general form covers and
   what it still refuses by name.
3. **Oracles, measured on your head:**
   - Per-flow complex JAMP dumps against MadGraph standalone (pinned 3.7.1 in
     the submodule, run with the main checkout's madgraph env python) on
     `g g > t1 t1~` at fixed points, per helicity. Also `u u~ > t1 t1~` as a
     control without the seagull. Name each test's blind spot, e.g. a
     transposed index that a Gram-type CF matrix cannot see.
   - The colour-factor matrix against MadGraph's `matrix.f` CF for the same
     process.
   - σ on an SMS-StopStop_mStop-600 card. Try `cms-sw/genproductions` cards
     over `raw.githubusercontent.com`; otherwise write `p p > t1 t1~` at 13.6
     TeV with the MSSM default spectrum, M(t1) = 600 GeV. Use ≥ 5 seeds
     against local MadGraph: unweighted mean, spread and χ²/dof. Name any
     substitution.
   - CI's `banked` job runs `validate` on every push and is the banked gate.
     Don't run `validate.sh` locally. A colour change reaches every coloured
     row, so the workspace tests must pass before you push.
   - Ask the manager before each MadGraph run or seed sweep.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-seagull`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-seagull`). If you
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
checkpoints (`git push -u origin sprintk-seagull`). Prefix logs `ks1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-seagull/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-seagull-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-S1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
