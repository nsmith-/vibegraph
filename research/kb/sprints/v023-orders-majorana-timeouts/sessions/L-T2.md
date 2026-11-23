---
type: Session Brief
title: "L-T2: per-family timeout verdicts from the v022-1009 phase sample"
description: "Read the 21-card phase-timed sample, name the dominant phase and cause per family, record the cap-limited retries as explained, draft the cause items, and close the triage item."
status: draft
agent: performance-dev (Opus)
depends_on: [L-T1]
closes: [multijet-mlm-cards-time-out-in-the-gridpack-campaign]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Second session of PR 3 of [sprint L](../sprint.md), on branch
`sprintl-timeout-triage` (PR #173). L-T1 built the instrument
([L-T1 report](L-T1-report.md)); `vg-validator` ran it on 21 of the 178
never-retried cards. The values are in
[v022-1009-phase-sample.md](../v022-1009-phase-sample.md). Read the item
(`pixi run backlog --item multijet-mlm-cards-time-out-in-the-gridpack-campaign`),
the sample page and L-T1's report first.

The instrument code on this branch is also on PR 3a (#183), which lands first.
Do not edit `vibegraph-cli/src/phase.rs` or the phase wiring in this session:
the manager merges main into this branch once 3a lands, and any instrument edit
here would conflict. This session is analysis plus the kb.

## Scope

1. **Verdict per family** (closes_when (b)). For each of the seven families,
   the dominant phase from the records (a killed card counts by its last
   started phase) and, where the sizes and code say so, the cause *within*
   the phase. Read the code paths for the phases that dominate; small local
   measurements on hand-written cards of the same shape are fine (dev
   profile, `timeout -s KILL 300`, one at a time, **ask the manager first**).
   Questions the sample raises, answer each or state why it stays open:
   - **Enumerate on W/Z/DY+4j** (58, 15, 1009 never left it in 1200 s at
     `-j 4 --parallel-diagrams`): what is "done N of M" counting, is the walk
     exponential in the line's legs, and which part of it would parallelise
     or prune? L-T1's W+4j smoke run is a second data point.
   - **Amplitude probe** (65: 1077 s; 37: died there at 4.6 GB): L-T1 Found
     that every set is compiled twice. Is the probe's cost and memory the
     duplicate compile, and would reusing the compile's evaluators remove it?
   - **VEGAS at 1 iteration** (GJ, TT, aQGC 195, older GJets, TT FCNC: "done
     1" with 1.4e5 to 2.2e6 ns/eval): is it per-eval cost (helicities ×
     flows × diagrams), the warm-up schedule L-T1 Found on ssWW, or the
     point budget `--neval 120000 --target-rel 0.01` asks for? Give the
     arithmetic: points the run needed × ns/eval against the cap.
   - **alpha_survey** (VBS 133, 14, 710; VBF 655: 400–900 s): what does the
     survey cost per point and why so many points?
   - **Card 290 now refuses** in 14 s (massless spacelike propagator, beam
     uncut) where v0.2.2 timed out. Is the refusal correct for that card's
     cuts (compare MadGraph's handling: does MG apply a default cut, or would
     it also integrate a divergence)? If the refusal is right, the card left
     the timeout list for a named reason; say so.
   - **Card 722 finished** in 1079 s under the 1200 s cap, where all five
     v0.2.2 seeds timed out. Borderline cost, or did something between
     v0.2.2 and 4aab24fb speed it up? `git log` between the tags is the
     evidence.
   - **Card 37 hit memory, not time.** Peak 4.6 GB in the probe for
     `p p > 5j`-class lines: name where the memory goes.
2. **(c) the cap-limited retries**: record them in the item's closing account
   (your report) as explained by the cap, citing 117 of 155 at microcentury
   and 36 of the remaining 38 at workday.
3. **(d) the causes as items.** Dev agents never create backlog items. Draft
   each cause in your report's **Found** as a ready-to-file item: slug,
   title, area, priority, `closes_when`, and a body with the evidence (card
   ids, phase times, sizes). The manager files them in this PR. Expect a
   handful (enumerate cost, the duplicate probe compile, VEGAS per-eval or
   budget, alpha_survey cost, memory), not one per card. Where an existing
   item already covers a cause (`pixi run backlog` lists them; look at least
   for `mlm-vanishing-flavour-groups-still-sampled`,
   `zero-weight-integrand-runs-to-point-cap`,
   `generate-parallel-diagrams-is-a-no-op`), cite it instead of drafting a
   duplicate.
4. **(a) coverage**: the sample covers every family with ≥2 cards and each
   'other' group with one. Check this against
   [v022-1009-never-retried.tsv](../v022-1009-never-retried.tsv) and state
   it.
5. **Close the item**: delete
   `research/kb/backlog/performance/multijet-mlm-cards-time-out-in-the-gridpack-campaign.md`
   in your final commit. Its account goes in your report. Check that
   nothing else in the kb links to it (`pixi run --skip-deps kb-lint`).

What this session cannot claim: a verdict extrapolated from 2–4 cards to a
family of up to 56 is a sample, not a census. Say per family how uniform the
records are and which sub-shapes the sample did not cover.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-timeout-triage`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-timeout-triage`) and `git branch --show-current` (`sprintl-timeout-triage`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `lt2--`
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
