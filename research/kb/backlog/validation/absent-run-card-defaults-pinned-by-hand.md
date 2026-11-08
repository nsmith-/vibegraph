---
type: Backlog Item
title: "The absent-run-card defaults are pinned against hand-read MadGraph output, not a committed dump"
description: "Tests pin the no-run-card defaults to values read by hand; dumping create_default_for_process's written card for a few processes would catch a MadGraph bump, and census its sde_strategy rule."
area: validation
state: open
priority: low
closes_when: "dump_runcard_defaults.py also dumps the card create_default_for_process writes for 5F/4F p p > j j, p p > w+, e+ e- > mu+ mu- and a mixed multiplicity, a test compares vibegraph's absent-card default with it, and the survey's written sde_strategy values are censused."
blocked_by: []
opened: 2026-11-07
tags: [run-card, defaults, madgraph-parity, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hj1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-J1-report.md", title: "H-J1 report, Found 3 and 4"}
---
Found by H-J1, which fixed the absent-card `maxjetflavor` rule.

- **Hand-read pins** (Found 4). `validation/madgraph/dump_runcard_defaults.py`
  dumps the template and the decay card, but not the card
  `create_default_for_process` writes for a given process. Dumping it for a few
  process definitions would make a MadGraph bump that changes a rule fail a test.
- **`sde_strategy` unverified** (Found 3). `banner.py:4994–5014` moves
  `sde_strategy` to 2 for single-colour-flow processes outside two exceptions,
  and vibegraph consumes it (configuration weights, the clustering scale source).
  `p p > w+ w-` writes 1, as vibegraph's template; a card that gets 2 was not
  found. A census of MadGraph's written value over the survey's cards settles it.

Sprint I closed the two differences H-J1 found (the lepton beams and the
mixed-multiplicity matching rule). It committed three written cards as test
fixtures (`vibegraph-lib/tests/data/mg_default_run_cards/`: mixed
multiplicity, `e+ e- > mu+ mu-`, and an `MLM`-limited SMEFT card) and pinned
`sde_strategy` on eight processes against local MadGraph `output`. Still open:
the dump script for the other process definitions, and the census over the
survey's cards.
