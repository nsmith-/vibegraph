---
type: Session Brief
title: "K-N1: two named refusals — the fixed-beam sampled-point scale fault and misset's √ŝ floor"
description: "Turn the fixed-beam path's per-event scale panic into the named sampled-point refusal the proton path already uses, and settle whether MadGraph's misset² term in setcuts.f's lepton ŝ floor can bind at misset = -1: apply it, or refuse by name where it binds."
status: draft
agent: validation-dev (Sonnet)
depends_on: []
closes: [fixed-beam-sampled-point-scale-error-panics, misset-negative-sqrt-shat-floor-unapplied]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
PR 12 of [sprint K](../sprint.md), on branch `sprintk-named-refusals` off main
f78fb60b. PR 10 (`sprintk-mlm-memory`) runs in parallel and owns the machine's
heavy-job slot: no seed sweep, MadGraph run or release build without asking
the sprint session first. Use `CARGO_BUILD_JOBS=2`.

Neither item is on main yet; both are filed in the sprint record (#148). Their
text is below, so there is no file to delete. Account for each in your report.

## Part A: the fixed-beam sampled-point scale panic (high)

`vibegraph-lib/src/hadronic.rs`, `apply_scale` (about line 2746 on main):
`event_scales_of(...).unwrap_or_else(|e| panic!("per-event scale on a sampled
point: {e}"))`. A panic is never acceptable. PR #153 (`74596ba3`) fixed the
same pattern on the proton path: `SampledPointFault`, `record_fault` into a
`OnceLock`, the point gets zero weight, and `integrate` / `generate` refuse
by name after the survey/integration (read `proton.rs` around
`SampledPointFault` and its callers in `multiplicity.rs`, `generate.rs` and
`vibegraph-cli/src/integrate.rs`). Do the same on the fixed-beam path, reusing
that type and those refusal sites rather than a parallel copy. Search the
fixed-beam path (and anything else reachable from a sampled point) for any
other `panic!`/`unwrap`/`expect` on a fallible per-point result and list what
you find; fix the ones of the same kind.

Oracle: a failing-first test that drives a fixed-beam sampled point into a
scale error (construct the error condition; say how) and shows a named
refusal, not a panic. Bit for bit (σ and grid sha256) against main on one
fixed-beam banked card and one hadronic card with default cuts.

## Part B: misset = -1 and setcuts.f's √ŝ floor

`setcuts.f` (pinned template, `research/refs/mg5amcnlo/Template/LO/SubProcesses/setcuts.f`,
around line 675) adds, for a subprocess with a charged lepton,
`max(smin_p**2, smin_m, mmnl**2, ptllmin**2, misset**2)` to `smin`. vibegraph
treats `misset <= 0` as off (`NONPOSITIVE_IS_OFF`, `cuts.rs`, whose comment
names this as a residual): at -1 MadGraph's floor gains a 1 GeV² term that
vibegraph does not apply. Seven campaign cards carry `misset = -1` (the older
template's off value, alongside `ptheavy` and `ptl1min`–`ptl4min` = -1).

1. Read how vibegraph builds its τ / ŝ floor and whether it reproduces this
   lepton term (and the rest of `smin`). State exactly when `misset**2` is the
   maximum, i.e. when the term can raise MadGraph's floor above vibegraph's.
2. Settle whether a raised floor can change σ or the event record: `smin` is a
   floor on the sampled τ, so it cuts only if the region it adds has support
   under the other cuts. Do not assume either way; the default lepton cuts
   (`ptl`, `mmll`) are not guaranteed on a CMS card.
3. Then, the manager's rule: if the floor provably cannot bind, record the
   proof in `cuts.rs`'s comment and the report. If it can bind, either
   **implement** the term (one more argument in the lepton max, exactly as
   setcuts.f has it) with a test that pins vibegraph's floor against
   MadGraph's for a card where the term is the maximum, or **refuse**
   `misset < 0` by name where the term would be the maximum. Prefer
   implementing if it is the one-term change it looks like. A per-event or
   per-card intermediate (the floor value itself, MadGraph's own computed
   `smin` from a compiled `setcuts`) is a finer oracle than σ, where the
   1 GeV² sliver is invisible; you may compile MadGraph's `setcuts` path on
   a small lepton process (ask first: it is a MadGraph run).
4. Bit for bit against main on banked cards where the term is not the maximum
   (any card with a lepton pT cut ≥ 1 GeV).

## Not in scope

`misset > 0` (the bbDM missing-ET cut, `misset-cut-unimplemented`) stays
refused.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-named-refusals`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-named-refusals`). If you
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
checkpoints (`git push -u origin sprintk-named-refusals`). Prefix logs `kn1-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-named-refusals/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-named-refusals-mg/`, never into the main checkout.
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

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-N1-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
