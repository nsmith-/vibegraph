---
type: Paper
title: "The four polarizations of the W at high energies"
description: "arXiv:2512.10015 (Basu, Ruiz): polarization interference, off-shell effects and gauge cancellations with polarized propagators, decomposed in covariant and axial gauges."
resource: "https://arxiv.org/abs/2512.10015"
status: draft
tags: [polarization, propagators, gauge, paper, interference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-trunc, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L492-L524", title: "Note 01, truncated propagator paradigm summary"}
  - {id: mg-pol-list, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/base_objects.py#L2095-L2098", title: "base_objects.py, polarization codes citing this paper"}
---

Basu and Ruiz (2025) build on the polarized-propagator approach of
[the MG5_aMC polarization paper](polarized-matrix-elements-mg5.md) and study
multi-leg processes with near-resonant weak bosons at the level of helicity
amplitudes, computing the interference between polarizations directly. Note 01
summarises it under the heading "Truncated Propagator Paradigm for Polarized
Amplitudes"[^n01-trunc].

## Content

- Analytic decompositions of the polarized propagator in covariant (`R_ξ`) and
  axial gauges, with explicit polarization vectors and completeness relations,
  which make the mass-over-energy power counting manifest.
- The scalar polarization (`λ = S`) and the Goldstone contributions in `R_ξ`
  gauge.
- Polarization interference is generically nonzero, even on shell, and can
  exceed the `Γ²/M²` corrections that bound the narrow-width and pole
  approximations at low energy.
- Longitudinal contributions are suppressed at high energy, consistent with the
  Goldstone equivalence theorem; helicity inversion generates interference, and
  s- and t-channel exchanges suppress it at high energy.
- A scheme for reducing gauge dependence in polarized rate predictions.

MadGraph 3.7.1's extra vector polarization codes (`{G}` metric, `{H}` Θ, `{Q}`,
`{W}` Ward-protected full propagator, `{S}` auxiliary plus width) cite this
paper[^mg-pol-list].

## Relevance to vibegraph

Two uses. The completeness relations are the check that a sum over polarized
states reproduces the unpolarized result, which is the sum vibegraph's
helicity loop computes ([helicity sum](../../amplitudes/helicity-sum-and-pruning.md)).
And the polarization-vector conventions bear on vibegraph's vector
wavefunctions, which follow HELAS's `vxxxxx`
([wavefunctions and propagators](../../amplitudes/wavefunctions-and-propagators.md)).
vibegraph supports polarization only on external legs; polarized propagators
and the codes this paper introduces are refused
([polarization](../../process/polarization.md),
[polarized-intermediate-resonances-refused](../../backlog/feature/polarized-intermediate-resonances-refused.md)).

[^n01-trunc]: Note 01, "Truncated Propagator Paradigm" summary; the arXiv title and authors are Basu and Ruiz, "The Four Polarizations of the W at High Energies".
[^mg-pol-list]: `madgraph/core/base_objects.py:2095–2098` and the polarization parser in `madgraph_interface.py` at `b7687064`.
