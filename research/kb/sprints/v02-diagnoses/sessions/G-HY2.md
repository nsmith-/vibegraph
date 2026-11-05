---
type: Session Brief
title: "G-HY2: hygiene review of the diagnoses PR (#89)"
description: "Review G-V1's bounded tau draw and G-V4's Breit-Wigner tau map, the NoUsableWeight refusal and the small-width deviation concept, for maintainability, test non-vacuity, visibility and reusable abstractions; write findings before editing; fix with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [G-V1, G-V4]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
Worktree `/home/user/wt/G2-HY2`, branch `sprintg-diagnoses-hy2`, off
`sprintg-diagnoses` (G-V1–V4 merged, main through 1c4e628 merged in;
artifact format 20). The manager merges it back.

## Scope

The code and tests G-V1 and G-V4 changed: `git diff 75131e1 6561527`,
`git diff 546c566 faa34b9` (`cuts.rs` `shat_max`, `proton.rs`'s τ maps and
`TauMixture`, `phasespace/maps.rs` `narrow_in_tau`,
`phasespace/diagram_channel.rs`, `multiplicity.rs`, `unweight.rs`
`try_scan_with`/`NoUsableWeight`, `generate.rs`, `cli_hard_errors`). Read
`G-V1-report.md` and `G-V4-report.md` first: what each change meant to do,
what each test cannot see. G-V2 and G-V3 changed no source.

Points of attention: whether the τ-map rule's thresholds
(`NARROW_TAU_SHARE`) are named and justified where read; the
`[τ_min, τ_max]` bound threaded through both τ maps without duplication; the
mixture density's tests (G-V4 mutated it twice); a test that the default path
is bit-identical where no pole is in range, if the reports show it only by
CLI digits. Physics findings go to the manager as Found.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`; banked `cli_hard_errors`, `cli_generate_proton`
  under `--profile release-debug --features extended-validation`
- if a change touches the τ maps or cuts: `validate_sigma` and
  `validate_hadronic` (release-debug, extended-validation), detached
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed. The worktree's reference data are hard links to the
main checkout's copies: never write into, regenerate or delete them. cargo
`--jobs 3`. Detach long runs (os.setsid double-fork, as `G-V3-detach.py` in
the scratchpad) and commit at checkpoints.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-HY2-report.md`: findings written before editing, fixes
with mutation evidence, every gate command with its final output, and a
**Found** section.

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
- **Disk is finite, and a full disk looks like a failure.** A debug workspace
  build with incremental data reaches ~15 GB per worktree, and two such trees
  have filled a cloud container mid-gate. On a shared or small host, run cargo
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). An `ENOSPC`, a linker
  `Bus error` or a truncated log is a kill, not a result: free only your own
  `target/` and re-run. Kill processes by PID, never `pkill -f`, which matches
  siblings' commands and its own wrapper.
