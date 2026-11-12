---
type: Session Brief
title: "J-A1: refuse diagram and integrate artifacts from another enumeration"
description: "Record an enumeration version in both artifacts, refuse a mismatch by name with the remedy, and pin the version to the enumeration output of a fixed card set with a known-answer test."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [diagram-artifacts-replay-stale-enumeration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
---
Read the item (`python3 scripts/kb.py backlog --item
diagram-artifacts-replay-stale-enumeration`; it is filed on `sprintj-plan`, so
read it from `/home/user/wt/sprintj-plan` if your branch lacks it),
[the integrate artifact](../../../pipeline/integrate-artifact.md) (its checks
and the amplitude probe's blind spots), `vibegraph-lib/src/diagrams/artifact.rs`
and `vibegraph-lib/src/artifact.rs`.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintj-a1`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintj-stale-artifacts`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `ja1-`.
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

**Report** in `research/kb/sprints/v021-fixes/sessions/J-A1-report.md` (a
`Session Report` concept, as sprint I's reports): what changed, every oracle
with the command alongside its output, what each test provably cannot detect,
and a **Found** section for new work (never create or edit backlog items).
Correct this brief where it is wrong.

## Scope

1. **The constant.** `ENUMERATION_VERSION`, documented as covering everything
   the banked `DiagramSet`s hold that enumeration or conversion computes: the
   diagram list, Fermi `sign`, `symmetry_factor`, momentum routing,
   interaction indices, and whatever else you find in the type. Record it in
   the diagram artifact and in the integrate artifact. Both format versions
   bump.
2. **The refusal.** Reading either artifact with a different enumeration
   version is refused by name, naming both versions and the remedy (`vibegraph
   enumerate` again, or `integrate` again from the card). It is refused before
   anything is integrated or generated. One test per artifact, failing before
   the change. Older format versions stay refused as they are today.
3. **The known-answer test.** Pin `(ENUMERATION_VERSION, diagram_set_digest)`
   for a small fixed card set that reaches fermion flow signs, a decay chain,
   identical particles across decays, a Majorana-free four-fermion contact
   vertex, and a symmetry factor ≠ 1. The test fails on any change to the
   output, and its message tells the developer to bump the version and update
   the pinned digests together. Show by mutation (flip a Fermi sign in
   conversion, record the failing output, revert) that it catches an unbumped
   change. Keep it fast enough for the default `cargo test`.
4. **Docs.** `integrate-artifact.md` (the check list, the blind spots: name
   the residual one, a change that moves only cards outside the set) and the
   diagram artifact's module doc.

**The alternatives were considered and rejected** (the plan): re-enumerating on
read costs minutes at high multiplicity, and a build identity refuses on every
rebuild. If you find the known-answer test cannot be made stable (for example,
if enumeration order is not deterministic across platforms), stop and report.
