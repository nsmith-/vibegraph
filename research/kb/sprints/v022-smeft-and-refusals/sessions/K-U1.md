---
type: Session Brief
title: "K-U1: UFO value forms and import model options"
description: "Close the cheapest UFO-loader refusal classes: orders inferred without coupling_orders.py, import model --modelname, Python expressions and bare names in parameter values, coupling lists on vertices, order expressions, particle attributes naming parameters, and SMEFTsim's function library."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [coupling-orders-not-inferred-without-coupling-orders-py, proc-card-import-model-options-refused, ufo-parameter-value-expressions-refused, ufo-vertex-coupling-lists-refused, ufo-coupling-order-expressions-refused, ufo-particle-attribute-parameter-references-refused, ufo-parameter-value-bare-name-refused, smeftsim-function-library-real-part]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 3 of [sprint K](../sprint.md). Read each of the eight items
(`pixi run backlog --item <slug>`). Each one's `closes_when` is its oracle.
PR 2 (K-R1) is running in parallel on the restriction path
(`ufo/parameters.rs` `apply_restrict`, the `import model` suffix handling).
Stay out of those, and say where you had to touch them.

## Scope

Work in this order, and commit after each item:
1. `coupling-orders-not-inferred-without-coupling-orders-py`
2. `proc-card-import-model-options-refused`. This includes the gridpack-side
   `gpdb` rewrite under `validation/cms_gridpacks/` and its Python tests
   (`pixi run test-gpdb`).
3. `ufo-parameter-value-expressions-refused` and
   `ufo-parameter-value-bare-name-refused`: one evaluator change.
4. `ufo-vertex-coupling-lists-refused`
5. `ufo-coupling-order-expressions-refused`
6. `ufo-particle-attribute-parameter-references-refused`
7. `smeftsim-function-library-real-part`. Report whether any banked SMEFTsim
   value moves. If one does, stop and report before changing it.

Rules:
- Each fix follows MadGraph's import (`models/import_ufo.py`,
  `models/model_reader.py` at the pinned 3.7.1, b7687064). Cite the lines you
  reproduce. Never accept a form by guessing what it means: an unknown form
  stays a named refusal.
- Each item's named models or cards come from the CMS model store where
  possible. Where a model is only on cvmfs, build the smallest authored UFO
  under `validation/ufo/` that has the form, and name the substitution in the
  report.
- Every behaviour has a test that fails before the change.
- If an item turns out larger than one session, stop on it and report rather
  than half-land it.
- CI's `banked` job runs `validate` on every push and is the banked gate; do not
  run `validate.sh` locally.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-ufo-values`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-ufo-values`). If you
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
checkpoints (`git push -u origin sprintk-ufo-values`). Prefix logs `ku1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-ufo-values/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-ufo-values-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-U1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
