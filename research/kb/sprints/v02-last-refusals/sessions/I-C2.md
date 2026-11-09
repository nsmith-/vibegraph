---
type: Session Brief
title: "I-C2: refuse every massless splitting that no cut regulates"
description: "Extend #109's photon–parton guard to every massless-pair collinear or soft limit (q → q g, g → g g, ℓ → ℓ γ, partons outside the jet classes) that no pT, pair or xqcut cut regulates, with a refusal test per class and every banked card still accepted."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [uncut-collinear-limits-beyond-photon-quark-unrefused]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
---
Read the item (`python3 scripts/kb.py backlog --item
uncut-collinear-limits-beyond-photon-quark-unrefused`), its sources
([H-G2](../../cms-gridpack-coverage/sessions/H-G2-report.md) Found 3,
[H-B1](../../cms-gridpack-coverage/sessions/H-B1-report.md) Found 1), and the
`cuts.rs` module doc's "Photon–parton collinear limit". The next session (I-C1)
restates the `cut_decays` exemption guard on top of what you build, so keep the
predicate "is this massless pair's limit regulated by the compiled cuts?"
reusable.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprinti-c2`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprinti-cuts`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `ic2-`.
pixi is not installed here: run gates as plain `cargo` and
`bash validation/validate.sh`, which fetches nothing (the `--skip-deps`
equivalent); the reference data is already copied into your worktree.

## Scope

1. **Which limits.** For each class, write down why it diverges and which cuts
   regulate it in MadGraph (single-leg pT, pair ΔR, invariant mass, `xqcut`
   under MLM, `ptheavy`…), from `cuts.f`/`setcuts.f`:
   - final-state `q → q g` and `g → g g` (two massless coloured outgoing legs
     at one vertex): `ptj = 0` and `drjj = 0` and no `mmjj` and no `xqcut`;
   - initial-state emission of a massless coloured leg with no pT cut
     (`ptj = 0` alone, if a soft/collinear pole off the beam is uncut);
   - `ℓ → ℓ γ` with a massless lepton at `dral = 0` (`e+ e- > mu+ mu- a` in the
     default SM);
   - a coloured parton above `maxjetflavor` that no class pair cut reaches;
   - whatever else the analysis turns up (say so; don't widen silently).
2. **The guard.** Generalise `photon_parton_vertices` /
   `refuse_uncut_photon_splitting` to every such pair, refusing with a typed
   error naming the legs and the cut that would regulate it. Keep
   `UncutPhotonSplitting`'s behaviour on its cards.
3. **Nothing banked is refused.** Run every banked card's cut compilation and
   show it is accepted; if a banked card is refused, stop and report: either the
   guard is wrong or the banked row is divergent, and that is the user's call.

## Gate

- One refusal test per class, each failing before the change (mutation
  evidence).
- Every banked card still accepted, every banked σ line unchanged
  (`bash validation/validate.sh`, after asking the manager).
- `cargo test --workspace` green.

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
