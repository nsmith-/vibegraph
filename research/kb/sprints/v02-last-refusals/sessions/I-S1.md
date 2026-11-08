---
type: Session Brief
title: "I-S1: bound each peripheral rung by the thresholds of the legs it can reach"
description: "Cuts::spacelike_floor caps every peripheral rung at the largest single-leg pT_min²; derive a provable bound per rung from its adjacent legs (or fall back to the smallest threshold), and gate p p > a j j at pta 40, ptj 0.1, draj 0.4 against the CERN host's MadGraph."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [spacelike-floor-cuts-phase-space-below-the-largest-pt-threshold]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
---
Read the item (`python3 scripts/kb.py backlog --item
spacelike-floor-cuts-phase-space-below-the-largest-pt-threshold`), its
performance-side twin `spacelike-floor-too-loose`, `Cuts::spacelike_floor`'s
doc (`vibegraph-lib/src/cuts.rs` ~:1107) and its uses in
`phasespace/diagram_channel.rs` and `proton.rs` (`UnionBounds`).

**User decision: the CERN host establishes the MadGraph reference first.** The
manager has asked the host for MadGraph 3.7.1 σ on `p p > a j j` at `pta 40`,
`ptj 0.1`, `draj 0.4` (and `ptj 1` as a control), with ≥ 5-seed vibegraph σ for
stock and for the smallest-threshold build. The manager relays the numbers;
until then do steps 1–3, which need no reference.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprinti-s1`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprinti-s1`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `is1-`.
pixi is not installed here: run gates as plain `cargo` and
`bash validation/validate.sh`, which fetches nothing (the `--skip-deps`
equivalent); the reference data is already copied into your worktree.

## Scope

1. **Derive the bound per rung.** The doc's argument: a massless leg emitted
   off a massless beam at `pT` transfers `|t| ≥ pT²`, and for three bodies
   transverse balance carries the jet's `pT` to the other side of the rung. Find
   exactly which legs' thresholds bound which rung's `|t|` (the legs adjacent to
   the rung, on either side of the ladder), state the conditions, and prove it.
   Check the doc's "always holds for three-body" claim, which the item's
   evidence contradicts on `a j j`. Where no proof is available, the rung's
   bound is the smallest threshold among the legs, or zero.
2. **Implement** the per-rung bound where the channel is built (the support
   bound and the density regulator are separate roles: say whether each moves).
3. **Test** that fails on the largest-threshold rule: e.g. a point the cuts
   accept on `p p > a j j` with a rung at `|t| < pta²`, which the old floor
   gives zero density.
4. **Gate**, once the reference arrives.

## Gate

- σ on `p p > a j j` at `pta 40`, `ptj 0.1`, `draj 0.4` and at `ptj 1` against
  the CERN host's MadGraph by the multi-seed criterion (≥ 5 seeds a side).
- Banked lines bit for bit where the bound cannot move, within error where it
  can, with the rows that move listed (`bash validation/validate.sh`,
  after asking the manager).
- `cargo test --workspace` green.

## Rules for this sprint

- Commits are authored by Nicholas Smith: `git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`, and end with exactly one trailer, `Assisted-by: claude-code:<your model id>`. Never `Co-authored-by` or `Signed-off-by`.
- Do not create, edit or delete backlog items, or anything else under `research/kb/`, unless your scope says so; the manager deletes the items your PR closes. New work goes in your report's **Found**.
- Comments describe the code in its own terms: never backlog items, sessions or sprints.
- Statistical claims: ≥ 5 seeds a side, unweighted mean, with the spread and χ²/dof (AGENTS.md, "Samplers gate statistically"). Every number in your report is a recorded measurement with its command beside it, on a stated commit.
- Lints are gates: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Correct this brief if its premise is wrong. Stop and report rather than build on a wrong premise.

## Report

Your final message is the report: the commits; each oracle with its command and
output; mutation evidence for each new test (it fails before the change); the
gates run, with their output; and a **Found** section. The manager records it
as this session's `-report.md`.
