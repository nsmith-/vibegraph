---
type: Sprint
title: "Peripheral spines without a single-leg pT cut"
description: "A final state of more than two legs gets its peripheral spine when the card has no single-leg pT cut: the pole floor always on from a default scale, the support bound only from a provable pT floor."
status: draft
active: true
tags: [phase-space, spine, t-channel, vbf, cms, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, every clause pinned by a test or a
recorded measurement.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `three-body-spine-requires-fiducial-scale` (rewritten in #80; until #80
  merges, read it from `origin/claude/fervent-mayer-2hv310`)

Part of sprint E (param card, systematics, this item), added by the user
through the planner on 2026-10-28. The evidence is the CERN host's VBF/EWK
Drell-Yan sweep, PR #71 comment 6438701885.

## Decisions (user, 2026-10-28)

- **T1.** The fallback scale is a fixed default, used only as the pole floor
  and tuned by measured variance. It never bounds the support: only a
  provable single-leg pT floor (`Cuts::spacelike_floor`) does that, as now.
  A scale read off `mmjj` or other cuts is not a provable transfer bound.
- **T2.** The spine is always built, with that pole floor. The item's
  narrower rule ("unless the emitted side is composite") is the fallback only
  if the bias check fails on composite ladders.
- **T3.** The 5-seed ladder on the three CMS cards runs in a sprint session
  in this container. The proc and run cards come from genproductions over
  HTTPS, and the model is `loop_sm` with the gridpacks' `ckm_no_b_mass`
  restriction. MadGraph's σ is 13.12, 7.574 and 117.6 pb (the sweep).

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| T-F1 | feature-dev (Opus) | – | hermetic, diagram_channel, fmt, clippy, kb-lint; bias and variance measured | – |
| T-V1 | validation-dev (Opus) | T-F1 | banked σ (≥5 seeds); the CMS ladder | the item |
| T-HY1 | hygiene-dev (Sonnet) | T-V1 | hermetic, fmt, clippy | – |
