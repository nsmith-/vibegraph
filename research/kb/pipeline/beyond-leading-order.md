---
type: Overview
title: What NLO and beyond would add to an LO pipeline
description: "Components NLO adds to an LO generator (virtuals, UV renormalisation, IR subtraction, matching to showers), NNLO tooling and computer algebra; out of release scope."
status: draft
tags: [nlo, scope, background, pipeline]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n00-nlo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/00-overview.md#L70-L139", title: "Note 00: Beyond LO — NLO and fixed-order calculations"}
  - {id: n03-powheg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/03-sherpa-powheg.md#L351-L368", title: "Note 03 §2.11: POWHEG-BOX relevance to vibegraph"}
  - {id: n38-room, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L227-L253", title: "Note 38 §3.2: room for MLM and NLO"}
  - {id: check-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/diagrams/check.rs#L12-L37", title: "Unsupported variants table (check.rs)"}
---
vibegraph generates leading-order (tree-level) matrix elements only. NLO is
outside the release goal ([release-scope decision](../decisions/release-scope-lo-mlm.md);
its enforcement is in [release scope](release-scope.md));
it is tracked as the backlog items
[nlo-generation-missing](../backlog/feature/nlo-generation-missing.md) and
[loop-level-ufo-models-unsupported](../backlog/feature/loop-level-ufo-models-unsupported.md).
This concept records what an NLO extension would need, so that the boundary
stays visible and the LO data structures are not built in a way that blocks it.

## Where the boundary sits in the code

- **The proc card parses NLO syntax and refuses it.** `[QCD]`, `[real=QCD]` and
  the `!a!` photon tag are in the AST and refused by `check_supported` as
  `Unsupported::LoopSpec` and `Unsupported::PhotonTag`
  (`vibegraph-lib/src/diagrams/check.rs`). They are not parse errors.[^check-rs]
- **Loop UFOs load as LO models.** The parser skips the `.counterterm` and
  `.loop_particles` attributes of `loop_sm`-style models, and never reads
  `CT_vertices.py`, `CT_couplings.py` or `CT_parameters.py`.
- **There is a slot for Born/real bookkeeping.** The diagram container handed
  to `helas` records each diagram's provenance (which `@N`, which decay-chain
  node, each propagator's on-shell flag). The same slot is where NLO's
  Born/real/virtual split would attach.[^n38-room] No other design exists.

## What NLO adds

The LO chain is mostly numerical. NLO needs analytic work on divergent
expressions *before* any numerical integration:[^n00-nlo]

```text
LO:   UFO → diagrams → HELAS numerical code → VEGAS
NLO:  UFO → diagrams → analytic reduction (CAS)
                     → UV renormalisation
                     → IR subtraction construction
                     → numerical one-loop evaluation
                     → VEGAS over (real + virtual − subtraction)
```

| Component | What it does | Typical tools |
|---|---|---|
| One-loop amplitudes | Virtual corrections; loop integrals in dimensional regularisation, with poles in ε | OpenLoops, GoSam, MadLoop, BlackHat |
| UV renormalisation | Counterterms that absorb UV divergences; model-dependent analytic work (the UFO `CT_*` files) | FeynArts/FeynCalc, FORM |
| IR subtraction | Local counterterms so real and virtual singularities cancel numerically | Catani–Seymour dipoles, FKS; automated in MadGraph5_aMC@NLO and Sherpa |
| Matching to a shower | MC@NLO subtraction, or POWHEG's B̃ function plus hardest-emission generation | MadGraph5_aMC@NLO, POWHEG-BOX |

Subtraction also needs **colour- and spin-correlated Born** matrix elements
(POWHEG's `bornjk` and `bmunu`). LO never needs them, so vibegraph's evaluator
does not produce them. See [Catani–Seymour](../references/papers/catani-seymour.md)
and the [POWHEG-BOX survey](../references/codebases/powheg-box.md).

POWHEG-BOX is the reference for how NLO+PS is organised:[^n03-powheg]

1. Its user interface (`setborn`, `setvirtual`, `sigreal_btl`) separates the
   matrix elements from the integration and generation machinery, a design
   principle worth keeping in vibegraph.
2. **MINT** is an alternative to VEGAS: adaptive grids with folding, which
   reduces variance without hand-chosen importance-sampling variables.
3. **B̃** is the formula to implement for POWHEG-style NLO+PS, as opposed to
   MC@NLO subtraction.

## NNLO and computer algebra

Beyond NLO, loop integrals are reduced to a small basis of master integrals by
integration-by-parts identities (the Laporta algorithm: FIRE6, Kira, LiteRed,
Reduze). The masters are then solved by differential equations or evaluated
numerically by sector decomposition (pySecDec, FIESTA).

The dominant computer-algebra tool for these calculations is **FORM**, which is
built for polynomial manipulation of very large expressions (Dirac traces,
colour algebra). Mathematica with FeynArts/FeynCalc is common for simpler or
pedagogical NLO work. FeynRules, which writes UFO models, is Mathematica-based.
FormCalc pipes FeynArts diagrams through FORM and LoopTools.

## Other generators' tree-level engines

Shower programs mostly read LO events from LHEF (Pythia 8, whose built-in hard
processes are a fixed list) or interface external one-loop providers (Herwig 7's
Matchbox). Sherpa ships two LO engines: AMEGIC, which is diagram-based like
MadGraph, and COMIX, which uses Berends–Giele recursion. Recursion scales
polynomially rather than factorially with the number of legs. The architectures
are compared in [generator architectures](../references/codebases/generator-architectures-compared.md)
and [COMIX](../references/papers/comix.md).

[^n00-nlo]: Note 00, "Beyond LO: NLO and Fixed-Order Calculations" (NLO structure, NNLO, computer algebra, summary).
[^n03-powheg]: Note 03 §2.11, POWHEG-BOX-V2 relevance to vibegraph.
[^n38-room]: Note 38 §3.2, room for MLM and NLO.
[^check-rs]: `vibegraph-lib/src/diagrams/check.rs`, the `Unsupported` table.
