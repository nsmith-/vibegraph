---
type: Session Brief
title: "K-C1: run-card off sentinels, the resolved-card report and the classification table"
description: "Compile the run-card 'off' sentinels MadGraph leaves inert to the default cuts, report the resolved run card and each key's class without sampling, and ship a generated per-release classification table."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [run-card-off-sentinel-values-refused, ptj-n-min-nonpositive-refused, run-card-resolution-not-reported, run-card-classification-table-missing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-18}
---
First half of PR 5 of [sprint K](../sprint.md), on branch `sprintk-runcard` off
main 0340a89. The second half is K-C2 (the jet and lepton rank cuts,
`ptj1`–`ptj4` and `ptl1`–`ptl4` min/max). It follows in the same worktree once
its MadGraph inputs arrive. Leave the real-value rank cuts refused. Your
sentinel rules must be the ones K-C2 can extend: a value that would cut stays
refused, by name.

PR 4 (the MLM matching panic) has not started and may touch the run-card or
cut code later. Whichever of the two lands second merges main in.

Read each item (`pixi run backlog --item <slug>`) and the concepts they link:
- `run-card-off-sentinel-values-refused`
- `ptj-n-min-nonpositive-refused`
- `run-card-resolution-not-reported`
- `run-card-classification-table-missing`

The code is `vibegraph-lib/src/cuts.rs` (`UNIMPLEMENTED_CUTS` near L981,
`NONPOSITIVE_IS_OFF` near L1035 and its use near L2256) and
`vibegraph-lib/src/runcard/classes.rs` (`FIELD_CLASSES`).

## Scope

1. **Off sentinels.** For each parameter in the two sentinel items, cite the
   cuts.f / setcuts.f guard (pinned mg5amcnlo in `research/refs/mg5amcnlo`,
   `Template/LO/SubProcesses/`) that makes the value inert. Covered parameters:
   - `ptj1min`–`ptj4min` ≤ 0;
   - `ptl1min`–`ptl4min` ≤ 0;
   - `misset` and `ptheavy` at −1 (or ≤ 0, if the guard says so);
   - `ht2max`–`ht4max` at their template off value (read the guard: whether
     it is "≥ off value" or "< 0", and what the template writes).

   Each cited-inert value compiles to the default cuts. Any other value stays
   refused by name. Do not mark a sentinel inert without a citation, and do
   not widen a rule beyond what the guard shows. Tests: one per rule, inert
   and refused sides both.
2. **Resolved-card report**, as `run-card-resolution-not-reported` asks. Put
   it on whichever surface fits the CLI's existing shape (an `enumerate
   --run-card` ldjson record or a `cards` subcommand), without sampling.
   Report:
   - every key: its resolved value, MadGraph's LO default, whether the card
     moves it, and its `FIELD_CLASSES` class, plus the refusal integrate would
     emit for it;
   - the process-implied defaults.

   Pin it with a test against `RunCard` parsing for the committed validation
   cards and one refused card. Document it in the CLI docs (`docs/src/cli/`).
3. **Classification table**, as `run-card-classification-table-missing` asks:
   machine-readable JSON generated from the same lists, not hand-kept, with a
   one-line note per parameter. A test fails when a code list gains a
   parameter without a table entry. Item 2 and item 3 come from one
   implementation (sprint.md). How the table ships with a release (command
   output, a release asset or both) is your call; record why.
4. **Campaign check.** The item names 31 campaign cards. You have no cvmfs.
   Rebuild a representative card for each sentinel family from
   `cms-sw/genproductions` over `raw.githubusercontent.com`, or from the
   item's description. Show each one getting past the refusal (`enumerate`
   or a short `integrate`, not a seed sweep). No σ oracle is needed for
   inert values, but bit for bit is: σ and grid sha256 for main 0340a89
   against your branch on ≥ 3 banked cards with default cuts (one
   fixed-beam, one hadronic, one MLM). Build main in a separate target dir
   and delete it afterwards.

Ask the manager before any run longer than a few minutes. CI's `banked` job
is the banked gate: don't run `validate.sh` locally.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-runcard`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-runcard`). If you
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
checkpoints (`git push -u origin sprintk-runcard`). Prefix logs `kc1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-runcard/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-runcard-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-C1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
