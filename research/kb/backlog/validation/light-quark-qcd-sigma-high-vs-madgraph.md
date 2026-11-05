---
type: Backlog Item
title: "Light-quark QCD 2→3 and 2→4 σ read 1–2% above MadGraph on the same PDF"
description: "On explicit-flavour u d > d g u and u u > g g u u, vibegraph is 1.3–2.2% above local MadGraph on LHAPDF 247000, single-subprocess controls included; banked pp_to_jj agrees to 0.2%."
area: validation
state: open
priority: high
closes_when: "The offset is traced to a per-point quantity (|M|², PDF × flux, cut, scale or MadGraph's iteration combination) by comparing at fixed phase-space points against MadGraph standalone and MadEvent, and either fixed with a test that fails before or shown to be on MadGraph's side and recorded."
blocked_by: []
opened: 2026-11-05
tags: [sigma, qcd, madgraph-parity, proton, sprint-h]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: hg1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md", title: "H-G1 report, σ on LHAPDF 247000 and Found 2–4"}
---
Measured in sprint H's H-G1 on `colour-split-groups` (merged on main through
#96). Local MadGraph ran with `pdlabel = lhapdf` and `lhaid = 247000`, and both
sides write `AQCDUP = 0.1300027`. Fixed μ = 91.188, 13 TeV, `ptj = 20`, `drjj = 0.4`.

- `u d > d g u` + `u~ d~ > d~ g u~`: vibegraph 188 192 ± 58 pb against
  MadGraph's 185 376 ± 281 pb (10 seeds), ratio 1.0152.
- HT-binned `u u > g g u u` + `u~ u~ > g g u~ u~`: ratio 1.0205.
- Single-subprocess controls: `u d` alone +1.56%, `u u` alone +2.19%. So this is
  not flavour grouping.
- Recombining MadGraph's per-channel iterations with equal weights (`G*/results.dat`)
  raises MadGraph by 0.2% (2→3) and 1.06% (HT). That leaves +1.33% (10σ) and
  +0.95% (2.9σ).
- Ruled out: vibegraph's budget (the ladders are flat), cut regions (each side's
  events pass the other's cuts), shape (the ratio is flat in x and in the gluon pT).
  The banked `pp_to_jj`, `pp_to_jj_ht`, `pp_to_llj` and `pp_to_ttx_0j1j_mlm` rows
  agree to 0.2%.

Next: compare per-point |M|² against MadGraph standalone (H-G1 banked
`ud_to_dgu.json`), then dσ at fixed points. Follow
[bit-exact amplitude debugging](../../validation/bit-exact-amplitude-debugging.md).
