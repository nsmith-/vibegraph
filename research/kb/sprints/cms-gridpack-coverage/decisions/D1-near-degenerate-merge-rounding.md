---
type: Design Decision
title: "Sprint H: near-degenerate flavour groups merge at rounding, split when physical; decay-chain duplicates decided after MadGraph is measured"
description: "D1: GROUP_REL_TOL rises to about 1e-9 so rounding-level pairs merge, and pairs above it (the CP-odd 8e-7 γ+jets split) are separate groups instead of refused. D2: item 5's behaviour is chosen after MadGraph's own is measured."
decided: 2026-11-04
decided_by: human:nsmith-
status: stable
tags: [flavour-groups, ckm, decay-chains, sprint-h]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
verified: [{by: "human:nsmith-", at: 2026-11-04}]
sources:
  - {id: log, resource: "../log.md", title: "cms-gridpack-coverage log, 2026-11-04: the user's answers"}
---
The user's answers to the sprint's two questions.

- **D1. Near-degenerate groups (item 2).** Not the recommended option. Raise
  `GROUP_REL_TOL` to about 1e-9, so a pair separated only by rounding (the
  1.5e-10 VBS_WPMZTo4Q pair) merges, and treat groups separated by more than
  that as separate rather than refusing them (the 8e-7 γ+jets `u b~` pair and
  its conjugate). The change moves the merge tolerance for every process, so
  the session measures the worst rounding-level separation on the banked
  processes and sets the new value with headroom over it. The physical split is
  pinned by a test showing that it vanishes when the CKM phase or the W width is
  zero.
- **D2. Decay-chain duplicate subprocess (item 5).** Decide after MadGraph is
  measured. The session first establishes, from the pinned source and from an
  M3 `output` of the survey cards, how MadGraph treats one subprocess reached by
  two decay assignments. It also measures how large the interference between
  the assignments is. The user then chooses between matching MadGraph and a
  coherent sum registered as a deviation.
