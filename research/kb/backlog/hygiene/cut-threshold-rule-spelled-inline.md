---
type: Backlog Item
title: "The cut default-off rule is spelled inline at every threshold in cuts.rs"
description: "'A maximum is read only when ≥ 0, a minimum only when > 0' is written out at about eleven sites in cuts.rs: HT, single-leg, pair, mmnl, ŝ window and pair_mass checks."
area: hygiene
state: open
priority: low
closes_when: "One threshold helper in cuts.rs (a Threshold { min, max } or fails(x, min, max)) carries the rule, every cut family uses it, and the compiled Cuts values and edge tests are unchanged."
blocked_by: []
opened: 2026-10-26
tags: [cuts, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/ht-cuts/sessions/HY1-report.md", title: "ht-cuts HY1 report, Found 1"}
---
Sites named by the [ht-cuts review](../../sprints/ht-cuts/sessions/HY1-report.md)
(H7), at its line numbers: `ht_cut` :574 and `ht_ok` :1052, :1056; the
single-leg `pt`/`e`/`eta` checks :963, :971, :975 and `admits_every_point`
:930-934; `PairCut::is_active` :313-317 and the pair checks :987-1007;
`mmnl_cut` :550 and its check :1027; the ŝ window :942-951; `pair_mass`
:1456-1461. `vibegraph-lib/src/cuts.rs` on `main` has 19 `>= 0.0` and 16
`> 0.0` comparisons.

The HT session followed the module's existing spelling on purpose; merging HT's
alone would have made it the odd family. An identity oracle on the compiled
`Cuts` (its `Debug` output over the edge tests' cards) fits the refactor.
