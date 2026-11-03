---
type: Session Brief
title: "T-V1: version bump, then is the CMS VBF σ well defined?"
description: "Bump the artifact version for the new channel maps; re-run mutation M1; then measure whether the three CMS cards' σ is regulated (ptj = 0, etaj = -1) on both vibegraph's and MadGraph's side, before any ladder."
status: draft
agent: validation-dev (Opus)
depends_on: [T-F1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/T`, branch `sprinte-spine-fallback` (PR #81), at the
commit that adds this brief. Pixi is not installed. The reference data is
hard-linked in; do not write into it. Read `T-F1-report.md` in full first.

## 1. The artifact version (do first)

[Artifact format versioning](../../../pipeline/artifact-format-versioning.md),
rule 3: a meaning change bumps the version even when the shape does not
change. T-F1 changed which phase-space maps a stored grid is replayed
through, for every run whose `spacelike_floor` is 0 (2→2 rows with no pT cut,
decay chains, on-shell veto rows, final states past two legs). An older
artifact would silently replay its grids against different maps.
- Bump `FORMAT_VERSION` and raise `OLDEST_READABLE_VERSION` to it. C11 says
  every artifact is written at the current version, and refusing an old one
  carries the existing "regenerate it with `vibegraph integrate`" remedy.
- Pin a refusal test to the old version's shape (rule 9).
- The systematics PR (#83) also bumps to 15. Whichever merges second takes
  16; note this in your report, and do nothing about it here.
- Do **not** add a channel-map digest. That is a separate question for the
  user (the manager is asking).

## 2. Mutation M1 against the walk/density test

T-F1 added a spine assertion to the walk/density test after its M1 run.
Re-run M1 (spine withheld) and show the test fails at its first diagram.

## 3. Is the CMS cards' σ well defined? (before any ladder)

The three cards are on branch `vbf-dy-spine-inputs` (4cc4011b), under
`validation/cms_gridpacks/inputs/<gridpack>/`. Fetch them with
`git show origin/vbf-dy-spine-inputs:<path>`; do not merge that branch. They
set `ptj = ptl = 0`, `etaj = etal = -1`, `drjj = drjl = 0` and only
`mmjj`/`mmll` (`mmllmax` on 4to50).

T-F1's hypothesis, unverified: with massless quarks, a t-channel photon
between the quark lines in `l+ l- j j` (QED = 4) is log-divergent in the
collinear limit, so nothing regulates σ. Then MadGraph's 13.12 / 7.574 /
117.6 pb would be an artefact of its sampling, and no ladder converges to
them.

Measure, don't argue:
- **vibegraph:** on the DY MLL-50 card at the restriction defaults (no param
  card is needed for a regulator scan), add an artificial jet regulator and
  shrink it: `ptj` = 3, 1, 0.3, 0.1 GeV, ≥5 seeds per point. Report σ
  against log(1/ptj). A σ that keeps growing linearly in the log is a
  divergence. Use a budget at which each point's seeds agree (χ²/dof near
  1), and say what it was.
- **MadGraph:** find what, if anything, regulates the same region in
  MadEvent with these cards. Read its cut and phase-space code in
  `research/refs/mg5amcnlo`: `setcuts.f`, `cuts.f`, `genps.f`/`gen_ximprove`,
  and any minimum on t-channel invariants or a hidden default. Name the
  file and line. If MadGraph has an implicit cut, vibegraph must apply the
  same one to compare, and that becomes a design question for the manager.
- Also report the per-diagram-class split (photon vs W/Z t-channel), if the
  channel diagnostics make it cheap.

Stop after §3 and report. The 5-seed ladder on the three cards runs only once
#82 (`integrate --param-card`) has merged and the regulator question is
settled. The cards' param card moves aEWM1, MZ, WZ, MW, MT, WT and ymt, which
shifts σ by about 10%.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace`
- `python3 scripts/kb.py lint` (0 errors)

Write and commit `sessions/T-V1-report.md`: commands with output, the scan
table, the MadGraph finding with its file and line, and **Found**.

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
