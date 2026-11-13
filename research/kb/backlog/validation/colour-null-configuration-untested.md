---
type: Backlog Item
title: "No test exercises a configuration whose diagrams reach no colour flow"
description: "Ranking ids are found from diagram content because config_members() drops colour-null diagrams; no test card has such a configuration, so that path is unverified."
area: validation
state: open
priority: low
closes_when: "A machine search over small SM cards finds one where config_members() differs from config_groups, and the ranking test covers it (ids match diagrams.json, colour_null reported)."
blocked_by: []
opened: 2026-11-10
tags: [ranking, colour, configurations, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr126, resource: "https://github.com/nsmith-/vibegraph/pull/126", title: "inspect --ranking (found there)"}
---
`inspect --ranking` (nsmith-/vibegraph#126) locates each banked share's
configuration as the `config_groups` group holding its diagrams, rather than
by the evaluator's AMP2 index, because `config_members()` drops diagrams that
no colour flow references: where a whole group is colour-null the two
numberings differ. None of the test cards has such a group, so the id mapping
in that case and the `colour_null` unranked entry are unverified. Find a card
by machine search, not by hand.
