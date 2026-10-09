---
type: Physics Convention
title: External wavefunctions and propagators
description: "HELAS/ALOHA external vector/scalar wavefunctions and storage; Weyl-basis fermion, Feynman/unitary-gauge vector and scalar propagators with fixed-width denominators."
status: draft
tags: [helas, wavefunctions, propagators, conventions, aloha]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n10-prims, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/10-lorentz-runtime-eval-plan.md#L140-L236", title: "Note 10 §4.1–§4.6 (vxxxxx, sxxxxx, Dirac/massless/massive propagators, GammaV)"}
  - {id: n10-open, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/10-lorentz-runtime-eval-plan.md#L579-L605", title: "Note 10 §11 (open questions)"}
  - {id: n12-causes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L37-L87", title: "Note 12, root causes (momentum routing, chain-phase normalisation)"}
  - {id: n12-stand, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L165-L173", title: "Note 12, where things stand"}
  - {id: code-wavefn, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/wavefn.rs", title: "DiracWf, VectorWf::vxxxxx, ScalarWf::sxxxxx"}
  - {id: code-kernel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/kernel.rs", title: "propagate_*_bare kernels"}
  - {id: code-run, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs", title: "build_external_core"}
  - {id: code-vertex, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/vertex.rs", title: "Reference HELAS/ALOHA ports (jioxxx, ffv2_3, fvixxx, …)"}
  - {id: code-diagram, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/diagram.rs", title: "Diagram::fermion_line_sign (massive reversed spines)"}
  - {id: aloha-om, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_writers.py#L624-L626", title: "ALOHA writer: OM = 1/M**2 for the massive vector numerator"}
---

# External wavefunctions and propagators

The external wavefunctions are transcriptions of the HELAS routines (KEK
report 91-11, Appendix A: `ixxxxx`, `oxxxxx`, `vxxxxx`, `sxxxxx`), so an
amplitude built from them agrees with MadGraph's generated code diagram by
diagram and helicity by helicity, phases included. Metric `(+,−,−,−)`;
four-vectors and vector wavefunctions are stored `[E, x, y, z]`; spinors as
four Weyl components (0,1 left-chiral; 2,3 right-chiral). HELAS packs momentum
into extra array entries; here it is a separate `momentum` field[^code-wavefn].

## Storage: the flow-signed momentum

Every constructor takes a flow flag and stores flag × p, not p. The momentum
passed in is always physical.

| constructor | flag | +1 | −1 | stored |
|---|---|---|---|---|
| `DiracWf::from_momentum` | `nsf: Charge` | `Particle` (u) | `Antiparticle` (v) | `nsf · p` |
| `VectorWf::vxxxxx` | `nsv: i32` | outgoing | incoming | `nsv · p` |
| `ScalarWf::sxxxxx` | `nss: i32` | outgoing | incoming | `nss · p` |

The signed momenta let off-shell currents add and subtract leg momenta
directly. A flow-in (ket) current subtracts the boson momentum
(`fvixxx`: q = fi − v) and a flow-out (bra) current adds it (`fvoxxx`:
q = fo + v); the vector current from a fermion pair is `jmom = fo − fi`.
Getting this routing wrong mis-routes every line with more than one vertex and
puts spurious poles in the amplitude[^n12-causes].

## External legs

`build_external_core` (`helas/eval/run.rs`) dispatches on the UFO spin code[^code-run]:

- **Spin 0** → `ScalarWf::sxxxxx(p, nss)`, value `1 + 0i`, no helicity.
  `nss = −1` for an incoming leg, `+1` outgoing.
- **Spin ½** → a ket (`InDiracWf`, HELAS `ixxxxx`) iff
  `is_incoming == is_particle`, otherwise a bra (`OutDiracWf`, `oxxxxx`, the
  Dirac conjugate `ψ†γ⁰` of the same construction). Helicity is ±1 only.
- **Spin 1** → `VectorWf::vxxxxx(p, m, λ, nsv)`, `nsv = −1` incoming, `+1`
  outgoing.

Helicity states per particle come from `Particle::helicity_states`: a scalar
`{0}`, a fermion `{−1, +1}`, a massive vector `{−1, 0, +1}`, a massless vector
`{−1, +1}`. A spin-3/2 particle gets `EvalError::UnsupportedSpin`.

### Vector polarisations

With `(ê_θ, ê_φ, p̂)` the right-handed triad of p⃗ and `p_T = √(p_x² + p_y²)`:

- Transverse (λ = ±1, massive and massless): `ε⁰ = 0`,
  `ε⃗ = (−λ ê_θ + i nsv ê_φ)/√2`.
- Longitudinal (λ = 0, massive only): `ε = (|p⃗|, E p̂)/m`, real and
  independent of `nsv`.
- At rest: `ε(±1) = (0, −λ, i nsv, 0)/√2`, `ε(0) = (0, 0, 0, 1)`.
- Along z (`p_T = 0`): `ê_θ = x̂`, `ê_φ = sgn(p_z) ŷ` with `sgn(0) = +1`. That is
  the limit from the side where p_x and p_z share a sign. The other side would
  negate the whole transverse vector, so this is a HELAS convention that must
  be kept.

`nsv` enters only through the imaginary parts: the incoming (`nsv = −1`) state
is `ε^μ(p, λ)` and the outgoing (`+1`) state is its complex conjugate. Every
state satisfies `ε·p = 0` and `ε(λ)·ε(λ′)* = −δ`. The massive states sum to
`−g^{μν} + p^μp^ν/m²`, the unitary-gauge numerator. The massless basis fixes
`ε⁰ = 0` (temporal gauge in the frame the momenta are given in), so its sum
carries extra `p^μ n^ν` terms that drop out of a gauge-invariant sum by the Ward
identity.

Edge cases: the massive branch clamps `|p⃗| → min(E, |p⃗|)` and
`p_T → min(|p⃗|, p_T)`, and the massless branch uses E for |p⃗|, so the clamps
bite only on rounding-level inconsistencies. `vxxxxx` with `nhel = 0` and
`vmass = 0` returns a non-zero vector that is not a polarisation state; only
`nhel = ±1` is meaningful there. HELAS's `nhel = 4` mode is not implemented.

## Propagators

Every propagator carries the same `−i/D` phase, with the fixed-width
Breit–Wigner denominator `D = q² − m² + imΓ` (`helas/eval/kernel.rs`)[^code-kernel]:

| line | kernel | action on the current |
|---|---|---|
| Dirac, ket | `propagate_fin_bare` | `−i (q̸ + m) ψ / D` |
| Dirac, bra | `propagate_fout_bare` | `−i ψ̄ (q̸ + m) / D` |
| massless vector (Feynman gauge) | `propagate_vector_bare`, `m = 0` | `−i ε^μ / q²` (no width) |
| massive vector (unitary gauge) | `propagate_vector_bare`, `m > 0` | `−i [ε^μ − q^μ (q·ε)/m²] / D` |
| scalar | `propagate_scalar_bare` | `−i φ / D` |

Conventions in that table:

- **The longitudinal term divides by real `m²`.** This is ALOHA's form: the
  generated routines declare `OM3 = 0; if (M3.ne.0) OM3 = 1/M3**2` with a real
  mass[^aloha-om]. Only the denominator is complex. The legacy HELAS port
  `helas/vertex.rs::jioxxx` instead divides by complex `m² − imΓ`; it is a
  test reference, not on the production path[^code-vertex], and its comparison test
  (`test_eval_jioxxx` in `run.rs`) uses massless external fermions whose
  current is conserved, `q·J = 0`, so it cannot tell the two forms apart.
- **The massless vector is Feynman gauge.** It ignores the width and does not
  guard `q² = 0`; the caller must keep the line off-shell.
- **No complex-mass scheme.** `set complex_mass_scheme True` in a proc card is
  refused by the supported-card check (`diagrams/check.rs`).
- **One phase for every chain type.** The Dirac and scalar propagators carry
  the same `−i/D` as the vector, so every off-shell chain type sits in phase
  with MadGraph's. The compensating signs that keep scalar chains right live in
  the scalar-sink vertex roots (`build_at_leg`), not in the propagator. This is
  pinned per diagram by the internal-H chains (ee→μμττ, the uux and b b̄ 2→6
  rows) and external-H chains (e⁺e⁻→τ⁺τ⁻H) against MadGraph `AMP()`. A
  per-chain-type phase that is uniform within one process is invisible to
  |M|² until diagram classes with different chain content mix. The full phase
  ledger is [global-phase-i-counting](../amplitudes/global-phase-i-counting.md)[^n12-causes].

The current reaching a propagator already carries its routed momentum, so the
propagator reads `q` from its input. Why propagators are separate ops rather
than fused into vertex routines is in
[propagator-separate-from-vertex](../amplitudes/propagator-separate-from-vertex.md).

### Weyl-basis slash

`q̸ = q_μ γ^μ` with `γ^μ = [[0, σ^μ], [σ̄^μ, 0]]`, `σ^μ = (1, σ⃗)`,
`σ̄^μ = (1, −σ⃗)`[^n10-prims]. In components:

```
q·σ  = [[q₀+q₃, q₁−iq₂], [q₁+iq₂, q₀−q₃]]
q·σ̄ = [[q₀−q₃, −q₁+iq₂], [−q₁−iq₂, q₀+q₃]]
```

The ket action `q̸ψ` puts `(σ·q)ψ_R` in the left block and `(σ̄·q)ψ_L` in the
right block. The bra action `ψ̄q̸` is the row–matrix product, the transpose of
the ket action rather than its chiral swap, so a plain dot of a bra with a
ket gives the Lorentz scalar (`DiracAdjoint::slash_bispinor` in
`repr/lorentz.rs`). The same kernel with complex polarisation components is
`ε̸ψ`, the vector-onto-fermion contraction (`GammaIout`/`GammaOout`), with no
denominator. On-shell, `(q̸ + m)` at `q² = m²` equals `Σ_s u_s ū_s`.

## The fermion-line sign on massive lines

The per-propagator −1 on a fermion line read against its arrow
(`Diagram::fermion_line_sign`) is a property of the line, not of whether the
propagators are massive. It is pinned bit-for-bit against MadGraph per-diagram
`AMP()` on massive tau lines that radiate a Higgs before annihilating
(`ta+ ta- > t t~ h` and `ta+ ta- > t t~ h h` under the SM default restrict card,
which keeps m_τ; rows `tata_to_ttxh` and `tata_to_ttxhh` in
`validation/madgraph/gen_standalone_jamps.py`)[^code-diagram][^n12-stand]. The sign itself
belongs to [fermion-line-sign](../amplitudes/fermion-line-sign.md).

## Not supported

- **Spin 3/2 and spin 2**[^n10-open]. Spin-3/2 externals are refused at compile time
  (`UnsupportedSpin`). Spin-2 particles get helicity states
  `{−2, …, +2}` from `helicity_states`, but `build_external_core` has no arm for
  them and panics if one is evaluated.
- **Majorana fermions.** No charge-conjugation structures; the rooting refuses
  them as `RootLorentzError::UnsupportedVertex`.

[^n10-prims]: Note 10 §4.1–§4.6. Its §4.1 doc sketch has the `nsv` sign backwards ("+1 for an incoming particle"); the code and HELAS use +1 for outgoing.
[^n10-open]: Note 10 §11. Its open questions are settled: colour is factored symbolically (not multiplied into couplings), amplitudes are summed coherently before squaring, the massless vector is Feynman gauge, and `Sigma`/`Epsilon` are implemented.
[^n12-causes]: Note 12, root causes 1 and 6. Fix 6 set the scalar propagator to `1/D`; the scalar propagator is now `−i/D`, with the compensation in the scalar-sink vertex roots.
[^n12-stand]: Note 12, "Where things stand". Its per-row numbers and "NCOLOR ≥ 2 not oracled" are superseded (colour oracles: [validation/colour-oracles](../validation/colour-oracles.md)); its massive-fermion reversed-spine caveat is settled by the tau rows above.
[^code-wavefn]: `vibegraph-lib/src/helas/wavefn.rs` module doc and constructors.
[^code-kernel]: `vibegraph-lib/src/helas/eval/kernel.rs`, the bare propagator kernels and the `propagate_scalar` doc comment.
[^code-run]: `vibegraph-lib/src/helas/eval/run.rs`, `build_external_core`; `vibegraph-lib/src/ufo/particles.rs`, `helicity_states`; `vibegraph-lib/src/helas/eval/compile.rs`, the `UnsupportedSpin` error.
[^code-vertex]: `vibegraph-lib/src/helas/vertex.rs`, reference ports used by unit tests.
[^code-diagram]: `vibegraph-lib/src/diagrams/diagram.rs`, `fermion_line_sign` doc comment.
[^aloha-om]: `aloha/aloha_writers.py` at the pinned commit: `OM{0} = 0; if (M{0}.ne.0) OM{0}=1/M{0}**2`.
