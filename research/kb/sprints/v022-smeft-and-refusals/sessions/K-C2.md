---
type: Session Brief
title: "K-C2: jet and lepton rank pT cuts"
description: "Implement ptj1–ptj4 and ptl1–ptl4 min/max as MadGraph's cuts.f rank cuts, with σ against MadGraph on DY1j pT bins, a biting lepton-rank cut, and the ZH SMEFTsim gate."
status: draft
agent: feature-dev (Opus)
depends_on: [K-C1]
closes: [rank-jet-pt-cuts-unimplemented, rank-lepton-pt-cuts-unimplemented]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-18}
---
Second half of PR 5 of [sprint K](../sprint.md), on `sprintk-runcard` after
K-C1 (head 488295c). Read [K-C1's report](K-C1-report.md) first. K-C1 made
the off sentinels inert and left every value that would cut refused, by name,
in `unimplemented_cut_refusal` (`vibegraph-lib/src/cuts.rs`). The `run-card`
report and table read the same lists, so keep them in step. PR 4 (#153,
`sprintk-mlm-panic`) runs in parallel in the clustering code.

Read the items: `rank-jet-pt-cuts-unimplemented` and
`rank-lepton-pt-cuts-unimplemented`.

## Scope

1. **Implement** `ptj1min`–`ptj4min`, `ptj1max`–`ptj4max`,
   `ptl1min`–`ptl4min` and `ptl1max`–`ptl4max` as MadGraph's rank cuts.
   Cite each piece from the pinned `Template/LO/SubProcesses/cuts.f` and
   `setcuts.f`:
   - which objects are ranked: jets with `maxjetflavor` and b handling;
     charged leptons only;
   - pT ordering, and what happens with fewer objects than the rank;
   - `cutuse` (any versus all) for the jet family, and whether the lepton
     family has an equivalent;
   - interplay with `cut_decays` and the decay-product exemption;
   - whether and how each bound enters the τ floor (`setcuts.f`, the
     `max(…, 0)` terms).

   Do not guess a convention. Any knob you do not implement (for example
   `cutuse = 1`, if you do not get to it) stays refused by name.
2. **Oracles, measured on your head:**
   - **Unit tests** on hand-built momenta. For each family, test ordering,
     fewer objects than the rank, min and max, and `cutuse`.
   - **Jet family.** σ on two `DY1jToLL_M-1to10_Pt-*` bins against local
     MadGraph 3.5.7, at ≥ 5 seeds each side (unweighted mean, spread,
     χ²/dof). Rebuild the cards from `cms-sw/genproductions` over
     `raw.githubusercontent.com`, or from the item's description; name the
     substitution.
   - **Lepton family, biting.** The CERN host's ZH references barely test
     the cut: at `ptl1min` = 10, MadGraph's σ moves only −0.1% from
     `ptl1min` = 0. Use the 832 (SM) inputs in
     `/home/user/wt/sprintk-runcard-mg/zh832/` and the model in
     `/home/user/wt/sprintk-runcard-mg/model/` (README there). Generate with
     local MadGraph at a `ptl1min` that moves σ by ≥ 10% (e.g. 50 GeV; check
     the shift first). Run MadGraph and vibegraph at ≥ 5 seeds each.
     - MadGraph 3.7.1 refuses `NPprop=0` on this restriction. If 3.5.7
       does too, drop it, as the CERN host did.
     - A cheaper non-SMEFT card with the same lepton topology (`p p > z h`,
       `z > l+ l-`, `h > b b~` in the SM) is acceptable if 832 is too slow.
       Name the substitution.
   - **ZH gate.** Integrate 832 and 837 at their real `ptl1min` = 10, 5
     seeds each, using each card's own `param_card.dat` (837 has
     `chw` = 0.4). Compare against the CERN host's MadGraph 3.7.1 references:
     SM 0.07481 ± 0.00005 pb and HW0p4 0.09970 ± 0.00006 pb. Note that 837's
     run card differs from 832's only in the order of its hidden lines.
   - **Seven cards unpatched.** Show the cards now integrate unpatched. Only
     832 and 837 are here. The other five are the same process with other
     SMEFT points, and the 832 card with their points stands in. Show that
     `integrate --check` passes, and that one short integrate per family
     runs.
   - **Bit for bit.** σ and grid sha256 for main 0340a89 against your branch
     on ≥ 3 banked cards with default rank cuts. Build main in a separate
     target dir and delete it afterwards.

   First run one seed of each MadGraph and vibegraph job and extrapolate.
   Then ask the sprint session for approval of the sweep, with its total
   cost.

Ask the sprint session before every MadGraph run or seed sweep: K-M1 shares
this 4-core box. CI's `banked` job is the banked gate; don't run
`validate.sh` locally.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintk-runcard`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintk-runcard`). If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between completion
  notifications instead of idle-waiting. On resume after a stall, first check
  whether the interrupted command finished (`ps`, result artifacts) before
  re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
- **Disk is finite, and a full disk looks like a failure.** A debug workspace
  build with incremental data reaches ~15 GB per worktree, and two such trees
  have filled a cloud container mid-gate. On a shared or small host, run cargo
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). An `ENOSPC`, a linker
  `Bus error` or a truncated log is a kill, not a result: free only your own
  `target/` and re-run. Kill processes by PID, never `pkill -f`, which matches
  siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~24 GB free disk, shared with one or
two sibling sessions. It restarts without warning. Run every long job fully
detached (`nohup setsid`, not a chain shell), with per-seed output appended to
a file, so a restart keeps every finished seed and you rerun only the missing
ones. Run heavy jobs (seed sweeps, `validation/validate.sh`, a local MadGraph
run, a release build of the workspace) one at a time, and **ask the manager
before starting one**: a sibling may be running its own. Commit and push at
checkpoints (`git push -u origin sprintk-runcard`). Prefix logs `kc2-`.
- Cargo: always `export CARGO_TARGET_DIR=/home/user/wt/sprintk-runcard/target` plus the
  three lean variables above. Each worktree has its own target dir: a shared one
  ran a sibling's stale test binaries in sprint J.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `mg5amcnlo` submodule content) is already in your worktree, hard-linked:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under your scratchpad
  or `/home/user/wt/sprintk-runcard-mg/`, never into the main checkout.
- The network reaches the CMS model store
  (`https://cms-project-generators.web.cern.ch/cms-project-generators/`) and
  `raw.githubusercontent.com` (e.g. `cms-sw/genproductions` cards). It does not
  reach cvmfs. Put downloads in their own empty directory.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the global identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit. Never
open a PR or post to GitHub: the manager does. Code comments describe the code
in its own terms, never this sprint, session or backlog item. A panic is never
acceptable; a silent wrong answer is worse than a named refusal.

**Report** in `research/kb/sprints/v022-smeft-and-refusals/sessions/K-C2-report.md`
(a `Session Report` concept, as sprint J's `J-E1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
