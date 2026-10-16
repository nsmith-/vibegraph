---
type: Backlog Item
title: "MadEvent generators bank an empty or non-numeric results.dat"
description: "mes_run_seed's callers outside the seeded σ driver accept a run that exits 0 with an empty results.dat, and cache that empty value under the card hash."
area: validation
state: open
priority: medium
closes_when: "Every generator that sources madevent_seeds.sh refuses a run whose results.dat first line is not a positive finite σ (and, for event runs, an event file short of nevents), so nothing empty is cached."
blocked_by: []
opened: 2026-10-16
tags: [validation, madgraph, refdata, generators]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
`bank_seeded_sigma.sh` runs every seed through `mes_check_run` (positive finite
σ, a complete event file of at least `nevents` events). The other callers of
`mes_run_seed` do not:

- `gen_decay_widths.sh`, `gen_decay_chain_*.sh`, `gen_onshell_veto.sh`,
  `gen_grammar_sigma.sh` and `gen_mlm_references.sh` accept a run that exits 0
  with an empty `results.dat`. `mes_run_seed` records the empty value with the
  card hash, and every later call reads it back from the cache.
- `gen_pta_windows.sh` `run_one`, run on its own stages, refuses only a missing
  or empty `results.dat`, not a non-numeric σ or a short event file.

To resolve: have `mes_run_seed` refuse a non-numeric or non-positive first line,
and call `mes_check_run` where a generator keeps events. The event-count half
does not apply to decay widths. Found in the
[F2 report](../../sprints/sigma-reference-scatter/sessions/F2-report.md) and the
[HY1 report](../../sprints/sigma-reference-scatter/sessions/HY1-report.md).
