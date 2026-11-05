---
type: Session Brief
title: "G-V1: why card 048 integrates to one nonzero point with μF = 0"
description: "Diagnose CMS card 048 (XtoYYprimeTo4Q, p p > vz, vz > h z): why one point per seed carries σ at the gridpack card, why the defaults' seeds disagree, why μF logs 0 and '0 resonant pairs'; fix or refuse each defect with a test."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [vector-triplet-chain-one-nonzero-point]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G2-V1`, branch `sprintg-diagnoses-v1`, off
`sprintg-diagnoses`. The manager merges it back.

## Read first

- `python3 scripts/kb.py backlog --item vector-triplet-chain-one-nonzero-point`.
- The CERN host's inputs and runs, on branch `origin/sprintg-gridpack-cards`
  (7b9e8f80, no PR), `validation/cms_gridpacks/cards/048-XtoYYprimeTo4Q_Par-MX-3500GeV-MY-600GeV-MYprime-170GeV/`:
  proc, customize and extramodels cards, the final run, param and
  proc_card_mg5 cards, the staged `ufomodel/`, `gridpack_generation.log`,
  and `vg-output/` with every run's full CLI output. No events ship. Check
  them out into your scratchpad (`git -C /home/user/wt/G2-V1 show
  origin/sprintg-gridpack-cards:<path>` or a `git worktree` of that branch
  in the scratchpad); do not commit them on this branch.
- What those runs say (main f128e483; every run needs
  `runpatch:etaonium=0.6`):
  - **A, model defaults** (MVz = 2000, WVz = 10): σ = 0.110, 0.113, 0.114,
    0.112, 0.107 pb, each ± ~0.0015; seed χ²/dof 3.6, so the quoted errors
    are ~1.9× too small.
  - **B, the gridpack card** (MVz = 3500, MH = 170, MZ = 600, widths 1 MeV):
    σ = 11416 ± 11416, 7680 ± 7680, 28409 ± 24264, 17607 ± 17607, 0 ± 0 pb,
    against MadGraph's survey 1.811e5 ± 779 pb.
  - Both log μF = 0 GeV (`dynamical_scale_choice = -1`) and "1 channels …
    0 resonant pairs".
- Sibling context: the refusals PR (#88) adds an integrate refusal for a σ
  carried by too few nonzero points (G-F4), which will refuse run B. That is a
  guard, not a diagnosis. The μF = 0 print is the CLI-output item
  `dynamical-scale-summary-prints-mu-f-zero` (#90) unless it is more than a
  print: find out which.

## Work

1. Reproduce B and A here with the gridpack's UFO, cards and `--param-card`.
2. Start from the per-event record, as the item says: the scale, the
   forced-window cut and the weight of each nonzero point, and the channel
   map. Why does the decay chain `vz > h z` with MZ = 600, MH = 170 (narrow,
   1 MeV) get one channel and no resonant pair? With widths of 1 MeV and no
   resonance mapping, a near-delta Breit–Wigner explains points of enormous
   weight: confirm or falsify it with a run that would differ if it were
   wrong.
3. Compare against MadGraph at the same card. MadGraph runs in this
   container from a copy of the pinned submodule
   (`research/refs/mg5amcnlo`; copy it to your scratchpad first, never write
   into the worktree's hard-linked copy; `apt-get install -y gfortran` is
   allowed). The gridpack's model must be loaded the way the survey does
   (`--adapt gpmodel`). A full MadEvent σ for this card on 4 cores may be
   feasible; if it is not, use MadGraph's channel/diagram structure, a
   standalone |M|² at fixed points, and the survey value 1.811e5 pb.
4. For each defect found: fix it with a test that fails before, or refuse it
   by name. If the fix is phase space (resonance detection in decay chains),
   the banked σ gates must stay green.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the `extended-validation` skill's rows for whatever you change
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly
(`pixi.toml`). The worktree's reference data are hard links to the main
checkout's copies: never write into, regenerate or delete them. Three
sessions share 4 cores and ~16 GB of disk: cargo `--jobs 1`, at most 2
integration threads, stop and report under 8 GB free. Commit work in progress
at checkpoints: the container has restarted once and killed background runs.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-V1-report.md` (`type: Session Report`): every command
with its output, the per-event table, the MadGraph comparison, each
hypothesis tested and how, what was fixed or refused, and a **Found**
section. Correct this brief where it is wrong.

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
