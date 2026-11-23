---
type: Session Brief
title: "L-M1: Majorana fermions and fermion-number violation, per diagram against MadGraph"
description: "Lift the Majorana and fermion-number-violating refusals with MadGraph's fermion-flow conventions (flow-flip, the C operator), validated per diagram and per helicity against MadGraph AMP() on a neutralino and an RPV/diquark process before any σ."
status: draft
agent: feature-dev (Opus); validation-dev for the oracle afterwards
depends_on: []
closes: [majorana-fermions-unsupported]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 2 of [sprint L](../sprint.md), on branch `sprintl-majorana` off main b2434a0.
This is sprint-sized; if it runs long it moves to v0.2.4 without holding the
tag, so land it in reviewable steps and commit at each.

Read the item (`pixi run backlog --item majorana-fermions-unsupported`) and the
concepts it links — especially [fermion-line sign](../../../amplitudes/fermion-line-sign.md),
the [fnv-refusal close-out](../../fnv-refusal/closeout.md), note 16's fermion-flow
slot swap and [bit-exact amplitude debugging](../../../validation/bit-exact-amplitude-debugging.md).
The refusal sites: `Diagram::check_fermion_flow` (`vibegraph-lib/src/diagrams/diagram.rs`),
the `C` operator in `vibegraph-lib/src/helas/eval/root_lorentz.rs` (near L591), and
whatever `layout.rs`/`kernel.rs` assume about Dirac-continuous lines.

## Scope

1. **Conventions from source, cited.** Read how MadGraph handles fermion flow
   for Majorana and FNV vertices (Denner et al.'s flow-reversal prescription):
   `madgraph/core/helas_objects.py` (fermion-flow flipping, `flip_flow`, the
   `C`-conjugate wavefunctions and `fermion_factor`), and ALOHA's conjugate
   routines (`aloha/` `C`-prefixed routines, `ConjugateRoutine`). Record the
   rules with file:line citations in the report before writing code.
2. **Implement** the flow-flip and charge-conjugated vertex/propagator handling
   for: an external Majorana; an internal Majorana propagator; a
   fermion-number-violating vertex (RPV/diquark/leptoquark-type). MadGraph's own
   refusal of Majorana fermions in four-fermion vertices stays a named refusal
   here too. Any configuration you do not validate stays refused by name.
3. **Oracle before |M|².** Per diagram × per helicity complex AMP() against
   MadGraph standalone (3.5.7 here; `validation/madgraph/build_amplitude.sh`,
   `compare_amps.py`, the probe `helas::eval::run::tests::probe_process_diagrams`)
   on at least:
   - `vibegraph_toy_majorana_UFO` (the first row; include `lt~ lt > sc sc~`,
     which ran unvalidated at σ = 0.5158 ± 0.0023 pb before the refusal);
   - one neutralino process in a real model (MSSM_SLHA2 from the submodule's
     `models/`, e.g. `e+ e- > n1 n2` or `u u~ > n1 n1`), with external and
     t-channel Majoranas;
   - one FNV process (`vibegraph_toy_fnv_UFO`, then an RPV or diquark model a CMS
     card uses — the [refusal census](../../../validation/cms-gridpack-refusals-202a540.md)
     lists them).
   Match relative signs between diagrams, not just |AMP|: per-diagram ratios
   must be one common phase. Then bank per-flow JAMP rows in
   `vibegraph-lib/tests/amplitude_oracle.rs` (or its successor) so CI enforces
   them, and only then σ on one row with ≥ 5 seeds (unweighted mean, spread,
   χ²/dof) against MadGraph. A banked σ needing MadGraph 3.7.1 goes to the
   long-run host via your report.
4. **Nothing else moves.** Dirac-only processes are bit for bit: σ and grid
   sha256, main b2434a0 against your branch, on ≥ 3 banked cards (one
   fixed-beam, one hadronic, one multi-fermion-line process such as
   `e+ e- > e+ e-` or a four-quark row). Build main in a separate target dir and
   delete it afterwards.
5. **Campaign.** Report which census cards (HNL, higgsino, gluino, RPV stop,
   diquark, leptoquark) now enumerate, and which stay refused and why.

The manager may dispatch a validation-dev session for the oracle after you;
leave the dumps and the commands that made them in the report so it can start
from them.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-majorana`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-majorana`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lm1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-majorana/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lm1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-majorana-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-majorana`). Never open a PR or post to GitHub: the
manager does. Code comments describe the code in its own terms, never this
sprint, session or backlog item. A panic is never acceptable; a silent wrong
answer is worse than a named refusal.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-M1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
