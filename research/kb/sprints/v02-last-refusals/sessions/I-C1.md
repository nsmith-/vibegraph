---
type: Session Brief
title: "I-C1: the cut_decays window rule, and the exemption guard over the exempting pairing"
description: "With cut_decays = F, exempt per point the pairing whose forced lines are inside the window (closest to the pole if several), restate refuse_ir_unsafe_exemption over that pairing and every massless gauge-charged leg, and gate σ against local MadGraph."
status: draft
agent: validation-dev (Opus)
depends_on: [I-C2]
closes: [cut-decays-identical-quarks-cut-a-stricter-observable, ir-unsafe-exemption-guard-sees-coloured-legs-only]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
---
**User decision: bring the window fix forward into v0.2.** Read both items
(`python3 scripts/kb.py backlog --item <slug>`),
[D5](../../cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md),
[H-C2](../../cms-gridpack-coverage/sessions/H-C2-report.md) (why the
configuration-1 exemption diverges on an every-pairing |M|²),
[H-C3](../../cms-gridpack-coverage/sessions/H-C3-report.md) Found 1 and 2,
[H-K1](../../cms-gridpack-coverage/sessions/H-K1-report.md) §5b, and
[identical particles across decays](../../../process/identical-particles-across-decays.md).
Then read I-C2's report: its regulated-limit predicate is what the guard
builds on.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprinti-c1`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprinti-c1`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `ic1-`.
pixi is not installed here: run gates as plain `cargo` and
`bash validation/validate.sh`, which fetches nothing (the `--skip-deps`
equivalent); the reference data is already copied into your worktree.

## Scope

1. **Every `from_decay` reader first.** List each place `setcuts.f` and
   `cuts.f` read `from_decay`/`do_cuts` and what each switches off, so the
   exemption switches off exactly those cuts (the `cuts.rs` module doc says
   `ptll` and `mmnl` stay on).
2. **The rule.** At each point, among the pairings (the kept diagrams' forced
   leg sets), exempt the decay products of the one whose forced lines are all
   inside the window |m − M| < bwcutoff·Γ, the one closest to the pole when
   several are. Define "closest" for a pairing with several forced lines and
   justify it (it moves σ only in the interference region). The exemption
   changes per point, so it cannot use the fixed `uncut_products` mask; keep the
   distinct-flavour path (one pairing) bit for bit, and say how you know.
3. **IR safety.** A pair containing a soft quark has m → 0 and is never in the
   window: prove it holds for every exempted leg, and pin with a test that a
   soft quark is never exempted.
4. **The guard.** `refuse_ir_unsafe_exemption` ORs over every kept diagram and
   would refuse this rule (H-C3 Found 2). Restate it over the exempting
   pairing's own diagrams, and cover every massless leg with a gauge charge
   (photon, massless lepton), not just coloured ones, using I-C2's predicate.
   Test: a card that would exempt a massless colourless core leg is refused.
5. **Docs.** Update the `cuts.rs` module doc. If the registered deviation's
   text changes, update `research/kb/pipeline/release-scope.md` ("Deviating
   from MadGraph"), `research/kb/process/identical-particles-across-decays.md`
   and the `madgraph-defects` row to match; the manager updates D5's record.

## Gate

- σ against local MadGraph (`python3 research/refs/mg5amcnlo/bin/mg5_aMC` from
  a copy in your scratch; `nn23lo1`, vibegraph with `NNPDF23_lo_as_0130_qed`),
  ≥ 5 seeds a side, on `g g > z u u~, z > u u~` (MadGraph 7.211 ± 0.028 pb at
  one seed, vibegraph 5.348 today) and on `g g > z j j, z > j j` (MadGraph
  150.61 at 5 seeds, vibegraph −6.8% today), at `cut_decays = F`, with the
  `cut_decays = T` rows as the control. Agreement is judged against the
  interference scale (about 0.2%). Use H-K1's cards (§5b) so the old numbers
  are comparable. Ask the manager before each sweep.
- Bit for bit on the banked decay-chain cards with no identical particles across
  decays, and every banked line unchanged.
- The soft-quark test and the guard test, each failing before the change.
- `cargo test --workspace` and `bash validation/validate.sh` green.

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
