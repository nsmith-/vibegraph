---
type: Session Brief
title: "K-O1: negative UFO coupling orders and the gluon-free WEIGHTED trial"
description: "Load couplings with a negative coupling order (SMEFTsim's QED:-1 vev insertion) the way MadGraph counts them, in the diagram order counting and WEIGHTED, and run MadGraph's gluon-free trial in the WEIGHTED search."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [negative-ufo-coupling-orders-refused, weighted-search-gluon-free-trial-missing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 1 of [sprint K](../sprint.md). Read the two items first
(`pixi run backlog --item negative-ufo-coupling-orders-refused`, and the same
for `weighted-search-gluon-free-trial-missing`), then
`research/kb/validation/bit-exact-amplitude-debugging.md`.

## Scope

1. **Read MadGraph first, and cite it** (file:line at the submodule's
   b7687064). How does a negative order enter the per-diagram order sum, the
   `WEIGHTED` weight (`hierarchy`), `QED<=n` / `NP<=1` bounds and
   `find_optimal_process_orders`? Is a negative contribution clamped anywhere?
   Write that reading into the report before you change any code.
2. **Reproduce MadGraph.** Orders must go signed through
   `vibegraph-lib/src/ufo/couplings.rs` (`extract_orders`) and every
   downstream consumer: diagram order counting, `WEIGHTED`, the bounds and the
   refusal texts. Banked diagram counts stay unchanged on every row. Add a test
   that loads a coupling with a negative order.
3. **Gluon-free trial.** Reproduce `diagram_generation.py:2040-2059`: drop
   final-state gluons in the lowest class from each trial, including the
   empty-final-state case. Un-ignore
   `a_final_state_gluon_pair_without_a_qcd_vertex_is_refused` and make it pass.
4. **Oracles, each measured on your head:**
   - Per-diagram × per-helicity complex amplitudes on a SMEFT_alpha or SMEFTsim
     v2 A-model process that has a `QED:-1` vertex, against MadGraph standalone
     at a fixed point. Get the model from the CMS model store. Choose a small
     process; ask the manager before the MadGraph run.
   - Diagram counts against MadGraph for that process under `NP<=1`.
   - σ on a GluGluH_HToUpsilon card at ≥ 5 seeds against MadGraph: unweighted
     mean, spread and χ²/dof. The card and model are on cvmfs, which is out of
     reach here; try `cms-sw/genproductions` on GitHub for the cards and the
     model store for the model. If the card can't be had, say so. Then
     substitute σ on the SMEFT process above at ≥ 5 seeds and name the
     substitution in the report.
   - Every banked row unchanged: workspace tests, then `validate.sh` with
     `--skip-deps` (ask first). The change reaches diagram enumeration, so
     invoke the `extended-validation` skill for which gates apply.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-negative-orders`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-negative-orders`). If you
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
checkpoints (`git push -u origin sprintk-negative-orders`). Prefix logs `ko1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-negative-orders/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-negative-orders-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-O1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
