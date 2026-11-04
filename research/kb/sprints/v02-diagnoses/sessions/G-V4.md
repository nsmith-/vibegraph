---
type: Session Brief
title: "G-V4: a Breit–Wigner τ map for a whole-state s-channel resonance; generate's panic; the small-width deviation"
description: "A hadronic process whose s-channel resonance carries the whole final state gets a Breit–Wigner τ map (p p > h z through a 1 MeV Vz, gated over ≥5 seeds against MadEvent); generate's 'no channel produced a usable weight' panic becomes a named refusal; MadEvent's small-width convention is registered as a deviation."
status: draft
agent: feature-dev (Opus)
depends_on: [G-V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G2-V4`, branch `sprintg-diagnoses-v4`, off
`sprintg-diagnoses` (G-V1–V3 merged). The manager merges it back.

## Read first

- Decisions G8 and G9 in `../sprint.md`.
- The G-V1 report (`G-V1-report.md`): the τ-ceiling fix (`Cuts::shat_max`,
  both τ maps in `proton.rs`), its MadGraph comparisons, and Found 1–3.
- How channel maps detect and map s-channel resonances today
  (`phasespace/`, the channel builder and `proton.rs`' τ maps), and the
  [MadGraph defect policy](../../../validation/madgraph-defect-policy.md) with
  [identical particles across decays](../../../process/identical-particles-across-decays.md)
  as the pattern for a registered deviation.
- MadEvent at the pin: how a channel whose s-channel propagator spans the
  whole final state maps τ (`genps.f`/`setcuts.f` and `transpole.f`).

## Work

1. **Breit–Wigner τ map (G8).** For a hadronic channel whose s-channel
   resonance carries the whole final state (outside a decay chain), draw τ
   with a Breit–Wigner map around M², as MadEvent does, composed with G-V1's
   [τ_min, τ_max] bounds. Reproduce the failure first: `p p > h z` with the
   card-048 gridpack UFO and card B (on `origin/sprintg-gridpack-cards`;
   check out into your scratchpad, never commit), p without c, 100000 × 8,
   ≥5 seeds: today 1.48e5–2.25e5 pb, in-run χ²/dof 51.6. After: ≥5 seeds
   consistent (seed χ²/dof and in-run χ²/dof near 1) and against a fresh
   MadEvent σ at the same card and PDF (MadGraph runs here from a copy of the
   pinned submodule in your scratchpad; gfortran may be installed). Expect
   the ~1.5 % small-width offset G-V1 measured (MadEvent floors Γ at M·1e-6);
   say how much of the residual it accounts for. Also re-run card 048's
   decay-chain form to show it is unchanged or improved.
   State whether the change moves sampling order on any banked row; the
   banked σ gates must stay green, and the default path must stay
   bit-identical where no such resonance exists (show it, as G-V1 did).
   The artifact format: if the map changes what a recorded grid means for
   any process, bump it (this branch is at 18; #88 also bumps to 18, and the
   manager renumbers at merge — just say whether a bump is needed).
2. **`generate`'s panic** (G-V1 Found 1): `unweight.rs:602` panics "no
   channel produced a usable weight" (exit 101) on run B's grid. Make it a
   named refusal with a CLI test that panics before the change.
3. **The small-width deviation (G9).** Write the registered deviation into a
   concept beside the identical-particles one: what MadEvent does
   (`transpole.f:44`, fk_W = M·1e-6), what vibegraph does, G-V1's
   measurement (predicted 1.0155, measured 1.0144; lineshape widths 1.48 vs
   3.86 MeV), and that MadGraph's standalone output uses the raw width so the
   amplitude oracle cannot see it. Link it from the defect policy and the
   release scope's "Deviating from MadGraph". The manager adds it to the
   compatibility-flag backlog item.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the skill's rows for phase space and unweighting: `validate_sigma`,
  `validate_hadronic`, `validate_unweighting`, `validate_generate_proton`,
  run detached, one at a time
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly. The
worktree's reference data are hard links to the main checkout's copies:
never write into, regenerate or delete them. cargo `--jobs 2`, at most 2
integration threads; stop and report under 8 GB free. The container has
restarted twice and killed background runs: detach every long run from the
start (an `os.setsid` double-fork, as G-V3's `G-V3-detach.py` in the
scratchpad does), and commit at checkpoints.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-V4-report.md` (`type: Session Report`): every command
with its output, the seed tables (mean, spread, χ²/dof), the MadEvent
comparison, the mutation evidence, and a **Found** section. Correct this
brief where it is wrong.

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
