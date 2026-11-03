---
type: Session Brief
title: "T-V4: gate the banked pp_to_epemjj_qcd0 row and refresh the spine concepts"
description: "On refdata-13, turn the planned no-pT-cut EWK e+ e- j j row into an enforced long-tier gate against the confirmed 20-seed MadGraph record (resolving H17), and bring the spine concepts up to date with the pole floor, the remainder mixture and DEFAULT_POLE_SCALE."
status: draft
agent: validation-dev (Opus)
depends_on: [T-HY1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-02}
---
Worktree `/home/user/wt/T`, branch `sprinte-spine-fallback` (PR #81), at the
commit that adds this brief. The commit before it, `validation: cut
refdata-13 and pin it`, pins refdata-13, and the bundle is already fetched
into `validation/madgraph/output` (its stamp reads `5a99624a…`). Don't write
into it. Pixi is not installed: run what a pixi task runs by hand.

Refdata-13 is refdata-12 plus one record,
`output/seeded_sigma/pp_to_epemjj_qcd0`. It holds 20 seeds at 10000 events
and 2 at 40000, and its SCATTER line reads CONFIRMED. MadGraph's 20-seed
unweighted mean is 3.593175 ± 0.001777 pb with χ²/dof 0.66. A few channels
(`P1_qq_llqq/G45`, `G13`) show real excess scatter, under 4% of σ.
MadEvent's own equal-weight combination sits +0.535% ± 0.089% above its
reported σ. Use the reported σ, the same quantity every other seeded row
compares against. The M3's write-up is on #81 (comment 6566404909). Read
`T-V3-report.md` and `T-HY1-report.md` first. T-HY1's H17 is yours.

## Tasks

1. **Enforce the row.** In `validation/manifest.toml`, take
   `pp_to_epemjj_qcd0` from PLANNED to enforced. Its note should record the
   reference value and how the record was banked, alongside this side's
   measurement already there. `validate_ewk_lljj_sigma.rs` must read the
   record from `seed_record.json` the way the other seeded rows do. It must
   also check this side's own χ²/dof band and the reference comparison
   together, so it can fail on each one (H17). Remove the `AWAITING THE BANK`
   path, or make it a hard failure when the record is missing. Check how the
   other seeded gates treat a missing record and match them. Find out
   whether CI runs this long-tier target, and say which in your report. The
   item closes only when this row gates vibegraph's σ in CI, so if CI does
   not run it, stop and report rather than wiring it in.
2. **Run the gate** on the five committed seeds, detached. The gate takes
   about 35 min on four cores, so make it resumable. Record the command and
   its full verdict output in the report: σ, pull, rel, the χ²/dof band, and
   the reference value it read.
3. **Mutation evidence.** Show the gate fails when:
   - the reference σ is shifted by 3%;
   - the χ²/dof band is narrowed so this side's measured value falls outside
     it;
   - the record is missing.

   Use a cheap probe, such as a doctored copy of the record pointed at
   through the gate's own inputs, rather than five more integrations. If the
   gate can't take a substitute path, do the arithmetic on the recorded
   seeds and say so.
4. **Refresh the spine concepts.** They still describe the spine before
   #81. The places I know of are:
   - `research/kb/phasespace/channel-contract.md:136-139`;
   - `spacelike-floor.md:110-113`, and its `dc-chain` anchor;
   - `t-channel-spine.md:116`.

   The line numbers are from the brief author's reading, so check them and
   grep for any others. Describe the code as it is now:
   - the pole floor is always on, at 1e-3 × `DEFAULT_POLE_SCALE` (1e3 GeV²);
   - the support bound comes only from a provable pT floor
     (`Cuts::spacelike_floor`);
   - the remainder invariants between rungs are drawn from the flat/1/ŝ
     mixture;
   - artifact version 15 records `channel_map_digest`.

   Describe the current behaviour only, with no history. Regenerate indexes
   with `python3 scripts/kb.py index` if needed.

Leave the backlog items alone. The manager deletes
`three-body-spine-requires-fiducial-scale` and fixes the links to it.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation` and with
  `-p vibegraph-lib --features extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint` (0 errors)
- `validate_ewk_lljj_sigma` on the committed seeds (task 2), and the banked
  manifest/inventory checks that read `manifest.toml`

Build with debuginfo off and `-j 2`. Write and commit
`sessions/T-V4-report.md`, with a **Found** section.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push,
and never amend a pushed commit. Do not edit backlog items. Commit
checkpoints as you go.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Use absolute paths in every command**, and never background a
  `cd … && …` chain.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based or `pgrep`-based watcher shells; draft your
  notes and report between completion notifications instead of idle-waiting.
  Never `pkill -f`; kill only PIDs you started.
- **No foreground command over ~8 minutes.** Detach longer runs.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
