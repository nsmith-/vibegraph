---
type: Session Brief
title: "L-D1: card 6 (dim6top tZq EFT CPV) reads 1.66× MadGraph; diagnose per diagram"
description: "Diagnosis only. dim6top_LO_UFO card 6, tZq_EFT_CPV_point_1, reads vg/MG 1.655 ± 0.074 over 5 seeds on main 40e87aa8, the first σ check ever on this model. Census and per-diagram × per-helicity complex values on one subprocess against MadGraph 3.7.1 standalone, with the imaginary-coefficient discriminator. No production code."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Diagnosis session of [sprint L](../sprint.md), branch `sprintl-dim6top-sigma` off
main 35ace596. It is a pre-tag blocker candidate for v0.2.3. **No production code**:
the deliverable is the census, the dumps, the fits and a located cause, with a
candidate fix checked arithmetically on the dumps. The planner reviews them before
anyone writes a fix.

## What is known (vg-validator)

- **Process (card 6):**
  `generate p p > t b~ j l+ l- $$ w+ w- / h FCNC=0 DIM6=1, (t > w+ b FCNC=0 DIM6=0, w+ > l+ vl)`
  plus the charge conjugate as `add process`. The full proc card is in
  `/home/user/wt/sprintl-dim6top-mg/proc_card_card6_vg.dat`.
- **Cross sections:**
  - vibegraph main 40e87aa8, 5 seeds at campaign budget: 0.07589, 0.07616, 0.08884,
    0.06852 and 0.08400 pb, mean 0.07868. Every seed is ≥ 1.44 × MG; χ²/dof is 1.85.
  - MadGraph 2.9.18 gridpack survey: 0.04754 ± 0.0000554 pb, base point, not
    reweighted.
  - vg/MG = 1.655 ± 0.074.
- **Provenance:**
  - The param card is byte-identical to the gridpack's. It includes the imaginary
    Wilson coefficients ctpI, ctWI, ctZI and ctGI.
  - MG ran `import model dim6top_LO_UFO-restrict`; vibegraph ran the unrestricted
    bundled model.
  - The restriction keeps exactly the 8 ctX operators and zeroes every other DIM6 and
    FCNC coefficient (`restrict_model_condensed.txt`).
  - MG's proc_card_mg5.dat also sets: `complex_mass_scheme False`, `gauge unitary`,
    `zerowidth_tchannel True`, `max_t_for_channel 99`, `default_unset_couplings 99`,
    `max_npoint_for_channel 0`, `group_subprocesses Auto`, and a 4-flavour base
    `p`/`j` before the `p = p b b~` extension. See `MG_SETTINGS.md`.
- **Validator observation:** vibegraph's result record echoes the process **without
  `/ h`**. Either the echo drops exclusions, or `/ h` never reaches the diagram set.
  Check this first: it is cheap.

All inputs are in `/home/user/wt/sprintl-dim6top-mg/`:
- the UFO `models/dim6top_LO_UFO` (from cms-project-generators). It matches the
  gridpack's copy on every amplitude file; only `object_library.py` and
  `write_param_card.py` differ.
- `param_card_card6.dat`, rebuilt from the validator's verbatim block lines with the
  headers regenerated, so its sha will not equal aa13ed0e…;
- the proc card and `MG_SETTINGS.md`.

## Suspects, cheapest first (the planner's list)

1. **`/ h` and `$$ w+ w-`.** Do they reach the diagram set, in combination with the
   decay-chain orders? Check the census against MadGraph's with and without `/ h`.
2. **DIM6=1 in the core.** Does it select the same diagrams and the same |M|² terms,
   including the DIM6² pieces? Also check what `default_unset_couplings 99` means for
   QED/QCD on both sides.
3. **Complex Wilson coefficients.** Is a coupling built from ctXI handled as MadGraph
   handles it, including conjugation on the reversed vertex?
4. **The dropped restriction.** Does it change which vertices exist? It should only
   remove zero couplings. Verify this; don't assume it.

## Method

Follow `research/kb/validation/bit-exact-amplitude-debugging.md`.

1. **Census.** Count diagrams per subprocess on both sides, for the whole card (both
   `generate` lines), by machine. Start with whether `/ h` and `$$ w+ w-` act. Then
   pick one subprocess, e.g. `u b > t d e+ e-` with the decay
   (`t > w+ b, w+ > e+ ve`), or whichever subprocess the census shows differs.
2. **Amplitudes.** On that subprocess, compare per diagram × per helicity complex
   values at fixed points against MadGraph 3.7.1 standalone. Use a copy of the pinned
   submodule (`research/refs/mg5amcnlo`) and this exact param card, banked with
   `validation/madgraph/gen_standalone_jamps.py`'s helpers, as L-I1 and L-G1 did. Match
   MadEvent's t-channel width convention line by line, from `output madevent` of the
   same process.
3. **Discriminator.** Rerun the same comparison with all imaginary Wilson coefficients
   set to 0 on both sides. If the gap closes, the bug is in complex-coupling handling.
4. **Locate, then propose.** Find which diagrams or vertices carry any disagreement,
   name the code site, and verify a candidate fix arithmetically on the dumped values.
   Do not edit production code.

If the census and amplitudes all agree, say so plainly, and name what is left: the
phase space, the decay-chain treatment, or the MG 2.9.18 reference itself. Then
request any σ-level run in the report rather than running long jobs here.

## What not to do

No seed sweeps, no σ runs beyond a reduced single seed, no `pixi run validate`, no
MadEvent σ. A run over 20 minutes goes into the report as a long-run request. Do not
post to GitHub.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-dim6top-sigma`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-dim6top-sigma`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `ld1-` — sibling sessions share that
  directory). A backgrounded command notifies on exit: never spawn sleep-based
  watcher shells (they compound into self-sustaining wake-up rings); draft your
  notes and report between completion notifications instead of idle-waiting. On
  resume after a stall, first check whether the interrupted command finished
  (`ps`, result artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration. Do
  not run `validation/validate.sh` or `pixi run validate` at all: CI's `banked`
  job runs it on every push. Run the specific `validate_*` test targets your
  change touches instead.
- **Disk is finite, and a full disk looks like a failure.** Always run cargo with
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-dim6top-sigma/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). Each worktree has its own
  target dir: a shared one ran a sibling's stale test binaries in an earlier
  sprint. An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID, never
  `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, shared with two sibling dev agents. It
can restart without warning. **One heavy job at a time across the container**,
enforced by a lock rather than by asking: wrap every cargo build/test/clippy
invocation, every MadGraph run and every seed sweep in
`flock /home/user/wt/heavy.lock <command>` (e.g.
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/ld1-test.log 2>&1`,
backgrounded). Hold the lock for one command at a time, never across an
interactive pause, and prefer targeted test filters to the whole workspace
while iterating; siblings queue behind you. `cargo fmt` and reading code need
no lock. A single job expected to run over ~20 minutes, a MadGraph 3.7.1
regeneration, a multi-seed σ on a card over ~20 minutes per seed, or anything
over 8 GB of memory does **not** run here: stop and put the request in your
report (branch, head SHA, cards, seeds, command, expected answer) — the
manager sends it to the long-run host. Run long jobs that do fit fully detached
(`nohup setsid … &`), with per-seed output appended to a file, so a restart
keeps every finished seed.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `research/refs/mg5amcnlo` submodule content) is already in your worktree:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under
  `/home/user/wt/sprintl-dim6top-sigma-mg/`, never into the main checkout.
- The network reaches `raw.githubusercontent.com` (e.g. `cms-sw/genproductions`
  cards) and the CMS model store
  (`https://cms-project-generators.web.cern.ch/cms-project-generators/`). It does
  not reach cvmfs. Put downloads in their own empty directory.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the container's identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae> %cn <%ce>'`
before every push. Never force-push, rebase or amend a pushed commit; keep the
branch current by merging `origin/main`. Commit and push at checkpoints
(`git push -u origin sprintl-dim6top-sigma`). Never open a PR or post to GitHub: the
manager does. Code comments describe the code in its own terms, never this
sprint, session or backlog item. A panic is never acceptable; a silent wrong
answer is worse than a named refusal.

**The hermetic layer has no submodule and no fetched data.** CI's `test` job
runs `cargo test --workspace` without `research/refs/mg5amcnlo`,
`validation/madgraph/output` or `validation/pdf`, and this worktree has all
three. A test that reads any of them must carry
`#[cfg(feature = "extended-validation")]` (or sit in a target with
`required-features = ["extended-validation"]`), as the existing MSSM
standalone rows do. Before reporting, run the workspace suite once with
those three paths moved aside (and restored afterwards); a green run with
them present does not show the hermetic layer is green.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-D1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
