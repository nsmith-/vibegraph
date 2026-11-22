---
type: Session Brief
title: "L-Q1: squared-order constraints by coupling-order split"
description: "Split each amplitude by coupling-order class and form only the cross terms a squared-order constraint selects, so NP^2==1 (interference only) and NP^2==2 match MadGraph per diagram class, in |M|² and in σ."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [squared-order-constraints-refused]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 1 of [sprint L](../sprint.md), on branch `sprintl-squared-orders` off main
b2434a0. PR 10 (helicity-aware reweighting, `reweight-helicity-summed-only`)
follows on the same split after this merges, so build the split as a reusable
graded evaluation — the amplitude carried as its coupling-order components —
rather than a special case of the card check.

Read the item (`pixi run backlog --item squared-order-constraints-refused`) and
the concepts it links. The code: the refusal in
`vibegraph-lib/src/diagrams/check.rs` (`Unsupported::SquaredOrder` near L290 and
its use near L705; the module table near L23), the decay-chain refusal in
`vibegraph-lib/src/diagrams/resolve.rs` (near L361), the test
`a_squared_order_constraint_is_a_hard_error`, and `vibegraph-lib/src/reweight/poly.rs`
(which already proves each diagram's coupling monomials).

## Scope

1. **Semantics first.** Before writing code, read MadGraph's squared-order
   semantics in the pinned `research/refs/mg5amcnlo` (`madgraph/core/`
   diagram generation and `madgraph/iolibs/export_v4.py` / the `matrix.f`
   template's `AMP2`/squared-order split: `==`, `<=`, `>`, and how an
   interference-only `NP^2==1` selects cross terms between amplitude orders).
   Write the semantics down in the report with file:line citations, including
   how MadGraph combines a squared-order constraint with the amplitude-level
   order limits it implies, and what it does with several squared constraints at
   once. Cite before you implement; do not infer a rule MadGraph does not show.
2. **The split.** Classify each diagram by its coupling-order vector, evaluate
   the amplitude per class (per colour flow, per helicity), and form |M|² from
   only the class pairs the constraint selects. Constraints MadGraph accepts and
   you do not implement stay refused **by name**; nothing passes silently.
   `WEIGHTED==`/`WEIGHTED>` (`Unsupported::WeightedOrder`) and the `aEW`/`aS`
   cases: take them if they fall out of the split, otherwise leave them refused
   and put them in Found. Decay chains: support or keep refusing by name.
3. **Oracles, finest first** (AGENTS.md Physics Validation):
   - per coupling-order class and per helicity against MadGraph standalone
     (MadGraph 3.5.7 here; `validation/madgraph/build_amplitude.sh` and
     `compare_amps.py` are the existing per-diagram tooling) on
     `e+ e- > t t~ NP^2==1` and `NP^2==2` under the banked SMEFTsim restrict
     cards (`validation/manifest.toml`'s SMEFTsim ladder), plus one QCD case
     such as a `QCD^2` constraint on a process with both QCD and QED diagrams;
   - then |M|² at fixed points, and σ for those SMEFTsim rows against MadGraph
     with ≥ 5 seeds, unweighted mean, spread and χ²/dof read. Bank the new rows
     so CI's `banked` gate enforces them. If a σ reference needs MadGraph 3.7.1
     (the pinned banking version), write the exact request in your report
     instead of banking from 3.5.7.
   - Know what each test cannot detect: a full-|M|² row is blind to a swapped
     class assignment that preserves the sum, so the per-class comparison is
     the real oracle.
4. **Nothing else moves.** Processes without squared orders must be bit for bit:
   σ and grid sha256 for main b2434a0 against your branch on ≥ 3 banked cards
   (one fixed-beam, one hadronic, one SMEFT at `NP<=1`). Build main in a
   separate target dir and delete it afterwards.
5. **Campaign cards.** The item names 24 campaign cards (21 squared-order
   refusals, plus three `M3^2==1` / `T6^2==1`). Report which of these you can
   show getting past the refusal with a short `enumerate`/`integrate` (cards from
   `cms-sw/genproductions` over raw.githubusercontent.com); the full set goes to
   the long-run host via your report.

Update the check-module table line ("amplitudes split by coupling order (not
planned)") and the CLI docs (`docs/src/cli/`) for the new grammar.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-squared-orders`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-squared-orders`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lq1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-squared-orders/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lq1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-squared-orders-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-squared-orders`). Never open a PR or post to GitHub: the
manager does. Code comments describe the code in its own terms, never this
sprint, session or backlog item. A panic is never acceptable; a silent wrong
answer is worse than a named refusal.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-Q1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
