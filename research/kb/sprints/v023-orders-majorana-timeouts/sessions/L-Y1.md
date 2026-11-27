---
type: Session Brief
title: "L-Y1: group index and run-global position on ldjson progress records"
description: "Add group, groups, label and channels to every per-group progress stage, and a monotone run-global phase/phases built on the phase records, as additive extras under schema_version 1."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [ldjson-progress-has-no-run-global-position]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
The only session of PR 14 of [sprint L](../sprint.md). It runs on branch
`sprintl-progress` off main 1058b78, which already has PR 3a's phase
records (`vibegraph-cli/src/phase.rs`).

Start by reading the item:
`pixi run backlog --item ldjson-progress-has-no-run-global-position`.

## Need

The need comes from a downstream consumer. On a multi-multiplicity card
(DY + 0–3 jets, @1–@4), the `alpha_survey` progress record is
`{stage, done, total=6}`, and that stage runs once per outgoing-leg group.
Users therefore see "1..6 of 6" restart several times and read it as the
survey restarting. Nothing in the record is run-global.

## Requirements

Add new progress *extras* only. There is no `schema_version` bump, and no
existing field changes meaning.

1. **Group extras.** Every per-group stage carries four fields:
   - `group` (1-based);
   - `groups` (N);
   - `label`: the group's `@N` tag, or the leg count when the card has no tag;
   - that group's `channels`.

   This covers `alpha_survey` and every other stage that repeats per group.
   Find each one in the code and list it in your report.
2. **Run-global position.** Add `phase`/`phases` (the index of the current
   phase and the count of phases this run executes), taken from the phase
   instrument's own sequence. Do not define a second notion of "phase". The
   value must never go backwards across the record stream.
   - `phases` is the number of phases the run will actually execute. It is
     known up front from the run's mode: proton or fixed-energy, `--check`,
     `generate`.
   - If you cannot make the count honest before the run starts, say why in
     your report, and choose a form that stays monotone.
   - Do not add an overall fraction unless you can prove it never goes
     backwards.
3. **Docs.** Document the extras in `docs/src/cli/overview.md`, next to the
   existing progress record and the "Phase timing" section. Explain how a
   reader derives a monotone overall position from them. If any CLI help text
   changes, regenerate the CLI reference with `scripts/gen-cli-docs.sh`;
   never hand-edit `docs/src/cli/reference.md`.

## Acceptance

All of these are hermetic tests in the default suite.

- **Two groups.** `generate p p > e+ e-` plus `add process p p > e+ e- j` on
  sm-no_b_mass:
  - the two α surveys' records carry group 1 of 2 and group 2 of 2;
  - the labels are distinct and match the groups;
  - `channels` matches the group's channel count;
  - the global position is non-decreasing across the whole record stream
    (`phase`, `progress` and the rest).

  The proton path needs a PDF set. Use the fetched NNPDF set under
  `extended-validation`, as the existing proton phase test does, or a
  hermetic PDF if the suite has one. Check before writing: the proton phase
  test failed CI once for needing reference data. **Run your suite with the
  reference data hidden:** move `research/refs/mg5amcnlo`,
  `validation/madgraph/output` and `validation/pdf/NNPDF23_lo_as_0130_qed` out,
  run it, then put them back.
- **One group.** A single-group run reports group 1 of 1.
- **Ordering unchanged.** A test pins that existing progress fields and
  record order are unchanged: the extras are additive only.
- **σ bit for bit.** σ and grid sha256 are unchanged against main on one
  fixed-beam card and one MLM card. Progress writes must not touch the
  sampling order. Build a main reference in its own detached worktree with
  its own target, and remove it afterwards.

When the tests pass, delete the item file in your final commit.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-progress`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-progress`) and `git branch --show-current` (`sprintl-progress`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `ly1--`
  in your scratchpad — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between
  completion notifications instead of idle-waiting. On resume after a stall,
  first check whether the interrupted command finished (`ps`, result
  artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
  CI's `banked` job runs `pixi run --skip-deps validate` on every push: do not
  run `validate` locally. Run the hermetic suite and the single banked targets
  your change touches.
- **Disk is finite, and a full disk looks like a failure.** Always
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-progress/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-progress`), and run anything long fully detached
with results appended to a file. **Ask the manager before any heavy job**
(a release build of the workspace, a seed sweep, a MadGraph run, anything over
~10 minutes of CPU): a sibling may be running one. Long runs (multi-seed σ over
~20 min per seed, MadGraph 3.7.1, timed campaign runs, >8 GB) go to the
validator host through the manager: write the request (branch, SHA, cards,
seeds, command, expected answer) into your report or a message, never run it
here.
- pixi 0.84.1: `export PATH=$HOME/.pixi/bin:$PATH`. The reference data
  (`validation/madgraph/output`, `validation/pdf/NNPDF23_lo_as_0130_qed`, the
  `research/refs/mg5amcnlo` content) is in your worktree as hard links: never
  edit it in place. No MadGraph env is installed; ask the manager if you need
  one, never `pixi install -e madgraph` in your worktree.
- Put downloads (model store, genproductions cards) in their own empty
  directory under your scratchpad; read them, never execute them.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the container's identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit; keep
current by merging main. Never open a PR or post to GitHub: the manager does.
Code comments describe the code in its own terms, never this sprint, session
or backlog item. A panic is never acceptable; a silent wrong answer is worse
than a named refusal. `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` are gates.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-Y1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
