---
type: Paper
title: FiniteFlow
description: "arXiv:1905.08019 (Peraro 2019): numerical algorithms as dataflow graphs evaluated over Z_p, with multivariate rational functions reconstructed from the samples; the method behind vibegraph's finite-field study."
resource: "https://arxiv.org/abs/1905.08019"
status: draft
tags: [finite-fields, reconstruction, rational-functions, paper, trace-form]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-swell, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L153-L189", title: "Note 41 §3.1, intermediate swell is not final size"}
  - {id: n41-method, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L329-L351", title: "Note 41 §8–8.1, functional reconstruction: what the method is"}
  - {id: n41-obstacles, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L375-L426", title: "Note 41 §8.4, obstacles specific to this codebase"}
  - {id: n41-box, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L452-L518", title: "Note 41 §9.1, the box"}
  - {id: n41-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L788-L810", title: "Note 41, references"}
  - {id: peraro16, resource: "https://arxiv.org/abs/1608.01902", title: "Peraro, Scattering amplitudes over finite fields and multivariate functional reconstruction (2016)"}
  - {id: firefly, resource: "https://arxiv.org/abs/1904.00009", title: "Klappert, Lange, FireFly: reconstructing rational functions from finite field evaluations (C++)"}
  - {id: multivariate-apart, resource: "https://arxiv.org/abs/2101.08283", title: "Heller, von Manteuffel, MultivariateApart: multivariate partial fractioning (Leinartas decomposition)"}
  - {id: ff-test, resource: "vibegraph-lib/tests/finite_field_msq.rs", title: "The Fz scalar, the reconstruction measurement and its oracles"}
---

FiniteFlow (Peraro, 2019) generalises his 2016 method for multivariate
functional reconstruction over finite fields (arXiv:1608.01902[^peraro16]) into a
framework of composable dataflow graphs[^n41-method].

## The method

- Treat a numerical algorithm as a black box mapping rational inputs to
  rational outputs, and evaluate it over prime fields `Z_p` with 63-bit primes,
  so arithmetic is exact and machine-word sized.
- From many evaluations, reconstruct the multivariate rational function the box
  computes: Thiele and Newton interpolation on univariate slices fix the
  degrees, multivariate Newton or sparse interpolation finds the coefficients,
  and rational reconstruction with the Chinese remainder theorem over a few
  primes lifts them to `Q`.
- A dataflow graph composes such algorithms (linear solves, substitutions,
  Laurent expansions) so that a chain is sampled as one box.

The symbolic intermediate expression is never formed. The cost is the number
of black-box evaluations, which scales with the number of terms in the *final*
answer, not with the intermediate swell. A tree-level evaluator at
microseconds per point can afford 10⁶–10⁷ samples per prime.

Reconstruction returns the function in whatever representation it fits,
typically an expanded numerator over a denominator. Getting a compact,
well-conditioned form is a separate step: multivariate partial fractions
(MultivariateApart[^multivariate-apart]), spinor-variable ansätze
([De Laurentis–Maître](de-laurentis-maitre-spinor-ansatz.md)), then Horner
schemes and CSE. FiniteFlow and FireFly[^firefly] are C++ libraries.

## Relevance to vibegraph

The method answers whether the helicity-summed `|M|²` has a compact closed form,
because reconstruction cost tracks the size of the answer. vibegraph ran that
measurement in-tree rather than through FiniteFlow
([trace-form feasibility](../../performance/trace-form-msq-feasibility.md)).
The black box is the unchanged evaluator, run over a test-only scalar `Fz`: an
element of `Z_p` (extended by `i`) carried with an `f64` shadow that decides
the comparisons the wavefunction routines branch on. The fit is a dense
monomial ansatz solved by exact row reduction mod `p`, lifted to `Q(i)` by
Chinese remaindering and Wang's rational reconstruction; Thiele/Newton and
sparse interpolation were not needed at the sizes measured[^ff-test].

Square roots in the wavefunctions are not a blocker over `Z_p`. The helicity
sum depends only on each leg's completeness relation, and taking the root
`x^((p+1)/4)` (the root that is itself a square) with `p ≡ 7 (mod 8)` keeps the
separately taken roots consistent. `fz_wavefunctions_satisfy_completeness`
pins that convention mod `p`[^n41-box]. The exact evaluator is reusable as an
oracle: exact zeros, exact per-pair identities, exact gauge cancellations
([finite-field evaluator](../../validation/finite-field-evaluator.md)).

Two caveats for any reuse. Parity-odd `ε(p_a,p_b,p_c,p_d)` terms are square
roots of Gram determinants in the invariants, so from 2 → 3 on they must be
reconstructed in momentum-twistor variables or split off as `R_even + ε·R_odd`.
And an expanded numerator in a monomial basis cancels badly near soft and
collinear limits, so a reconstructed form must be checked against the `f64`
evaluator across phase space, not at one point[^n41-obstacles].

[^peraro16]: arXiv:1608.01902; note 41 cites it alongside FiniteFlow.
[^n41-method]: Note 41 §8 and §8.1.
[^multivariate-apart]: arXiv:2101.08283, cited in note 41 §8.2.
[^firefly]: arXiv:1904.00009, cited in note 41 §8.4.
[^ff-test]: `vibegraph-lib/tests/finite_field_msq.rs`; note 41 §9.1.
[^n41-box]: Note 41 §9.1, "Square roots"; the test is at `tests/finite_field_msq.rs:1286`.
[^n41-obstacles]: Note 41 §8.4, obstacles 2 and 4.
