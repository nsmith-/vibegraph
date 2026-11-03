---
type: Sprint
title: "integrate --param-card"
description: "integrate binds the model to a param card and records the moved parameters; generate takes no card and rebinds from the record; a moved-parameter σ matches MadGraph at the same card."
status: draft
active: true
tags: [cli, param-card, slha, artifact, cms, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, every clause pinned by a test or a
recorded measurement.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `cli-has-no-param-card-option`

Part of sprint E (param card, systematics, peripheral spine). The design is
settled in [run inputs in the artifact](../../pipeline/run-inputs-in-the-artifact.md)
§1, §3 and §6 and in [the integrate-generate split](../../pipeline/integrate-generate-split.md):
the param card feeds integration only.

## Decisions (user, 2026-10-28)

- **P1.** `DECAY <pdg> Auto` is refused at `integrate`. MadGraph computes such
  widths with MadWidth at launch and writes the numbers into the card, so a
  gridpack's final card has none. The refusal says to give the width (or use
  the card MadGraph wrote) and that computing Auto widths is not supported
  yet; it may become a feature.
- **P2.** A card value for a dependent (internal) parameter, such as MW in the
  SM, is ignored with a warning naming the computed value, as MadGraph's
  Fortran ignores it. Nothing is recorded for it.
- **P3.** `generate` writes the bound param card as MadGraph's `<slha>` header
  block, so Pythia reads the run's masses and widths.
- **P4.** A card that moves one parameter of a pair the restriction merged is
  refused.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P-F1 | feature-dev (Opus) | – | hermetic, fmt, clippy, kb-lint | – |
| P-V1 | validation-dev (Opus) | P-F1, M3 banking | moved-parameter σ, ≥5 seeds per side | the item |
| P-HY1 | hygiene-dev (Sonnet) | P-V1 | hermetic, fmt, clippy | – |
