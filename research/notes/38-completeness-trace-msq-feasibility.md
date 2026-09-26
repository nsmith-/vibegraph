# 38 — Completeness relations for the helicity-summed |M|²: feasibility (2026-09-26)

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
rather than read off propagator poles.

## 0. Verdict

**Feasible for 2 → 2 and small 2 → 3, and a large per-point win there (roughly
10–50× on the matrix element). It does not speed up the integration stage "a
lot":** the evaluator is 50–62% of the integrand wherever it has been profiled
(note 30 §7.1), so an infinitely fast |M|² bounds the stage at 2.0–2.7×, and the
processes where the trace form wins are the ones that already integrate in
seconds. **Where the integration time actually goes (2 → 4 and up) the trace
form's runtime cost is unknown, not settled**: the *intermediate* expansion
grows as the square of the diagram count times a factorial in trace length,
but that is a compile-time cost, and the simplified result can be far shorter
(§3.1). No compact forms are known for the massive/electroweak 2 → 4 rows, and
every multi-leg generator evaluates them numerically, but that is precedent,
not a bound. The final-size scaling is a measurement (§7).

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
- Kuipers, Ruijl, Vermaseren, "Code optimization in FORM", arXiv:1310.7007:
  Horner + CSE for large polynomials.
- Zhang et al., egglog (note 14); note 15 §4.1 for the extraction blocker.
