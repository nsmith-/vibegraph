---
type: Backlog Item
title: "Channel members pairing a group member's configurations to the representative's is untested for charge-conjugate members"
description: "A flavour-group member whose configurations pair with the representative's neither by partition nor by tag gets no diagrams; charge-conjugate pairings are untested."
area: validation
state: open
priority: low
closes_when: "A test integrates a card whose flavour group joins charge-conjugate subprocesses (e.g. u d > ... with u~ d~ > ...) and asserts every channel member has diagrams and a share."
blocked_by: []
opened: 2026-11-10
tags: [channels, flavour-groups, configurations, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares (found there)"}
---
Found while building channel membership (nsmith-/vibegraph#124). The member
configurations are paired with the representative's first by `config_groups`
partition, then by `config_tag`; when neither matches, the member is recorded
with no diagrams. No row in the configuration-share oracle hits it (asserted),
but those rows have no charge-conjugate pairing, which is where the colour
bookkeeping is reindexed ([flavour groups](../../hadronic/flavour-groups.md)).
