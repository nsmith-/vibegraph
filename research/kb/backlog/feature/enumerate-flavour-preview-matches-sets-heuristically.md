---
type: Backlog Item
title: "enumerate's flavour-group preview enumerates again and matches groups to sets by heuristic"
description: "enumerate's flavour-group preview enumerates the card a second time and matches groups to sets heuristically, which is ambiguous for decay chains."
area: feature
state: open
priority: low
closes_when: "The preview builds groups from the already-enumerated sets (multiplicity_groups_of) and maps members back through FlavorGroup's recorded member origins, and a decay-chain card with two assignments reaching one subprocess is tested."
blocked_by: []
opened: 2026-11-10
tags: [enumerate, flavour-groups, preview]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr120, resource: "https://github.com/nsmith-/vibegraph/pull/120", title: "vibegraph enumerate (found there)"}
  - {id: pr121, resource: "https://github.com/nsmith-/vibegraph/pull/121", title: "multiplicity_groups_of"}
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "FlavorGroup member origins"}
---
Found when `enumerate --run-card` was built (nsmith-/vibegraph#120). Since
then #121 added `multiplicity_groups_of(sets, …)`, which takes enumerated
sets, and #124 added `FlavorGroup::member_origins`, which finds each member's
enumerated set by digest. Once those land, the preview should use both: one
enumeration instead of two (about 5 s saved on `p p > e+ e- j j j`), and no
heuristic matching.
