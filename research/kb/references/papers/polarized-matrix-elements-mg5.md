---
type: Paper
title: Polarized matrix elements in MG5_aMC
description: "arXiv:1912.01725 (Buarque Franzosi, Mattelaer, Ruiz, Shil): fixed-helicity particles via truncated propagators, and MadGraph's {0}/{T}/{L}/{R} polarization syntax."
resource: "https://arxiv.org/abs/1912.01725"
status: draft
tags: [polarization, helicity, madgraph, paper, propagators]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-pol, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L456-L491", title: "Note 01, polarized matrix element automation summary"}
  - {id: mg-pol-parse, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/interface/madgraph_interface.py#L5095-L5185", title: "madgraph_interface.py, the {…} polarization parser"}
  - {id: mg-pol-list, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/base_objects.py#L2095-L2098", title: "base_objects.py, list_of_allowed_polarizations"}
---

"Automated Predictions from Polarized Matrix Elements" (Buarque Franzosi,
Mattelaer, Ruiz, Shil, 2019) automates helicity-polarized matrix elements in
MadGraph5_aMC@NLO. Fermions and weak bosons can be given a fixed helicity while
spin correlations and off-shell effects are kept, for production and decay at
leading order[^n01-pol].

## Method

For an intermediate particle of definite helicity `λ`, the propagator's
numerator is replaced by the projector onto that helicity state:

```text
Δ_μν(q)  →  ε*_μ(q, λ) ε_ν(q, λ)
```

Everything else is standard HELAS/ALOHA evaluation. The particle still runs
off-shell over all momenta; only its spin state is fixed. Helicities are
defined relative to the particle's momentum in a chosen frame (laboratory,
partonic centre of mass, or user-defined).

## MadGraph's syntax

A polarization is written in braces after the particle,
`generate p p > w+{0} z{T}`. At the pin, the parser maps the labels as
follows[^mg-pol-parse]:

| Label | Code | Meaning |
|---|---|---|
| `{T}` | `[1, −1]` | transverse (spin-1 only) |
| `{0}` | `[0]` | longitudinal (massive spin 1) |
| `{L}`, `{R}` | `[−1]`, `[1]` | left- and right-handed; for a vector, `L` is the −1 helicity, not longitudinal |
| `{A}` | `99` | auxiliary (spin-1 only) |
| `{+n}`, `{−n}` | `±n`, `n ≤ 3` | an explicit helicity |
| `{G}`, `{H}`, `{Q}`, `{W}`, `{S}` | `4`, `5`, `6`, `7`, `9` | metric, Θ, longitudinal minus Θ, Ward-protected full propagator, auxiliary plus width; extensions from arXiv:2512.10015 |

`base_objects.py` points to this paper for the fermion and vector definitions
and to arXiv:2512.10015 for the extensions[^mg-pol-list]. There is no "axial
±2" vector label.

## Relevance to vibegraph

vibegraph accepts MadGraph's polarization syntax on external legs and reproduces
its helicity conventions, which are HELAS's
([polarization](../../process/polarization.md)). A polarization on a leg that
is then decayed, which is where this paper's truncated propagator is needed,
and the propagator-only codes (`{A}`, `{G}`, `{H}`, `{Q}`, `{W}`, `{S}`) are
refused: [polarized-intermediate-resonances-refused](../../backlog/feature/polarized-intermediate-resonances-refused.md).
The follow-up analysis of polarized propagators is
[Basu and Ruiz](truncated-propagator-polarization.md).

[^n01-pol]: Note 01, polarized matrix element automation summary; authors confirmed on arXiv (note 01 lists different ones).
[^mg-pol-parse]: `madgraph/interface/madgraph_interface.py` at `b7687064`, the polarization branch of `extract_process`.
[^mg-pol-list]: `madgraph/core/base_objects.py:2095–2098` at `b7687064`.
