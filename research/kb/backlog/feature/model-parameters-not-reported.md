---
type: Backlog Item
title: "A restricted model's parameters and default param card are not reported in machine-readable form"
description: "simshop's param-card and reweight-hypothesis editors need each parameter's class, default and reweight verdict; today they would have to parse parameters.py and restrict_*.dat themselves."
area: feature
state: open
priority: medium
closes_when: "For a model and restriction (e.g. SMEFTsim_topU3l_MwScheme_UFO-massless_VH), vibegraph writes one documented record per external parameter (name, block, code, post-restriction default, class: free / zero / one / merged into X with factor / internal; MASS/DECAY particle and whether it is external in the process; reweight verdict and polynomial entry; binder value constraints) and the default param_card.dat, and a test pins one parameter of each class."
blocked_by: []
opened: 2026-11-18
tags: [cli, model, param-card, reweight, ui, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-18}
---
Reported by simshop, which is designing a param-card editor for its New
sample form and a reweight-hypothesis editor for its generate step. Both need
to know, for a model and restriction, which parameters a user may set and to
what. It is the model-side counterpart of the `vibegraph run-card` report,
which gives each run-card key's resolved value and class without sampling:
vibegraph already decides all of it, so a second reader in simshop would drift.

vibegraph already holds each piece:
- the restriction's free / zero / one / merged decisions (the loader, as
  MadGraph applies a restriction card);
- the binder's value checks (`ufo/bind.rs`: `DECAY Auto` refused, a negative
  mass refused except on a Majorana fermion);
- the reweight classification (`reweight/`: Locked, FixedOne, SplitsMerged,
  StrongCoupling, ExternalMass) and whether a parameter enters polynomially,
  which `--reweight-couplings` needs.

Possible home: a `parameters.json` next to `diagrams.json` from the
`enumerate` run simshop's check already makes, where the process is known.
A model-only entry point with no process would also serve; ExternalMass then
needs the process. Either way the record shape is documented.

Lower priority, same theme: `enumerate --reweight-card`, binding the
hypotheses at check time as `--param-card` already does, so a bad hypothesis
is refused before the user waits for integrate.
