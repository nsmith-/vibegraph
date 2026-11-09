---
type: Session Brief
title: "I-R1: the absent-run-card defaults follow create_default_for_process; an untabulated parton is refused"
description: "Reproduce every field banner.py's create_default_for_process changes for a scattering process (MLM matching on mixed jet multiplicities, lepton, e p, neutrino and EVA beams) in RunCard::scattering_default, and refuse any initial-state parton the PDF does not tabulate."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [absent-run-card-mixed-multiplicity-unmatched, absent-run-card-lepton-beams-panic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
---
Read the two items first (`python3 scripts/kb.py backlog --item <slug>`) and
[the sprint overview](../sprint.md), "What is known before the sprint". The
user's decision for the mixed-multiplicity item is **match MadGraph**.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprinti-r1`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprinti-runcard`). If you
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
be running its own. Commit and push at checkpoints. Prefix logs `ir1-`.
pixi is not installed here: run gates as plain `cargo` and
`bash validation/validate.sh`, which fetches nothing (the `--skip-deps`
equivalent); the reference data is already copied into your worktree.

## Scope

1. **Read `create_default_for_process`** (`research/refs/mg5amcnlo/madgraph/various/banner.py:4767-5060`)
   and list every field it changes for a process with two incoming particles,
   with the condition for each. For each field, either reproduce it in
   `RunCard::scattering_default` (`vibegraph-lib/src/runcard.rs`, next to #101's
   `maxjetflavor` rule), or show that the value it writes is then refused by an
   existing rule (e.g. `UnsupportedLpp` for `e p` or EVA beams, a `polbeam`
   refusal for neutrino beams), so the user sees a named refusal, never a
   default-card run MadGraph would not do. `sde_strategy` and the display-only
   blocks need an explicit argument either way. Note how `matching` is decided:
   on `proc[0]`'s legs of each process definition, with `j` expanded, and the
   `MLM` model limitation that turns it back off.
2. **Mixed multiplicity.** With no run card, `p p > e+ e-` plus
   `add process p p > e+ e- j` gets `ickkw = 1`, `xqcut = 30`, `drjj = 0`,
   `drjl = 0`, `sys_alpsfact`, the `--alps=0.5,1,2` systematics argument and
   `dynamical_scale_choice = -1`, exactly as MadGraph's written card. The
   matched defaults must then pass `runcard::matching::resolve` (a resolved
   `ptj` below `xqcut` is refused today: check what MadGraph's default card does
   with `ptj` there, and report it rather than paper over it).
3. **Beams.** `e+ e- > mu+ mu-` with no run card gets `lpp = 0`,
   `ebeam = 500`, `use_syst = F`. Check `e- p > e- j`, `ve p > e- j`, and a W/Z
   beam card for what MadGraph writes and what vibegraph then does.
4. **Untabulated partons.** Any initial-state parton the PDF does not tabulate
   (`vibegraph-lib/src/proton.rs`, the `panic!` near :1074, "is not a tabulated
   parton density") is a typed refusal at integrand construction, before any
   point: e.g. `e+ e- > mu+ mu-` with an explicit `lpp = 1` card.

## Gate

- A test pins `ickkw`/`xqcut`/`drjl`/`drjj`/the systematics arguments of the
  mixed card, and `lpp`/`ebeam`/`use_syst` of the lepton card, against the
  `run_card.dat` local MadGraph's `output` writes (`python3
  research/refs/mg5amcnlo/bin/mg5_aMC`; run it from a copy of the submodule in
  your scratch directory, never inside the shared checkout). Commit those two
  written cards as test fixtures, trimmed if large.
- A σ row with **no run card** on the mixed card against local MadGraph
  `launch` at its default card (`pdlabel = nn23lo1`; vibegraph with
  `NNPDF23_lo_as_0130_qed` from `validation/pdf`), ≥ 5 seeds a side. Ask the
  manager before starting the sweeps.
- A refusal test for the untabulated parton that panics before the change.
- `cargo test --workspace` and `bash validation/validate.sh` green, and
  every banked line unchanged (no banked card is run without a run card: say
  whether that is so).

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
