---
type: Backlog Item
title: "MadGraph's per-PDG pT minimum, pt_min_pdg, is refused"
description: "A run card that sets pt_min_pdg (e.g. {1023: 100}) is refused as an unimplemented cut; 2 CMS gridpacks (DarkPhoton-monojet) stop there."
area: feature
state: open
priority: medium
closes_when: "pt_min_pdg is applied as MadGraph applies it (each particle and antiparticle of a listed PDG, with MadGraph's own refusals of light PDGs), pinned per event, and a σ row with it matches MadGraph within the σ-gate tolerance."
blocked_by: []
opened: 2026-11-04
tags: [cuts, run-card, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4700-L4760", title: "banner.py: pt_min_pdg into pdg_cut / ptmin4pdg"}
  - {id: setcuts-f, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L310-L330", title: "setcuts.f: etmin for a listed PDG"}
---
`pt_min_pdg` is in `UNIMPLEMENTED_CUTS` (`vibegraph-lib/src/cuts.rs`), so any
value is refused. MadGraph turns it into `pdg_cut` / `ptmin4pdg`
(`banner.py:4700`), sets `etmin` of every outgoing leg whose `|PDG|` is listed
(`setcuts.f:318`), folds it into the `smin` hint (`setcuts.f:678`), and refuses
negative PDGs and the light PDGs 1–5, 11, 13, 15, 21 and 22. In the campaign
`main-23d3a76-200-pc` on #71 this refuses both DarkPhoton-monojet gridpacks
(`{1023: 100}`).

The other per-PDG cuts (`pt_max_pdg`, `E_*_pdg`, `eta_*_pdg`, `mxx_min_pdg`)
are not part of this item and stay refused.
