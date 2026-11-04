---
type: Session Brief
title: "G-F3: classify every construct of the param, restrict and reweight cards"
description: "A classification table per input card (consumed, benign with an argument, or refused when it could bite), asserted by a test as the run card's FIELD_CLASSES is; plus the UFO propagators.py required fields."
status: draft
agent: feature-dev (Opus)
depends_on: [G-F2]
closes: [input-cards-lack-field-classification, ufo-required-fields-default-silently]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G1-F3`, branch `sprintg-refusals-f3`, off
`sprintg-refusals` (G-F1 and G-F2 merged). The manager merges it back.

## Read first

- `python3 scripts/kb.py backlog --item input-cards-lack-field-classification`.
- [Run-card field classification](../../../run-card/field-classification.md)
  and `vibegraph-lib/src/runcard/classes.rs`: the pattern to follow (one
  class per construct, a written inertness argument for each benign one, a
  test asserting the table covers what the reader accepts).
- Decision G4 in `../sprint.md`: all three cards, the param card first.
- The G-F2 report (`G-F2-report.md`, both sections): the restrict card's
  `Auto` refusal, the zero rule, `ParameterSet::ones`/`merged`.
- The param-card sprint's decisions P1–P4 (`../../param-card/sprint.md`) and
  `ufo/bind.rs`, which already refuses several param-card constructs.
- MadGraph at the pin: `models/check_param_card.py` (ParamCard, Block,
  DECAY with branching ratios, QNUMBERS), `madgraph/interface/reweight_interface.py`
  (the reweight commands and options), and `models/import_ufo.py`'s
  restriction (`RestrictModel`).

## Work

For each card, in this order — **param, restrict, reweight**:

1. List every construct MadGraph's reader accepts (blocks, entry shapes,
   `DECAY` with BR lines, `QNUMBERS`, comments/ordering, for reweight every
   command and option of `reweight_interface` that a card can hold), citing
   the MadGraph line.
2. Classify each: `Consumed`, `IgnoredBenign` (with an argument that it
   cannot reach σ, a weight or the event record), or refused when it could
   bite (a named error, reporting every problem in the card at once where the
   reader allows). Reuse refusals that already exist; add the missing ones.
3. A test per card asserting the table covers every construct listed, and a
   refusal test per new refusal, failing before the change.
4. Write each table into a concept beside the run card's (`research/kb/`,
   linked from [release scope](../../../pipeline/release-scope.md)'s table).

Also: **`propagators.py` required fields** (G-F2 Found 1). Read
`Propagator`'s `require_args` in a model that ships it (SMEFTsim does) and
apply `Fields` to `ufo/propagators.rs` as G-F2 did for the other objects.

If the reweight card turns out much larger than the other two, finish param
and restrict completely first, then stop and report what reweight needs.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the skill's rows for the param card and coupling evaluation
  (`coupling_oracle`, `sm_interned_blob`) and reweighting
  (`validate-reweight`'s hermetic comparison and the banked
  `cli_reweight_proton` audit); every banked card still loads unchanged
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
