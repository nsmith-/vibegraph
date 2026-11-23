---
type: Backlog Item
title: "gpdb records an external-UFO card's model as sm and keeps no copy of its UFO"
description: "Sixteen released cards (zprime_UFO ×7, HAHM ×7, LPstable_cascade ×2) cannot be re-staged, so the flavour prune's enumerate diff could not cover those three models."
area: validation
state: open
priority: medium
closes_when: "gpdb records each card's real model and stages its UFO (or a pinned fetch of it), the sixteen cards re-stage, and the PR 3b old-against-new enumerate diff is run on them."
blocked_by: []
opened: 2026-11-23
tags: [gpdb, campaign, external-ufo, enumerate, flavour-prune]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Sprint L's flavour prune (PR 3b) was checked by enumerating every released
gridpack card on the old and new builds and diffing the artifacts. Pass 1
covered 365 of 381 cards, all identical. The other 16 use external UFO
models: zprime_UFO ×7, HAHM ×7 and LPstable_cascade ×2. gpdb records their
model as `sm` and holds no UFO for them, so they could not be re-staged.

The prune is therefore unverified on those three models. Any later campaign
that re-runs released cards has the same gap.
