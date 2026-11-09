---
type: Session Brief
title: "I-R2: refuse a zero τ floor, honour mxx_only_part_antipart, end the run on a worker panic"
description: "Three small refusal and robustness fixes: a card with no ŝ floor is refused before any point, mxx_only_part_antipart's keys enter the per-PDG cuts as in MadGraph (or it is refused off default), and a rayon worker panic ends the CLI promptly with its message."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [zero-shat-floor-panics-with-nan-x, mxx-only-part-antipart-silently-ignored, rayon-worker-panic-leaves-process-alive]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
---
Read the three items first (`python3 scripts/kb.py backlog --item <slug>`) and
[the sprint overview](../sprint.md). A sibling session (I-R1) is editing
`RunCard::scattering_default` in `runcard.rs`; stay out of it, and keep your
`classes.rs` edits to the one row this needs.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprinti-r2`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprinti-r2`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `ir2-`.
pixi is not installed here: run gates as plain `cargo` and
`bash validation/validate.sh`, which fetches nothing (the `--skip-deps`
equivalent); the reference data is already copied into your worktree.

## Scope

1. **Zero τ floor.** `p p > b b~` under `sm-no_b_mass` at `ptb = 0` panics at
   the first PDF lookup with `x = NaN` (`pdf/extrap.rs:85`). Find MadGraph's
   check that refuses such a card (the item cites `setcuts.f:808`, which in the
   pinned source is the grouping trap: find the real one and quote it), and
   confirm with a local MadGraph `launch` of the same card. Refuse a card
   whose τ floor is zero with a typed error naming the uncut leg, at integrand
   construction, before any point is drawn. If the collinear guards a sibling
   session (I-C2) is adding would also catch this card, the τ-floor refusal is
   still needed: it is the one that cannot be reached by a NaN.
2. **`mxx_only_part_antipart`.** It is filed `IgnoredBenign` (`classes.rs:450`).
   In MadGraph its keys join `pdg_cut` (`banner.py:4708-4750`, each with the
   per-PDG defaults `0`/`-1`), and `setcuts.f:318-334` then replaces those legs'
   class cuts. H-C1 found the light-code guard at `banner.py:4722` never fires
   (the keys are strings), so `{'default': False, 22: True}` does strip `pta`
   from photons in MadEvent. Implement it by folding its keys into the
   per-PDG cut handling of `pt_min_pdg` (`cuts.rs`), with the same light-code
   handling `pt_min_pdg` already has, if that is cheap. Oracle: σ against
   local MadGraph on a finite card, e.g. `p p > e+ e-` with an `mmll` window
   and `{'default': False, 11: True}` (and the per-event lepton pT showing the
   class cut gone), ≥ 5 seeds a side. If it is not cheap, reclassify it
   `IgnoredPhysics` so that off its default it is refused, with a test.
3. **Worker panic.** An instrumented CERN-host run stayed alive about 40
   minutes after a rayon worker panicked. Explain why from the code
   (`vibegraph-cli/src/parallel.rs`, the integration and generation pools), and
   make a worker panic end the CLI within seconds, with the panic's message and
   a non-zero exit, in text and ldjson modes. Pin it with a CLI test driven by a
   test-only panic hook (an env var or `cfg(test)` feature that production
   builds cannot set by accident; say which and why).

## Gate

- The τ-floor refusal test on the item's card, failing (panicking) before the
  change.
- The `mxx_only_part_antipart` σ comparison, or the refusal test.
- The panic CLI test, both modes.
- `cargo test --workspace` and `bash validation/validate.sh` green, every
  banked line unchanged.

## Rules for this sprint

- Commits are authored by Nicholas Smith: `git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`, and end with exactly one trailer, `Assisted-by: claude-code:<your model id>`. Never `Co-authored-by` or `Signed-off-by`.
- Do not create, edit or delete backlog items, or anything else under `research/kb/`, unless your scope says so; the manager deletes the items your PR closes. New work goes in your report's **Found**.
- Comments describe the code in its own terms: never backlog items, sessions or sprints.
- Statistical claims: ≥ 5 seeds a side, unweighted mean, with the spread and χ²/dof (AGENTS.md, "Samplers gate statistically"). Every number in your report is a recorded measurement with its command beside it, on a stated commit.
- Lints are gates: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Correct this brief if its premise is wrong. Stop and report rather than build on a wrong premise.

## Report

Your final message is the report: the commits; each oracle with its command and
output; mutation evidence for each new test (it fails before the change); the
gates run, with their output; and a **Found** section. The manager records it
as this session's `-report.md`.
