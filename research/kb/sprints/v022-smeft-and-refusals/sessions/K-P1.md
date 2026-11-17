---
type: Session Brief
title: "K-P1: one-body final-state phase space"
description: "Integrate 2 → 1 processes (p p > z, e+ e- > z) through a δ(ŝ − M²) one-body phase space following MadGraph's genps.f τ = m²/s, and remove the OneBodyFinalState refusals."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [one-body-final-state-phase-space-missing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 7 of [sprint K](../sprint.md). Read the item
(`pixi run backlog --item one-body-final-state-phase-space-missing`) and follow
its links.

## Scope

1. **Read MadGraph and cite it**: `genps.f` near line 276 at b7687064, for
   one final-state particle. Find the Jacobian (τ = m²/s, the δ(ŝ − M²) factor
   and its 2π/π conventions), how the width enters, if it does, and what
   MadEvent does on **fixed beams**, where √s is not M. Does it refuse, return
   0, or integrate a Breit–Wigner? Measure that with local MadGraph before you
   design the fixed-beam case; the item's "e+ e- > z" oracle assumes MadGraph
   gives a σ there. If it doesn't, the right answer may be a named refusal, or
   a σ at √s = M_Z. Say which in the report and correct this brief.
2. **Implement** the one-body map. A hadronic run integrates only the
   rapidity and the parton densities. Remove `ProtonError::OneBodyFinalState`
   and `refuse_one_body_final_state` where the new map covers the case. A
   multi-multiplicity card (`p p > z (j)` under MLM) must run its one-body part
   through the new map; check how the subprocess and channel setup handle a
   group with one final-state leg. PR 9 also edits the sampler's subprocess
   setup in parallel. Keep your changes there minimal and say where you
   touched it.
3. **Events**: the resonance is at rest in the partonic frame and boosted to
   the lab, and the LHEF record is valid. Run the LHEF gate (`validate-lhef`
   with `--skip-deps`).
4. **Oracles, measured on your head:**
   - σ for `p p > z` (hadronic, the NNPDF23 set the banked rows use) and for
     the fixed-beam case (as settled in step 1), each at ≥ 5 seeds against
     local MadGraph: unweighted mean, spread and χ²/dof.
   - A matched `p p > z (j)` card integrates (σ for the record, against
     MadGraph if time allows).
   - Every banked σ line unchanged. Your change cannot reach a card with ≥ 2
     final-state particles, so those lines stay bit for bit; show it.
   - Ask the manager before any MadGraph run or seed sweep.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-one-body`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-one-body`). If you
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
checkpoints (`git push -u origin sprintk-one-body`). Prefix logs `kp1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-one-body/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-one-body-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-P1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
