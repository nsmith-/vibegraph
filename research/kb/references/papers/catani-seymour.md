---
type: Paper
title: Catani-Seymour dipole subtraction
description: "hep-ph/9605323 (Catani, Seymour): the general dipole subtraction method for NLO QCD infrared divergences; the reference for what an NLO extension would need."
resource: "https://arxiv.org/abs/hep-ph/9605323"
status: draft
tags: [nlo, subtraction, dipoles, paper, infrared]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n01-cs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L296-L344", title: "Note 01, Catani-Seymour summary"}
  - {id: cdst, resource: "https://arxiv.org/abs/hep-ph/0201036", title: "Catani, Dittmaier, Seymour, Trócsányi, The dipole formalism for NLO QCD calculations with massive partons (2002)"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
---

Catani and Seymour (Nucl. Phys. B485 (1997) 291) give a general method for
cancelling infrared divergences at next-to-leading order in QCD[^n01-cs]. It is
not needed at leading order.

## The problem

An NLO cross section adds two contributions, each divergent on its own:

- **real emission**, with `m + 1` partons, singular when a parton is soft or two
  are collinear;
- **virtual correction**, with `m` partons and one loop, carrying explicit
  `1/ε` poles in dimensional regularisation.

Their sum is finite for infrared-safe observables; the task is to make each
piece numerically integrable.

## The subtraction

Add and subtract a local counterterm `dσ^A` that has the same pointwise
singular behaviour as `dσ^R` and can be integrated analytically over the
one-parton emission phase space:

```text
σ^NLO = ∫_{m+1} [dσ^R − dσ^A]_{ε=0}          (finite; integrate numerically)
      + ∫_m    [dσ^V + ∫_1 dσ^A]_{ε=0}        (poles cancel analytically)
```

The counterterm is a sum of **dipoles**, each with an emitter `i`, an emitted
parton `j` and a spectator `k`, and a momentum mapping `(i, j) → ĩ` onto
`m`-parton kinematics that interpolates smoothly between the soft and
collinear limits and keeps every parton on shell. The paper gives every
dipole formula needed to implement the method, for final- and initial-state
emitters and spectators, for massless partons; massive quarks are the later
extension by Catani, Dittmaier, Seymour and Trócsányi[^cdst].

## Relevance to vibegraph

vibegraph is leading order, and NLO process requests are parsed and refused
([nlo-generation-missing](../../backlog/feature/nlo-generation-missing.md)).
This paper is the reference for what an NLO extension would add
([beyond leading order](../../pipeline/beyond-leading-order.md)). Sherpa
implements Catani–Seymour dipoles; MadGraph5_aMC@NLO (MadFKS) and POWHEG-BOX use
FKS-style subtraction instead ([POWHEG-BOX](../codebases/powheg-box.md),
[generators compared](../codebases/generator-architectures-compared.md)).

[^n01-cs]: Note 01, Catani-Seymour summary. Note 01 says MadFKS uses the dipole method; MadFKS implements FKS subtraction.
[^cdst]: arXiv:hep-ph/0201036. Note 01 says the 1996 paper includes massive quarks; its abstract and scope are massless.
