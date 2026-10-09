---
type: Paper
title: "FeynRules: Feynman rules made easy"
description: "arXiv:0806.4194 (Christensen, Duhr): the Mathematica package that derives Feynman rules from a Lagrangian and exports them; the UFO files vibegraph reads are its output."
resource: "https://arxiv.org/abs/0806.4194"
status: draft
tags: [feynrules, ufo, model, paper]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-feynrules, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L217-L252", title: "Note 01, FeynRules summary"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
  - {id: feynrules2, resource: "https://arxiv.org/abs/1310.1921", title: "Alloul, Christensen, Degrande, Duhr, Fuks, FeynRules 2.0 (full UFO output)"}
---

FeynRules (Christensen and Duhr, 2009) is a Mathematica package. The user
writes a model file with the particle content, the parameters and the
Lagrangian; FeynRules extracts every interaction vertex, stores it in a
generic internal form, and exports it through translation interfaces to the
formats of the matrix-element generators (UFO, CalcHEP, FeynArts, Sherpa,
…)[^n01-feynrules]. FeynRules 2.0 (Alloul et al., arXiv:1310.1921) added full
[UFO](ufo.md) output[^feynrules2].

The internal representation, which UFO serialises:

- **particle**: PDG code, mass symbol, spin, colour representation, charge;
- **vertex**: the participating particles, a Lorentz structure, a colour
  factor and a coupling symbol;
- **parameter**: external (an input with a value) or internal (derived).

## Relevance to vibegraph

vibegraph never runs FeynRules. It reads the UFO files FeynRules (or a model
author) produced, so FeynRules is the reason the format looks the way it does:
the UFO `Vertex`, `Lorentz` and `Coupling` objects are FeynRules' vertex
representation written out as Python ([UFO parsing](../../model/ufo-parsing.md)).
The bundled SM UFO's `particles.py` records that FeynRules 1.7.69 generated it
([the MadGraph survey](../codebases/madgraph5-amcnlo.md)). The SMEFTsim model
vibegraph vendors is another FeynRules export
([SMEFTsim topU3l](../../model/smeftsim-topu3l.md)).

[^n01-feynrules]: Note 01, FeynRules summary.
[^feynrules2]: arXiv:1310.1921; the notes mention it in one sentence.
