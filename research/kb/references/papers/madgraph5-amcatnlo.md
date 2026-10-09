---
type: Paper
title: MadGraph5_aMC@NLO
description: "arXiv:1405.0301 (Alwall et al.): the automated LO and NLO pipeline with shower matching and merging; its LO subset is the scope vibegraph reproduces."
resource: "https://arxiv.org/abs/1405.0301"
status: draft
tags: [madgraph, pipeline, lo, nlo, paper]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-mg5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L120-L147", title: "Note 01, MadGraph5_aMC@NLO summary"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
---

The MadGraph5_aMC@NLO paper describes the complete automation of tree-level and
NLO QCD cross sections, parton-shower matching and multi-leg merging in one
framework. The LO case is §2.3 (LO computations) and §3.1 (LO-type
generation and running), with LO scale and PDF uncertainties and reweighting
in Appendix B.4–B.5; read with
[the original MadGraph paper](madgraph-stelzer-long.md) for diagram
generation[^n01-mg5].

## The LO pipeline it describes

1. Load a UFO model into particle and vertex tables.
2. Generate helicity routines for every vertex with [ALOHA](aloha.md).
3. Generate diagrams (combination over the model's interactions, then
   flavour assignment).
4. At each phase-space point, evaluate all diagrams, sum the amplitudes and
   square.
5. Integrate over phase space with VEGAS-style adaptation to get `σ ± δσ`.
6. Unweight events by accept/reject.

Sub-diagrams shared between diagrams are computed once, so the cost grows with
the number of distinct wavefunctions rather than with the product of diagrams
and legs.

## Relevance to vibegraph

vibegraph's scope is this paper's LO pipeline, reproduced process for process
against MadGraph 3.7.1 ([release scope](../../pipeline/release-scope.md),
[pipeline overview](../../pipeline/overview.md)). The NLO, matching and
merging parts are out of scope except LO MLM matching
([MLM matching](../../scales-pdf/mlm-matching.md)); what NLO would add is
[beyond leading order](../../pipeline/beyond-leading-order.md). The code that
implements the paper, at the pinned version, is
[the MadGraph survey](../codebases/madgraph5-amcnlo.md).

[^n01-mg5]: Note 01, MadGraph5_aMC@NLO summary. Note 01 says "sections 2–4 cover the LO case"; the section numbers here are from the paper's table of contents (arXiv HTML), where §2–§4 interleave LO and NLO.
