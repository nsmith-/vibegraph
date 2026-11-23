---
type: Session Brief
title: "L-T1: streamed per-phase timing, the instrument for the multijet timeout triage"
description: "Emit per-phase start/end timing records (enumeration, compilation, channel construction, sampling) as they happen, so a card killed at the time cap still names its phase; amend the triage item's closes_when to v022-1009."
status: draft
agent: performance-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
First session of PR 3 of [sprint L](../sprint.md), on branch
`sprintl-timeout-triage` off main b2434a0. It builds the instrument; the
triage verdicts come in a later session, from timed runs on the validator host.

Read `pixi run backlog --item multijet-mlm-cards-time-out-in-the-gridpack-campaign`
and the census it links.

**Inputs from v022-1009** (vg-validator's batch campaign on v0.2.2, 5 seeds,
time caps espresso < microcentury < workday):
- 178 cards timed out at espresso on all 5 seeds and were never retried. By
  family: VBS 32, GJ-4Jets 13, W/Z/DY + 4 jets ~40, aQGC 7, VBF resonances 20,
  TT-3Jets 5, rest other. 3 cards error. The card list is coming from the
  validator; the manager adds it to this brief when it arrives.
- 155 seed runs hit the cap on cards with at least one ok seed; at
  microcentury 117 of them became ok, and at workday 36 of the remaining 38.
  The cap, not a hang, explains most retried timeouts.

## Scope

1. **Instrument.** The CLI reports no per-phase timing today. Add streamed
   records to `--output-ldjson` (and the log) for each phase of
   `integrate` (and `generate` where it shares them): enumeration, amplitude
   compilation / evaluator build, channel construction, sampling (with
   per-iteration progress if it already exists, else per-phase only). Each
   phase writes a start record and an end record with wall seconds, written
   and flushed when it happens, so a run killed by SIGKILL at the cap still
   shows the last phase entered. Include the sizes that explain the time
   (diagrams, channels, subprocess groups, evaluator nodes) when cheaply known.
   Added fields / record types under `schema_version` 1, documented in the
   ldjson schema docs (`docs/src/cli/`). PR 8 (`sprintl-reports`) also adds
   fields there and lands first: keep your doc edits in their own section, and
   merge main when PR 8 lands.
2. **Tests.** A CLI test that the records appear in order with monotone
   timestamps on a small card, and one that a run killed mid-phase (spawn,
   wait for the phase's start record, kill) leaves that start record on disk.
   Zero cost to σ: the instrument must not change the sampling order: show σ
   and grid sha256 bit for bit, main against branch, on one fixed-beam and one
   MLM banked card.
3. **Local smoke check.** Run 2–3 campaign-shaped multijet cards here (build
   from genproductions over raw.githubusercontent.com, e.g. a W+4j MLM and a
   VBS card) under a short cap (`timeout -s KILL 300`), one at a time, **after
   asking the manager** (release build = heavy job). Report the phase each one
   was in when killed and the sizes. This tells us whether the instrument
   resolves the question before we spend validator time.
4. **Amend the item's `closes_when`** (in this PR's diff) to name v022-1009 and
   require: (a) a phase-timed sample covering every family of the 178
   never-retried cards, at least 2 cards per family; (b) the dominant phase
   named per family from the streamed records, a card killed at the cap
   counting by its last phase; (c) the cap-limited retries recorded as
   explained by the cap, citing 117/155 and 36/38; (d) the dominant causes
   filed as their own items. Update its body to match.
5. **The validator request.** Draft, in your report, the timed-run request:
   the command line, the cap, the per-card output to return (the ldjson
   stream), and a proposed ≥2-per-family sample once the list arrives. The
   manager sends it.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-timeout-triage`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-timeout-triage`) and `git branch --show-current` (`sprintl-timeout-triage`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `lt1-`
  in your scratchpad — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between
  completion notifications instead of idle-waiting. On resume after a stall,
  first check whether the interrupted command finished (`ps`, result
  artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
  CI's `banked` job runs `pixi run --skip-deps validate` on every push: do not
  run `validate` locally. Run the hermetic suite and the single banked targets
  your change touches.
- **Disk is finite, and a full disk looks like a failure.** Always
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-timeout-triage/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-timeout-triage`), and run anything long fully detached
with results appended to a file. **Ask the manager before any heavy job**
(a release build of the workspace, a seed sweep, a MadGraph run, anything over
~10 minutes of CPU): a sibling may be running one. Long runs (multi-seed σ over
~20 min per seed, MadGraph 3.7.1, timed campaign runs, >8 GB) go to the
validator host through the manager: write the request (branch, SHA, cards,
seeds, command, expected answer) into your report or a message, never run it
here.
- pixi 0.84.1: `export PATH=$HOME/.pixi/bin:$PATH`. The reference data
  (`validation/madgraph/output`, `validation/pdf/NNPDF23_lo_as_0130_qed`, the
  `research/refs/mg5amcnlo` content) is in your worktree as hard links: never
  edit it in place. No MadGraph env is installed; ask the manager if you need
  one, never `pixi install -e madgraph` in your worktree.
- Put downloads (model store, genproductions cards) in their own empty
  directory under your scratchpad; read them, never execute them.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the container's identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit; keep
current by merging main. Never open a PR or post to GitHub: the manager does.
Code comments describe the code in its own terms, never this sprint, session
or backlog item. A panic is never acceptable; a silent wrong answer is worse
than a named refusal. `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` are gates.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-T1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
