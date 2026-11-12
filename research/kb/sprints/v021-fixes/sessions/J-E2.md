---
type: Session Brief
title: "J-E2: fixed-energy sde_strategy = 2 with several subprocesses"
description: "Establish whether any card reaches the fixed-energy sde_strategy = 2 path with two subprocesses of different diagram content; fix or refuse it if so, otherwise pin the refusal that makes it unreachable and re-scope the item with the evidence."
status: draft
agent: validation-dev (Sonnet)
depends_on: []
closes: [sde-strategy-2-weights-only-first-fixed-energy-subprocess]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
---
Read the item (`python3 scripts/kb.py backlog --item
sde-strategy-2-weights-only-first-fixed-energy-subprocess`) and
[MadEvent single-diagram enhancement](../../../phase-space/madevent-single-diagram-enhancement.md).
A reviewer tried three two-subprocess fixed-energy cards; each was refused by
name before reaching the path.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintj-e2`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintj-sde`). If you
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

This container restarts without warning. Run every long job fully detached
(`nohup setsid`, not a chain shell), with per-seed output appended to a file,
so a restart keeps every finished seed and you rerun only the missing ones.
Run heavy jobs (seed sweeps, `validation/validate.sh`, local MadGraph) one at a
time (4 cores), and ask the manager before starting one: a sibling session may
be running its own. Commit and push at checkpoints. Prefix logs `je2-`.
All sessions share `CARGO_TARGET_DIR=/home/user/cargo-target` (export it in every
cargo command), so dependencies build once; a build may wait on another
session's cargo lock, which is expected. The reference data (`validation/madgraph/output`,
`validation/pdf` sets, the `mg5amcnlo` submodule) is already in your worktree.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the global identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit. Never
open a PR or post to GitHub: the manager does. Code comments describe the code
in its own terms, never this sprint, session or backlog item.

**Report** in `research/kb/sprints/v021-fixes/sessions/J-E2-report.md` (a
`Session Report` concept, as sprint I's reports): what changed, every oracle
with the command alongside its output, what each test provably cannot detect,
and a **Found** section for new work (never create or edit backlog items).
Correct this brief where it is wrong.

## Scope

1. **Reachability.** From the code, list every way a fixed-energy card yields
   more than one subprocess (`add process`, multi-particle labels, decay-chain
   lines, polarizations) and which refusal (`InconsistentExternals`,
   enumeration's duplicate-subprocess check, others) stops each before
   `use_running_coupling`. Try the candidates the item names: decay-chain lines
   reaching the same ordered final state through different assignments, and
   polarized legs. Record each card and its outcome.
2. **If reachable:** a test on that card that fails before the change (the
   panic, or a wrong configuration draw shown by its `ChannelSet`), then either
   derive one `ChannelSet` per subprocess as the proton path does, indexed by
   subprocess in the share and event-generation paths, or refuse it by name.
   Say which and why.
3. **If unreachable:** a test that pins the refusal making it so, with an
   argument for why no other card gets through. Then add a `debug_assert` or a
   typed error at the site, so a later change that opens the path fails loudly
   rather than drawing from the wrong forest. Report the re-scoped
   `closes_when` you recommend; the manager edits or closes the item.

The manager merges your branch into `sprintj-panics`; do not touch J-E1's
branch.
