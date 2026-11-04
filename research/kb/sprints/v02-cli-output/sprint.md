---
type: Sprint
title: "v0.2 CLI output that misleads"
description: "The integrate summary prints σ with significant figures, χ²/dof as n/a over fewer than two iterations, and the scale choice instead of μF = 0 GeV; optional part three of sprint G, one PR."
status: draft
active: true
tags: [cli, output, artifact, sprint-g, v0.2]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
## Goal and exit criteria

The optional third part of sprint G (split explained in
`v02-refusals` (sprint folder on its own PR)), run if capacity allows. Done when
each claimed item's `closes_when` holds, pinned by CLI tests, and CI is green.

## Scope

- `iteration-chi2-printed-as-zero-with-one-result`
- `dynamical-scale-summary-prints-mu-f-zero`
- `sigma-printed-to-fixed-decimals` (filed by #85, merged)

These change the summary line and may change the artifact's format (an
undefined χ²/dof and an absent constant μF recorded distinctly from 0), so
the PR checks the artifact's version rules and goes last. It is stacked on
v02-refusals (#88): its base branch is `sprintg-refusals`, whose zero-σ
refusal and artifact format 18 it builds on. The CMS survey
database (#71) parses the summary line; its parser is updated in the same
change.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| G-HY3 | hygiene-dev (Opus) | stacked on `sprintg-refusals` (#88), which has G-F4 and G-F5 | hermetic, lints, CLI tests, `validation/cms_gridpacks` parser tests | all three |
