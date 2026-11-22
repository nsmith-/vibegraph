---
type: Session Brief
title: "L-U1: run-card placeholders, non-string and conditional UFO parameters, coupling-order expressions"
description: "Refuse a run card left with $ placeholders by name; load the ctGpatched and dim6top UFO forms MadGraph loads (or refuse them with MadGraph's reason); time_of_flight if small."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [run-card-dollar-lines-skipped-silently, ufo-non-string-parameter-value-refused, ufo-conditional-parameter-definitions-refused, ufo-coupling-order-expressions-refused, time-of-flight-unimplemented]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 7 of [sprint L](../sprint.md), on branch `sprintl-refusals` off main
b2434a0. Order: the run-card item first (high priority, a silent wrong
answer), then the UFO items, then time_of_flight only if it is small.

Read each item (`pixi run backlog --item <slug>`) and the concepts they link:
- `run-card-dollar-lines-skipped-silently` (high)
- `ufo-non-string-parameter-value-refused`
- `ufo-conditional-parameter-definitions-refused`, which unblocks
  `ufo-coupling-order-expressions-refused` (the evaluator K-U1 added already
  handles the order expressions; this PR closes both once dim6top gets past
  model load)
- `time-of-flight-unimplemented` (low; only if small)

Code: `vibegraph-lib/src/runcard/`, `vibegraph-cli/src/run_card.rs`, the UFO
loader under `vibegraph-lib/src/ufo/` (parameter parsing and the configuration
evaluator), the LHEF writer for time_of_flight. MadGraph's reading is in the
pinned `research/refs/mg5amcnlo` (`models/import_ufo.py`, `models/model_reader.py`,
`madgraph/various/banner.py` for run cards).

## Scope

1. **`$` placeholders.** integrate, generate and `--check` refuse by name a run
   card whose value for any key is an unfilled `$` placeholder, naming the key,
   the placeholder, and that CMS's gridpack script fills it at build time; the
   run-card report marks the key refused, not honoured. CLI test on a CMS
   InputCards-style card (both `$DEFAULT_PDF_SETS = lhaid` and
   `$DEFAULT_PDF_MEMBERS = reweight_PDF` forms; check genproductions'
   `bin/MadGraph5_aMCatNLO/gridpack_generation.sh` over raw.githubusercontent.com
   for the full placeholder set it substitutes, and cover each).
2. **ctGpatched.** First fetch `SMEFTsim_topU3l_MwScheme_UFO_ctGpatched` from
   the CMS model store
   (`https://cms-project-generators.web.cern.ch/cms-project-generators/`) and
   record the exact non-string `value` form of `Gstrong`. Read how
   `import_ufo.py` / `model_reader.py` handle it, and cite the lines. If
   MadGraph loads it, accept the same forms (no wider), with a minimal-UFO
   test, and show the 11 restrictions enumerate simshop's store probe (use
   `e+ e- > t t~` or the item's probe if named). If MadGraph refuses it, the
   refusal names MadGraph's reason.
3. **dim6top.** Evaluate parameter definitions under
   `if configuration.<flag>:` blocks and string-valued conditional value
   expressions with `configuration.py`'s defaults, as a stated subset of
   Python, refusing anything outside it by name. Tests on a minimal UFO;
   `dim6top_LO_UFO` (model store) gets past model load and enumerates the
   tZq_EFT_CPV_point_1_13p6 process (from genproductions). That closes both
   dim6top items. An amplitude check of dim6top against MadGraph is not in
   scope unless loading changes a value an existing banked model sees; say so.
4. **time_of_flight.** If writing MadGraph's per-particle lifetime into the
   LHE VTIMUP field is small (the formula in MadGraph's event writer, cited,
   and a test), do it; checking the field against a MadGraph sample needs a
   MadGraph run, so ask the manager. Otherwise document the off value as the
   only supported reading in `research/kb/pipeline/release-scope.md` and close
   on that, as the item allows.

No σ line can move for items 1–3 except by refusal; show bit-for-bit σ on one
banked fixed-beam and one hadronic card, main against your branch, if the
loader changes touch the SM path.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-refusals`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-refusals`) and `git branch --show-current` (`sprintl-refusals`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `lu1-`
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-refusals/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-refusals`), and run anything long fully detached
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-U1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
