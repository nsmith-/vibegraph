---
type: Paper
title: "ALOHA: Automatic Libraries Of Helicity Amplitudes"
description: "arXiv:1108.2041 (de Aquino et al.): generates HELAS-style wavefunction, off-shell current and amplitude routines from UFO Lorentz structures; vibegraph mirrors its conventions."
resource: "https://arxiv.org/abs/1108.2041"
status: draft
tags: [aloha, helas, ufo, lorentz, paper]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-aloha, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L17-L48", title: "Note 01, ALOHA summary"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
---

ALOHA takes a UFO model and writes a helicity-amplitude routine for every
vertex, in Fortran, C++ or Python, in the HELAS style[^n01-aloha]. It ships inside
MadGraph5_aMC@NLO (`aloha/`; see [the MadGraph survey](../codebases/madgraph5-amcnlo.md)).

## What it does

1. For each UFO vertex, take its Lorentz structure (a tensor expression such as
   `Gamma(3,2,1)`).
2. Contract it with every combination of external wavefunctions, leaving one
   leg off-shell and inserting that leg's propagator, or with all legs
   external for the amplitude.
3. Write each resulting expression as a callable routine.

Spins 0, 1/2, 1 and 2 are supported, each with its canonical propagator.

The vocabulary, which vibegraph uses throughout:

- **wavefunction routine**: an external state (`ixxxxx` incoming fermion,
  `oxxxxx` outgoing fermion, `vxxxxx` vector);
- **vertex routine**: contracts wavefunctions or currents at a vertex into an
  off-shell current. The suffix names the off-shell leg: `FFV1_3` is the FFV1
  structure with leg 3 (the vector) off-shell;
- **amplitude routine**: the full contraction to a complex number (`FFV1_0`).

The paper is the generalisation of the hand-written [HELAS](helas.md) library
to arbitrary [UFO](ufo.md) Lorentz structures.

## Relevance to vibegraph

vibegraph does not generate per-vertex code. It parses each UFO Lorentz
structure into a typed node tree and evaluates it at runtime, with fused
kernels picked by intertwiner-basis coordinates where they exist
([intertwiner basis and peephole](../../amplitudes/intertwiner-basis-and-peephole.md)).
ALOHA's role is the convention and the oracle: wavefunction, current and
amplitude follow its calling pattern, its sign and index conventions for
`Epsilon` and `Sigma` are adopted ([Levi-Civita and Sigma conventions](../../amplitudes/levi-civita-and-sigma-conventions.md)),
and the per-diagram amplitudes MadGraph computes with ALOHA's routines are what
the [amplitude oracle](../../validation/amplitude-oracle.md) compares
against. How UFO spin codes map onto ALOHA's value families is
[the UFO/ALOHA type matrix](../../model/ufo-aloha-type-matrix.md).

[^n01-aloha]: Note 01, ALOHA summary.
