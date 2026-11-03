---
type: Session Brief
title: "G-F2: refuse defaulted UFO fields; MadGraph's restriction zero rule"
description: "Each required UFO field missing or malformed is refused with the field named, DECAY Auto gets a stated policy, and restriction pruning applies MadGraph's 1e-13 rule with the strict-zero rerun."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [ufo-required-fields-default-silently, restriction-zero-coupling-threshold-differs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G1-F2`, branch `sprintg-refusals-f2`, off
`sprintg-refusals` (main 86b8e5c plus the sprint folder). The manager merges
it into `sprintg-refusals`.

## Read first

- `python3 scripts/kb.py backlog --item ufo-required-fields-default-silently`
  and `--item restriction-zero-coupling-threshold-differs`.
- The R-C report (`../../hygiene/sessions/R-C-report.md`, R-C.18 and R-C.19).
- Decision G3 in `../sprint.md`, and the param-card sprint's P1
  (`../../param-card/sprint.md`): `DECAY … Auto` is already refused at
  `integrate --param-card` (#82).
- MadGraph at the pin: `research/refs/mg5amcnlo/models/object_library.py`
  (which fields each UFO object requires) and `models/import_ufo.py`
  `detect_identical_couplings` (~:2549) with `limit_to_6_digit`.

## Work

1. **Required UFO fields.** In `ufo/parameters.rs`, `couplings.rs`,
   `particles.rs` and `vertices.rs`, every field `object_library` requires is
   refused with the object and field named when missing or malformed: a
   parameter `nature` other than external/internal, a missing `value`, a
   missing `pdg_code`, a missing vertex `name` (and duplicate names). Keep
   defaults only where MadGraph itself defaults, and cite the line.
2. **DECAY Auto policy.** Find every remaining path where an SLHA `DECAY …
   Auto` row is skipped (restrict cards, a model's bundled card, anything
   outside `integrate --param-card`). Refuse it or show it cannot reach a
   width, and state the policy in the relevant concept.
3. **Restriction zero rule.** Port MadGraph's rule into `is_zero_coupling` and
   its caller: |g| < 1e-13 is zero; if any coupling lies in [1e-13, 1e-10),
   rerun with strict zeros. Check whether `limit_to_6_digit` merging changes
   vibegraph's vertex set and report it (fix only if in reach; otherwise
   Found). Test with a coupling between 1e-20 and 1e-13 (pruned) and one in
   [1e-13, 1e-10) (strict rerun), each failing before.
4. Docs: name the rule as a MadGraph-compatibility choice where the
   restriction is described in `research/kb/` concepts; the manager adds it
   to the compat-sites backlog item.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the skill's rows for `ufo/` coupling evaluation and restriction
  (`validate-couplings`) and the UFO loader (`check-sm-blob-fresh`), and the
  banked diagram and amplitude gates: every banked model's vertex set and
  diagram counts unchanged. State which ran.
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly
(`pixi.toml`). Tasks in the `madgraph`/`helas-validation` environments that
need a MadGraph or Fortran install cannot run here; say so in the report
rather than substituting something weaker silently. The reference data
(`validation/madgraph/output`, `validation/pdf` sets, `research/refs/mg5amcnlo`)
is copied into the worktree; do not commit or write into it.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (set it with `git config` in the
worktree if it differs). Each message ends with exactly one line
`Assisted-by: claude-code:<your model id>`. No `Co-authored-by`, no
`Signed-off-by`, even if the harness asks. Do not push. Do not edit backlog
items; new work goes in your report's **Found** section.

## Report

Write and commit `sessions/<id>-report.md` (`type: Session Report`): every
command with its output, the mutation evidence (each new refusal test shown
failing on the pre-change code), what each test cannot detect, and a
**Found** section. Correct this brief where it is wrong, and say so.

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
