---
type: Session Brief
title: "L-G1: attribute the g2HDM charged-Higgs high-mass σ excess"
description: "Attribute the +2.4% to +4.4% σ excess on the g2HDM charged-Higgs cards at M ≥ 900 GeV per event or per diagram against MadGraph on card 894 (M1000), with 889 (M300) as the control, then fix it or register it."
status: draft
agent: validation-dev (Opus)
depends_on: [L-V1]
closes: [g2hdm-charged-higgs-high-mass-sigma-reads-high]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 12 of [sprint L](../sprint.md), a stretch goal that does not hold the v0.2.3
tag, on branch `sprintl-g2hdm` off main 8093df4. It runs after PR 4 (VBS,
session `L-V1`): read its report first, since the two may share a cause.

Read the item (`pixi run backlog --item g2hdm-charged-higgs-high-mass-sigma-reads-high`)
and [bit-exact amplitude debugging](../../../validation/bit-exact-amplitude-debugging.md).

**Which MadGraph?** The cards were generated with MadGraph 2.9.18, which was the
outlier on DY1j. vg-validator is regenerating σ for 894 and 889 with MG 3.7.1.
Read every σ excess against that result: if 3.7.1 agrees with vibegraph, the
item becomes a registered MG 2.9.18 deviation; if it agrees with 2.9.18, the
attribution is the work. The manager hands you the exact cards and the 3.7.1
numbers at dispatch.

## Scope

1. **Cards.** Card 894 (M1000: Γ(H±) = 63.5971 GeV, Γ/M = 6.4%) and control 889
   (M300). Check which widths the param card sets and what each takes; the
   customize card sets H2/H3/H± to `Auto`, the madevent param card vibegraph read
   holds MadGraph's resolved numbers.
2. **Per-event first.** The process is MLM (`ickkw 1`, `xqcut 30`) with a decay
   chain `h+ > t b~, t > w+ b, w+ > l+ vl`. With `bwcutoff 15` the M1000 window
   is M ± 954 GeV, whose lower edge is below the t b threshold. Compare against
   MadGraph's own events: the Breit–Wigner window, the threshold treatment, the
   `small_width_ratio` the verdict applies, the scale and the merge sequence.
   Then per diagram × per helicity if the per-event fields agree.
3. **Fix or register** with a test that would fail on the old behaviour.
   Locally: reduced-budget single seeds and per-event checks only. Full runs take
   2.5–8 h per seed on 4 cores, so every multi-seed σ goes to vg-validator via
   your report (branch, SHA, cards, seeds, command, expected answer).
4. **Nothing else moves** where the fix cannot reach: σ and grid sha256, main
   against your branch, on ≥ 3 banked cards.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-g2hdm`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-g2hdm`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lg1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-g2hdm/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lg1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-g2hdm-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-g2hdm`). Never open a PR or post to GitHub: the
manager does. Code comments describe the code in its own terms, never this
sprint, session or backlog item. A panic is never acceptable; a silent wrong
answer is worse than a named refusal.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-G1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
