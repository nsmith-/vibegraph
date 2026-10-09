---
type: Feasibility Study
title: "Helicity-summed |M|² from completeness relations"
description: "Whether a trace-form |M|² speeds integration: cost by multiplicity, the Amdahl cap, obstacles, the reconstructed per-pair and full forms timed against eval_m2, and the recommendation."
status: draft
tags: [performance, matrix-element, helicity-sum, finite-fields, trace-form]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  landed_in: ef84a12
  pr: 15
  host: "4-core cloud container, release profile"
  command: "cargo test --release -p vibegraph-lib --test finite_field_msq measure_trace_form -- --ignored --nocapture (and measure_full_msq)"
sources:
  - {id: n41-0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L11-L89", title: "Note 41 §0–§1, verdict and what the helicity sum costs"}
  - {id: n41-2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L90-L216", title: "Note 41 §2–§4, completeness, cost scaling, the Amdahl bound"}
  - {id: n41-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L217-L328", title: "Note 41 §5–§7, obstacles, egglog, recommendation"}
  - {id: n41-8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L335-L433", title: "Note 41 §8, functional reconstruction"}
  - {id: n41-9, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L463-L612", title: "Note 41 §9, the per-pair trace form measured"}
  - {id: n41-10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-completeness-trace-msq-feasibility.md#L622-L787", title: "Note 41 §10, the full |M|² measured"}
  - {id: n21, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L512-L544", title: "Note 21, helicity and colour handling during integration and events"}
  - {id: ff-test, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/finite_field_msq.rs", title: "vibegraph-lib/tests/finite_field_msq.rs"}
---

# Helicity-summed |M|² from completeness relations

The question: replace the explicit helicity sum with completeness relations
(`Σ u ū = p̸ + m`, `Σ v v̄ = p̸ − m`, `Σ ε_μ ε*_ν = −g_μν (+ p_μ p_ν / M²)`), so that
`Σ_hel |M|²` becomes Dirac traces contracted into dot products. Would that speed up
integration, and how do you obtain the simplified form? Nothing was built into production;
the measurements are in `vibegraph-lib/tests/finite_field_msq.rs`.[^n41-0]

## Verdict

- **2 → 2 and 2 → 3 with an external gauge boson: a reconstructed closed form of the full
  `|M|²` is 4–10× faster per point than `eval_m2`.** `ee_to_mumu` 10×, the llj subprocesses
  4–5×.
- **Multi-radiator 2 → 3** (`ee_to_mumua`): 1.3× *slower* over one common denominator; only
  partial fractions could help (unmeasured).
- **2 → 4 four-fermion: dropped.** Per pair the form is 12–23× slower than `eval_m2`, and the
  full `|M|²` has nothing to cancel.
- **The stage gain is capped at about 2×** even with `|M|²` free, and the per-helicity program
  has to stay for events.
- **Helicity sampling is the direct lever** on the expensive rows.

## What the helicity sum costs today

The helicity sum is already near the minimum for the helicity-amplitude method: every
surviving combination is interned into one arena, so each distinct current is computed once
per point (sharing 1.8–2.8×), and combinations that vanish at generic points are pruned like
MadGraph's `GOODHEL` (16 of 256 kept on the 2 → 6). See
[helicity expansion](helicity-expansion.md). Per point it runs near MATRIX1
([matrix element against MadGraph](matrix-element-vs-madgraph.md)). So what is left has to
come from changing the method.

**Completeness applied leg by leg, numerically, does not help.** A `−g_μν` sum replaces a
vector's 2–3 helicities with 4 Lorentz components, and `p̸ + m` is rank 2, the same count as
two helicities. The win exists only when all legs are traced out symbolically and the result
simplified: a different program derived from the diagrams, with the arena as its oracle, not
a rewrite of the HELAS arena.[^n41-2]

## Cost scaling and the Amdahl cap

A fermion loop with `n` gamma matrices expands into `(n−1)!!` terms before simplification, and
an interference pair joins the lines of two diagrams. Pair counts `D(D+1)/2` from MadGraph's
diagram counts: `ee_to_mumu` 3, the llj subprocesses 10, `ee_to_mumua` 36, the 2 → 4 rows
325–630, the 2 → 6 167 910. The expanded size is paid once at compile time; the per-point cost
is set by the simplified expression, and that is what the measurements below size. The
historical precedent (CompHEP/CalcHEP top out around 2 → 4; EXCALIBUR and every multi-leg
generator since moved to helicity amplitudes) holds on these rows.

The evaluator is 51–62% of the busiest thread wherever integration and generation were
profiled ([integrate profiles](integrate-profiles.md)), so an infinitely fast `|M|²` gives at
most:

| profile | evaluator share | stage speedup, `|M|²` → 0 | `|M|²` 10× faster |
|---|--:|--:|--:|
| integrate, partonic | 51% | 2.0× | 1.9× |
| integrate, `pp_to_llj_dyn` | 52% | 2.1× | 1.9× |
| sample, proton | 62% | 2.7× | 2.3× |

The rows where the trace form wins already integrate in seconds, and `pp_to_llj`'s measured
deficit is the sampler (many more points than MadGraph), not the per-point cost.

## Obstacles specific to this codebase

1. **External gluons.** `Σ ε ε* → −g` holds for at most one external non-abelian boson; with
   two or more, use the physical sum with a reference vector or squared ghost diagrams (the
   CompHEP route). Massive vectors in unitary gauge and photons are fine.
2. **γ5 and ε tensors.** Chiral couplings give `ε(p_a,p_b,p_c,p_d)` terms from 2 → 3 on, one
   more convention claim to pin (`AGENTS.md`).
3. **Cancellations worsen at the squared level.** Gauge-growing terms cancelled at amplitude
   level (`ee_to_wpwm`) cancel between squared interference terms instead, losing about twice
   the digits (~4 at √s = 1 TeV). A flop-counting extraction cannot see conditioning.
4. **Events still need helicity amplitudes.** `SPINUP` selection draws from `eval_hel_m2` and
   colour selection from `eval_jamp2`; a trace program could supply JAMP2 as flow-diagonal
   blocks but has no per-helicity content, so it would sit beside the per-helicity program,
   doubling the compile surface. `AMP2_c` for the configuration draw can come from the same
   pair matrix.
5. The trace form is Lorentz invariant, so it has no partonic-CM frame contract, and exactly
   symmetric under the beam reflection `proton.rs` evaluates twice. Running couplings and
   widths are no obstacle (couplings symbolic, `1/(D_i D_j*)` per pair).

**Where egglog fits.** Dirac algebra and trace evaluation are a terminating, confluent
normalisation, where an e-graph's keep-every-form semantics only costs memory (with AC
blow-up on polynomials); that stage is FORM's home ground or an in-tree normaliser. The
e-graph fits the *choice* problems (which momentum to eliminate, invariant basis, factoring
denominators), but each pays off only through sharing and so needs the global extractor that
is still missing. It is not on the critical path. See
[the egglog rewrite stage](egglog-rewrite-stage.md) and
[DAG extraction](egraph-dag-extraction.md).[^n41-5]

## The method: functional reconstruction over finite fields

FiniteFlow-style reconstruction (Peraro; see
[finite-field reconstruction](../references/papers/peraro-finite-field-reconstruction.md) and
[FiniteFlow](../references/papers/finiteflow.md)) treats a numerical algorithm as a black box
over prime fields `Z_p` and reconstructs the rational function it computes, never forming the
trace expansion; its cost tracks the size of the *final* answer. It measures exactly the open
question, how big the simplified form is. It does not make it small: a compact,
well-conditioned form needs partial fractions, spinor-variable ansätze
([De Laurentis–Maître](../references/papers/de-laurentis-maitre-spinor-ansatz.md)) and Horner/CSE
afterwards.[^n41-8]

**The box is the evaluator itself**, run over a test-only scalar `Fz`: an element of `Z_p`
carried with an `f64` shadow that decides every comparison the wavefunction routines branch on.
`num_complex` adjoins `i`, which for `p ≡ 3 (mod 4)` gives `F_{p²}`, whose Frobenius is complex
conjugation.[^n41-9]

- **Square roots were not a blocker**: the helicity sum sees only each leg's completeness
  relation. The root taken is `x^((p+1)/4)`, the root that is itself a square, and with
  `p ≡ 7 (mod 8)` the separately taken roots in `weyl_ixxxxx` stay consistent. This is a
  convention claim, pinned mod `p` by `fz_wavefunctions_satisfy_completeness`
  (`Σ u ū = p̸ + m`, `Σ v v̄ = p̸ − m`, both vector sums with HELAS's `n = (p⁰, −p⃗)`).
- **Missing roots**: about half the random draws per root have none in `Z_p`. The full-|M|²
  curves build every free leg so each root it takes is a rational square
  (`square_root_leg`), pinned by `square_root_legs_never_miss_a_root`; legs along the z axis
  are excluded (the root of `|p⃗|²` can come out `−|p⃗|`).
- **External massless vectors** break per-pair Lorentz invariance (the polarization sum's frame
  vector `n`), so per-pair fits on those rows were sampled in the partonic CM with an extra
  `(p·P)²` denominator. Their per-pair counts are those of an axial-type gauge and overstate
  what a Feynman-gauge symbolic program would get.
- **Oracles**: the per-pair fits lifted to `Q(i)` (Chinese remaindering, Wang's rational
  reconstruction, 8–12 primes, coefficients to 324 bits) match the f64 evaluator's per-pair
  `Σ_hel A_i A_j*` to 7.5e-15 / 1.2e-15 relative; `full_msq_matches_the_textbook_closed_form`
  checks `u ū → e⁺e⁻ g` against the crossed `e⁺e⁻ → q q̄ g` form exactly mod `p`; a second prime
  reproduces every count. **Blind spot**: the box is the evaluator, so this pins the field
  arithmetic, roots and fit, not the amplitudes, which the MadGraph amplitude gate holds. The
  2 → 4 per-pair fits were not lifted to `Q`.

## Results

`eval_m2` is the production helicity-pruned evaluator on the same container; it varies ±20%
between runs there and read about 1.9× note 32's M3 Max figures. Form timings are naive
scalar f64 evaluations (monomials one multiplication each, a sparse dot product, the known
denominators).[^n41-9][^n41-10]

| row | pairs | per-pair: terms, degree | per-pair ÷ `eval_m2` | full `|M|²`: poles losing a power, numerator degree, terms | full ÷ `eval_m2` |
|---|--:|---|--:|---|--:|
| `ee_to_mumu` | 3 | 9, 2 | 0.16 | 0 of 2, 4, 9 | **0.10** |
| `uux_to_epemg` | 10 | 968, 5 | 1.0 | 3 of 4, 4, 78 | **0.19** |
| `gu_to_epemu` | 10 | 261, 5 | 0.50 | 3 of 4, 4, 116 | **0.26** |
| `ee_to_mumua` | 36 | 3 608, 5 | 1.6 | 6 of 8, 9, 1 195 | 1.31 |
| `ud_to_epemud_qcd0` | 630 | 117 568, 4 | 22.6 | 0 of 15, 28, not fitted | — |
| `ee_to_mumu_tata_qcd0` | 325 | 85 435, 5 | 12.1 | 0 of 13, ≤ 25, not fitted | — |

- **Cancellations are real where a gauge boson is external.** On the 2 → 3 rows every massless
  propagator drops from the pairwise double pole to a single one (the eikonal and collinear
  structure); the llj numerators have 8–12× fewer terms than the per-pair form and two degrees
  less. The per-pair "break-even" for 2 → 3 was an artefact of the per-pair form.
- **One common denominator is the wrong container when pole sets differ**: `ee_to_mumua` has
  disjoint ISR and FSR collinear poles, so its numerator over their product reaches degree 9.
- **The four-fermion 2 → 4 rows have nothing to cancel**: with no external gauge boson no Ward
  identity ties the pairs, no propagator loses a power, and the dense ansatz at degree 28 would
  have 10⁸ columns. A partial-fraction form can do no better than the per-pair poles. External
  masses are not the cause (`ud_to_epemud_qcd0` is massless).
- **Per pair, 2 → 4 is large**: 190–260 terms per pair, about 10⁵ per row. Streaming the
  coefficients through FMAs and the low-rank factorisation (rank 58 on `ud_to_epemud_qcd0`) might
  together reach break-even on the best row. The low rank is the helicity method reappearing:
  `Σ_hel A_i A_j*` is a short sum of products.
- **Where compactness lives**: the famous compact forms are per-helicity amplitudes in spinor
  brackets. Squaring, summing helicities and rewriting brackets in invariants destroy that, and
  the helicity evaluator keeps the compact objects by squaring numbers. The trace-form question
  in its natural variables is whether per-helicity amplitudes are compact in momentum-twistor
  variables (De Laurentis–Maître): no conjugation, no root rejection, but HELAS's wavefunction
  normalisation must be divided out. Not scheduled; the prior expectation is that recursive
  evaluation still wins from 2 → 4.

## Recommendation and open work

- **2 → 2 and llj-type 2 → 3**: reconstruct, lift to `Q`, emit, after checking conditioning near
  the collinear poles against the f64 evaluator, since a common-denominator numerator cancels
  there. Worth building only if those rows' integration cost matters; the stage cap still
  applies ([backlog](../backlog/performance/generated-closed-form-msq-2to2.md)). Generated Rust for
  any larger form must be chunked ([rustc limits](rustc-limits-on-generated-code.md)).
- **Helicity sampling** (MadEvent's `nhel = 1`): one helicity combination per point drawn with
  adapted probability `p_h`, weighted by `1/p_h`. The cost side saves up to `N_kept / sharing`
  per point (about 16/1.8 ≈ 9× on the 2 → 6, where the evaluator dominates); the extra variance
  `ΔV = ∫ (Σ_h f_h²/p_h − f²) dx ≥ 0` is computable offline from `eval_hel_m2` vectors with the
  point-independent optimum `p_h ∝ √∫f_h²`, so `(V + ΔV)·c₁` against `V·c_sum` per row can be
  measured before building a sampler. It changes the estimator (no bit-for-bit comparison with
  banked σ; needs the ≥ 5-seed sweep) and forgoes cross-helicity sharing. It is in scope and
  tracked as [helicity Monte Carlo](../backlog/feature/nhel1-run-cards-refused.md), which also
  unlocks `wpwm_to_wpwmz_cw`.
- **Today**: integration sums helicities (pruned and recycled) and contracts colour; no channel
  is weighted by a single helicity. At event writing one helicity is selected per accepted event
  with probability `|M_h|² / Σ_h |M_h|²` (MadGraph's `SELECT_HEL`), with no effect on σ.[^n21]
  See [colour and helicity selection](../events/colour-and-helicity-selection.md) and
  [helicity sum and pruning](../amplitudes/helicity-sum-and-pruning.md).
- **The box is reusable**: an exact field-valued evaluator gives exact zeros (helicity pruning
  without a threshold), exact per-pair identities and exact gauge-cancellation checks.

Related papers: [CalcHEP](../references/papers/calchep.md),
[EXCALIBUR](../references/papers/excalibur.md).

[^n41-0]: Note 41 §0–§1 (2026-09-26, `db5fd03`).
[^n41-2]: Note 41 §2–§4.
[^n41-5]: Note 41 §5–§6.
[^n41-8]: Note 41 §8.
[^n41-9]: Note 41 §9, measured on PR #15 (squash `ef84a12`).
[^n41-10]: Note 41 §10, same PR.
[^n21]: Note 21, "Helicity & color handling".
