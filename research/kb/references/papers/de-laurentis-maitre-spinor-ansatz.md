---
type: Paper
title: De Laurentis-Maître spinor-helicity ansatz
description: "arXiv:1904.04067 (De Laurentis, Maître): analytic amplitudes reconstructed from numerical evaluations with spinor-helicity ansätze; where a compact tree form would have to be sought."
resource: "https://arxiv.org/abs/1904.04067"
status: draft
tags: [spinor-helicity, reconstruction, analytic-amplitudes, paper, trace-form]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n41-swell, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L153-L189", title: "Note 41 §3.1, intermediate swell is not final size"}
  - {id: n41-compact, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L729-L771", title: "Note 41 §10.4, where compactness lives"}
  - {id: n41-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L788-L810", title: "Note 41, references"}
---

"Extracting analytical one-loop amplitudes from numerical evaluations" (De
Laurentis and Maître, 2019) reconstructs analytic amplitudes from numerical
evaluations, fitting them to ansätze in spinor-helicity variables. Instead of
expanding in invariants, it divides an amplitude by a prefactor that carries its
helicity weight and reconstructs the remainder as a rational function of
momentum-twistor variables[^n41-swell].

## Why the variables matter

The famous compact tree forms (Parke–Taylor, four-fermion amplitudes such as
`⟨13⟩²[24]/(s s′ s″)`) are per-helicity amplitudes in spinor brackets, not
helicity-summed `|M|²` in dot products. Two steps destroy that
compactness[^n41-compact]:

- **Squaring.** An `n`-term amplitude gives about `n²` cross terms, and summing
  helicities adds expansions that share no structure in dot products.
- **Rewriting in invariants.** `|⟨ij⟩|² = s_ij`, so brackets are square roots of
  invariants; their phases have no rational form in the `s_ij`, and the result
  expands into a much larger polynomial.

## Relevance to vibegraph

vibegraph measured the helicity-summed `|M|²` by finite-field reconstruction
([FiniteFlow](finiteflow.md)) and found its trace form no faster than the
helicity evaluator from 2 → 4 on
([trace-form feasibility](../../performance/trace-form-msq-feasibility.md)).
The evaluator wins there because it keeps the compact objects: it evaluates
amplitudes numerically, shares sub-currents across diagrams, and squares
numbers. The low rank of the measured coefficient matrix (58 linear forms
spanning 630 pairs on `ud_to_epemud_qcd0`) is that structure showing through.

If analytic compactness is pursued, this paper is the route: reconstruct
per-helicity amplitudes in spinor variables, not `|M|²` in invariants. That is
a different measurement from the one made:

- **no conjugation**: the box is the per-helicity amplitude (`eval_amplitude`)
  over `F_p`, so momenta can be complex twistor points and the real-momentum
  machinery and root rejection of the `|M|²` study fall away;
- **spinor-built wavefunctions**: HELAS's `√E` normalisation and phases must be
  replaced or divided out per helicity. Massless spinors scaled by `√p⁺` and
  polarizations `⟨n|γ^μ|k]` with HELAS's reference `n` are polynomial in the
  momentum, with a rational weight per helicity; a massive helicity state keeps
  one root, `|p⃗|`. The per-diagram amplitude gate against MadGraph would have
  to divide out the per-helicity factor, and that is where the cost of such a
  study sits ([finite-field evaluator](../../validation/finite-field-evaluator.md)).

The prior expectation is that recursive numerical evaluation still wins from
2 → 4 on; the precedent is mixed, since some NLO codes use closed-form tree
amplitudes at low multiplicity. The study is not scheduled.

[^n41-swell]: Note 41 §3.1.
[^n41-compact]: Note 41 §10.4.
