---
type: Backlog Item
title: "ptj4min = −1 is refused, though MadGraph treats a non-positive value as no cut"
description: "ZtoAlpGto3G now stops at ptj4min = −1 after the xpt* ≤ 0 fix lifted xptb; MadGraph's jet-rank cuts test ptjNmin > 0 and clamp with max(…, 0), so −1 is inert there."
area: feature
state: open
priority: medium
closes_when: "A non-positive ptj1min–ptj4min compiles to the default cuts (as xpt* ≤ 0 does), a positive value is still refused until implemented, both with tests, and ZtoAlpGto3G gets past this cut."
blocked_by: []
opened: 2026-11-07
tags: [cuts, run-card, refusal, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: closing campaign main-202a540-200-pc"}
  - {id: hc1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C1-report.md", title: "H-C1: xpt* ≤ 0 compiles to the default cuts"}
---
In the CERN host's closing campaign `main-202a540-200-pc`, `ZtoAlpGto3G` now
stops at `ptj4min = −1`. H-C1's fix made `xpt*` ≤ 0 inert, which lifted its
`xptb`; the next refusal is this one.

MadGraph (mg5amcnlo b768706) guards it the same way as `xpt*`:
- `Template/LO/SubProcesses/cuts.f:817–820` requires a fourth jet only when
  `ptj4min.gt.0` (likewise `ptj1min`–`ptj3min` on `:817–819`);
- `cuts.f:855` fails a jet when `ptjet(i).lt.ptjmin4(i)`, never true at −1;
- `setcuts.f:549–562` clamps each term of the τ floor with `max(…, 0d0)`.

vibegraph lists `ptj1min`–`ptj4min` in `UNIMPLEMENTED_CUTS`
(`vibegraph-lib/src/cuts.rs:459`), and `NONPOSITIVE_IS_OFF` (`cuts.rs:513`)
holds the four `xpt*` only. Adding the `ptjNmin` to it is the likely fix
(**inferred** for `ptj1min`–`ptj3min` from the same guards; only `ptj4min` was
seen on a card).
