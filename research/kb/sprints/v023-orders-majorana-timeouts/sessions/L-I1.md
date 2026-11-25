---
type: Session Brief
title: "L-I1: the squared-order interference sign on VLQ 612, per diagram against MadGraph standalone"
description: "Diagnosis only: VLQ card 612's VLQ^2==2 interference σ is positive in vibegraph and negative in MadGraph while |M0|² and |M2|² partition consistently. Fit one complex constant per amplitude class against MadGraph 3.7.1 standalone at one point, per diagram and per helicity, and bring a candidate fix checked arithmetically on the dumps. No production code."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Diagnosis session of [sprint L](../sprint.md), branch `sprintl-interference-sign` off
main 3f4dd815. The item is `squared-order-interference-sign-opposite-to-madgraph`
(validation, high), filed on kb PR #207. It is a possible release blocker for v0.2.3.
**No production code in this session**: the deliverable is the dumps, the fit and a
candidate fix verified arithmetically on the dumped amplitudes. The planner reviews
them before anyone writes the fix. Scratch probes and test-side edits stay on this
branch and are listed in the report.

## What is known (vg-validator, bin-pr1 9f196713)

Card 612, `TprimeBToTH_LH_M800_decay20pct_int` (gridpack MG 2.9.18):

```
import model VLQ_v4_4FNS_UFO-3rdL_noCKM
define tt = t t~
define bb = b b~
define ww = w+ w-
define tpp = tp tp~
generate p p > h tt bb j / bp x y VLQ^2==2
```

- MadGraph's signed σ: −1.101e-3 ± 3.19e-5 pb (gen_log; `results.dat` column 10).
  vibegraph, 5 seeds at campaign budget: +4.779e-4 pb, sd 4.1%. Card 614 is the
  same with ratio −1.32.
- vibegraph's order classes, seed 1, campaign budget (pb): ^2==0 6.59122e-2,
  ^2==2 4.71877e-4, ^2==4 1.28125e-3. Their sum matches VLQ<=2 (6.71938e-2) at
  1.1σ, and ^2<=2 matches ==0 + ==2 at 1.1σ. ==1 and ==3 have no diagrams.
- So the selection partitions |M|² consistently and only 2Re(M0·M2*) disagrees.
  |M0|² and |M2|² cannot see a relative sign or phase between the VLQ=0 and VLQ=2
  amplitudes; the interference can. The hypothesis is a relative sign between
  diagram classes (a vertex or fermion-flow convention), not a selection bug.
  Treat that as a hypothesis to test.

## The task

Follow `research/kb/validation/bit-exact-amplitude-debugging.md`.

1. **Provenance first.**
   - The UFO is in `/home/user/wt/sprintl-interference-sign-mg/models/VLQ_v4_4FNS_UFO`, a
     local copy from earlier work. Its manifest (sorted `relpath\tsha256` of the 15
     `*.py`) is `vlq_manifest_local.txt` beside it, sha256 83bdef2c…. The validator's
     copy hashes to 52d46852…; the difference is being resolved file by file through
     the manager. **Until the manager confirms the amplitude files match, treat the
     UFO as unconfirmed**, and record which copy every result used.
   - The param card is 612's own (validator sha 5ebc3f1f…). The manager will put it at
     `/home/user/wt/sprintl-interference-sign-mg/param_card_612.dat` when it arrives. Until
     then use the non-default entries below, and rerun the final comparison on the
     real card:
     - KBLH 3 5.092830e-01; KBLW 3 1.291330e-01; KBLZ 3 1.234540e-01;
       KTLH 3 5.097720e-01; KTLW 3 1.234480e-01; KTLZ 3 1.291390e-01;
       KXLW 3 1.830000e-01; KYLW 3 1.750000e-01
     - mb 4.7, mt 172, mta 1.777, mz 91.1876, mw 79.82436, mh 125, mx 1000,
       mtp 800, mbp 1000, my 1000; light quarks and leptons massless
     - aewm1 127.9, as 0.1184
     - wt 1.508336, wz 2.4952, ww 2.085, wh 4.07e-3, wx 5, wtp 160, wbp 5, wy 5
   - The T' width is 160 GeV. Match the width convention on every line, the T' line
     included, on both sides: standalone keeps widths on spacelike lines, MadEvent zeroes
     them. Do the first comparison with every width zero on both sides (as L-V1's `_zw`
     rows), then with widths.
2. **One subprocess**: `g u > h t b~ d` (the generate refusal names it), with the
   VLQ^2==2 split. MadGraph 3.7.1 standalone from a copy of the pinned submodule
   (`research/refs/mg5amcnlo`, version 3.7.1), banked with
   `validation/madgraph/gen_standalone_jamps.py`'s `bank()` from a scratch driver, as
   L-V1 and L-G1 did. Check whether the harness can carry the squared-order split or
   needs the two classes as separate outputs (VLQ=0 and VLQ=2 amplitude orders).
3. **The oracle**: per diagram × per helicity (× per flow) complex values at one
   point, with `tests/standalone_jamps.rs`'s `measure`/`check` on a scratch copy.
   Fit one complex constant **per amplitude class**: G0 for the VLQ=0 diagrams, G2 for
   the VLQ=2 diagrams. A relative sign shows as G2/G0 = −1 while |M0|² and |M2|² agree.
   Record |M0|² and |M2|² agreement as a precondition only. Then find which diagrams
   carry the sign (per-diagram ratios inside each class), and which vertex, Lorentz
   structure or fermion-flow orientation they share.
4. **The helicity-pruning check**: L-Z1 left `helas/eval/run.rs` pruning on |T_c|
   (around line 719, live in `integrate`) without a hermetic test. Run 612 ^2==2 on
   the probe's unpruned path, or with a scratch patch disabling
   `prune_zero_helicities`, at a reduced budget that fits here, and compare σ with the
   pruned run at the same seed. This is a scratch patch, never committed to
   production code.
5. **Candidate fix**: name the code site, and verify the fix arithmetically on the
   dumped amplitudes (apply the proposed sign or phase to the dumped vibegraph values
   and show the fit closes to G2/G0 = 1 and the interference |M|² matches MadGraph's).
   Do not edit production code. If the site is one of L-Z1's three untested
   signed-|M|² sites (split `JAMP2` `.abs()` near `run.rs:462`, pruning near
   `run.rs:719`, the signed helicity draw near `compile.rs:346`), say so.

## What not to do

No seed sweeps, no full-card σ beyond the reduced pruning check, no `pixi run
validate`, no MadEvent regeneration. A run over 20 minutes goes into the report as a
long-run request. Do not post to GitHub.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-interference-sign`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-interference-sign`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `li1-` — sibling sessions share that
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
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-interference-sign/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
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
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/li1-test.log 2>&1`,
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
  `/home/user/wt/sprintl-interference-sign-mg/`, never into the main checkout.
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
(`git push -u origin sprintl-interference-sign`). Never open a PR or post to GitHub: the
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

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-I1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
