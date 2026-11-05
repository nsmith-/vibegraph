---
type: Sprint
title: "v0.2 output modes: plain, rich and ldjson, and vibegraph inspect"
description: "One output mode per run formats logs, progress and the result or refusal together; --output-ldjson gives a versioned machine-readable stream; vibegraph inspect reads a saved artifact; the survey reads ldjson. Part four of sprint G."
status: draft
active: false
tags: [cli, output, ldjson, tui, tracing, artifact, cms-survey, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
## Goal and exit criteria

The fourth part of sprint G, queued by the user after #90. Done when
`integrate-output-not-machine-readable`'s `closes_when` holds, every clause
pinned by a CLI test, and CI is green. The design is
[CLI output modes](../../tooling/cli-output-modes.md).

## Scope

- `integrate-output-not-machine-readable`

**Stacked on #90.** The final object carries χ²/dof-or-null, the named μF and
artifact format 19, which exist only on `sprintg-cli-output` until #90
merges. The branch starts from it (82d9d90, which has main through cbfaec6)
and brings main in, retargeting the PR to `main`, once #90 merges.

## Decisions (user, 2026-11-04)

- **G10. A small stable error kind.** `CliError` gains a kind from a short,
  versioned list (for example usage, card, model, process, support,
  artifact, io, network), set where each error is built. The ldjson refusal's
  `error_kind` is that name; the message stays the human text.
- **G11. Per-multiplicity totals are recorded in the artifact.** `integrate`
  writes the totals it already computes, so `inspect` and the ldjson final
  object read the numbers `integrate` printed. This bumps the artifact
  format. #89 also bumps it; whichever merges second renumbers.
- **G12. `generate` emits ldjson too; `check-events` refuses the flag.**
  `generate`'s final object carries the events written, the output path, and
  the sample σ with its pull against the integration, or a refusal.
  `check-events` refuses `--output-ldjson` with a usage error.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| G-F6 | feature-dev (Opus) | – | hermetic, lints, CLI schema tests, banked CLI targets, survey parser tests | integrate-output-not-machine-readable |
| G-HY5 | hygiene-dev (Opus) | G-F6 | hermetic, lints | – |
