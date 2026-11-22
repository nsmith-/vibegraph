---
type: Backlog Item
title: "An amplitude-order == or > does not set the squared order MadGraph implies"
description: "MadGraph derives a squared-order default from an amplitude-order constraint like p p > t t~ QED==0; vibegraph's run-card defaults miss it, a silent difference from MadGraph."
area: feature
state: open
priority: high
closes_when: "The squared-order defaults MadGraph derives from amplitude-order == and > constraints are reproduced, or the case is refused by name; a test pins a card such as p p > t t~ QED==0 against MadGraph's derived orders."
blocked_by: []
opened: 2026-11-22
tags: [squared-orders, coupling-orders, madgraph-parity, silent-difference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: pr1, resource: "https://github.com/nsmith-/vibegraph/pull/168", title: "Sprint L PR 1 (squared orders), Found (b)"}
---
Reported by sprint L's PR 1 agent: for a process line with an amplitude-order
`==` or `>` constraint, MadGraph's `banner.py` (about L4972–4988 at the pin)
derives a squared-order default that vibegraph does not. The difference is
pre-existing and silent: the card runs, with a different order selection.

First confirm the claim against the pinned source and one MadGraph process
directory, then decide between reproducing the default and refusing by name. A
silent difference is worse than a refusal.
