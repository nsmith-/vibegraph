---
type: Session Brief
title: "L-H1: per-event-helicity reweighting (MadGraph's change helicity True)"
description: "Sprint L PR 10: a per-run option writes |M_hyp(λ)|²/|M_card(λ)|² in the event's recorded helicity configuration, matching MadGraph's default reweighting event by event."
session: L-H1
agent: feature-dev
pr: 10
backlog: [reweight-helicity-summed-only]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
# L-H1: helicity-aware reweighting

**Item:** `research/kb/backlog/feature/reweight-helicity-summed-only.md`. Its
`closes_when` is the definition of done: a per-run option (for example
`--reweight-helicity summed|event`) writes the per-helicity ratio, and the
`gen_reweight_oracle.py` rows with `change helicity True` agree event by event.
Run `pixi run backlog --item reweight-helicity-summed-only` to see it with its
links, including the
[reweight-card classification](../../../events/reweight-card-classification.md).

## Context

- `generate --reweight-card` writes the helicity-summed ratio
  `|M_hyp|²/|M_card|²`. MadGraph's default (`change helicity True`) writes the
  ratio in the helicity configuration the event records in `SPINUP`.
- The card parser handles `change helicity` in one of two ways today: it
  refuses it, or it accepts a card that omits the line (MadGraph's default is
  per-helicity) and records the divergence. Read the classification page and
  the parser to see which applies where. Once the per-event mode exists, decide
  what an omitted `change helicity` line means, cite MadGraph's default
  (file:line in `research/refs/mg5amcnlo`), and justify the decision. Changing
  what an existing accepted card produces is a user-visible change: report it
  plainly.
- The helicity that selection picks is written to the event but not kept on the
  event in memory. The reweighter needs it. Find where it is chosen (helicity
  selection in generate or unweighting) and carry it through.
- **PR 1 merged the graded evaluator** (`helas/eval`: `order_classes`,
  `jamp_slots`, `n_jamps`, `jamp_cf`, `is_signed`). Per-helicity |M|² is
  already a contraction path there. The polynomial reweight path
  (`reweight::poly` and `reweight/engine.rs`) contracts a Gram matrix. For the
  per-event mode it must contract over one helicity combination, not the sum.
  Use what PR 1 exposes where it serves, and say where it doesn't.
- A **signed** |M|² is one under a squared-order selection with interference
  only. `generate` refuses it, so you only need to make sure the new mode
  doesn't open a path around that refusal.

## MadGraph semantics first

Before writing code, read and cite (file:line) in the pinned checkout how
MadGraph's reweight module (`madgraph/interface/reweight_interface.py` and the
f2py matrix-element calls it makes) does each of the following:
- reads the event's helicities: SPINUP order, the meaning of 9 or 0 for
  unpolarised or unknown, and crossing and initial-state conventions;
- evaluates `|M(λ)|²` for that one configuration, including which
  normalisation, averaging and identical-particle factors cancel in the ratio;
- handles an event whose recorded configuration has `|M_card(λ)|² = 0`, or
  `|M_hyp(λ)|²` with a card value of 0;
- handles a decay chain or a process with polarised legs, and whether
  `change helicity` interacts with either.

Match each one, or refuse it by name.

## Oracles

1. **Event by event against MadGraph.** Extend
   `validation/madgraph/gen_reweight_oracle.py` to bank `change helicity True`
   rows alongside the existing `False` ones, then extend
   `vibegraph-lib/tests/reweight_mg_oracle.rs` to compare them weight by weight.
   - Cover at least one e+e- row, one hadronic row and one SMEFT (polynomial
     path) row. Pick rows whose hypotheses actually move helicity amplitudes
     unevenly, so the per-helicity ratio differs from the summed one; show that
     difference on the banked events.
   - Set the tolerance at the scale the existing summed rows use, with measured
     headroom.
   - The pinned MadGraph (`mg5_pinned.sh`, `pixi run -e madgraph`) runs here.
     One row's regeneration is short local work.
2. **Unbiasedness.** On one row, the per-event-helicity weights and the summed
   weights must give the same reweighted σ within statistics.
   - Use 5 seeds: report the unweighted mean of each and the spread of their
     difference.
   - The two estimators share events, so compare the per-seed difference, not
     two independent errors.
3. **Mutations.** Each of these must make the event-by-event oracle fail. Show
   the command and the FAILED output, then the restore:
   - use the summed ratio in event mode;
   - use the wrong helicity (for example, reversed leg order);
   - drop a helicity's sign convention.
4. **Bit for bit:** with the default mode, banked reweight outputs are
   unchanged against main f4d6eba. Show this on at least 2 banked reweight
   rows plus one non-reweight row, using `generate` with a fixed seed and
   comparing the event file sha256 or weights.

## Refusals

A panic is never acceptable. Each of these must be a named refusal:
- an event with no recorded helicity (SPINUP 9) in event mode;
- `|M_card(λ)|² = 0` in the recorded configuration, unless MadGraph has a
  defined behaviour you then match;
- a combination you cannot support, such as decay chains or polarised legs,
  if MadGraph's semantics there are unclear.

## Gates

- fmt and clippy `-D warnings`.
- The workspace suite **with the submodule and fetched data hidden**, as the
  discipline section below requires.
- The touched `extended-validation` targets (`reweight_mg_oracle` and any
  reweight `validate_*`).

New work goes in Found. Long runs go to vg-validator as requests with branch,
SHA, cards, seeds, command and expected answer.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-helicity-reweight`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-helicity-reweight`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lh1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-helicity-reweight/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lh1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-helicity-reweight-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-helicity-reweight`). Never open a PR or post to GitHub: the
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-H1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
