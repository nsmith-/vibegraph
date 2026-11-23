---
type: Session Brief
title: "L-T3: enumeration and set compilation across the cores, the flavour prune, one compile per set"
description: "Skip subprocesses that break a model-conserved additive number before topology assignment; run the subprocess loop, the set-compilation loops and enumeration's default parallelism across the pool; let the amplitude probe reuse the run's evaluators. All order-preserving, bit for bit."
status: draft
agent: performance-dev (Opus)
depends_on: [L-T2]
closes: [enumeration-hands-flavour-violating-subprocesses-to-feyngraph, set-compilation-loops-are-serial, amplitude-probe-compiles-each-set-twice]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
First session of PR 3b of [sprint L](../sprint.md), on branch
`sprintl-timeout-fixes` off main a5bf8fca. PR 3's triage named the causes
(the L-T2 report `sessions/L-T2-report.md` and the sample page
`v022-1009-phase-sample.md`, both on PR 3's branch until it merges).

**Two of the items aren't on main yet.**
`enumeration-hands-flavour-violating-subprocesses-to-feyngraph` and
`set-compilation-loops-are-serial` were filed on PR 3's branch, and the PR 3
merge adds them to main. Read them, and the report, with
`git show origin/sprintl-timeout-triage:research/kb/backlog/performance/<slug>.md`
and `git show origin/sprintl-timeout-triage:research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md`.

`amplitude-probe-compiles-each-set-twice` is on main, and PR 3 raises it to
high. The manager merges main into this branch after PR 3 lands, and the
item deletions happen then. Don't recreate the items here.

## Scope

1. **Flavour prune** (`enumeration-hands-flavour-violating-subprocesses-to-feyngraph`).
   Before `assign_topologies`, skip any assignment whose total of some
   additive quantum number is nonzero, where the number is conserved by every
   vertex of the loaded model.
   - **Derive the numbers from the vertex table.** Never assume the SM:
     - the lepton numbers per generation, baryon number and the quark
       generations are candidates only if the model's vertices conserve them;
     - a CKM, FCNC (TopFCNC), RPV or leptoquark model must prune nothing it
       shouldn't.
   - **Tests:**
     - a hermetic test that pruned and unpruned diagram sets are identical on
       a flavour-violating model (TopFCNC or ckm-on sm, whichever is in the
       hermetic store) and on sm-no_b_mass;
     - a test that the conserved numbers derived for sm-no_b_mass are the
       expected ones;
     - a test that a model with flavour mixing derives fewer numbers.
   - **Measure:** the enumerate wall time on a W(→ℓν)+0–4j MLM card and on
     Z(→νν)+4j, before and after.
2. **Parallel subprocess loop, and enumeration parallel by default.** The
   candidate loop in `generate_sets_inner` is serial, and
   `--parallel-diagrams` only parallelises inside one subprocess's
   assignment. Run the subprocess loop across the rayon pool, with results
   in input order. The user-approved scope is folded in: see the next section.
3. **Set compilation across the pool** (`set-compilation-loops-are-serial`).
   `derive_flavor_groups` (`proton.rs` ~1107) and `amplitude_probe::measure`
   (`amplitude_probe.rs` ~262) compile sets in serial iterators. Run them on
   the pool, with results in set order.
4. **One compile per set** (`amplitude-probe-compiles-each-set-twice`). The
   probe evaluates through the evaluators the run compiles, on both the
   fixed-energy and proton paths, in `integrate` and in `generate`. This
   changes phase boundaries: the probe's cost moves into, or merges with,
   `compile`. Keep the phase records truthful (see Constraints), and record
   the measured saving on `p p > e+ e- j j j` as the item asks.
5. **`generate-parallel-diagrams-is-a-no-op`**, if it fits: make `generate`
   honour or refuse the flag. Regenerate the CLI reference with
   `scripts/gen-cli-docs.sh`; never hand-edit `docs/src/cli/reference.md`.

### Enumerate scales across cores by default

This is the user-approved need, relayed by the planner on 2026-11-23.

The card is DY+0–3j: sm-no_b_mass, maxjetflavor 5, `generate p p > e+ e-`
plus `add process` with 1, 2 and 3 j. It has 260 subprocesses and 10310
diagrams, run with `--svg none` on a 2-CPU VM at v0.2.2:

| run | wall | user |
|---|---|---|
| -j1 | 27.3 s | 27.0 s |
| -j2 | 27.2 s | — |
| -j2 --parallel-diagrams | 20.8 s | 28.9 s |

At -j2 without the flag there is no gain, because `--parallel-diagrams` is
opt-in. With the flag, the enumerate stage alone drops from 20.5 s to
13.9 s. Both runs carry about 7 s of serial work: the compile stage over
flavour groups and cuts, plus a 3.0 s silent gap between the last "compiled
cuts" line and the final "10310 diagrams in 260 subprocesses" line.

Scope:
1. Parallel enumeration is the default when it pays. Use a cost estimate
   available before enumeration starts. `--parallel-diagrams` stays as
   force-on, with a force-off if one is needed. Measure the threshold and
   record it; don't guess it.
2. Explain the 1.5× and parallelise inside it if that is cheap. One
   candidate: a single process line's walk is serial and dominates.
3. Identify the serial tail (the 3 s gap and the compile stage) and move it
   onto the pool wherever the output cannot change.

Acceptance:
- that card at -j2 with no flag enumerates in ≈14 s or less;
- output byte-identical to serial: diagram sets, artifact digest,
  diagrams.json;
- small processes no slower: ee→μμ and a 2→2 pp card, at -j1 and -j4;
- a hermetic test pins parallel against serial byte identity.

## Constraints and gates

- **Order-preserving.** Bit for bit against main on σ, its error, grid
  sha256, diagram-set and artifact digests: one fixed-beam and one MLM
  banked card at -j 1 and -j 4, plus the DY+0–3j card's diagrams.json and
  digest. A main reference build lives in its own detached worktree with its
  own target, and you remove it after.
- **No panic** from a parallel path. A refusal raised inside the pool
  surfaces as the same named refusal the serial path gives. Test one.
- **The phase instrument** is being merged separately (PR 3a, #183). It isn't
  on main yet, so don't add phase records in this session. If #183 lands
  while you work, the manager merges main into this branch. After that, any
  phase you move needs its records moved too, and the phase-timing test must
  still pass.
- **Host-side measurements** go to the manager as a request; never run them
  here:
  - the enumerate diff, v0.2.2 against this head on every released v022-1009
    card that enumerates within the cap, identical card for card;
  - re-timing cards 58, 15, 1009 and 65.

  Draft the request in your report: branch, SHA, cards, command, expected
  answer.
- **Local timing** of the DY+0–3j and W/Z+4j cards is a heavy job. Ask the
  manager before each one. Build profiles must match on both sides of any
  comparison.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-timeout-fixes`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-timeout-fixes`) and `git branch --show-current` (`sprintl-timeout-fixes`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `lt3--`
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-timeout-fixes/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-timeout-fixes`), and run anything long fully detached
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-T3-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
