---
type: Session Brief
title: "L-V1: attribute and fix the VBS stable-boson σ excess"
description: "Attribute the +3.4% to +12.7% σ excess on five CMS VBS cards per event or per diagram against MadGraph, on one polarised and one unpolarised card, then fix it or register it with its predicted size."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [vbs-stable-boson-sigma-reads-high]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 4 of [sprint L](../sprint.md), on branch `sprintl-vbs` off main b2434a0.

Read the item (`pixi run backlog --item vbs-stable-boson-sigma-reads-high`) and
[bit-exact amplitude debugging](../../../validation/bit-exact-amplitude-debugging.md).
Follow [expose, don't fix](../../../workflow/expose-dont-fix.md) for the diagnosis,
then fix it in this PR once the cause is named and checked by arithmetic.

The cards are CMS gridpacks 134, 135, 698 and 702 (polarised) and 700
(unpolarised): stable W/Z, `QED=4 QCD=0`, `etaj = 6.5`, `mmjj = 10`, `ptj = 10`.
Find them in `cms-sw/genproductions` (VBS / `WLWL`-style polarised cards); if you
cannot identify which gridpack numbers they are, say so early in a checkpoint
commit message and work from the closest polarised and unpolarised VBS cards with
those settings — the manager is asking the long-run host for the names.

## Scope

1. **Finest oracle first.** Reduce each card to the smallest subprocess that
   shows the excess (one partonic channel, e.g. `u d > w+ w+ d u`-type, fixed
   √ŝ or one PDF point). Compare |M|² point by point and then per diagram ×
   per helicity against MadGraph standalone (3.5.7 here; `build_amplitude.sh`,
   `compare_amps.py`). If |M|² agrees, move to the per-event fields: the scale,
   the cut decision, the PDF weight, the phase-space weight on MadGraph's own
   events (a per-event field is a finer oracle than σ).
2. **Candidates the item names:** t-channel photon exchange at low `mmjj`, the
   scale choice, Γ = 0 versus finite width on t-channel bosons, and — because
   the unpolarised card is high too — anything shared by both. Also check the
   polarised-boson definition (MadGraph's helicity-frame choice for `w+{0}`).
   Rule each in or out with a recorded measurement.
3. **Fix** with a test that would fail on the old behaviour, then σ on one
   polarised and the unpolarised card, ≥ 5 seeds, unweighted mean, spread and
   χ²/dof, against MadGraph. A card over ~20 minutes per seed goes to the
   long-run host via your report, with the expected answer; all five cards at
   the campaign budget go there too. If the cause is a MadGraph-side
   convention we should not copy, register it as a deviation with its predicted
   size instead.
4. **Nothing else moves** where the fix cannot reach: σ and grid sha256, main
   b2434a0 against your branch, on ≥ 3 banked cards. Build main in a separate
   target dir and delete it afterwards.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-vbs`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-vbs`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lv1-` — sibling sessions share that
  directory). A backgrounded command notifies on exit: never spawn sleep-based
  watcher shells (they compound into self-sustaining wake-up rings); draft your
  notes and report between completion notifications instead of idle-waiting. On
  resume after a stall, first check whether the interrupted command finished
  (`ps`, result artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration. Do
  not run `validation/validate.sh` or `pixi run validate` at all: CI's `banked`
  job runs it on every push. Run the specific `validate_*` test targets your
  change touches instead.
- **Disk is finite, and a full disk looks like a failure.** Always run cargo with
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-vbs/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). Each worktree has its own
  target dir: a shared one ran a sibling's stale test binaries in an earlier
  sprint. An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID, never
  `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, shared with two sibling dev agents. It
can restart without warning. **One heavy job at a time across the container**,
enforced by a lock rather than by asking: wrap every cargo build/test/clippy
invocation, every MadGraph run and every seed sweep in
`flock /home/user/wt/heavy.lock <command>` (e.g.
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lv1-test.log 2>&1`,
backgrounded). Hold the lock for one command at a time, never across an
interactive pause, and prefer targeted test filters to the whole workspace
while iterating; siblings queue behind you. `cargo fmt` and reading code need
no lock. A single job expected to run over ~20 minutes, a MadGraph 3.7.1
regeneration, a multi-seed σ on a card over ~20 minutes per seed, or anything
over 8 GB of memory does **not** run here: stop and put the request in your
report (branch, head SHA, cards, seeds, command, expected answer) — the
manager sends it to the long-run host. Run long jobs that do fit fully detached
(`nohup setsid … &`), with per-seed output appended to a file, so a restart
keeps every finished seed.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `research/refs/mg5amcnlo` submodule content) is already in your worktree:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under
  `/home/user/wt/sprintl-vbs-mg/`, never into the main checkout.
- The network reaches `raw.githubusercontent.com` (e.g. `cms-sw/genproductions`
  cards) and the CMS model store
  (`https://cms-project-generators.web.cern.ch/cms-project-generators/`). It does
  not reach cvmfs. Put downloads in their own empty directory.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the container's identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae> %cn <%ce>'`
before every push. Never force-push, rebase or amend a pushed commit; keep the
branch current by merging `origin/main`. Commit and push at checkpoints
(`git push -u origin sprintl-vbs`). Never open a PR or post to GitHub: the
manager does. Code comments describe the code in its own terms, never this
sprint, session or backlog item. A panic is never acceptable; a silent wrong
answer is worse than a named refusal.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-V1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
