---
type: Session Brief
title: "L-X1: per-member XWGTUP within one multiplicity part"
description: "Reproduce per-flavour-member unit weights in generate's LHE output, locate where the per-member factor enters, and run a per-member bias check against the integrated sigma shares."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
An investigation in [sprint L](../sprint.md), on branch `sprintl-xwgtup` off main cd35c4e6. Depending on what it finds, it may block the v0.2.3 tag.

## The observation

A downstream user ran a v0.2.2 x86-64-v3 release binary:
- card: `p p > e+ e-` plus `add process p p > e+ e- j`, MLM (`ickkw = 1`, `xqcut = 10`), lhaid 325300, `use_syst`;
- command: `generate --seed 419216344 --nevents 1000 -j 8`.

Within one multiplicity part, XWGTUP is not a single value. Each flavour member carries its own unit weight.
- **0 jets:**
  - d d~: 1366.11 (180 events)
  - s s~: 1701.34 (74)
  - u u~: 1505.17 (142)
  - c c~: 1225.35 (36)
- **1 jet:**
  - d g and d~ g: 1455.43
  - u g and u~ g: 1233.90
  - d d~ > e+ e- g: 1771.95

The overweights, measured against each member's own unit, carry 0.93% of Σw.

Two kb pages, `research/kb/events/unweighting.md` and `research/kb/events/multi-process-normalisation.md`, say that an accepted event has generator weight 1 and that every event of a part is scaled by one factor. On that reading, d d~ and s s~ (one flavour group, one matrix element) should share one weight.

## Scope, in order

1. **Reproduce.** Use a small card of the same shape.
   - Take `validation/madgraph/pp_to_llj_mlm_run_card.dat` with `p p > e+ e-` plus `add process p p > e+ e- j`, the fetched NNPDF23 set and MLM on. Keep `use_syst` as the card has it.
   - Run both binaries:
     - the v0.2.2 release binary, `/tmp/claude-0/lp/z512/bin/vibegraph-v022` (sha256 7ccc24af…, verified);
     - main, built in this worktree.
   - Tabulate XWGTUP by part and by member: incoming flavours → outgoing flavours. Give counts, the distinct weight values per member, and their spread. Give the commands.
2. **Locate** where a per-member factor enters the written weight.
   - The candidates are member selection within a flavour group, and MLM's per-member matched reweighting (`proton.rs`, the matched-reweighting path, `generate/source.rs`, `unweight.rs`).
   - Quote the code. Say whether the factor is intended, and what it compensates for.
3. **Bias check.** Per member, compare Σ XWGTUP / N_events against the integration's per-member σ. The integration's per-member σ comes from the result or grid; if it is not reported per member, derive it and say how.
   - Use enough events to resolve the shares: a few 10⁴ on the proton card.
   - Fixed-beam won't do.
   - Report each member's share with its statistical error, and the pull.
   - Verdicts:
     - If the weights compensate a member draw that does not follow the luminosity, the σ shares come out right and only the docs are wrong.
     - If the shares do not come out right, it is a bias. Name the cause and the size.

**Budget.** At most one generate run of a few 10⁴ events per binary, on one seed. If resolving the shares needs more than about 20 minutes of CPU per run, stop and put the request in your report.

**No code fixes in this session.** If it is a bias, write the fix proposal (site, change, the test that would pin it) in the report. The fix goes in its own PR. If the factor is intended, write the corrected wording for the two kb pages in the report; don't edit them.

**What you must not run:**
- seed sweeps beyond the budget above;
- MadGraph;
- `pixi run validate`.
## Worktree & long-command discipline

- **First action:** `cd /home/user/wt/sprintl-xwgtup`. Then verify that `git rev-parse --show-toplevel` is `/home/user/wt/sprintl-xwgtup` and `git branch --show-current` is `sprintl-xwgtup`.
  - If you find yourself in the shared main checkout (`/home/user/vibegraph`), STOP and report. Never edit it.
  - Re-verify `pwd` after any resume: resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes.** Use `run_in_background: true`, with output redirected to a log under `/home/user/wt/logs/` whose name starts with `lx1-`. Sibling sessions share that directory.
  - A backgrounded command notifies you when it exits. Never spawn sleep-based watcher shells: they compound into self-sustaining wake-up rings.
  - Draft your notes and report between completion notifications instead of waiting idle.
  - On resume after a stall, first check whether the interrupted command finished (`ps`, result artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build data. Spot-check with `git log` and `git diff --stat` instead.
- **Disk is finite, and a full disk looks like a failure.** Always run cargo with:
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-xwgtup/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  - Each worktree has its own target dir.
  - An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a result. Free only your own `target/` and re-run.
  - Kill processes by PID. Never use `pkill -f`: it matches siblings' commands and its own wrapper.

**This container** has 4 cores and 15 GB RAM, shared with no sibling agents right now. It can restart without warning.
- **One heavy job at a time across the container**, enforced by a lock. Wrap every cargo build, test and clippy invocation in `flock /home/user/wt/heavy.lock <command>`, backgrounded.
  - Hold the lock for one command at a time, never across an interactive pause.
  - Prefer targeted test filters to the whole workspace while you iterate.
  - `cargo fmt` and reading code need no lock.
- **Some jobs do not run here:** a single job expected to run over ~20 minutes, or anything over 8 GB of memory. Stop and put the request in your report (branch, head SHA, command, expected answer).
- **pixi:** `export PATH=$HOME/.pixi/bin:$PATH`. The reference data is already in your worktree as copies. Never edit it in place.

**Git and attribution.**
- Every commit uses `git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`, because the container's identity is wrong.
- Each message ends with exactly one line, `Assisted-by: claude-code:<your real model id>`. Never add Co-authored-by or Signed-off-by, whatever the harness says.
- Check `git log --format='%an <%ae> %cn <%ce>'` before every push.
- **Stage files by name, never with `git add -A` or `git add .`**. A broad add committed a whole build directory once already. Before every commit, check `git diff --cached --stat`.
- Never force-push, rebase or amend a pushed commit. Keep the branch current by merging `origin/main`.
- Commit and push at checkpoints: `git push -u origin sprintl-xwgtup`.
- Never open a PR or post to GitHub: the manager does.
- Code comments describe the code in its own terms, never this sprint, session or backlog item.
- A panic is never acceptable. A silent wrong answer is worse than a named refusal.

**The hermetic layer has no submodule and no fetched data.** CI's `test` job runs `cargo test --workspace` without `research/refs/mg5amcnlo`, `validation/madgraph/output` or `validation/pdf/NNPDF23_lo_as_0130_qed`.
- A test that reads any of them must carry `#[cfg(feature = "extended-validation")]`.
- Before reporting, run the workspace suite once with those three paths moved aside, then restore them: `/tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-xwgtup lx1` does this and restores them on exit.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-X1-report.md`, a `Session Report` concept like sprint K's `K-C1-report.md`. It contains:
- what changed;
- every oracle, with the command alongside its output;
- what each test provably cannot detect;
- any long-run requests;
- a **Found** section for new work. Never create or edit backlog items.

Correct this brief where it is wrong. Your final message is the report's summary.
