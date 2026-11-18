---
type: Session Brief
title: "K-X1: split subprocess groups by outgoing masses"
description: "Let flavour grouping split subprocesses whose outgoing masses differ, as it already splits incoming masses, so ZPrimeToTT-type cards integrate; σ against MadGraph at ≥ 5 seeds."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [mixed-outgoing-mass-subprocesses-not-split]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 9 of [sprint K](../sprint.md). Read the item
(`pixi run backlog --item mixed-outgoing-mass-subprocesses-not-split`) and the
census it links.

PR 7 (#142, one-body phase space, not yet merged) also edits the sampler's
subprocess setup: the one-body block in `proton.rs` `derive_flavor_groups` and
one arm of `multiplicity.rs` `multiplicity_groups_of`. Stay out of those
blocks where you can. The second of the two PRs to merge will merge main in.

## Scope

1. **Reproduce** the refusal on a small card with the same structure: a decay
   chain whose lepton multiparticle includes the massive τ next to massless e
   and μ (e.g. `p p > t t~, t > b w+, w+ > l+ vl` with `l` including `ta`).
2. **Implement** the split. Group by outgoing masses the way incoming masses
   are already split, and sum the parts. Read how MadGraph groups
   subprocesses (`group_subprocs` and its mass criterion at the pinned 3.7.1)
   and cite it.
3. **Oracles, measured on your head:**
   - A test on the small card: the split parts and their union are the
     original subprocess set.
   - Cards whose groups already have uniform masses are bit for bit: σ and
     grid hash for main 703f57f against your branch on ≥ 3 banked cards (one
     fixed-beam, one hadronic, one MLM). Build main in a separate target dir
     and delete it afterwards.
   - σ on ZPrimeToTT_M2000GeV_W200GeV at ≥ 5 seeds against local MadGraph:
     unweighted mean, spread and χ²/dof. The card is on cvmfs; try
     `cms-sw/genproductions` over `raw.githubusercontent.com`, or rebuild
     it from the item's process string. First run one seed on each side and
     extrapolate. If the full card costs more than about 2 hours on this
     4-core box, stop and report: the manager decides whether a smaller card
     with the same mixed-mass structure stands in.
   - CI's `banked` job is the banked gate. Don't run `validate.sh` locally.
   - Ask the manager before each MadGraph run or seed sweep.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-mass-split`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-mass-split`). If you
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
checkpoints (`git push -u origin sprintk-mass-split`). Prefix logs `kx1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-mass-split/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-mass-split-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-X1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
