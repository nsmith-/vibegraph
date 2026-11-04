---
type: Session Brief
title: "G-F5: carry a param card's decay tables; MadGraph's negative-mass width sign"
description: "A param card's DECAY branching-ratio tables reach the LHEF <slha> header (recorded in the artifact so generate can write them), and a negative mass flips its width's sign as MadGraph's write_inc_file does, pinned per helicity against MadGraph on MSSM_SLHA2."
status: draft
agent: feature-dev (Opus)
depends_on: [G-F3]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G1-F5`, branch `sprintg-refusals-f5`, off
`sprintg-refusals` (G-F1–F3 merged). The manager merges it back.

## Read first

- Decisions G6 and G7 in `../sprint.md`.
- The G-F3 report (`G-F3-report.md`): the `DecayTable` and `NegativeMass`
  refusals you replace, Found 1 and Found 2, and the param-card table
  (`vibegraph-lib/src/ufo/card_classes.rs`,
  `research/kb/model/param-card-classification.md`), whose rows you move to
  `Consumed`.
- [Run inputs in the artifact](../../../pipeline/run-inputs-in-the-artifact.md):
  `generate` takes no param card and rebuilds it from what `integrate`
  recorded, so anything the header writes must be in the artifact.
- The param-card sprint's P3 (`../../param-card/sprint.md`) and
  `lhef::write::slha_block`.

## Work

1. **Decay tables (G6).** `integrate --param-card` accepts `DECAY` lines
   followed by branching-ratio rows. The tables are recorded in the artifact
   (check its version rules; bump the format if the shape changes, with the
   refusal of older/newer versions the rules require), and `generate` writes
   them into the `<slha>` header in MadGraph's layout, under each `DECAY`
   line. Compare against a MadGraph event file's header written from a card
   with BRs (make one in this container, see below) and say what differs.
   Widths still bind exactly as today. A BR table that does not sum to one,
   or names a particle the model lacks: classify (MadGraph's
   `check_param_card.py` behaviour decides) and test.
2. **Negative mass (G7).** Port MadGraph's rule (`write_inc_file`,
   `check_param_card.py` or `export_v4.py` at the pin, ~L692; find the exact
   site) into model evaluation so every propagator and wavefunction sees the
   width MadGraph's Fortran sees. Lift G-F3's `NegativeMass` refusal. Say
   whether the rule applies to masses from the model's defaults, a restrict
   card and a param card alike (it should: MadGraph's Fortran reads the one
   card).
3. **Pin G7 against MadGraph.** MadGraph runs in this container from the
   pinned submodule (`python3 research/refs/mg5amcnlo/bin/mg5_aMC`, pure
   Python for `import model`; copy the submodule to your scratchpad first, never
   write into the worktree's hard-linked copy). For per-helicity amplitudes
   use a standalone output; if it needs a Fortran compiler, `apt-get install
   -y gfortran` is allowed. Choose an MSSM_SLHA2 process with an internal
   `n3` propagator near its pole (e.g. `e+ e- > n1 n3` or one where `n3`
   is s-channel/t-channel), and compare per-diagram × per-helicity complex
   values, as the bit-exact methodology
   (`research/kb/validation/bit-exact-amplitude-debugging.md`) asks. Show the
   comparison failing before the port and passing after. Bank the reference
   as a committed table if a hermetic test can read it (follow how
   `standalone_jamps` / `amplitude_oracle` tables are committed).
4. Docs and the classification table updated; the release-scope table if a
   refusal row changes.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the skill's rows for the LHEF writer (`validate-lhef`'s cargo test) and
  amplitude evaluation (`amplitude_oracle`, `standalone_jamps`), and the
  param-card σ gate (`validate_param_card_sigma`); every banked amplitude
  unchanged (no banked model has a negative mass: check and say so)
- `python3 scripts/kb.py lint` (0 errors)

Sprint F may be editing `generate` and the LHEF writer; list every line you
touch in `lhef/` and `generate` in the report so the manager can flag it.

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
