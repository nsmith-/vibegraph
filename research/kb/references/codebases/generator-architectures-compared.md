---
type: Overview
title: MadGraph, Sherpa and POWHEG-BOX compared
description: "Diagram enumeration, Berends-Giele recursion and an NLO framework side by side: amplitude unit, colour, helicity, model loading, integration, output and where each scales."
status: draft
tags: [madgraph, sherpa, powheg, architecture, comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n00-ps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L76-L92", title: "Note 00, what parton-shower programs do at ME level"}
  - {id: n03-compare-sherpa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L146-L162", title: "Note 03 §1.6, COMIX against MadGraph/HELAS"}
  - {id: n03-compare-all, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L369-L383", title: "Note 03 Part 3, comparison of all surveyed generators"}
---

Three reference codes take three different positions on how a leading-order
matrix element is produced. vibegraph follows MadGraph's: one amplitude per
Feynman diagram, colour by a precomputed colour-factor matrix, validated
against MadGraph's own numbers. The surveys are
[MadGraph5_aMC@NLO](madgraph5-amcnlo.md), [Sherpa / COMIX](sherpa.md) and
[POWHEG-BOX-V2](powheg-box.md)[^n03-compare-all].

| | MadGraph5_aMC@NLO | Sherpa / COMIX | POWHEG-BOX-V2 |
|---|---|---|---|
| What it is | a generator: model, diagrams, code, integration, events | a full framework; COMIX is its tree-level ME generator | an NLO+PS framework; no ME generation |
| Amplitude unit | one Feynman diagram, a chain of ALOHA (HELAS-style) calls in generated Fortran | an off-shell current `J(S)`, built recursively by `Vertex::Evaluate()` | the user's `setborn`, `setvirtual`, `setreal` |
| Cost growth | factorial in the number of legs (number of diagrams), with sub-diagram reuse | about `3ⁿ` recursion steps | whatever the user's code costs |
| Colour | diagrams accumulate into colour-stripped partial amplitudes (JAMPs) contracted with a precomputed colour matrix | colour-dressed currents carry explicit indices; external colours summed or sampled by `Color_Integrator` | colour-correlated Born `bornjk` for subtraction |
| Helicity | an explicit loop over helicity configurations calling the generated routines, with run-time filtering of vanishing ones | `Spin_Structure` holds all configurations; `Helicity_Integrator` sums or samples | inside the user's code |
| Wavefunctions | HELAS spinors and vectors, no colour | `CObject`s with colour attached | — |
| Models | imports the UFO's Python modules and converts them (`import_ufo.py`) | native C++ UFO reader (`UFO_Model`) | none; the user writes flavour tables |
| NLO | MadFKS (FKS subtraction) | Catani–Seymour dipoles in the Sherpa framework | FKS-style counterterms in `B̃` |
| Integration | MadEvent: one VEGAS grid and job per configuration, summed | PHASIC++ adaptive multichannel | MINT, VEGAS-like with folding |
| Output | LHEF | HepMC, LHEF | LHEF 3.0 |
| Language | Python, generated Fortran | C++ | Fortran 77/90 |
| Primary reference | arXiv:1405.0301 | arXiv:0808.3674 | arXiv:hep-ph/0409146, arXiv:1002.2581 |

## Where each one wins

Per-diagram evaluation is easy to audit, since every diagram is a separate,
comparable number, and with sub-current reuse it is efficient at low
multiplicity. MadGraph remains the reference for processes up to about six
external legs. Berends–Giele recursion shares every sub-current across all
diagrams at once, so its cost grows exponentially rather than factorially, and
it wins clearly beyond about six legs ([COMIX](../papers/comix.md)). Sherpa
also ships AMEGIC++, a diagram-based generator in the MadGraph style.

The other general-purpose shower programs mostly consume matrix elements from
outside: Pythia 8 reads Les Houches event files and has a limited internal
hard-process library; Herwig 7 automates NLO through Matchbox with external
one-loop providers (OpenLoops, GoSam). Sherpa supports NLO through
OpenLoops and BlackHat.

POWHEG-BOX sits a layer above all of this: it needs a matrix-element provider
and supplies what NLO+PS adds on top. What an NLO extension would need is
[beyond leading order](../../pipeline/beyond-leading-order.md).

[^n03-compare-all]: Note 03 Part 3.
