---
type: Session Brief
title: "K-R1: restriction cards as MadGraph applies them"
description: "Read MadGraph's restriction path and reproduce it: values equal to 1 (and 9.999999e-1), entries absent from the card, and the -full suffix; so the SMEFTsim per-operator restrictions load with MadGraph's restricted model."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [restriction-card-near-one-values-refused, restriction-card-missing-ckm-entry-refused, ufo-full-restriction-unsupported, restrict-card-unit-values-not-fixed]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
PR 2 of [sprint K](../sprint.md). Read the four items first
(`pixi run backlog --item <slug>` for each); their `closes_when` lines are
binding. PR 3 (K-U1) is running in parallel. It edits the UFO value parser and
the coupling and particle loaders, so stay inside the restriction path
(`ufo/parameters.rs` `apply_restrict` and the `import model` name and suffix
handling). Say where you had to step outside it.

## Scope

1. **Read MadGraph and cite it** (file:line at the pinned 3.7.1, b7687064). The
   restriction code is in `models/import_ufo.py` (`RestrictModel`, `restrict_model`,
   `detect_identical_couplings`, `put_parameters_to_zero`/`fix_parameter_values`,
   and how a value counts as 1: the tolerance, sign, and whether −1 counts too).
   Also: what happens to a parameter the card does not mention, and how `-full`
   and a bare name differ. Write the reading into the report before you change
   code.
2. **Reproduce it.** Fixed-at-1 parameters, near-one values, absent entries and
   `-full`. If MadGraph's rule is a tolerance, use its constant and cite it.
3. **Oracles, measured on your head:**
   - On one SMEFTsim per-operator restriction from the CMS model store (e.g. a
     `SMEFTsim_U35_MwScheme_UFO_b_massless` card that uses 9.999999e-1 and
     omits a CKM entry): the restricted model's free-parameter set and its
     non-zero coupling set equal MadGraph's. Get MadGraph's from
     `import model X-<restriction>` followed by `output standalone` (its
     `param_card.dat` and `couplings.f`/`coupling` list) or from its restricted
     model dump. Ask the manager before the MadGraph run.
   - All 22 restrictions named in `restriction-card-missing-ckm-entry-refused`
     load. List them with pass/fail.
   - `-full` on a model that ships `restrict_default.dat` (the SM does) loads
     unrestricted. Test that its parameter set differs from the bare name's
     where `restrict_default` fixes something.
   - Everything banked is unchanged. A restriction change can move SMEFTsim
     banked rows, so name which ones it reaches. CI's `banked` job runs
     `validate` on every push and is the gate; do not run `validate.sh`
     locally.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-restriction`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-restriction`). If you
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
checkpoints (`git push -u origin sprintk-restriction`). Prefix logs `kr1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-restriction/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-restriction-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-R1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
