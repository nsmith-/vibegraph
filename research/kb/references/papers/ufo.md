---
type: Paper
title: "UFO: Universal FeynRules Output"
description: "arXiv:1108.2040 (Degrande et al.): the model format as a Python module of particles, parameters, vertices, Lorentz structures and couplings; vibegraph's model input."
resource: "https://arxiv.org/abs/1108.2040"
status: draft
tags: [ufo, model, feynrules, paper, input-format]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L49-L86", title: "Note 01, UFO summary"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
  - {id: n01-feynrules2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L217-L220", title: "Note 01, FeynRules entry (FeynRules 2.0 adds full UFO output)"}
  - {id: feynrules2, resource: "https://arxiv.org/abs/1310.1921", title: "Alloul et al., FeynRules 2.0 (full UFO output)"}
  - {id: cargo, resource: "vibegraph-lib/Cargo.toml#L38-L39", title: "peg and rustpython-parser dependencies"}
---

UFO defines a model as a Python module rather than a text format, with no
built-in assumption about which Lorentz or colour structures may appear. It is
read by MadGraph5, GoSam, Sherpa and others[^n01-ufo], and written by
[FeynRules](feynrules.md) (full UFO output since FeynRules 2.0[^feynrules2]).

## Structure

| File | Contents |
|---|---|
| `particles.py` | `Particle` objects: PDG code, spin, colour, mass, width, antiparticle |
| `parameters.py` | external (input) and internal (derived) parameters |
| `vertices.py` | `Vertex` objects: particles, Lorentz structure references, coupling references |
| `lorentz.py` | Lorentz structures as expression strings (`Gamma(mu,1,2)`) |
| `couplings.py` | coupling values as expression strings, with coupling orders |
| `coupling_orders.py` | order bookkeeping (QCD, QED, …) |
| `object_library.py` | the base classes |

A vertex:

```python
V_1 = Vertex(name='V_1',
             particles=[P.e__minus__, P.e__plus__, P.a],   # e- e+ γ
             color=['1'],
             lorentz=['FFV1'],
             couplings={(0,0): C.GC_3})                   # -i e γ^μ
```

`couplings` is keyed by `(colour index, Lorentz index)`. A real model's files
can go beyond this (decay tables from [MadWidth](madwidth.md), loop
information, form factors, attribute assignments after construction).

## Relevance to vibegraph

UFO is vibegraph's only model input. It parses the files without running
Python: `rustpython-parser` turns each file into a Python AST that is read
structurally, and `peg` grammars parse the expression strings inside it
(Lorentz structures, colour structures, coupling and parameter
expressions)[^cargo]. That design and its limits are
[UFO parsing](../../model/ufo-parsing.md) and the
[UFO string grammars](../../model/ufo-string-grammars.md). How spin codes and
Lorentz structures map onto the amplitude layer is
[the UFO/ALOHA type matrix](../../model/ufo-aloha-type-matrix.md); writing a
UFO that MadGraph and vibegraph read the same way is
[UFO authoring for MadGraph](../../model/ufo-authoring-for-madgraph.md). The
routines generated from these Lorentz structures in MadGraph are
[ALOHA](aloha.md)'s.

[^n01-ufo]: Note 01, UFO summary.
[^feynrules2]: arXiv:1310.1921.
[^cargo]: `vibegraph-lib/Cargo.toml:38–39`.
