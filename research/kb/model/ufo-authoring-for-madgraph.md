---
type: Caveat
title: Writing a UFO MadGraph reads the way we do
description: "A non-self-conjugate vertex needs its h.c. listed; without a T(a,i,j) vertex MadGraph guesses 3/3̄ labels (fatal with an epsilon); a model must declare QCD."
status: draft
tags: [ufo, madgraph, toy-models, colour, authoring]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n35-t1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L905-L991", title: "Note 35 §6 T1, authoring vibegraph_toy_UFO and banking its oracle"}
  - {id: mg-color-rep, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/models/import_ufo.py#L1651-L1700", title: "MadGraph import_ufo.py find_color_anti_color_rep"}
  - {id: ufo-readme, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/ufo/README.md", title: "validation/ufo/README.md"}
  - {id: toy-color-vertices, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/ufo/vibegraph_toy_color_UFO/vertices.py#L1-L33", title: "vibegraph_toy_color_UFO vertices.py module documentation"}
---

A hand-written UFO is the cheapest way to put one Lorentz or colour structure into one
vertex with a coupling of our choosing (the case for it is in
[toy UFO models](../validation/toy-ufo-models.md)). The model is only useful as an oracle
if MadGraph reads it as intended, and MadGraph's importer has conventions a model
written for vibegraph alone would not need. Three of them are easy to miss, and
`vibegraph_toy_UFO` and `vibegraph_toy_color_UFO` work around each. [^n35-t1]

## 1. List the hermitian conjugate of every non-self-conjugate vertex

MadGraph finds an interaction by the sorted tuple of its legs' PDG codes, with the
initial legs conjugated. A vertex whose particle multiset is not its own conjugate (for
example a diquark coupling `q q D3~`) is not found for the conjugate process unless the
model also lists its h.c. vertex, which is what a FeynRules-written model emits anyway.
Write both, with the same coupling when the strength is real. [^toy-color-vertices]

## 2. Without a `T(a,i,j)` vertex, MadGraph guesses which leg is the 3

UFO colour strings name slots, not representations, so MadGraph has to decide which
member of a 3/3̄ pair is the **3**. `import_ufo.find_color_anti_color_rep` learns it only
from three-point vertices carrying an adjoint index, `T(3,2,1)` or `T(3,1,2)` (or an
`Identity` next to a particle it has already labelled) [^mg-color-rep]:

```python
if colors[:2] == [3,3]:
    if 'T(3,2,1)' in interaction_info.color:
        color, anticolor, other = interaction_info.particles
    elif 'T(3,1,2)' in interaction_info.color:
        anticolor, color, _ = interaction_info.particles
```

It records the *first* particle of a `T(3,2,1)` vertex as the fundamental, which for the
Standard Model's `u~ u g` is the antiparticle. A model with no such vertex falls through
to a default that reads each particle's own colour sign, which is the opposite
labelling, and an `Identity(1,2)` between a 3 and a 3̄ then comes out index-reversed.

- **Without an `Epsilon`** the reversal is invisible: it transposes the whole colour
  basis uniformly, and |M|² and the colour matrix are blind to a uniform transpose.
- **With an `Epsilon`** or `K6` it is fatal: MadGraph's colour matrix stops reducing and
  generation dies in `set_Nc`.

`vibegraph_toy_color_UFO` has no gluon vertex at all and avoids the guess by writing its
3/3̄ deltas as an explicit `T(2,1)` instead of `Identity(1,2)` [^toy-color-vertices]. Either a `T(a,i,j)`
vertex for the field or explicit `T(i,j)` deltas work; an `Identity` between triplets in
a model without one does not, once the model also has a baryonic or sextet tensor.

## 3. Declare `QCD` in `coupling_orders.py`

MadGraph's `WEIGHTED` coupling-order search reads the `QCD` order by name and raises
`KeyError` on a model that has none, so an unbounded `generate` fails before it
enumerates anything. Declare `QCD` even when no coupling carries it, as
`vibegraph_toy_color_UFO` does (its only real order is `NP`).

## Choices the toy models made for MadGraph's sake

These are not MadGraph rules, but a new model meets the same constraints:

- **Distinct triplets under a baryonic `Epsilon`.** `Epsilon(1,2,3)` is antisymmetric,
  so `vibegraph_toy_color_UFO` couples two *distinct* triplet scalars `p3`, `r3`; two
  identical bosonic legs would cancel it.
- **Sextet leg first.** `K6Bar(3,1,2)` is written with the sextet slot first, as
  `color_algebra.py` defines `K6`/`K6Bar`.
- **All-scalar colour model.** Two fermions reach a diquark only through a
  fermion-number-violating vertex, which needs charge conjugation; Majorana and `C`
  handling are outside what the engine evaluates
  ([backlog](../backlog/feature/majorana-fermions-unsupported.md)).
- **Two spellings, two orders.** The toy model writes its tensor four-fermion contact
  twice, once with a literal `Sigma` and once as the γγ expansion, under different
  coupling orders, so MadGraph splits them into two diagrams that can be compared per
  helicity inside one process. [^ufo-readme]
- **`Sigma` is ALOHA's.** ALOHA's `Sigma` is half the textbook `(i/2)[γ^μ, γ^ν]`
  (measured on the toy rows). A model author writing a coupling for a literal `Sigma`
  writes it against ALOHA's normalisation; see
  [gamma chains, Gamma5 and Epsilon](../amplitudes/gamma-chains-gamma5-and-epsilon.md).

How the epsilon and sextet atoms then flow through crossing and the colour basis, and
which of them stay refused, is in
[colour crossing, epsilon and sextets](../amplitudes/colour-crossing-epsilon-and-sextets.md).

[^n35-t1]: Note 35 §6, authoring the toy UFO and banking its oracle: the two MadGraph conventions recorded for model authors, and the `QCD` declaration requirement.
[^mg-color-rep]: `models/import_ufo.py` `find_color_anti_color_rep`, L1651.
[^ufo-readme]: `validation/ufo/README.md`, the `vibegraph_toy_UFO` section.
[^toy-color-vertices]: `validation/ufo/vibegraph_toy_color_UFO/vertices.py` module documentation: the h.c. listing and the explicit `T(2,1)` deltas.
