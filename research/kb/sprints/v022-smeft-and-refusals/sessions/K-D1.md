---
type: Session Brief
title: "K-D1: σ 4× MadGraph's on a decay chain with repeated decays"
description: "Attribute and fix the factor of 4 on p p > w+ h01, (h01 > a01 a01, a01 > b b~, a01 > b b~): per-diagram amplitudes and the symmetry factor against MadGraph standalone, the cause checked by arithmetic on the dumps, then σ on the 4 HToAATo4B cards."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [decay-chain-repeated-decay-sigma-four-times-high]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 11 of [sprint K](../sprint.md), added by the user on 2026-11-17. It is a
**silent wrong answer, so it blocks the release.**

The item, `decay-chain-repeated-decay-sigma-four-times-high`, is being added to
main by #149 and is not on your base yet. Its text:

> **closes_when:** The factor of 4 is attributed: per-diagram amplitudes and the
> symmetry factor are compared with MadGraph standalone on the repro, and the
> cause is named. It is fixed so the 4 cards match MadGraph by the multi-seed
> criterion, with a test that fails before the fix. The library's decay-chain
> symmetry factor is asserted against MadGraph's identical_particle_factor on
> this card and on a repeated-decay census case.
>
> Reported by the CERN host, stage 2 of its 1009-gridpack survey (5 seeds,
> v0.2.1). Cards: WH, VBFH and TTH `HToAATo4B`, gridpacks 88, 89, 448 and 452,
> at M-15/40/45. Process `p p > w+ h01 , (h01 > a01 a01, a01 > b b~, a01 > b b~)`.
> σ_vg/σ_MG = 3.993, 3.992, 4.022, 4.013. Two readings: the 1/(2!·2!) for the
> two identical b and b̄ is missing, or each of the two `a01 > b b~` specs is
> applied to both a01 (a doubled assignment).

The related open item on your base is
`pixi run backlog --item decay-chain-identical-factor-unasserted`. Read
`research/kb/validation/bit-exact-amplitude-debugging.md` first: it is the
method here. AGENTS.md's physics-validation rules are binding.

## Scope

1. **Get the model and cards.** The HToAATo4B cards are in `cms-sw/genproductions`.
   A blobless `git clone --filter=blob:none` works here; `raw.githubusercontent.com`
   paths have 404'd. The model (h01, a01: likely an NMSSM/2HDM+S-type UFO) may be
   in the CMS model store. If neither has it, say so. The manager can get the
   card from the CERN host, and the process line alone may be enough with a
   HAHM-like model.
2. **Diagnose before changing code.**
   - Dump the diagram/assignment census and the symmetry factor vibegraph uses
     for this process.
   - Dump MadGraph's diagrams, `IDEN` and `identical_particle_factor` for the
     same process.
   - Compare per-diagram × per-helicity amplitudes at a fixed point against
     MadGraph standalone.
   - Name which reading holds (or another), and show by arithmetic on the dumped
     numbers that it accounts for exactly 4 before writing the fix.
3. **Fix**, with a test that fails before the change. Also assert the library's
   decay-chain symmetry factor against MadGraph's on this card and on one more
   repeated-decay case, e.g. `h > z z, z > l+ l-, z > l+ l-`, or a census case
   from the banked decay rows.
4. **Oracles, measured on your head:**
   - σ on the HToAATo4B cards (as many of the 4 as you can obtain, at least 2
     masses) at ≥ 5 seeds against local MadGraph: unweighted mean, spread and
     χ²/dof.
   - Cards without repeated decays are bit for bit: σ and grid hash, main
     703f57f against your branch, on ≥ 3 banked decay-chain cards. Build main in
     a separate target dir and delete it afterwards. K-X1's
     `/home/user/wt/sprintk-mass-split-mg/` has a bitdiff driver you can adapt.
   - CI's `banked` job is the banked gate. Do not run `validate.sh` locally.
   - Ask the manager before each MadGraph run or seed sweep.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-repeated-decay`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-repeated-decay`). If you
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
checkpoints (`git push -u origin sprintk-repeated-decay`). Prefix logs `kd1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-repeated-decay/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-repeated-decay-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-D1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
