---
type: Session Brief
title: "L-D2: complex couplings on two-fermion contacts and self-conjugate dipoles bind as MadGraph's"
description: "Fix UFOModel::conjugate_couplings so every two-fermion vertex takes its Hermitian partner's coupling on the conjugated structure, times the sign conjugation produces (a self-conjugate vertex is its own partner); return that sign from bound_coupling; pin it with a committed per-diagram row with imaginary Wilson coefficients that fails on main. Signed off by the planner."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Implementation session of [sprint L](../sprint.md), branch `sprintl-cp-odd-couplings` off
main c095a1cf. The diagnosis is in `sessions/L-D1-report.md` on branch
`sprintl-dim6top-sigma` (commit be45128c). Read it first: `git show
origin/sprintl-dim6top-sigma:research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-D1-report.md`.
Its scratch harness is in `/home/user/wt/sprintl-dim6top-mg/harness/` (`ld1_fixcheck.py`,
`ld1_hackcmp.py`, the probe). Its tables are in `tables/`. The dim6top UFO is in `models/`.

## The defect (L-D1)

With nonzero imaginary Wilson coefficients (card 6's point: ctpI 0.020136, ctWI 0.318131,
ctZI 0.668223, ctGI −0.800347), 23 of the 75 diagrams of
`g u > t b~ d mu+ mu- (t > w+ b, w+ > e+ ve)` disagree with MadGraph 3.7.1 standalone.
They fall in two classes:

- **(A) Two-fermion contact vertices.** These are t-b-W-γ and t-b-W-Z, `V_382` and `V_402`.
  vibegraph binds the vertex's own coupling (GC_795) where MadGraph binds its Hermitian
  partner's (GC_796 = conj(GC_795)), so the phase is off by −c/c*. Cause: the early return
  in `UFOModel::conjugate_couplings` (`vibegraph-lib/src/ufo/mod.rs`, around line 820) for
  any vertex without exactly three particles.
- **(B) Self-conjugate top dipoles.** These are t-t-g, t-t-γ and t-t-Z, `V_189`, `V_49` and
  `V_51`. The σ·q·γ⁵ term (FFV9) has the wrong sign relative to the σ·q term (FFV11).
  Cause: the `own == anti` early return (around line 844).

With real coefficients the correction factor is exactly 1, so every enforced row agrees
today.

**Candidate rule, checked on the dumps.** Every vertex with two fermion slots takes its
Hermitian partner's coupling on the conjugated structure, times the sign the conjugation
produces. A self-conjugate vertex is its own partner. Concretely: a contact vertex takes
−c_hc/c, and the FFV9 term is negated. In production, the coupling match must allow an
overall sign and `bound_coupling` must return it.

Checked on the dumps:
- the fitted subprocess closes to 8.3e-14 per diagram;
- its charge conjugate `g d > t~ b u mu+ mu-`, not used to build the rule, closes to
  1.9e-14;
- fixing only A or only B leaves 4–15%.

## The planner's conditions for the PR

1. **A committed per-diagram × per-helicity row** with nonzero imaginary coefficients on
   both vertex classes. It must fail on main and pass with the fix. Use card 6's point on
   `g u > t b~ d mu+ mu-` and its charge conjugate.
   - Vendor dim6top_LO_UFO only if its licence allows. The public tarball ships no licence
     text: check its authors' terms (FeynRules model page, the dim6top paper
     arXiv:1802.07237) and record what you find.
   - If it cannot be vendored, use a model that can be vendored and has both vertex
     classes, such as a toy UFO in the test tree like the existing toy models. Record which
     you used and why.
   - The banked MG table needs a committed generation recipe, like the existing standalone
     rows.
2. **A test pins the overall sign `bound_coupling` returns.** It must fail if the sign is
   dropped. Show it failing with the sign removed.
3. **No regression in the extended-validation gates.** List each with its command and
   output. Rows with real coefficients must stay bit-identical; the correction factor is
   exactly 1 there. Name the enforced rows the change touches, as L-D1's report lists them,
   and show those rows identical.
4. **Artifact staleness.** Check whether any artifact embeds bound couplings or the
   vertex-to-coupling map. If one does, bump `ENUMERATION_VERSION` (`diagrams/artifact.rs`)
   to 5 in the same PR. Measure this; don't assume it. #178 → #211 is this sprint's lesson
   on exactly this.
5. **Fallback.** If the fix is not converging, stop and report. The alternative is a named
   refusal for an imaginary coupling on a two-fermion contact or a self-conjugate dipole,
   but the manager decides on that, not you.

## Scope

- **Production change:** the `ufo` coupling binding, `conjugate_couplings` and
  `bound_coupling`, plus whatever carries the sign to the evaluator.
- **New tests:** the committed row, the sign pin, and unit tests for the two vertex classes
  at the model layer.
- **Out of scope:** card 6's σ gap, which is a run-card question handled in L-D1's
  follow-up; the `/ h` echo; and `FCNC=0` acceptance. Those are separate items.
- Hygiene, as for any PR: comments describe the code in its own terms, and no panics.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-cp-odd-couplings`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-cp-odd-couplings`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `ld2-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-cp-odd-couplings/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/ld2-test.log 2>&1`,
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
  `/home/user/wt/sprintl-cp-odd-couplings-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-cp-odd-couplings`). Never open a PR or post to GitHub: the
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-D2-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
