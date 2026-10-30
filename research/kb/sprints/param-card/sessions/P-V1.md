---
type: Session Brief
title: "P-V1: banking inputs, MadGraph's coupling fusion, and the silent-card fixes"
description: "Prepare the two moved-parameter MadGraph rows (cards, driver fixes), check whether MadGraph's merged couplings break at a moved MZ or ymt, fix the SLHA parser's silent drops and the unlocked 1-valued parameters, and write the σ gate awaiting the bank."
status: draft
agent: validation-dev (Opus)
depends_on: [P-F1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card` (PR #82), at the
commit that adds this brief. Pixi is not installed. The reference data is
hard-linked in; do not write into it. Read `P-F1-report.md` in full first.

## 1. MadGraph's fused couplings (P-F1 Found 3), before anything is banked

At restriction time MadGraph merges couplings that are equal at the
restriction point (`detect_identical_couplings`, `merge_iden_couplings`). If
a moved `MZ` or `ymt` makes two fused couplings differ, MadGraph evaluates
the fused one, and the σ gate would see a disagreement that is MadGraph's
merge, not a vibegraph bug. For the `sm` default restriction and the two
proposed rows (`p p > e+ e-` at MZ = 95; `e+ e- > t t~ h` at ymt = 250):
- List the coupling pairs MadGraph fuses that involve MZ- or ymt-dependent
  expressions. Machine-check this from MadGraph's restricted model or the
  banked `couplings` output (`validation/madgraph/couplings`), never by hand.
- If any fused pair splits at the moved point, say what that does to each
  row, and propose a row that avoids it.

## 2. The banking inputs

From P-F1's proposal:
- `validation/madgraph/dy13_mz95_param_card.dat`, plus the `bank_seeded_sigma.sh`
  `dy13` param-card hook;
- `validation/madgraph/scripts/ee_to_tth_ymt250.mg5`, plus a `mes_launch_sets`
  fix that skips three-argument `set` lines (param-card sets);
- the manifest rows (`validation/manifest.toml`), flagged as awaiting their
  bank, the way the `AWAITING_REBANK_RUNS` rows are.

Check the driver changes with `bash -n` and a dry run that prints the cards
it would use. MadGraph does not run here. Write the exact M3 commands, the
expected outputs, and pass/fail into your report; the manager posts them.

## 3. The σ gate, written now and run once banked

A test comparing vibegraph's moved-card σ with each banked row under the seed
policy: ≥5 seeds per side, unweighted mean, spread and χ²/dof read, not one
pull. vibegraph integrates with `--param-card` at the row's card. Until the
rows are banked, it reports them as awaiting the bank; it does not skip
silently. Keep P-F1's informational ratio against the reweight oracle as a
reported row.

## 4. Two silent-card fixes (refuse what is not honoured)

- **P-F1 Found 4:** the SLHA parser silently drops a DECAY width that is
  neither a number nor `Auto`, entries with non-integer keys, and duplicate
  `(block, code)` pairs (the last one wins, where MadGraph raises). Make each
  a parse error that names its line.
- **P-F1 Found 1:** MadGraph also locks parameters that a restriction sets to
  1 (`detect_special_parameters`, `fix_parameter_values`). vibegraph locks
  only zeros. Refuse a card that moves a 1-locked parameter, tested on a
  restriction that has one (SMEFTsim's, if any; otherwise a synthetic one).

Out of scope: Found 2 (anti-restriction values), Found 5 (Pythia), Found 6.
The manager files them.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace`
- `python3 scripts/kb.py lint` (0 errors)
- the reweight gates (`validate-reweight` in `pixi.toml`), detached

Each new refusal gets a test with mutation evidence. Write and commit
`sessions/P-V1-report.md`, with **Found**.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push.
Do not edit backlog items.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
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
