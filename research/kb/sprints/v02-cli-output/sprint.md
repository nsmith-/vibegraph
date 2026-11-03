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
- `sigma-printed-to-fixed-decimals`, claimed once PR #85 (which files it) merges

These change the summary line and may change the artifact's format (an
undefined χ²/dof and an absent constant μF recorded distinctly from 0), so
the PR checks the artifact's version rules and goes last, after v02-refusals,
whose zero-σ refusal edits the same end of `integrate`. The CMS survey
database (#71) parses the summary line; its parser is updated in the same
change.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| G-HY3 | hygiene-dev (Sonnet) | v02-refusals' G-F4 merged into main; #85 merged | hermetic, lints, CLI tests, `validation/cms_gridpacks` parser tests | all three |
