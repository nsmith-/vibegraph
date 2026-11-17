---
type: Backlog Item
title: "UFO couplings with a negative coupling order are refused at model load"
description: "The largest UFO refusal class in the CMS model store (162 of 337 refused loads, 17 SMEFTsim/SMEFT variants, QED: -1 vev insertions) plus GluGluH_HToUpsilon ×2; MadGraph accepts these."
area: feature
state: open
priority: high
closes_when: "MadGraph's handling of a negative coupling order (in the diagram order counting and WEIGHTED) is read and reproduced, a test loads a coupling with a negative order, and a GluGluH_HToUpsilon card integrates with σ matching MadGraph by the multi-seed criterion; or the refusal moves past model load with a stated reason."
blocked_by: []
opened: 2026-11-07
tags: [ufo, coupling-orders, refusal, cms, smeftsim, simshop]
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

Which coupling carries the negative order in the Upsilon gridpacks, and what
MadGraph does with it, were not recorded.

**Scale (simshop, 2026-11-13).** simshop, a CMS private-MC web app, loaded all
452 UFO roots in the CMS model store with vibegraph 0.2.1, bare and under each
`restrict_*.dat`. This refusal accounted for 162 of the 337 refused loads,
across 17 SMEFTsim/SMEFT variants. An example is `SMEFT_alpha_UFO.tar.gz`,
`couplings.py`: `GC_5284 = Coupling(..., order = {'NP':1,'QED':-1})`. That is
SMEFTsim's vev insertion, which MadGraph accepts. The store is at
`https://cms-project-generators.web.cern.ch/cms-project-generators/`.
