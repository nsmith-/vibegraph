---
type: Session Brief
title: "L-Z1: hygiene pass on squared orders and per-event-helicity reweighting"
description: "Review sprint L's merged PR 1 (#168) and PR 10 (#182) for maintainability, test non-vacuity, visibility and reusable abstractions; findings first, then fixes with mutation evidence, σ bit for bit."
status: draft
agent: hygiene-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Hygiene pass of [sprint L](../sprint.md) (dispatch step 4), on branch
`sprintl-hyg-amp` off main 033044e4. It covers these merged PRs. Read each squash
commit's diff (`git show <sha>`) and its session briefs and reports under
`sessions/`:

- #168 `f4d6eba9`: squared-order constraints by coupling-order split (PR 1). Brief and report: `sessions/L-Q1.md`, `L-Q1-report.md`.
- #182 `853028f8`: per-event-helicity reweighting (PR 10). Brief and report: `sessions/L-H1.md`, `L-H1-report.md`.

Both change how amplitudes are split (`helas/eval/run.rs`, `reweight/engine.rs`), so review them together: a second copy of the split logic between them is the likeliest finding.

Up to two sibling hygiene passes run alongside you. Stay inside the files
these PRs touched, plus their direct callers and tests. The other clusters
are: amplitude split (#168/#182, L-Z1); fermion flow and UFO particles
(#169/#178, L-Z2); widths and resonance records (#179/#188, L-Z3); multichannel
bound (#184, L-Z4). Do not edit code another cluster owns except where a
finding needs it, and then say so.

## Scope

1. **Findings first.** Before editing anything, write the findings section of
   your report and commit it. For each finding give the file:line, what is
   wrong, the class (dead or duplicated code, a vacuous or tautological test, a
   public item that should be private, a hand-kept list that should be derived,
   a missing reusable abstraction, a comment that breaks AGENTS.md's comment
   rules, a panic or unwrap on a fallible path), the evidence (a command with
   its output, or the quoted lines) and the fix you propose. Rank them and mark
   which ones you will fix here.
2. **Test non-vacuity.** For every test these PRs added, make one mutation of
   the code it guards that it should catch, run it, and record whether it
   failed. A test that survives its mutation is a finding: strengthen it and
   show it now fails under the same mutation. Batch the mutations with a
   script that applies, runs and reverts each one.
3. **Fix** the findings you marked. Behaviour is unchanged, and you prove it
   with the identity oracle below, chosen before the first edit: main built
   in a separate target dir under `/home/user/wt/lz1-main-target` and
   deleted afterwards.
4. **Physics or performance findings** (a wrong convention, a missing oracle,
   a slow path) are not yours to fix. List them under Found.

**Time matters.** Every sprint branch must be merged before the user rewrites
main's history for the v0.2.3 tag. Keep the pass tight: the change's own code
and tests, not a codebase sweep. Ship what is fixed and verified; anything
large goes in Found.

No seed sweeps, MadGraph runs or local `pixi run validate`: CI's banked job is
the banked gate. The gates are fmt, clippy `-D warnings` (plain and with
`--features extended-validation`), the hermetic `cargo test --workspace` with
the three data paths hidden, the `extended-validation` targets these PRs touch,
and kb-lint (`python3 scripts/kb.py lint`, or `pixi run kb-lint`).

**Identity oracle.**
- σ and grid sha256 bit for bit against main on `dy13_default` (hadronic) and `pp_to_llj_mlm` (MLM): `integrate --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`.
- One squared-order process from `tests/squared_orders.rs` (a `QCD^2==` or `QED^2<=` constraint) with the same argv.
- One `reweight` run byte-identical against main: the `cli_reweight_proton` setup, or a `gen_reweight_oracle.py` row with `change helicity True`, on the same events.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-hyg-amp`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-hyg-amp`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lz1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-hyg-amp/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lz1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-hyg-amp-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-hyg-amp`). Never open a PR or post to GitHub: the
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-Z1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
