---
type: Session Brief
title: "G-F4: refuse σ = 0 and a σ carried by too few nonzero points"
description: "integrate exits non-zero with no artifact when every point has zero weight, or when fewer nonzero points than a measured threshold carry σ, naming the cut-pass count and whether |M|² vanished; generate refuses a σ = 0 artifact."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [zero-cross-section-reported-as-ok]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G1-F4`, branch `sprintg-refusals-f4`, off
`sprintg-refusals`. The manager merges it back.

## Read first

- `python3 scripts/kb.py backlog --item zero-cross-section-reported-as-ok`.
- Decision G2 in `../sprint.md`.
- **The item's "First" step is done** (CERN host, main f128e483, relayed
  2026-11-04): both `H2ToH1H3To2Mu` gridpacks give σ = 0 ± 0 at the model's
  default parameters (mh1 = 120, mh2 = 130, mh3 = 140 close h2 → h1 h3) and
  agree with MadGraph at their own param card (pulls +0.97, +1.79). The zero
  is the parameters, not a vibegraph bug.
- **A second motivating case**, card 048 of the CMS survey at its own param
  card: five seeds read σ = 11416 ± 11416, 7680 ± 7680, 28409 ± 24264,
  17607 ± 17607, 0 ± 0 pb; each σ carried by one or two points. This is the
  "too few nonzero points" refusal's target. (Why it happens is G-V1's job on
  the diagnoses PR; do not diagnose it here.) Its cards are on branch
  `origin/sprintg-gridpack-cards`, `validation/cms_gridpacks/cards/048-*`,
  if you want to reproduce; do not commit them.

## Work

1. At the end of `integrate` (before any artifact is written), count, over
   every evaluated point: points passing the cuts, and points with nonzero
   weight. If no point has nonzero weight, refuse with a typed error naming
   both counts and which cause they point at (cuts reject every point; or
   |M|² is zero on every point that passed: a coupling or width at the
   parameters in force, a restriction). Exit non-zero, no artifact.
2. **Too few nonzero points.** Measure the nonzero-point count on the banked
   σ rows (`validate_sigma`, `validate_hadronic` configurations) at their
   budgets, and on the low-acceptance cases you can find, then set the
   threshold far below the smallest banked count and well above what card 048
   shows. Count per run, per channel, or as an effective sample size
   `(Σw)²/Σw²` — choose, justify with the measurements, and say what the
   choice cannot see. Refuse below it, same exit/no-artifact behaviour, with
   the counts in the message.
3. `generate` refuses an artifact recorded at σ = 0 (older artifacts can hold
   one). Sprint F works on `generate`; keep this to the artifact-read check
   and say exactly which lines you touched.
4. CLI tests: a toy-model card whose cuts reject everything; one whose
   coupling is zero at the parameters in force; a too-few-points case; and
   `generate` on a hand-made σ = 0 artifact. Each fails before the change.
5. Docs: the CLI docs and the release-scope table name the new refusals.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the banked σ gates (`validate_sigma`, `validate_hadronic`,
  `validate_generate_proton`) still pass: no banked row trips the new
  refusal. Run them detached, one at a time.
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
