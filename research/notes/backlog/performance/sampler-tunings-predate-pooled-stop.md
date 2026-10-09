---
type: Backlog Item
title: Sampler tunings were chosen under a stop rule and a combination that have since changed
description: "The map auto rules, the --max-iters sizing and VEGAS_ALPHA_MAPPED = 0.5 were measured under the χ²-scaled stop, 1/σ² combination or per-diagram channels; none is re-measured on the current sampler."
area: performance
state: open
priority: low
closes_when: "MapOptions::resolve's ratios, the --max-iters headroom and the mapped VEGAS damping are re-measured under the pooled stop, the unweighted combination and merged channels (≥5 seeds per arm), and each kept or changed."
blocked_by: []
opened: 2026-10-09
tags: [vegas, stop-rule, map-choices, damping, re-measure]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: maps, resource: "../../../../vibegraph-lib/src/phasespace/maps.rs", title: "phasespace/maps.rs MapOptions::resolve doc (~:218-243), measured to a χ²-scaled 0.1%"}
  - {id: integrate, resource: "../../../../vibegraph-cli/src/integrate.rs", title: "vibegraph-cli/src/integrate.rs --max-iters (~:320-328)"}
  - {id: hadronic, resource: "../../../../vibegraph-lib/src/hadronic.rs", title: "hadronic.rs VEGAS_ALPHA_MAPPED rationale (~:71-87)"}
---
Three settings rest on measurements made under rules the sampler no longer uses:

- **Map auto rules.** `MapOptions::resolve`'s ratios (`phasespace/maps.rs`
  ~:218-243) are "evaluations the convergence stop needs to reach a χ²-scaled
  0.1%". The stop now widens by the pooled factor `max(1, emp/quoted)`
  (`budget.rs`, `pooled_scale`, from 403cff8), and channels are merged per
  MadGraph configuration.
- **`--max-iters`.** Its default is sized for "the widest gated process needing
  ~156 iterations" (`vibegraph-cli/src/integrate.rs` ~:320-328), counted under
  the old stop and before channel merging.
- **`VEGAS_ALPHA_MAPPED = 0.5`.** Its rationale (`hadronic.rs` ~:71-87) is that
  confident wrong iterations dominate "since iterations are combined by `1/σ²`".
  The default combination is now `IterationCombination::Unweighted`.

None of these changes a cross section beyond its error: they set cost and stop
behaviour. Re-measure before the next tuning decision rests on them. The stale
comments are listed in
[sampler-and-phase-space-doc-comments-stale](../hygiene/sampler-and-phase-space-doc-comments-stale.md);
the stop-rule item is
[stop-scale-inflated-by-few-accepted-iterations](stop-scale-inflated-by-few-accepted-iterations.md).
Found by drafter D7 and verifier V6.
