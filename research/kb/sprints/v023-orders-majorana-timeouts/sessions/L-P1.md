---
type: Session Brief
title: "L-P1: name the region TprimeBToTZ M1800's multichannel misses"
description: "Sprint L PR 5, first session (validation-dev): re-establish the card 126 seed-dependent miss on post-PR-2 main, name the missed region from per-event output, and identify why a subdominant channel alone reaches it."
session: L-P1
agent: validation-dev
pr: 5
backlog: [multichannel-subdominant-channel-carries-dominant-region]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
# L-P1: the TprimeBToTZ multichannel miss

**Item:** `research/kb/backlog/validation/multichannel-subdominant-channel-carries-dominant-region.md`.
Run `pixi run backlog --item multichannel-subdominant-channel-carries-dominant-region`
to see it with its links, including the related
`ttz2b-sigma-carried-by-one-point`. Its `closes_when` needs two things: the
region named from per-event output, and a sampler that reaches it on every
seed at the campaign budget. **This session does the first half (diagnosis)
plus a fix only if one falls out cleanly and is cheap to validate.**
Otherwise a performance-dev session builds the fix from your diagnosis.

## Inputs

- **Card 126 (`TprimeBToTZ_LH_twentypctLO_M1800`)** as the campaign staged it:
  the proc, run and param cards, the UFO and its restriction, MadGraph's σ
  (2.504e-4 ± 2.3e-6 pb), and the host's per-seed and `inspect --ranking`
  data. These are in `/home/user/wt/cards-sprintL/tprime126.md`. Reproduce the
  card exactly from that file. If something is missing, say what rather than
  guess.
- **The model** is `VLQ_v4_4FNS_UFO` with the `3rdL_noCKM` restriction. Fetch
  it from the CMS model store; the discipline section below lists the hosts
  that are reachable. Check its sha256 against the one in the cards file.

## First: re-establish the symptom on post-fix code

The host's 5-seed row ran on v0.2.2. That release read **charged and
flavour-changing scalar vertices with the wrong chirality**, a bug PR 2 fixed
on main, and this model has such vertices (`bp~ d H` and others). So:

1. Find out whether card 126's diagram set uses an affected vertex. The rule
   is in `vibegraph-lib/src/helas/eval/diagram_eval.rs` under
   `conjugate_standalone_projectors`: a non-self-conjugate vertex whose Lorentz
   term has ProjM, ProjP or Gamma5 but no Gamma or Sigma. vg-validator's
   census may already answer this; check `tprime126.md`.
2. Reproduce on **main a5bf8fca**, your worktree's base, using the base
   recipe: `--neval 120000 --target-rel 0.01 -j 4` plus whatever else the
   cards file records.
   - Start with seed 1 (the hit) and seed 2 (a miss).
   - One seed's wall time decides how many more seeds you run locally.
     Measure it and report it before going further.
3. If the seed-to-seed disagreement does not reproduce on post-fix code,
   stop and report that. The diagnosis then changes.

## Which seeds are wrong is a hypothesis, not a fact

The item reads seed 1 (2.529e-4 ± 2.4e-6, channel 4 carrying about 9% of σ) as
the anomaly. Seeds 2–4 agree on 2.39–2.40e-4 with errors about 5× smaller, in
4 iterations. **A low σ with a small error is AGENTS.md's "confidently wrong"
signature**, so seeds 2–4 may be the ones missing a region that channel 4
finds on seed 1. The item's own knob sweep leans that way: σ climbs toward
seed 1 as the budget grows (`--neval 1.92M`: 5 of 5 seeds at 1.004 × MG). But
that MG reference is in another scheme (see the cards file), so it settles
nothing.

Settle it with an oracle that does not share the sampler's blind spot:
- vibegraph at a much higher budget, ≥5 seeds, reading where σ settles. Run it
  locally: one seed at the base budget takes 15–45 s here, so 4×–16× budgets
  are affordable within the long-command discipline. Compare it with the
  host's variant sweeps when the manager forwards them.
- the same-scheme MadGraph reference (MG 3.7.1, complex-mass scheme off), when
  vg-validator delivers it.

Name the region under whichever reading the oracle supports, and say what
each piece of evidence cannot see. The configuration ranking shows what each
configuration's own map covers, not where |M|² lives.

## Then: name the region from per-event output

The host's evidence: about 5% of σ is missing on configuration 2, the dominant
diagram (share 0.637). On the hit seed that σ comes from points drawn in
**channel 4** (propagators tp, tp~, W−; diagram share 0.6%). Channel 4's
inter-iteration χ²/dof is 190–5500. Its guess is the below-peak tail of the
broad T′ (Γ/M = 20%), but nobody has named the region.

- Use the finest per-event field available, not σ:
  - per-point weights by channel;
  - the invariant masses of every propagator in configuration 2 and in
    channel 4;
  - the multichannel weights α and the per-channel Jacobians at the heavy
    points.
  Find which kinematic region carries configuration 2's |M|² there. Then
  explain why configuration 2's own channel map does not reach it while
  channel 4's does: a mapping limit, a BW window cut, the threshold of
  `narrow_in_tau` in `phasespace/maps.rs`, or a VEGAS grid that never adapts.
- Read the mapping code paths you name, and cite them by file:line.
- **Every oracle has a blind spot**, and AGENTS.md has the sampler rules: a
  fixed-seed pull is not evidence; sweep ≥5 seeds; read the spread and the
  χ²/dof. For each conclusion, say what it cannot see.

## Coordination constraints

- **Wide-resonance mapping belongs to another session.** The sibling session's
  PR 11 is changing the BW-map threshold (`NARROW_TAU_SHARE` and
  `narrow_in_tau` in `phasespace/maps.rs`). Before you *change* how a wide
  resonance is mapped, windowed or thresholded, stop and report it in your
  report as a proposal. The manager has to clear it with the planner, because
  it touches the banked σ on every row with a BW map.
- Judge every σ against vibegraph's own high-budget value, or against the
  complex-mass-scheme-off MadGraph reference once it exists. **Never judge it
  against the gridpack's 2.504e-4**, which is in the complex-mass scheme.
- Changes to the multichannel α survey, budget floors or channel weighting are
  yours to prototype. Show any prototype against ≥5 seeds of this card **and**
  bit-for-bit or within-error on banked multichannel rows. Don't push such a
  change before the manager has seen the evidence.

## Long runs

Campaign-budget multi-seed runs go to vg-validator as requests: ≥5 seeds at
`--neval 120000 --target-rel 0.01`, plus any knob sweep. Each request names
the branch, the SHA, the card, the seeds, the command and the expected answer.
Run locally only what fits the long-command discipline below.

## Gates (if you change code)

- fmt and clippy `-D warnings`.
- The hermetic suite with the submodule and fetched data hidden, as the
  discipline section below requires.
- The touched `extended-validation` targets.
- Bit for bit on ≥3 banked cards against main a5bf8fca.

New work goes in Found.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-multichannel`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-multichannel`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lp1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-multichannel/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lp1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-multichannel-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-multichannel`). Never open a PR or post to GitHub: the
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-P1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
