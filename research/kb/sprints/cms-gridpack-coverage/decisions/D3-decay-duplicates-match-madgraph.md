---
type: Design Decision
title: "Sprint H: decay-chain duplicate subprocesses match MadGraph; per-group cut indicators come into scope"
description: "D3: a subprocess reached by two decay assignments is kept as two subprocesses summed incoherently, each with its own core cuts, as MadGraph does. D4: one cut indicator per flavour group is in sprint H's scope, since it blocks every hadronic decay chain with jets in the decay and 5F p p > j j j."
decided: 2026-11-04
decided_by: human:nsmith-
status: stable
tags: [decay-chains, cuts, flavour-groups, sprint-h]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
verified: [{by: "human:nsmith-", at: 2026-11-04}]
sources:
  - {id: hd1, resource: "../sessions/H-D1-report.md", title: "H-D1 report: decay-chain duplicates against MadGraph"}
---
The user's answers after H-D1's measurement.

- **D3. Item 5 matches MadGraph.** Both decay assignments of one flavour
  content are kept as separate subprocesses, summed incoherently. Each
  assignment cuts its own core particles under `cut_decays = F`. The
  interference left out is ≤ 1e-3 (H-D1). This differs from
  [identical particles across decays](../../../process/identical-particles-across-decays.md),
  where every pairing is kept. There, all pairings cut the same particles, so
  keeping them changes only the interference. Here the two assignments cut
  different quarks. A coherent subprocess would need a new cut rule:
  cutting all four quarks reads 23–26% below MadGraph, and cutting per
  assignment is IR-unsafe. The user asked whether this was the ZZ case
  again, and chose this option after that distinction was explained.
- **D4. Per-group cut indicators are in scope.** The sprint brief excluded
  them. H-D1 found that every hadronic decay chain with jets in the decay
  stops at `GroupCutsDiffer`, with or without a duplicate, so item 5 alone
  lifts none of its 11 gridpacks. Item 8 (5F `p p > j j j` at
  `maxjetflavor = 4`) needs the same capability.
