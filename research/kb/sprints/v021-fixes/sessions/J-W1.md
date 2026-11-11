---
type: Session Brief
title: "J-W1: the pairing-weighted cut_decays default and the per-pairing limit guard"
description: "With cut_decays = F, apply Σ_p w_p Θ_p |M|² with w_p = |M_p|²/Σ_q |M_q|², run the limit guard per pairing with its own exemptions, and record σ against local MadGraph with the difference attributed."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [cut-decays-identical-quarks-cut-a-stricter-observable]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
---
**Not dispatched until the user answers the flag question in
[the plan](../sprint.md) (PR 1).** This brief assumes option A: the weighted cut
is the only behaviour, and no `--madgraph-compat` flag is built.

Read the item (`python3 scripts/kb.py backlog --item
cut-decays-identical-quarks-cut-a-stricter-observable`),
[D2](../../v02-last-refusals/decisions/D2-cut-decays-keep-the-stricter-cut.md),
[D3](../../v02-last-refusals/decisions/D3-identical-decays-behind-madgraph-compat.md),
[I-C1's report](../../v02-last-refusals/sessions/I-C1-report.md) (its probe,
`ic1_window_probe.rs`, is in the report),
[H-C3's report](../../cms-gridpack-coverage/sessions/H-C3-report.md) Found 2,
and [identical particles across decays](../../../process/identical-particles-across-decays.md).
#113's limit guard lives in `cuts.rs`.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintj-w1`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintj-weighted-cut`). If you
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

This container restarts without warning. Run every long job fully detached
(`nohup setsid`, not a chain shell), with per-seed output appended to a file,
so a restart keeps every finished seed and you rerun only the missing ones.
Run heavy jobs (seed sweeps, `validation/validate.sh`, local MadGraph) one at a
time (4 cores), and ask the manager before starting one: a sibling session may
be running its own. Commit and push at checkpoints. Prefix logs `jw1-`.
All sessions share `CARGO_TARGET_DIR=/home/user/cargo-target` (export it in every
cargo command), so dependencies build once; a build may wait on another
session's cargo lock, which is expected. The reference data (`validation/madgraph/output`,
`validation/pdf` sets, the `mg5amcnlo` submodule) is already in your worktree.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the global identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit. Never
open a PR or post to GitHub: the manager does. Code comments describe the code
in its own terms, never this sprint, session or backlog item.

**Report** in `research/kb/sprints/v021-fixes/sessions/J-W1-report.md` (a
`Session Report` concept, as sprint I's reports): what changed, every oracle
with the command alongside its output, what each test provably cannot detect,
and a **Found** section for new work (never create or edit backlog items).
Correct this brief where it is wrong.

## Scope

1. **Per-pairing |M_p|².** Partition each subprocess's diagrams by pairing
   (which identical final-state legs the decay products are), and evaluate
   |M_p|² per pairing alongside the coherent |M|² at the same point. Say how
   the partition is read off the diagrams, and what it costs per point.
2. **The weighted cut.** With `cut_decays = F` and identical particles across
   a decay, the integrand is Σ_p w_p Θ_p |M|², w_p = |M_p|²/Σ_q |M_q|², where
   Θ_p exempts pairing p's decay products and cuts p's core legs. Every other
   card goes through the existing path unchanged: bit for bit, pinned by a test
   and by every banked σ line.
3. **The guard.** Run #113's limit check over each pairing's own diagrams with
   that pairing's exemptions. Add a refusal test on a multi-body decay with
   massless products where the weighted rule is unsafe, failing before the
   change.
4. **The probe.** I-C1's three configurations on the weighted integrand:
   w_p Θ_p |M|² ε² → 0 between ε = 1e-2 and 1e-4. Keep the unweighted
   control, still constant, so the probe can see a weight that does nothing.
5. **σ against MadGraph** (ask the manager before each heavy run). `g g > z u
   u~, z > u u~` (MadGraph 7.211 ± 0.028 pb, one seed; rerun it locally at ≥ 5
   seeds) and the full `g g > z j j, z > j j` card (MadGraph 150.61, 5 seeds).
   vibegraph ≥ 5 seeds a side, unweighted mean, spread and χ²/dof. The result
   is not expected to match. Attribute the difference: the pairing (~0.2%), the
   interference, and the exemption (weighted cut vs. MadGraph's
   configuration-1 exemption on one pairing).
6. **Docs.** `release-scope.md`'s "Deviating from MadGraph", the
   identical-particles concept, the `cut_decays` text in `cuts.rs`, and site
   (d) of `madgraph-compat-sites-unconditional` (report the wording change; the
   manager edits the item).
