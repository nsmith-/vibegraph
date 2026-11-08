---
type: Backlog Item
title: "UFO couplings with a negative coupling order are refused at model load"
description: "GluGluH_HToUpsilon ×2 stop at model load on negative coupling orders, per the CERN host; extract_orders accepts only non-negative integers, while MadGraph produced these gridpacks."
area: feature
state: open
priority: medium
closes_when: "MadGraph's handling of a negative coupling order (in the diagram order counting and WEIGHTED) is read and reproduced, a test loads a coupling with a negative order, and a GluGluH_HToUpsilon card integrates with σ matching MadGraph by the multi-seed criterion; or the refusal moves past model load with a stated reason."
blocked_by: []
opened: 2026-11-07
tags: [ufo, coupling-orders, refusal, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: closing campaign main-202a540-200-pc"}
---
Reported by the CERN host from the closing campaign `main-202a540-200-pc`: both
`GluGluH_HToUpsilon` gridpacks are refused at model load on negative coupling
orders.

The likely site is `extract_orders` in `vibegraph-lib/src/ufo/couplings.rs`,
whose orders are `usize` ("a dict of order names to non-negative integers");
that it is the site is **inferred** from the refusal's description, not
reproduced. Coupling orders are `usize` downstream too, so accepting a negative
order touches order counting and the `WEIGHTED` hierarchy, not only the parser.

Which coupling carries the negative order, and what MadGraph does with it,
were not recorded.
