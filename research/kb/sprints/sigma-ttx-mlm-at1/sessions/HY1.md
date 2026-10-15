---
type: Session Brief
title: "HY1: hygiene pass on the CM-clustering fix"
description: "Review commit 6740954 (CM-frame clustering in the hadronic integrand, the CM replay in validate_scales) for maintainability, test non-vacuity, visibility and reuse, then fix what is local."
status: draft
agent: hygiene-dev (Opus)
depends_on: [F1, V3, V4]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
The change under review is `6740954`
(`git show 6740954 --stat`: `proton.rs`, `coupling/scales.rs`, `hadronic.rs`,
`tests/validate_scales.rs`, `tests/validate_hadronic.rs`). The F1 entry in
[log.md](../log.md) and V2's report say what it fixes and why.

## Scope

- Follow `hygiene-dev`'s protocol: write the findings first, then fix the
  local ones with mutation evidence.
- **Non-vacuity, specifically:**
  - Does `clustered_scales_are_read_in_the_partonic_centre_of_mass` fail for
    each way the frame could regress (direct term, mirrored term)?
  - Does `validate_scales`' tightened zero-miss assertion still exercise the
    configuration search?
  - Does the new `partonic_cm` helper duplicate a boost the library already
    has? If it does, reuse the library's.
- **Docs:** the frame statements on `ProtonIntegrand`, `term_at` and
  `ScaleEvent::incoming` must agree with one another and with the code.
- Physics findings go back to the manager for a validation session. Do not
  change what the code computes.
- `helas/` is off limits.

## Gate

fmt; clippy `-D warnings`; `cargo test --workspace` (`-j 2`, check `df`
first, delete `target/debug` afterwards); and, if a test or a scale path
changed, `cargo test -p vibegraph-lib --profile release-debug --features
extended-validation --test validate_scales` plus `--test validate_hadronic`
(the non-ignored tests).

Briefs carry errors: if this one is wrong on any point, say so in the report.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
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

- **This host** is a 4-core cloud container with ~28 GB free disk, shared with
  sibling sessions. One heavy suite or MadEvent batch at a time from your
  session; kill only PIDs from your own process tree. Delete MadEvent
  scratch directories you no longer need.

## Banking rule (binding)

Diagnostic MadGraph/MadEvent runs (`pixi run -e madgraph ...`) are allowed and
encouraged: throwaway, never committed or banked, run in the scratchpad.
They use the pinned MadGraph 3.7.1 (`validation/madgraph/mg5_pinned.sh` under
`pixi run -e madgraph`, `VG_MG5_ROOT` for a patched copy), never the packaged
3.5.7 `mg5_aMC` on PATH, and LHAPDF sets
([MadGraph pinning](../../../validation/madgraph-oracle-pinning.md)).
**New banked reference data** (anything under the refdata bundle: MadEvent σ
or event references, re-banked rows, new seeds/budgets for banked rows) is
produced only by the user on the M3 host. If your conclusion needs it, write
the exact request in your report (task or script, rows, seeds, budgets,
patches, expected outputs); the manager posts it. MadGraph *standalone*
amplitude tables under `validation/madgraph/standalone/` are exempt.

## Boundary

A parallel sprint owns `vibegraph-lib/src/helas/` (especially `helas/eval`)
and the standalone-amplitude tests. Do not modify them. If your diagnosis
points into them, stop and report.
