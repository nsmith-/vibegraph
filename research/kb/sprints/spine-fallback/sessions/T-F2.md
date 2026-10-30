---
type: Session Brief
title: "T-F2: the γγ → ℓℓ corner and the default pole scale"
description: "Find why DY's overweight tail sits at the 10 GeV² pole floor (γγ → ℓℓ with both photon rungs near it), fix the channel or the floor, keep every bias check, and re-measure DY's budget climb."
status: draft
agent: feature-dev (Opus)
depends_on: [T-V2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Code changes go in `/home/user/wt/T`, branch `sprinte-spine-fallback`
(PR #81). Measurements on the CMS card need `--param-card`, so they run in
`/home/user/wt/TV2` (branch `scratch-t-v2`). That branch is a throwaway local
merge of #81 and #82: never push it. Bring your #81 commits into it with a
local `git merge sprinte-spine-fallback`. Pixi is not installed.

Read first: `T-F1-report.md`, `T-V1-report.md`, `T-V2-report.md` (its Found
list, especially the γγ corner), and the module docs of
`vibegraph-lib/src/phasespace/diagram_channel.rs`.

## The defect (T-V2 Found)

On DY MLL-50, overweight events carry 75% of the sample's σ. The worst is
1547× the unweighting maximum: `u u > e+ e- u u` with both photon virtualities
at 10.7 and 28.9 GeV², right at the default pole floor
(`DEFAULT_POLE_SCALE = 1e4`, so a floor of 10 GeV²). The seeds climb with
budget (+0.54 ± 0.27 pb, R1 → R4, paired), the fingerprint of a region the
channels under-sample. T-F1 chose that scale on `u c` and `ee → eeμμ`, which
have no double-photon corner.

## Work

1. **Diagnose at the point.** For the worst events (T-V2's samples, or new
   ones with weight-ranked dumps), report:
   - which channel drew them, and the multichannel density of every channel
     there;
   - |M|², and the ratio |M|²/density;
   - whether a channel exists with *both* photon rungs as spacelike lines (a
     two-sided ladder, γγ fusion), and how its rungs' poles are floored.

   Say in one table why the weight is large: no channel maps both rungs, the
   floor sits above where |M|² peaks, or the channel weights.
2. **Fix it.** Possible fixes include:
   - a lower default floor, chosen by measurement;
   - a floor per exchanged species (photon versus massive);
   - the missing two-sided channel.

   Rules:
   - The support bound stays tied to a provable pT floor, as T-F1 decided.
   - The pole floor must stay far above the edge's rounding noise (T-F1's
   walk/density result).
   - Do not tune toward MadGraph's number.
3. **Keep every bias check.**
   - Re-run T-F1's walk/density sweep with `WALK_DENSITY_TOL` unchanged.
   - Re-run its σ-level comparisons (`ee → eeμμ` against flat RAMBO; the `u c`
     pole-scale agreement), ≥5 seeds per arm.
   - `spine_sigma` must still pass, and must still fail under M1.
   - If the default changes, re-calibrate its spread bound from 40 seeds per
     arm, as T-V2 did.
4. **Re-measure on the CMS DY card**, in TV2 with the shipped param card:
   - the overweight share and the largest weight / maximum, before and after,
     at R1;
   - then the paired R1 → R4 climb over ≥12 seeds.

   Both before-and-after measurements must use the same harness. Plan the
   wall time before launching and write the plan into your report. This host
   is shared and has 4 cores. A full CMS ladder is not this session's job.
5. **The τ part** (T-V2 Found: DY τ part at 1.06 of MadGraph's P directory).
   Only if 1–4 leave time: report what the τ part does under the fix. The
   MadGraph side of that comparison is being checked separately; the CERN
   host is reading the tarballs' convergence record.

## Gate (on `/home/user/wt/T`)

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`, and with
  `-p vibegraph-lib --features extended-validation`
- `cargo test --workspace --no-fail-fast`
- `spine_sigma` under its feature flag
- `python3 scripts/kb.py lint`, with 0 errors
- If channel construction changes for banked rows, list which rows changed
  (as T-F1 did). Re-gate them as separate detached targets under the seed
  policy.

Each rule gets a test with mutation evidence. Remember the channel-map
digest: a map change is caught by it, and that is intended. Write and commit
`sessions/T-F2-report.md`, with a **Found** section.

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
