---
type: Session Brief
title: "G-HY4: σ and its error decimal-aligned in one SI-prefixed barn unit"
description: "The user's review change to #90: integrate prints σ and its error in one SI-prefixed barn unit chosen from σ, both to the error's decimal place (8.6812 ± 0.0023 ab), reusing si.rs; the survey parser reads the unit."
status: draft
agent: hygiene-dev (Opus)
depends_on: [G-HY3]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G3-HY4`, branch `sprintg-cli-output-hy4`, off
`sprintg-cli-output` (G-HY3 merged, main at d571a5d merged in). The
manager merges it back.

## The change (the user's review of #90, relayed 2026-11-04)

G-HY3 prints a 9e-6 pb σ as `8.6812e-6 ± 2.3e-9`. It should be
**decimal-aligned in one SI-prefixed unit**: `8.6812 ± 0.0023 ab`, "using an
SI auto-prefix as the CLI does elsewhere".

- Choose **one** SI prefix from σ's magnitude, mantissa in [1, 1000), and
  print σ and its error both in that unit. Units are barn-based: ab, fb, pb,
  nb, µb, mb, b.
- The error keeps its **two significant figures**, as now. σ is printed **to
  the same decimal place as the error**, so both numbers have the same number
  of decimals. Examples: 9e-6 pb → `8.6812 ± 0.0023 ab`;
  1.234567 ± 0.000123 pb → `1.23457 ± 0.00012 pb`.
- **Reuse `vibegraph-cli/src/si.rs`** (`fmt_si` / `si_components`, used by
  the TUI) rather than writing a second formatter. Its `PREFIXES` table starts
  at f, so add a (−18), and z if you like; check that the TUI footer's
  fixed-width tests still hold. G-HY3's `value_and_error` should become a thin
  user of it or go.
- **Edge cases, each with a test and a stated choice:** an error of 0, an error
  that is not finite, and an error larger than σ (σ's prefix may then put the
  error's figures above the units: say what prints). Also σ = 0 cannot reach
  the line (#88 refuses it); say what the function does if called with it.
- The result line and the per-multiplicity lines both use the format.
- **Survey parser** (`validation/cms_gridpacks`): read the unit suffix and
  convert to pb, and keep parsing old campaigns' output (both the `{:.6}` pb
  form and G-HY3's `e`-notation form, which never reached main but may be in
  a local database).
- Update the CLI docs that show the line, and the G-HY3 claims in
  `../closeout.md` that quote the old example. The manager updates the PR
  body.

## Read first

- `G-HY3-report.md` (this folder): `value_and_error`, its tests and the
  mutation driver (`G-HY3-mutate.py` in the scratchpad).
- `vibegraph-cli/src/si.rs` and its callers in `tui/`.

## Gate

- Mutation evidence for the alignment (σ's decimals equal the error's) and
  for the prefix choice (σ's magnitude picks it, at both edges of [1, 1000)).
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`; the banked `cli_integrate` and
  `cli_generate_proton` under `--profile release-debug --features extended-validation`
- `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'`
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed. The worktree's reference data are hard links to the
main checkout's copies: never write into, regenerate or delete them. cargo
`--jobs 3` (you are the only session). Detach long runs (an os.setsid
double-fork, as `G-V3-detach.py` in the scratchpad) and commit at
checkpoints.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-HY4-report.md`: the change, the edge-case choices,
mutation evidence, every gate command with its final output, and a **Found**
section.

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
