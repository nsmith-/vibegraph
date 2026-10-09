---
type: Working Note
original_type: "Feasibility Study"
title: "Completeness relations for the helicity-summed |M|²: feasibility"
description: "Whether a trace-form |M|² from completeness relations speeds integration: finite-field reconstruction measurements, the egglog verdict, and helicity sampling as the cheaper lever."
note: "41"
created: 2026-09-26
status: deprecated
tags: [matrix-element, helicity-sum, finite-fields, egglog, performance]
generated: {by: claude-code, at: 2026-09-26}
replaced_by: [performance/trace-form-msq-feasibility, validation/finite-field-evaluator, references/papers/de-laurentis-maitre-spinor-ansatz, performance/egglog-rewrite-stage, references/papers/helicity-recycling-mg5]
---
# 41 — Completeness relations for the helicity-summed |M|²: feasibility (2026-09-26)

The question (user): replace the explicit helicity sum in the integrand with
completeness relations — `Σ u ū = p̸ + m`, `Σ v v̄ = p̸ − m`, `Σ ε_μ ε*_ν = −g_μν
(+ p_μ p_ν / M²)` — so that `Σ_hel |M|²` becomes Dirac traces contracted into
scalar products of momenta. Would that speed up the integration stage a lot, and
is egglog the tool for it? Examined against what the evaluator and the
profiles have measured. Nothing here was built; the numbers are from notes 15,
30, 31 and 32, and the cost estimates in §3 are counting arguments, marked as
such.

**Status: backlog topic, low priority** (user, 2026-09-26). This note is the
whole record; it has no `TODO.md` entry. It carries two measurable follow-ups
(§7): the trace-form |M|² with its pre-registered kill and the final-size
scaling question, and helicity sampling as the direct lever on the expensive
rows. An idea the topic also covers: the explicit-invariant form of |M|² is the
natural input to a phase-space map derived from the integrand's own structure
rather than read off propagator poles. §8 (2026-10-03) assesses functional
reconstruction over finite fields (FiniteFlow) as the way to obtain the
simplified trace form, and turns §7.1's scaling question into a measurement
that needs no FORM pipeline. §9 (2026-10-03) is that measurement, for the
per-diagram-pair trace form: the evaluator run over a prime field, the pair
numerators reconstructed exactly, on two 2 → 2/2 → 3 rows lifted back to `Q` and
matched to the f64 evaluator, and timed against `eval_m2`
(`vibegraph-lib/tests/finite_field_msq.rs`). §10 (2026-10-03) reconstructs the
full `|M|²` instead, where cancellations between diagram pairs are visible.

## 0. Verdict

**Feasible for 2 → 2 and small 2 → 3, and a large per-point win there (roughly
10–50× on the matrix element). It does not speed up the integration stage "a
lot":** the evaluator is 50–62% of the integrand wherever it has been profiled
(note 30 §7.1), so an infinitely fast |M|² bounds the stage at 2.0–2.7×, and the
processes where the trace form wins are the ones that already integrate in
seconds. **Where the integration time actually goes (2 → 4 and up) the
per-pair trace form is measured too large (§9)**: on `ud_to_epemud_qcd0` and
`ee_to_mumu_tata_qcd0` the pair numerators hold 85–118 thousand terms of degree
4–5 in 9 dot products and 5 ε contractions, and a direct f64 evaluation of them
runs 12–23× slower than `eval_m2`. The full `|M|²` does not rescue them (§10):
on `ud_to_epemud_qcd0` and `ee_to_mumu_tata_qcd0` not one propagator (of 15
and 13) loses a power between pairs, and over the minimal common denominator
the numerator has degree 28 (and at most 25).
**Where the process has an external gauge boson the cancellations are large and
the full form wins**: on the llj subprocesses every massless propagator drops
from the pairwise double pole to a single one, the numerator over the minimal
denominator has 78–116 terms (the per-pair form has 261–968), and it evaluates
4–5× faster than `eval_m2`. 2 → 2 is 10× faster. The per-pair 2 → 3 verdict of §9
("break-even") was an artifact of the per-pair form.

**egglog is the wrong engine for the part that does the work.** Trace evaluation
and index contraction are a terminating, confluent normalization, where an
e-graph's keep-every-form semantics only costs memory (with AC blow-up on
polynomials). The e-graph fits the optimization of the *resulting* polynomial
and the choice problems (which momentum to eliminate, invariant basis), and
there it inherits the DAG-extraction blocker note 15 §4.1 already recorded.

The same goal, fewer helicity evaluations per point, has a cheaper route with
MadGraph precedent: **helicity sampling** (`nhel = 1`), which helps most on the
large processes where the evaluator dominates. §6 proposes a measurement for
each.

## 1. What the helicity sum costs today

The helicity sum is already close to the minimum for the helicity-amplitude
method:

- **Helicity recycling** (note 15 §2.2): every surviving helicity combination is
  baked into one arena and interned across combinations, so each distinct
  current is computed once per point. Sharing measured 1.8–2.8×.
- **Helicity pruning** (note 15 §2.3): combinations that vanish at generic
  points are removed, matching MadGraph's `NHEL` tables (e.g. 16/256 kept on the
  2 → 6).
- **Per-point cost vs MadGraph's MATRIX1** (note 32 §6): geomean 0.87×, from
  198 ns (`ee_to_zh`) through 1 296 ns (`gg_to_gg`), 4.4 µs (2 → 4), and 89–149 µs
  (2 → 6).

So what's left to gain has to come from changing the *method*, not from
recycling more.

## 2. What completeness buys, and what it does not

Written per diagram pair, the helicity-summed squared amplitude is

```
Σ_hel |M|² = Σ_{i,j} C_ij · Σ_hel A_i A_j*
```

with `C_ij` the diagram-level colour matrix. Completeness turns each
`Σ_hel A_i A_j*` into a product of closed Dirac traces (fermion lines of
diagram i joined to the conjugated lines of diagram j through `p̸ ± m`)
contracted with polarization sums. After traces and contractions the result is
a rational function of the invariants `p_a·p_b`, the couplings, and the
propagator denominators, plus `ε(p_a,p_b,p_c,p_d)` terms wherever γ5 enters
and four independent momenta exist (2 → 3 and up).

**Applied leg by leg, numerically, completeness does not help.** Summing one
external vector leg by `−g_μν` replaces its 2 (massless) or 3 (massive)
helicity states with 4 Lorentz components, and a spinor's `p̸ + m` is a rank-2
matrix, the same count as its two helicities. The win exists only when all
legs are traced out *symbolically* and the result simplified, which is a
computer-algebra pipeline, not an evaluator rewrite. One consequence: this
cannot be a set of rewrite rules on the existing HELAS arena (the
`helas::eval::egraph` seam). It is a different program, derived from the
diagrams, with the arena as its oracle.

## 3. Cost scaling (estimates, not measurements)

A fermion loop carrying `n` gamma matrices expands into `(n−1)!!` terms before
simplification (3, 15, 105, 945, 10 395, 135 135 for n = 4 … 14). An
interference pair joins each fermion line of diagram i with a line of diagram j,
so the loop length is about twice the number of vertices and propagators on the
line, plus the two external `p̸`. Diagram counts are MadGraph's
(`validation/madgraph/diagrams.json`):

| row | diagrams | pairs `D(D+1)/2` | longest loop (≈ n) | terms per pair, pre-simplification | today, ns/point |
|---|--:|--:|--:|---|--:|
| `ee_to_mumu` | 2 | 3 | 4 | ~10 | 240 |
| `gg_to_gg` | 6 | 21 | — (no fermions; see §5) | small; known closed form | 1 296 |
| `uux_to_epemg` (llj) | 4 | 10 | 8 | ~10² | 578 |
| `ee_to_mumua` | 8 | 36 | 8 | ~10² | 1 012 |
| `ee_to_mumu_tata_qcd0` | 25 | 325 | 8–12, three loops | 10³–10⁶ | 4 366 |
| `ud_to_epemud_qcd0` | 35 | 630 | 8–12, three loops | 10³–10⁶ | 4 104 |
| `uux_to_ccx_emmm_qcd0` | 579 | 167 910 | four loops | ≫10⁶ | 89 052 |
| `wpwm_to_wpwmz_cw` | 222 | 24 753 | — (bosonic, gauge cancellations) | large | — |

These columns are the *intermediate* expansion, which is paid once at compile
time. The per-point cost is set by the simplified expression, which §3.1
discusses. The historical ordering:

- **The 2 → 2 closed forms are tiny.** `gg → gg` is
  `(9/2) g⁴ (3 − tu/s² − su/t² − st/u²)`, and `e⁺e⁻ → μ⁺μ⁻` through γ/Z is a few
  dozen flops plus one Breit–Wigner. Against 240–1 300 ns today that is a
  10–50× per-point gain.
- **2 → 3 with a handful of diagrams stays compact.** `q q̄ → ℓℓ g` is tens to
  low hundreds of terms. The gain is likely still ≥ 5×.
- **2 → 4 is roughly break-even, and 2 → 6 is out of reach.** This is what
  happened historically. CompHEP/CalcHEP generate code for squared diagrams
  by traces and are competitive at low multiplicity, but they top out at around
  2 → 4 for this reason. Four-fermion LEP2 generators (EXCALIBUR, Berends–Kleiss–
  Pittau 1994) and every multi-leg generator since (MadGraph, COMIX, …) moved to
  helicity amplitudes for the same reason.

### 3.1 Intermediate swell is not final size

The expanded trace sum is routinely orders of magnitude longer than the
simplified |M|². Gauge invariance cancels whole classes of terms, and the
result is pinned by its poles and factorization limits, so it can collapse:

- `gg → gg`: 6 diagrams (one with a four-gluon vertex) and ghost or
  physical-polarization terms in the intermediate state, one line out.
- `e⁺e⁻ → q q̄ g`: `∝ (x₁² + x₂²)/((1 − x₁)(1 − x₂))`.
- `gg → ggg`, helicity-summed (Gottschalk–Sivers 1980; Berends et al. 1981):
  `∝ (Σ_{i<j} s_ij⁴) · Σ_{perms} 1/(s₁₂ s₂₃ s₃₄ s₄₅ s₅₁)` — 25 diagrams in,
  one symmetric expression out. It is compact because at five points every
  non-vanishing helicity amplitude is MHV (Parke–Taylor).

What the evidence does not show is that the collapse persists. The famous
compact forms are massless, ≤ 5 points, and mostly pure QCD or a single vector
boson. At six gluons non-MHV amplitudes enter and no comparably compact
helicity-summed form is known. `e⁺e⁻ → 4 partons` (Ellis–Ross–Terrano 1981) runs
to pages. None is known for the massive, finite-width, electroweak four-fermion
rows here. The compact forms also depend on the right variables (spinor
products, partial fractions in the `s_ij`); a naive monomial basis in the
3n − 10 independent invariants can stay large even when a short form exists.

Two consequences:

- **The 2 → 4 runtime verdict is open.** It is decided by the size of the
  simplified, CSE'd expression against the helicity program's ~10⁴ flops, and
  that is measurable (§7).
- **The swell itself can be skipped.** Functional reconstruction (Peraro,
  arXiv:1608.01902; FiniteFlow, arXiv:1905.08019; for spinor-helicity ansätze,
  De Laurentis–Maître, arXiv:1904.04067) samples a numerical evaluator at exact
  rational or finite-field kinematics and reconstructs the rational function
  directly, never forming the trace expansion. The existing evaluator would be
  the sampler. It is generic over `F: Real`, but finite-field evaluation needs
  a square-root-free parametrization of the kinematics (momentum twistors),
  because external spinors involve `√(E ± p_z)`.

## 4. The Amdahl bound on the integration stage

Note 30 §7.1's profiles put `helas::*` at **51.4%** (partonic σ gate), **52.2%**
(`pp_to_llj_dyn`), and **62.5%** (proton generation) of the busiest thread's
self time. The rest is PDF interpolation (14–19% on proton beams), the
multichannel map, the allocator, and kT clustering. Taking the matrix element
to zero gives at most:

| profile | evaluator share | stage speedup, ME → 0 | ME 10× faster |
|---|--:|--:|--:|
| integrate, partonic | 51% | 2.0× | 1.9× |
| integrate, `llj_dyn` | 52% | 2.1× | 1.9× |
| sample, proton | 62% | 2.7× | 2.3× |

The wall-time picture points the same way. Note 32 §7's time to 0.1% on σ is
0.43 s (`ee_to_mumu`), 4.0 s (`gg_to_gg`) and 19 s (`pp_to_jj`): the 2 → 2
rows are already cheap. `pp_to_llj` (87 s, capped) is a 2 → 3 the trace form
*could* handle, but that row's measured deficit is the sampler, at 14.5× more
points than MadGraph, not the per-point cost. The expensive per-point rows are
the 2 → 4 and 2 → 6, where §3 says the trace form does not win.

The dynamical-scale path also evaluates `eval_amp2` on every point (MadEvent's
per-configuration weight, `hadronic.rs::configuration_weights`,
`proton.rs` scale draw). A trace program can supply `AMP2_c` as the diagonal
configuration blocks of the same pair matrix, so that path does not rule the
approach out; it just has to be produced too.

## 5. Obstacles specific to this codebase

1. **External gluons.** `Σ ε_μ ε*_ν → −g_μν` is valid for at most one
   external non-abelian gauge boson. With two or more (`gg_to_gg`,
   `gg_to_ttx`, every `p p > j j` subprocess with gluons) the unphysical
   polarizations do not cancel. You need either the physical sum with a
   reference vector `−g + (p n + n p)/(p·n)`, which adds terms and a gauge
   choice, or squared ghost diagrams (the CompHEP route), which adds diagrams.
   Massive vectors in unitary gauge (`−g + p p/M²`) and photons are fine.
2. **γ5 and ε tensors.** Chiral couplings make traces with γ5. In four
   dimensions at tree level that is unambiguous, but from 2 → 3 on it yields
   `ε(p_a,p_b,p_c,p_d)` terms, which are one more convention claim to pin
   (AGENTS.md: convention claims are hypotheses).
3. **Cancellations get worse at the squared level.** Where the helicity program
   cancels gauge-growing terms at amplitude level (`ee_to_wpwm`, W/Z
   scattering), the trace form cancels them between squared interference terms,
   losing about twice as many digits (at √s = 1 TeV, `s/M_W² ≈ 150`: ~2 digits
   in the amplitude, ~4 in the square). An extraction cost that counts flops
   cannot see conditioning, so an e-graph could just as well pick the unstable
   form. AGENTS.md's rule applies: reformulate, don't loosen.
4. **Event generation still needs helicity amplitudes.** `SPINUP` selection
   draws from `eval_hel_m2`, and colour selection draws from `eval_jamp2`
   (per-flow `Σ_hel |JAMP_i|²`). A trace program can supply JAMP2 as flow-diagonal
   blocks but has no per-helicity content, so it would sit *beside* the
   per-helicity program, doubling the
   compile surface per subprocess.
5. **A side benefit: the trace form is Lorentz invariant.** It depends only on
   invariants, so it has no partonic-CM, beams-along-z contract (the pruned
   evaluator's frame-bound zeros, note 15 §2.3), and it is exactly symmetric
   under the beam-ordering reflection `proton.rs` evaluates twice.
6. **Running couplings and widths** are no obstacle. Keep couplings symbolic
   so the per-event `αs` pools still apply, and keep the propagator
   denominators as complex factors `1/(D_i D_j*)` per pair.

## 6. Where egglog fits

The trace pipeline has three stages, and the e-graph suits only the last:

- **Dirac algebra and trace evaluation**: contraction identities, `γ^μ γ_μ = 4`,
  the trace recursion, `p̸p̸ = p²`. These rules are terminating and confluent,
  so a normalizer computes the answer directly. Equality saturation keeps
  every intermediate form, and on sums and products it meets associativity and
  commutativity, the classic e-graph blow-up. egglog 2.0 does ship `MultiSet`
  and `BigRat` container sorts (`egglog-2.0.0/src/sort/{multiset,bigrat}.rs`), so a
  polynomial could be one AC-normal term, but then the e-graph only stores a
  normal form some other code computed. This stage is FORM's home ground (GPL,
  fine as an offline generator, awkward as a build dependency). In Rust it
  would be an in-tree normalizer, which the "never hand-write a standard
  primitive" rule permits because no suitable crate exists (Symbolica's licence
  restricts multi-core use).
- **Choosing a representation**: which momentum to eliminate by momentum
  conservation, which invariants form the basis, Schouten and Gram relations
  for ε terms, factoring out common propagator denominators. These are real
  choice problems with no canonical answer, which is what e-graphs are for.
  But every one of them pays off only through *sharing*, so it runs into note 15
  §4.1: tree-cost extraction cannot see the payoff, and greedy DAG extraction
  cannot co-commit across terms. The global/ILP extractor and a compute-aware
  cost model are still prerequisites.
- **Evaluating the polynomial**: the known-good methods are multivariate Horner
  schemes plus CSE (FORM's `Optimize`; Kuipers, Ruijl, Vermaseren,
  arXiv:1310.7007, with Horner orderings chosen by MCTS). The existing hash-cons
  CSE in `lower.rs` covers the CSE half.

So egglog helps in the middle stage and only once §4.1's extractor exists. It is
not on the critical path to a first measurement. §3.1 makes that middle stage
the one that matters, since it is where the short form is found. But finding a
compact form by saturating a huge expanded input is the hard way round;
reconstruction against an ansatz is the established tool, and an e-graph fits
as a post-pass on an already-reconstructed expression.

## 7. Recommendation

1. **If the trace form is pursued, measure before building.** Derive the
   trace-form `Σ_hel |M|²` and `AMP2_c` offline (FORM or by hand) for
   `uux_to_epemg`/`gu_to_epemu` (the llj subprocesses) and `ee_to_mumu`, and
   hand-code them behind the `Integrand` interface on a throwaway branch.
   - *Oracle*: the existing evaluator's per-pair `Σ_hel A_i A_j*` matrix, not
     just |M|². Per-pair agreement is sensitive to relative diagram phases and
     fermion signs, which |M|² can hide.
   - *Kill criterion (pre-registered)*: less than 1.3× on `pp_to_llj_dyn`
     CPU-to-target over ≥ 5 seeds. §4 predicts about 1.9× at best.
   - Only a pass justifies the symbolic pipeline (in-tree trace normalizer →
     polynomial → Horner/CSE, auto-selected per subprocess below a diagram-count
     threshold, per-helicity program kept for events).
   - *The scaling question (§3.1), separately*: obtain the simplified,
     Horner/CSE-optimized `Σ_hel |M|²` for `ee_to_mumua` (2 → 3, 8 diagrams)
     and `ee_to_mumu_tata_qcd0` (2 → 4, 25 diagrams), offline with FORM, and
     count its flops against the evaluator's per-point cost. If the 2 → 4
     expression is well under the helicity program, the "loses from 2 → 4"
     precedent does not hold for these rows and the per-point case reopens
     (the Amdahl cap of §4 still applies to the stage).
2. **For the stated goal, integration speed, measure helicity sampling first.**
   MadEvent's `nhel = 1` evaluates one helicity combination per point, drawn
   with adapted probabilities `p_h`, and weights by `1/p_h`. MadGraph chose it
   for `wpwm_to_wpwmz_cw`, and we refuse that card today (validation backlog,
   "Deferred coverage").
   - *Cost side*: one unexpanded combination instead of the recycled sum saves
     up to `N_kept / sharing` per point, e.g. about 16/1.8 ≈ 9× on the 2 → 6,
     which is exactly where the evaluator dominates.
   - *Variance side*: the extra variance is
     `ΔV = ∫ (Σ_h f_h²/p_h − f²) dx ≥ 0`, which is computable **offline** from
     `eval_hel_m2` vectors recorded on an existing run, with `p_h ∝ √∫f_h²` as
     the point-independent optimum.
   - This is a Stage-1 measurement in the same shape as the per-flow α item:
     report `(V + ΔV)·c₁` against `V·c_sum` per row before building a sampler.
     It also composes with the backlog's "helicity strata" axis, and with event
     generation, where the sampled helicity *is* the event's `SPINUP`.
   - Caveats: it changes the estimator, so it cannot be bit-for-bit against
     banked σ artifacts and needs the ≥ 5-seed sweep gate; and it forgoes the
     cross-helicity sharing, so on rows with high sharing the gain shrinks.
     Both are measurable.

## 8. Functional reconstruction as the route to the trace form (2026-10-03)

The question (user): FiniteFlow (Peraro, arXiv:1905.08019) as the way to reduce
the evaluator to a small rational function, especially the helicity-summed
`Σ_hel |M|²`. This section takes §3.1's pointer further. Nothing was built.

### 8.1 What the method is

FiniteFlow treats a numerical algorithm as a black box that maps rational inputs
to rational outputs, and evaluates it over prime fields `Z_p` (63-bit primes, so
arithmetic is exact and machine-word sized). From many such evaluations it
reconstructs the multivariate rational function the box computes: Thiele and
Newton interpolation on univariate slices fix the degrees, multivariate Newton
or sparse interpolation gets the coefficients, and rational reconstruction plus
the Chinese remainder theorem across a few primes lift them to `Q`. The
"dataflow graph" lets a chain of such algorithms (linear solves, substitutions,
Laurent expansions) be composed and sampled together. The symbolic intermediate
expression is never formed, so the trace expansion of §3 (`(n−1)!!` terms per
loop, `D²` pairs) costs nothing. What the method costs is the number of
black-box evaluations, which scales with the number of terms in the *final*
answer. A tree-level evaluator at µs per point can afford 10⁶–10⁷ samples per
prime.

### 8.2 What it does for this question

It answers §7.1's scaling question without building a trace pipeline. The
open question is how big the simplified 2 → 3 and 2 → 4 `Σ_hel |M|²` is, and
reconstruction measures exactly that size, because its cost tracks it. It does
not make the answer small. Reconstruction returns the function in whatever
representation it fits (expanded numerator over a denominator). Getting from
there to a compact, well-conditioned form is a separate step: multivariate
partial fractions (Heller, von Manteuffel, arXiv:2101.08283), spinor-variable
ansätze (De Laurentis, Maître, arXiv:1904.04067), and then Horner and CSE
(§6). §3.1's caveat stands: a short form may exist only in the right variables.

### 8.3 Exploitable structure: the denominator is known

For each diagram pair, `Σ_hel A_i A_j*` has the denominator `D_i D_j*`, the
product of that pair's propagators, which is read off the diagram set. Multiply
it out and the black box becomes a *polynomial* in the invariants, with its
degree bounded by mass dimension. Sparse polynomial interpolation is much
cheaper than general rational reconstruction, and it needs no denominator
guessing. Reconstructing per pair (or per colour-flow pair) and per coupling
monomial also keeps couplings out of the variable set. The per-pair results are
the oracle §7.1 already asks for, at no extra cost.

### 8.4 Obstacles specific to this codebase

1. **The evaluator cannot run over `Z_p` as written.** It is generic over
   `helas::repr::Real`, which is `num_traits::Float`: it needs `sqrt`,
   ordering and `min`, and none of those exist in a prime field. The
   wavefunction routines (`helas/wavefn.rs`) build spinors from
   `√(E ± |p⃗|)` and polarization vectors from `1/√2` and `p_T`. The summed
   `Σ_hel |M|²` is rational in the momentum components, but the individual
   wavefunctions are not. Two ways round it:
   - **Rational wavefunctions.** `|M|²` summed over helicities does not depend
     on each leg's basis or phases, only on the completeness relation. So any
     set of external states that is rational in the kinematics and satisfies
     `Σ u ū = p̸ + m` (and the vector analogue) gives the same sum. Massless
     spinors from a momentum-twistor parametrization are rational. A massive
     momentum splits as `p = k + (m²/2k·q) q` with `k` and `q` massless, which
     keeps it rational. Polarization vectors `⟨q|γ^μ|k]/⟨qk⟩` are rational in
     the spinors. This needs a second wavefunction set and a trait for the
     field with no `Float` bound: `+ − × ÷` and `i`, where `i` comes from
     `p ≡ 1 (mod 4)` or from working in `Z_p[i]`.
   - **Completeness-matrix legs.** Feed `p̸ ± m` and `−g + pp/M²` in as open
     external indices and contract at the end. That is rational with no change
     to the kinematics. §2 shows it is no faster at runtime, but the black box
     does not need to be fast. Its cost grows as `4ⁿ` open components, so it
     suits small `n` only.
2. **Parity-odd terms are not rational in the invariants.** From 2 → 3 on,
   `ε(p_a,p_b,p_c,p_d)` terms survive in the unpolarized sum wherever an
   imaginary coupling or width product multiplies them (finite-width
   propagators make `Im(D_i D_j*) ≠ 0`). Since `ε²` is a Gram determinant,
   `ε` is a square root of a polynomial in the `s_ij`. Either reconstruct in
   momentum-twistor variables, where `ε` is rational, or split the result as
   `R_even + ε·R_odd` and reconstruct the two separately. §5.2's
   convention-pinning requirement applies to the sign of `R_odd`.
3. **Masses and widths are parameters.** Pinning them to their f64 values,
   which are exact dyadic rationals, gives coefficients with large numerators
   and so needs many primes. Keeping `M²` and `MΓ` as variables adds two per
   resonance. With the pair-wise known denominators of §8.3, they enter only
   the numerator through on-shell masses and polarization sums, so the extra
   cost is degree, not new poles.
4. **Conditioning.** An expanded numerator in a monomial basis of the `s_ij`
   cancels between large alternating terms near collinear and soft limits and
   in the high-energy gauge cancellations of §5.3. Reconstruction cannot tell
   which form is stable. That is one more reason the partial-fraction step of
   §8.2 is part of the method rather than a polish pass, and the result has to
   be checked against the f64 evaluator across the phase space, not at one
   point.
5. **Tooling.** FiniteFlow and FireFly (Klappert, Lange) are C++. As offline
   generators for a measurement they are fine. In-tree reconstruction would
   need Rust modular arithmetic (a crate) and the reconstruction algorithms.
   The "no hand-written standard primitive" rule makes looking for a crate the
   first step; Symbolica has the algorithms, but its licence is the §6
   problem.

### 8.5 What it does not change

§4's Amdahl cap (2.0–2.7× on the stage even with `|M|²` free) and §5.4's need
to keep the per-helicity program for `SPINUP` and colour selection hold for any
fast `Σ_hel |M|²`, however it is obtained. Reconstruction changes how the
trace-form expression is found, not how much a fast one is worth.

### 8.6 Proposed measurement

This replaces the FORM route in §7.1's scaling question:

- **Box.** The completeness-matrix black box from §8.4.1, which needs no
  kinematic reparametrization. Run it over `Z_p` on rational phase-space
  points, for `ee_to_mumua` (2 → 3, 8 diagrams) and `ee_to_mumu_tata_qcd0`
  (2 → 4, 25 diagrams), per diagram pair, with the pair's propagators
  multiplied out.
- **Oracle.** The f64 evaluator's per-pair `Σ_hel A_i A_j*` at the same
  points, checked to the tolerance of its rounding.
- **Output.** Term counts of each pair's numerator, then the flop count after
  partial fractions and Horner/CSE, against the evaluator's measured per-point
  cost (`ee_to_mumua` 1 012 ns, `ee_to_mumu_tata_qcd0` 4 366 ns).
- **Kill.** §7.1's criteria are unchanged. If the 2 → 4 expression is not well
  below the helicity program's flops, the trace form stays confined to 2 → 2
  and small 2 → 3.

## 9. Measurement: the per-pair trace form by reconstruction (2026-10-03)

§8.6 run, with the field-arithmetic route of §8.4.1's first option rather than
completeness-matrix legs. Everything is in
`vibegraph-lib/tests/finite_field_msq.rs`: two fast tests (3 s) gate the
machinery, and `measure_trace_form` (ignored) prints the census below:

```
cargo test --release -p vibegraph-lib --test finite_field_msq measure_trace_form -- --ignored --nocapture
```

### 9.1 The box

The evaluator is unchanged. It runs over a test-only scalar `Fz`, an element of
`Z_p` carried with an `f64` shadow of the same computation. `num_complex` adjoins
`i`, and for `p ≡ 3 (mod 4)` that is the field `F_{p²}`, whose Frobenius is complex
conjugation. Arithmetic is exact mod `p`, and the shadow decides every comparison
the wavefunction routines branch on (`min`, `max`, `== 0`). Three points needed
care:

- **Square roots.** §8.4.1 named them as the blocker. They are not one, because
  the helicity sum only sees each leg's completeness relation. The root taken is
  `x^((p+1)/4)`, the root that is itself a square, which makes it multiplicative.
  With `p ≡ 7 (mod 8)` the separately taken roots in `weyl_ixxxxx` (`χ₀` against
  `√(2|p|(|p|+p_z))`) then stay consistent. That is a convention claim, and
  `fz_wavefunctions_satisfy_completeness` pins it mod `p`: `Σ u ū = p̸ + m`,
  `Σ v v̄ = p̸ − m`, the massive vector sum, and the massless one with HELAS's
  `n = (p⁰, −p⃗)`. Its first run failed on the massive antiparticle, and the
  same identity in f64 showed the test was at fault (a consumed poison flag), not
  the root convention. A leg whose roots do not exist in `Z_p` (about half the
  draws per root) is redrawn on its own; redrawing whole points instead cost 100×
  on the massive-τ row.
- **External massless vectors** break per-pair Lorentz invariance. The HELAS
  polarization sum `−g + (p n + n p)/(p·n)` has a frame vector `n`, and its gauge
  terms cancel only in the sum over diagrams. The first 2 → 3 fit failed to
  degree 8 for this reason. In the partonic CM frame `n` is rational in `p` and
  `P = p₁ + p₂`, so those rows are sampled there, with one more known
  denominator `(p·P)²` per vector leg. Their counts are therefore those of an
  axial-type gauge, not of the Feynman-gauge `−g` a symbolic trace program would
  use, and they overstate what such a program would get.
- **Denominators** are read off the diagram set: `D_i = Π (q² − M² + i M Γ)`, with no
  width on spacelike lines, as `lower.rs` does. A wrong denominator would make
  the numerator non-polynomial and the fit would fail, so the fit checks them.

The ansatz for `N_ij = D_i D_j* Σ_hel A_i A_j*` is dense: every monomial of total
degree `≤ d` in the independent dot products, the last momentum eliminated, plus
every `ε(p_a,p_b,p_c,p_d)` times monomials of degree `≤ d − 2`. It is solved by
exact row reduction mod `p` on random rational points in random frames, with 12
held-out points; `d` grows until they agree. Dependent columns (Schouten
relations among the ε's) drop out as non-pivots. Sparse interpolation was not
needed: the largest system is 3 114 × 3 102.

**Oracles.**

- `per_pair_numerators_lift_to_the_f64_evaluator` lifts the fits to `Q(i)` by
  Chinese remaindering and Wang's rational reconstruction, confirmed by one
  further prime. That takes 8 primes for `ee_to_mumu` and 12 for `ee_to_mumua`,
  whose coefficients reach 324 bits. It then evaluates the result in f64 at
  fresh physical points against the evaluator's own per-pair `Σ_hel A_i A_j*`.
  Worst deviation, relative to the largest term: `7.5e-15` and `1.2e-15`.
- On every measured row a second prime reproduces every pair's term count.

Blind spots: the box *is* the evaluator, so this pins the field arithmetic, the
roots and the fit, not the amplitudes; the amplitude gate holds those against
MadGraph. A coefficient divisible by `p` reads as zero, which is why the counts
must agree across two primes. The lift is not run on the 2 → 4 rows.

### 9.2 Results

The default SM card, this container (4 cores, release profile). `eval_m2` is the
production, helicity-pruned evaluator; here it measures about 1.9× note 32's
figures (`ud_to_epemud_qcd0` 8.0 against 4.1 µs). The per-pair column times a
plain scalar f64 evaluation of the fitted form: monomials built one
multiplication each, every pair's numerator a sparse complex dot product with
them, times `1/D_i · 1/D_j*`.

| row | diagrams / pairs | variables | degree | terms (mean / max per pair) | best basis | rank | `eval_m2` | per-pair | ratio |
|---|--:|---|--:|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 2 / 3 | 2 | 2 | 9 (3 / 3) | 6 | 2 | 388 ns | 61 ns | 0.16 |
| `ee_to_mumua` | 8 / 36 | 5 + 1 ε | 5ᵃ | 3 608 (100 / 128) | 2 538 | 15 | 2 057 ns | 3 216 ns | 1.6 |
| `uux_to_epemg` | 4 / 10 | 5 + 1 ε | 5ᵃ | 968 (97 / 105) | 403 | 8 | 1 082 ns | 1 066 ns | 1.0 |
| `gu_to_epemu` | 4 / 10 | 5 + 1 ε | 5ᵃ | 261 (26 / 40) | 187 | 8 | 1 180 ns | 592 ns | 0.50 |
| `ud_to_epemud_qcd0` | 35 / 630 | 9 + 5 ε | 4 | 117 568 (187 / 404) | 88 619 | 58 | 8.0 µs | 182 µs | 22.6 |
| `ee_to_mumu_tata_qcd0` | 25 / 325 | 9 + 5 ε | 5ᵇ | 85 435 (263 / 429) | — | 221 | 9.8 µs | 119 µs | 12.1 |

- ᵃ Includes the `(p·P)²` gauge factor of the external photon or gluon.
- ᵇ The 36 pairs among the 8 diagrams with a Z on the `τ⁺τ⁻` pair need degree 5:
  the unitary-gauge `q q/M_Z²` term survives on a massive line through the axial
  current (`∝ m_τ`). Every other pair fits at degree 4.

Columns:

- **Best basis:** each pair counted in whichever choice of eliminated momentum
  suits it best, which is optimistic, since evaluating needs every basis's
  monomials.
- **Rank:** the rank of the pairs × monomials coefficient matrix, the number of
  linear forms in the monomials every numerator is a combination of.

The 2 → 4 ansätze have 990 columns (940 independent) and 3 102 (2 826).
Reconstruction took 7 s and 78 s per prime.

### 9.3 What the numbers say

- **The reconstruction is cheap and the answer is exact.** Seconds to a minute
  per prime at 2 → 4, against the evaluator as it stands. §8.1's sample budget is
  not the constraint; the dense row reduction is (`O(n³)` in the ansatz size), and
  a 2 → 5 row at degree 5 in 14 + 15 variables would want sparse interpolation.
- **2 → 2 is small and fast**, as §3 said: 3 terms per pair, 6× faster than
  `eval_m2` even in this naive form. The form also lifts to exact rationals, so
  it could be generated rather than hand-derived.
- **2 → 3 is break-even per point in this form** (0.5–1.6×), with the
  external-vector rows inflated by the gauge factor. Per-pair best bases cut
  30–60%. §10 supersedes this: the full `|M|²` on the same rows is 4–5× faster.
- **2 → 4 is large.** About 10⁵ terms per row, a mean of 190–260 per pair, at
  degree 4–5. As written the form is 12–23× slower than `eval_m2`. The scalar loop
  runs at 1.5 ns per term. A layout that streams the coefficients through 4-wide
  FMAs might reach ~0.3 ns, and on `ud_to_epemud_qcd0` the rank-58 factorisation
  `N = U (V x)` cuts the arithmetic by about 2.4×. Together those reach roughly
  break-even on the best row, after real engineering, against an evaluator that
  has its own SIMD lane path. No representation measured here gives the
  order-of-magnitude win that would matter, and §4 caps the stage at 2.0–2.7×
  regardless. The precedent in §3 ("2 → 4 is roughly break-even") now stands as a
  measurement for these two rows.
- **Merging pairs that share a denominator does nothing here.** Every pair on
  every row has a distinct `D_i D_j*` (γ and Z lines differ in mass), so the 2 → 4
  counts are already the merged ones.
- **The low rank is the helicity method reappearing.** 58 linear forms span 434
  monomials across 630 pairs because `Σ_hel A_i A_j*` is a short sum of products.
  Factorising the trace form back into that structure is what helicity
  amplitudes already do.

### 9.4 What is not measured

- **Cross-pair simplification** (measured in §10). The compact classical forms (§3.1) come from
  gauge cancellations between pairs and from partial fractions, which a per-pair
  ansatz cannot express. The gauge-invariant object is the full `|M|²`, and its
  natural denominator is the product of every distinct propagator: 8 on
  `ee_to_mumua`, 13 on `ee_to_mumu_tata_qcd0`. Reconstructing it needs the
  rational-function route (Thiele along lines to fix the degrees, then sparse
  interpolation) rather than a dense ansatz. That is the remaining open question
  of §3.1. It is measurable with this box, but the cost is now known to be the
  right order to spend only if the 2 → 4 rows matter.
- **Feynman-gauge external vectors.** These are not reachable through the HELAS
  polarization sum. The completeness-matrix legs of §8.4.1 would give them.
- **Spinor-helicity variables** (De Laurentis–Maître) are a different basis the
  fit could be run in. Momentum twistors would also remove the root rejection.

### 9.5 Recommendation (updates §7.1)

- **Drop the trace form for the 2 → 4 rows.** The per-pair form measured 12–23×
  slower, and the levers above at best reach parity. Only the cross-pair
  measurement of §9.4 could reopen it, and §4's cap makes that a low-value
  question.
- **For 2 → 2, a generated closed form is now cheap to obtain:** reconstruct,
  lift, emit. It is worth building only if a 2 → 2 row's integration cost ever
  matters, and today they integrate in seconds (§4).
- **Helicity sampling (§7.2) remains the direct lever** on the expensive rows.
- **The box itself is reusable.** An exact, field-valued evaluator is an oracle
  that floating point cannot provide: exact zeros (helicity pruning without a
  threshold), exact per-pair identities, and exact gauge-cancellation checks.

## 10. The full |M|² (2026-10-03)

§9.4 left cancellations *between* diagram pairs unmeasured. A per-pair numerator
cannot show them: the gauge cancellation that turns a collinear `1/s²` into
`1/s` happens only in the sum. Here the box is `eval_m2` itself, over the same
field. The full `|M|²` is gauge invariant, so it is Lorentz invariant even with
external gluons and photons: no frame restriction and no `(p·P)²` factor. It is
reconstructed over its *minimal* common denominator, in three exact steps.

### 10.1 Method

1. **Pole exponents.** For each distinct propagator (`q² − m²`, or
   `|q² − M² + iMΓ|²` for a line with a width), the exponent it keeps in `|M|²`.
   `|M|²` is reconstructed as a univariate rational function along a random
   rational curve through phase space. That uses Thiele interpolation over `Z_p`,
   2 → 4 needs about 1 400 nodes, and no degree has to be known in advance. The
   exponent is then the multiplicity of the propagator's polynomial in `|M|²(t)`'s
   denominator.

   Two curve artifacts had to be handled. A massless momentum is an energy times
   a direction, so invariants share energy factors. And closing momentum
   conservation by a lightlike split puts the closing pair's invariant into every
   invariant of those two legs. A propagator is therefore tested only on the part
   of its polynomial it shares with no other pairwise invariant, and on a curve
   that does not close on it; two curves, an s-type and a t-type closing, cover
   every line. A first version without this read the photon's `1/s²` in
   `ee_to_mumu` as exponent 0.

   A node is usable only if every wavefunction root exists in `Z_p`: about half
   the time per root, and a massive leg takes three. On `ee_to_mumu_tata_qcd0`
   that left one node in ~1 000 (measured: 0.50 per massless leg, 0.12 per τ),
   and the run was stopped after 45 minutes. The curves therefore build every
   free leg so that each root it takes is a rational square
   (`square_root_leg`). The direction has rational half-angles, so
   `(1 ± n_z)/2 = cos²(θ/2), sin²(θ/2)`. A massless leg has `E = 2w²`, so
   `E + p_z = (2w cos θ/2)²`. A massive one has `E ± |p⃗| = a², m²/a²`. Both
   signs a perfect square's root can take land on squares, so nothing is
   rejected except the closing pair (1/2 each). This is a kinematics choice in
   the test; the wavefunction conventions are untouched.
   `square_root_legs_never_miss_a_root` pins the claim: no root missing and
   completeness intact, for massless and massive spinors and vectors, over two
   primes. It also found the one exception, a leg along the z axis: there the
   root of `|p⃗|²` can come out `−|p⃗|`, making `|p⃗| + p_z = 0` on the regular
   branch the f64 shadow takes, so such legs are excluded. With these legs the
   exponents on the other rows come out identical on fresh curves, and the τ
   row takes 69 s.
2. **Numerator degree.** Under `p → μ² p` (massless legs only, which stay on shell)
   `Q·|M|²` must be a polynomial in `s = μ⁴`. Its degree is the numerator's total
   degree, and the run asserts the polynomial property, which checks `Q`.
3. **Numerator.** A dense fit at that degree, the parity-even and -odd parts
   separated by evaluating each point and its mirror image (`p⃗ → −p⃗`).

**Oracles.**

- `full_msq_matches_the_textbook_closed_form` checks the box against physics
  rather than against the evaluator. `u ū → e⁺e⁻ g / z` must satisfy
  `|M|² · s₃₄ s₁₅ s₂₅ / (s₁₃² + s₁₄² + s₂₃² + s₂₄²) = const`, the crossing of
  `e⁺e⁻ → q q̄ g`, and it does, exactly mod `p`, at 20 points. This pins the
  single poles the census reports, through `eval_m2`'s colour and helicity sums.
- A second prime reproduces every count.
- The scaling step's polynomial check confirms each `Q`.

### 10.2 Results

`measure_full_msq` (ignored), release profile, this container:

| row | propagators that lose a power | `deg Q` full (pairwise lcm) | numerator degree | terms | per-pair terms (§9) | `eval_m2` | full form | ratio |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 0 of 2 | 4 (4) | 4 | 9 | 9 | 441 ns | 45 ns | 0.10 |
| `uux_to_epemg` | 3 of 4 | 5 (8) | 4 | 78 | 968 | 1 141 ns | 215 ns | 0.19 |
| `gu_to_epemu` | 3 of 4 | 5 (8) | 4 | 116 | 261 | 928 ns | 240 ns | 0.26 |
| `ee_to_mumua` | 6 of 8 | 10 (16) | 9 | 1 195 (90 of them ε) | 3 608 | 2 115 ns | 2 778 ns | 1.31 |
| `ud_to_epemud_qcd0` | 0 of 15 | 30 (30) | 28 | not fitted | 117 568 | — | — | — |
| `ee_to_mumu_tata_qcd0` | 0 of 13 | 26 (26) | ≤ 25 | not fitted | 85 435 | — | — | — |

- **Losing a power** means the propagator goes from the pairwise double pole to
  a single pole. The Z Breit–Wigners keep theirs everywhere.
- **The full-form timing** is a scalar f64 evaluation: the monomials a term
  uses, built one multiplication each, a sparse dot product, then division by
  `Q`. `eval_m2` varies ±20% between runs here (388–533 ns on `ee_to_mumu`).
- **`ud_to_epemud_qcd0`:** the dense ansatz at degree 28 has 1.2 × 10⁸ even and
  3.5 × 10⁸ odd columns.
- **`ee_to_mumu_tata_qcd0`:** poles from 3 194 curve nodes in 69 s. The scaling
  family does not keep massive legs on shell, so the numerator degree is a
  bound: §9's per-pair fits have degree ≤ 5 over pair denominators of degree 6,
  so `|M|²` has degree ≤ −1 and the numerator ≤ 25. A dense ansatz at 25 has
  5.2 × 10⁷ even and 1.4 × 10⁸ odd columns.

### 10.3 What the numbers say

- **The cancellations are real where gauge bosons are external.** On the 2 → 3
  rows every massless propagator drops from the pairwise double pole to a single
  pole: the eikonal/collinear structure, the `1/s` of `γ* → ℓℓ` included. On the
  llj subprocesses the full numerator has 8–12× fewer terms than the per-pair
  form and is two degrees lower. It evaluates 4–5× faster than `eval_m2`, where
  §9 had per-pair break-even.
- **One common denominator is the wrong container when the pole sets differ.**
  `ee_to_mumua` has initial- and final-state radiation, whose collinear poles
  are disjoint. Over their product the numerator reaches degree 9 and 1 195 terms,
  and the form is 1.3× slower than `eval_m2`. Partial fractions in the
  propagators (Leinartas; MultivariateApart, arXiv:2101.08283) would keep the ISR,
  FSR and interference pieces each over their own poles. Not measured.
- **The four-fermion 2 → 4 rows have nothing to cancel.** With no external
  gauge boson, no Ward identity ties the pairs together, and no propagator loses
  a power, on `ud_to_epemud_qcd0` (15 propagators) or `ee_to_mumu_tata_qcd0`
  (13). The minimal common denominator is the full pairwise lcm (degree 30 and
  26), so the numerator has degree 28 and at most 25. That form is far larger
  than the per-pair one (§9), and a partial-fraction form can do no better than
  the per-pair poles already do. What remains unmeasured on these rows is
  numerator-level cancellation inside a partial-fraction basis. With every pole
  intact, no mechanism for it is known here.
- **External masses are not what blocks it.** `ud_to_epemud_qcd0` has only
  massless external fermions and shows the same nothing-cancels result as the
  massive-τ row. The τ mass adds degree (the unitary-gauge `q q/M_Z²` term,
  §9.2), not structure.

### 10.4 Where compactness lives

The famous compact forms (Parke–Taylor; four-fermion amplitudes like
`⟨13⟩²[24]/(s s′ s″)`) are **per-helicity amplitudes in spinor brackets**, not
helicity-summed `|M|²` in dot products. Two steps between them and what §9–§10
measured destroy that compactness:

- **Squaring.** An amplitude of `n` terms gives about `n²` cross terms in
  `|A|²`, and the sum over helicities adds the expansions of different helicity
  configurations, which share no structure in dot products.
- **Rewriting in invariants.** `|⟨ij⟩|² = s_ij`, so brackets are square roots of
  invariants. Their phases (`⟨ij⟩/[ij]`) have no rational form in the `s_ij`, and
  the result expands into a much larger polynomial in them.

The helicity evaluator keeps the compact objects, evaluating the amplitudes
numerically and squaring them as numbers. That is why it wins at 2 → 4. The low
rank of §9's coefficient matrix (58 linear forms spanning 630 pairs on
`ud_to_epemud_qcd0`) is that structure showing through the trace form.

So the trace-form question in its natural variables is: **are the per-helicity
amplitudes compact in spinor variables?** That is the De Laurentis–Maître
approach (arXiv:1904.04067): reconstruct `A_h` divided by a helicity-carrying
prefactor as a rational function of momentum-twistor variables. Two properties
make it a different measurement from this one:

- **No conjugation.** The box is the per-helicity amplitude (`eval_amplitude`)
  over `F_p`, with no `|·|²`. Momenta can be complex twistor points, and the
  real-momenta/Frobenius machinery and the root rejection of §8.4.1 and §10.1 go
  away.
- **Spinor-built external wavefunctions.** HELAS's `√E` normalisation and phase
  conventions must be replaced, or divided out per helicity. Massless spinors
  scaled by `√p⁺` (`u₊ = (p⁺, p_x + i p_y)`) and polarizations written as
  `⟨n|γ^μ|k]` with HELAS's reference `n` are polynomial in the momentum, with
  a rational weight per helicity. A massive helicity state keeps one root,
  `|p⃗|`, because the helicity axis is `p⃗/|p⃗|`. The per-diagram MadGraph
  amplitude gate would have to divide out the per-helicity factor. This is
  where the cost of such a study sits.

Prior expectation: recursive numerical evaluation still wins from 2 → 4 on,
because it shares sub-currents across diagrams in a way closed forms do not. The
precedent is mixed, though: some NLO codes use closed-form tree amplitudes at
low multiplicity. Not scheduled.

### 10.5 Recommendation (updates §9.5)

- **2 → 2 and 2 → 3 with an external gauge boson:** a reconstructed closed form
  of the full `|M|²` is 4–10× faster per point than `eval_m2`, and this pipeline
  produces it. Reconstruct, lift to `Q` (§9.1), emit. Before emitting, check its
  conditioning near the collinear poles against the f64 evaluator, since a
  common-denominator numerator cancels there. `pp_to_llj`'s subprocesses are the
  candidates. §4's cap still bounds the stage at about 2×, and that row's
  measured deficit is the sampler.
- **Multi-radiator 2 → 3 (`ee_to_mumua`):** worth it only with partial fractions.
- **2 → 4 four-fermion:** dropped, now on the full `|M|²` as well as per pair.
- **Analytic compactness, if pursued, means per-helicity amplitudes in spinor
  variables (§10.4)**, not `|M|²` in invariants.
- **Helicity sampling (§7.2) remains the lever** on the expensive rows. The
  per-helicity program stays in every case for `SPINUP` and colour (§5.4).

## References

- Frederix et al., "Speeding up MadGraph5_aMC@NLO", EPJC 81:435 (2021),
  arXiv:2102.00773: helicity recycling (note 15 §1.1).
- Belyaev, Christensen, Pukhov, "CalcHEP 3.4", arXiv:1207.6082; Boos et al.,
  "CompHEP 4.4", hep-ph/0403113: squared-diagram trace codegen and its
  multiplicity limit.
- Berends, Kleiss, Pittau, "EXCALIBUR", Nucl. Phys. B424 (1994) 308, and
  Comput. Phys. Commun. 85 (1995) 437: four-fermion production by helicity
  amplitudes.
- Peraro, arXiv:1608.01902, and FiniteFlow, arXiv:1905.08019: functional
  reconstruction over finite fields; De Laurentis, Maître, arXiv:1904.04067:
  analytic forms from numerical evaluations with spinor-helicity ansätze.
- Klappert, Lange, "FireFly", arXiv:1904.00009: finite-field rational
  function reconstruction (C++).
- Heller, von Manteuffel, "MultivariateApart", arXiv:2101.08283: multivariate
  partial fractioning of reconstructed rational functions; Leinartas, "Factorization
  of rational functions of several variables into partial fractions" (1978), the
  decomposition it implements.
- Kuipers, Ruijl, Vermaseren, "Code optimization in FORM", arXiv:1310.7007:
  Horner + CSE for large polynomials.
- Zhang et al., egglog (note 14); note 15 §4.1 for the extraction blocker.
