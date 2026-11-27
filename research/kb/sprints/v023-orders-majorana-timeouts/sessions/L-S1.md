---
type: Session Brief
title: "L-S1: multicore generate, with accept/reject on deterministic substreams independent of -j"
description: "Parallelise generate's accept/reject over N threads on per-block random substreams so the output is fixed by the seed alone; measure the speed-up and record the two 2-to-6 samples cells."
status: draft
agent: performance-dev (Opus)
depends_on: []
closes: [two-to-six-samples-serial-unweighting-cost]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 13 of [sprint L](../sprint.md), on branch `sprintl-generate` off main
e8da7a4. The user added it on 2026-11-22.

Read `pixi run backlog --item two-to-six-samples-serial-unweighting-cost` and
what it links (the samples cells in `validation/manifest.toml`, note 32 §5.1).

**Why.** `generate -j N` on v0.2.2 keeps about one core busy. Measured by simshop:

| Card | CPUs | CPU / wall | Utilisation |
|---|---|---|---|
| DY+jet SMEFT, 868 diagrams | 2 | 107 s / 111 s | 48% |
| ttbar | 2 | — | 42% |
| shard | 4 | — | 26% |

Only the w_max scan is parallel. The accept/reject loop advances one random trial
at a time.

## Scope

1. **Parallel accept/reject.** `generate -j N` scales accept/reject across N
   threads.
   - Suggested shape:
     - Split the trial sequence into fixed-size blocks.
     - Give each block its own random substream derived from (seed, block index),
       the way integrate derives its point substreams. Read how integrate does it
       and reuse that derivation; don't invent a new one.
     - Process blocks in parallel, and take accepted events **in block order**
       until the requested count.
     - Discard speculative work past that point, so nothing depends on
       scheduling.
   - Overweight handling, truncation, the per-channel bookkeeping and every
     reported statistic (sample σ, pull, efficiency) must follow the same rules
     as the serial path.
   - If LHE writing or anything after accept/reject becomes the next bottleneck,
     measure it and say so. Fix it only if the fix is cheap.
2. **Determinism.** The output is fixed by the seed and **independent of N**.
   - A seed's event stream will differ from v0.2.2 even at `-j 1`; that is
     accepted.
   - Write the release-note line in the user-facing changelog or release notes
     the repository keeps: same grid, same seed, different events; the sample is
     statistically equivalent; the output no longer depends on -j.
   - The integrate side is out of scope: don't change integrate's streams or the
     artifact format. Every banked σ line must stay bit for bit.
3. **Gates.**
   - **Hermetic test:** `-j 1`, `-j 2` and `-j 4` give byte-identical LHE files
     on at least two processes, one of them a multi-channel hadronic card. Use
     the banked NNPDF set behind `extended-validation` where a proton card needs
     it, as the other proton CLI tests do.
   - **Statistical gate:** the unweighting and samples gates pass at ≥5 seeds.
     Read the spread and χ²/dof, not one seed's pull. Any committed fixed-seed
     reference that encodes the old stream must be re-derived, never loosened.
     Say which ones were and why.
   - **Speed-up:** measured at -j 1, -j 2 and -j 4 on a ttbar card and on a
     heavier multi-diagram card, using this container's 4 cores. Report wall,
     CPU and utilisation. Run these one at a time, after asking the manager.
   - **2→6 samples cells:** `uux_to_ccx_emmm_qcd0` and `bbx_to_ccx_emmm_qcd0`
     must be run and recorded, as the item's closes_when asks.
     - Estimate their cost at -j 4 from the probe first.
     - If they run past about 20 minutes even in parallel, stop and send the
       manager a validator-host request (branch, SHA, command, expected answer)
       instead of running them here.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-generate`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-generate`) and `git branch --show-current` (`sprintl-generate`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `ls1-`
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-generate/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-generate`), and run anything long fully detached
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
