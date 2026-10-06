---
type: Caveat
title: MadGraph pixi tasks regenerate any missing run directory
description: Tasks that depend on build-diagrams silently re-run MadGraph for every card whose output directory is missing, unless invoked with --skip-deps.
tags: [pixi, madgraph, tooling, skip-deps, worktrees]
status: draft
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L562-L565", title: "TODO.md entry T045"}
---
In `pixi.toml`, `extract-diagrams`, `extract-configs`, `extract-sigma`,
`build-amplitude`, `validate-color-cf`, `validate-unweighting`, `validate-lhef` and
others declare `depends-on = ["build-diagrams"]`. `build-diagrams` is
`validation/madgraph/build.sh`. It skips only scripts whose output directory
already exists, and **regenerates every one that is missing**, printing a
"Skipping" line for the others and nothing louder.

Consequences for a reader or an agent:
- A run directory moved aside (to test a fetched-only work area, or to hold out a
  run) comes back as a fresh MadGraph job, possibly hours long, the next time any
  dependent task runs without `--skip-deps`. With a held-out run, use only
  `pixi run --skip-deps <task>`.
- A fresh worktree without `validation/madgraph/output` copied in has the same
  problem (see `AGENTS.md`, "Own the worktrees").
- A regenerated multi-group run such as `pp_to_jj` is a different valid event
  sample, not the banked bytes ([note 28](../28-kt-spine-feature-sprint-plan.md) C.5).

The `extended-validation` skill explains `--skip-deps` in terms of
stale references, but it does not mention this missing-directory trigger.
