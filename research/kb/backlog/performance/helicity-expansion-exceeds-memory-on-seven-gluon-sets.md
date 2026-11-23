---
type: Backlog Item
title: "Binding the g g > 5g evaluator expands all 128 helicities into a 4.5–4.9 GB arena"
description: "The first bind of a g g > g g g g g evaluator materialises nodes × helicities at once and peaks near 5 GB; QCD-5Jets was killed at the campaign's 4.5 GB memory cap."
area: performance
state: open
priority: high
closes_when: "Peak RSS of `integrate --check` on `g g > g g g g g` is below 2 GB (measured), with |M|² bit for bit or within 1e-12 on the amplitude oracle and σ unchanged on the banked layer; the compile phase's memory on a `p p > 5j` card (all sets held) is measured."
blocked_by: []
opened: 2026-11-23
tags: [memory, helicity, evaluator, multijet, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Card 37 (QCD-5Jets_HT-40) was killed at 4.60 GB in the amplitude probe
under `MemoryMax 4500M`. Locally (dev, `-j 2`), compiling `g g > 5g` (2485
diagrams, 138 323 arena nodes, 128 helicities) runs at 70 MB for 66.7 s;
RSS then grows from the first bind in `prune_zero_helicities`
(`expand_helicities` over all helicities) to a 4.93 GB peak, 5.00 GB in the
compile phase.

Pruning keeps ~112 of 128, so pruning first is not the fix: evaluate
without materialising nodes × helicities at once (by helicity chunk, or an
unexpanded arena for wide sets). The compile phase holds every set's
evaluator, so this is needed whether or not the probe's duplicate compile
goes. Evidence: [L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
