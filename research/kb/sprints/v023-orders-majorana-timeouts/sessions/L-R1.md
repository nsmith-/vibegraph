---
type: Session Brief
title: "L-R1: the model-parameter report and machine-readable artifact provenance"
description: "Report a model's external parameters (class, default, reweight verdict) and the parameters entering a process, write the default param card, and give integrate/inspect results and artifact refusals machine-readable provenance."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [model-parameters-not-reported, artifact-provenance-not-machine-readable]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 8 of [sprint L](../sprint.md), on branch `sprintl-reports` off main
b2434a0. simshop asked for both items in v0.2.3. **PR 9 (Auto widths, the
sibling sprint session) reports its widths through the parameter record you
define**, so the record shape is the first thing to fix: write it into the CLI
docs and commit it early, then tell the manager in your report (or stop and
report if the shape is in doubt) so PR 9 can design against it.

Read each item (`pixi run backlog --item <slug>`) and the concepts they link:
- `model-parameters-not-reported`
- `artifact-provenance-not-machine-readable`

Starting points: `vibegraph-cli/src/{enumerate,inspect,integrate,output,check,error}.rs`,
`vibegraph-lib/src/ufo/` (loader, restriction handling, `bind.rs`),
`vibegraph-lib/src/reweight/` (parameter classification, polynomial monomials),
the ldjson schema docs in `docs/src/cli/` (`overview.md`, `reference.md`), and
the existing `vibegraph run-card` report (`vibegraph-cli/src/run_card.rs`) as the
model-side counterpart's precedent.

## Scope

1. **Parameter record.** As the item's `closes_when` lists: one record per
   external parameter (name, block, code, post-restriction default; class
   free / zero / one / merged-into-X-with-factor / internal; MASS/DECAY
   particle and whether it is external in the process; reweight verdict and
   polynomial entry; binder value constraints). Design it so a later field
   (a computed width, its method) is an *added* field, not a reshape: PR 9 adds
   auto-computed widths here. When a process is given, the list of external
   parameters entering at least one diagram's couplings. Write the default
   `param_card.dat`. Choose the surface (`parameters.json` beside
   `diagrams.json` from `enumerate`, and/or a model-only subcommand) and record
   why. Tests: one parameter of each class, and the entering list for one
   SMEFTsim process (the vendored `validation/ufo/SMEFTsim_topU3l_MwScheme_UFO`,
   restriction `massless` or similar that exists in the tree). Check the
   default param card against the model's own restriction card value for value
   in a test.
2. **Provenance.** `integrate` and `inspect` results carry the artifact's
   `enumeration_version` and the writing vibegraph version; an `artifact`-kind
   refusal carries a stable `reason` (enumeration_version found/expected,
   format_version, digest mismatch, model mismatch, collider energy). Added
   fields under `schema_version` 1 — no new error kind. CLI tests per reason,
   including a version-2 artifact (construct one; do not depend on an old
   binary).
3. Document both record shapes in the ldjson schema docs. Hermetic suite,
   fmt, clippy; no σ line can move (say so with the reason, or show it).
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-reports`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-reports`) and `git branch --show-current` (`sprintl-reports`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `lr1-`
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-reports/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-reports`), and run anything long fully detached
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-R1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
