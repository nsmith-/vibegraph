---
type: Paper
title: General-purpose event generators for LHC physics
description: "arXiv:1101.2599 (Buckley et al., MCnet review): where LO matrix elements sit in the full simulation chain; factorisation, ME generators, scales and PDFs."
resource: "https://arxiv.org/abs/1101.2599"
status: draft
tags: [review, event-generators, factorisation, paper, context]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-review, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L197-L216", title: "Note 01, MC event generators review summary"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
---

The MCnet review of general-purpose event generators (Ariadne, Herwig++,
Pythia 8, Sherpa) for proton–proton collisions. Most of it covers parton
showers, hadronisation and the underlying event, which are outside vibegraph's
scope[^n01-review].

The parts that bear on a leading-order generator:

- **§3.1**: the QCD factorisation formula, which defines the hard cross
  section a matrix-element generator computes;
- **§3.2**: LO matrix-element generators, surveying how Alpgen, MadGraph and
  Sherpa/COMIX generate tree-level amplitudes;
- **§3.3–3.4**: scale choices and parton distributions.

## Relevance to vibegraph

It is context for where vibegraph's output goes: the `σ_hard` and the
unweighted parton-level events it produces are the input a shower such as
Pythia 8 takes over ([pipeline overview](../../pipeline/overview.md),
[Pythia interop](../../events/pythia-interop.md)). The scale and PDF choices the
review discusses are implemented in vibegraph as MadGraph makes them
([MadGraph scale choice](../../scales-pdf/madgraph-scale-choice.md)). The
architectural comparison of matrix-element generators is
[generators compared](../codebases/generator-architectures-compared.md).

[^n01-review]: Note 01, MC event generators review summary.
