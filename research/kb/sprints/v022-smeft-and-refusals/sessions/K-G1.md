---
type: Session Brief
title: "K-G1: integrate memory growth on the MLM γ + 4-jet card"
description: "Measure integrate's RSS against the VEGAS iteration count on GJ-4Jets HT-40to100 under a hard cap, name what grows with a heap profile, bound it, and show peak RSS flat in the iteration count with σ bit for bit where the change cannot reach it."
status: draft
agent: performance-dev (Opus)
depends_on: []
closes: [mlm-photon-four-jet-integrate-memory-grows-unbounded]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
PR 10 of [sprint K](../sprint.md), on branch `sprintk-mlm-memory` off main
f78fb60b. Read the item first:
`pixi run backlog --item mlm-photon-four-jet-integrate-memory-grows-unbounded`.
A small sibling PR (`sprintk-named-refusals`) builds and tests in parallel on
2 cores; you own the heavy-job slot.

**Inputs.** `/home/user/wt/sprintk-mlm-memory-mg/gj244/` holds gridpack 244's
`proc_card_mg5.dat` and `run_card.dat` (data; never edit; README one level
up). MLM γ + 0–4 jets, `sm-ckm_no_b_mass`, five-flavour `p`, xqcut 10,
htjmin 40 / htjmax 100, pta 200, lhaid 325300. Fetch lhaid 325300 from
`lhapdfsets.web.cern.ch` into `/home/user/inputs/pdf/` if it is not already
there (K-M1 may have fetched it; check first) and point `LHAPDF_DATA_PATH`
at it. Copy the cards into your own run directory if a tool wants to write
next to them.

The CERN host's observation: v0.2.1, seed 1, 4 threads, neval 120000, 1%
target, 23.9 GB RSS by VEGAS iteration 45 of 180. Other GJ-4Jets cards held
about 1.23 GB. That growth tracks iterations is **inferred** from one run.

## Memory safety: binding

This container has 15 GB RAM shared with the sprint session and a sibling.
**Every run of this card is capped at 8 GB RSS, hard.** Run it detached
(Python `os.setsid()` double-fork) under a watchdog script that samples the
process's RSS every 5 s to a log (`/proc/<pid>/status` VmRSS, with a
timestamp) and kills it **by PID** at 8 GB, writing `STOP: rss cap` as its
marker. Also apply the three-part discipline from AGENTS.md: the driver
writes a terminal marker on every exit path (`DONE`, `STOP: <why>`,
`FAILED rc=N`) and its PID; your watcher wakes on the marker or on
`! kill -0 <pid>`; and you read the first unit of real output (the first
VEGAS iteration's log line and RSS) and extrapolate before leaving the rest
to run. Use 2 threads (`-j 2`). Release build of `vibegraph` only
(`cargo build --release -p vibegraph`), not the workspace.

## Scope

1. **Measure before fixing.** RSS per VEGAS iteration on the card at reduced
   budget (choose `--neval` and the iteration cap so a run fits in well under
   an hour), with the iteration boundaries from the integrate log matched to
   the RSS samples. Report the slope (MB per iteration) and its shape (linear,
   steps at adaptation or MLM survey boundaries, saturating). If it does not
   grow at reduced budget, find the budget where it does; do not assume.
2. **Name what grows.** A heap profile (heaptrack, `dhat`, bytehound, jemalloc
   profiling: an existing tool or crate, never a hand-written allocator
   counter) of a short run, naming the allocation sites whose live bytes grow
   with iterations. Back it with arithmetic: per-item size × count = the
   measured slope.
3. **Bound it** (or justify and document it, if it is genuinely needed): peak
   RSS flat in the iteration count, measured on the same reduced run, before
   vs after. A test that would fail if the growth returned, where the
   structure allows one (e.g. a cache's size after N iterations).
4. **σ bit for bit** (σ and grid sha256) against main on `pp_to_llj_mlm` and
   one non-MLM banked card, if your change cannot reach σ; if it can, the
   multi-seed criterion (ask first: that is a sweep).
5. **Close-out run.** The item closes on the full card completing under a
   12 GB limit. If a bounded full run fits here under the 8 GB cap in a
   reasonable time, do it; otherwise say what it needs and the sprint
   session sends it to the CERN host.

Ask the sprint session before any run expected to exceed ~1 hour, any seed
sweep, and any MadGraph run.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-mlm-memory`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-mlm-memory`). If you
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
checkpoints (`git push -u origin sprintk-mlm-memory`). Prefix logs `kg1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-mlm-memory/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-mlm-memory-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-G1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
