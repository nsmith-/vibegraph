---
type: Session Brief
title: "J-E1: the SMEFTsim four-quark compile panic"
description: "Root-cause the empty Op::Add that panics AmplitudeEvaluator::compile on SMEFTsim u u~ > t t~ b b~ NP<=1, fix it if tractable or refuse by name, and compare per diagram against MadGraph standalone."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [smeft-four-quark-compile-panics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
---
Read the item (`python3 scripts/kb.py backlog --item
smeft-four-quark-compile-panics`): it has the reproducer and what is already
ruled out. Read `research/kb/validation/bit-exact-amplitude-debugging.md` for
the per-diagram comparison method.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintj-e1`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintj-panics`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `je1-`.
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

**Report** in `research/kb/sprints/v021-fixes/sessions/J-E1-report.md` (a
`Session Report` concept, as sprint I's reports): what changed, every oracle
with the command alongside its output, what each test provably cannot detect,
and a **Found** section for new work (never create or edit backlog items).
Correct this brief where it is wrong.

## Scope

1. **Reproduce** on this branch and capture the panic.
2. **Name the cause.** Find which stage (lowering, `optimize`, rooting,
   folding) produced an `Op::Add` with no operands, and from what. Decide
   which it is: a sum whose every term vanished (benign, so the fix gives it a
   defined value or does not emit it), or a colour flow or configuration that
   lost its contributions (a dropped diagram, so the fix is upstream). Show
   the evidence for the call.
3. **Fix or refuse.** Fix the root cause with a regression test on the
   smallest card that reaches it, failing before the change. If a fix is not
   tractable, refuse the card by name before compilation, with a test. It must
   never panic.
4. **Oracle.** Per-diagram × per-helicity amplitudes against MadGraph
   standalone at a fixed point, as the other SMEFTsim rows do. Ask the manager
   before a MadGraph run: standalone output of 2,517 diagrams may be slow, so
   measure a smaller sibling first and extrapolate. A short `integrate` σ for
   the record.
5. Everything banked is unchanged: workspace tests, and `validate.sh` if the
   change reaches compilation.
