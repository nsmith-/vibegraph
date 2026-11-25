---
type: Backlog Item
title: "The standalone harness cannot bank or compare a squared-order split, so no gate checks a signed interference against MadGraph"
description: "bank() assumes the unsplit MATRIX and standalone_jamps.rs compares only unsplit flows; run_flows panics on a split evaluator. A wrong |M2|² or interference sign passed every committed gate."
area: validation
state: open
priority: medium
closes_when: "The standalone harness banks MG's per-order-class amplitudes (RES per squared order) and a committed row compares a split evaluator's per-class |M|² and signed interference per helicity. run_flows refuses a split evaluator instead of panicking. The split evaluator has a public per-helicity signed-|M|² accessor. One VLQ-style row with an FFS chiral vertex is banked. If the VLQ UFO cannot be vendored, a vendorable model with the same vertex class is used."
blocked_by: []
opened: 2026-11-25
tags: [standalone, squared-orders, interference, oracle, test-tooling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: l-i1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-I1-report.md", title: "L-I1 report, Found"}
---
Found by sprint L's L-I1 probe. It needed an out-of-tree script, `li1_bank.py`,
to bank MG 3.7.1's split amplitudes on `g u > h t b~ d`. The in-tree harness has
three gaps:
- `bank()` assumes the unsplit `MATRIX`.
- `standalone_jamps.rs` compares only unsplit flows.
- `run_flows` panics on a split evaluator with "run_forward_flows_typed on a
  non-Flows amplitude". That is a panic in test tooling.

The binary that lacked #169 got |M2|² wrong by up to 1.8× and the
interference sign wrong. No committed gate could have caught it, because none
compares a per-class amplitude.
