//! Lorentz-primitive evaluation kernels.
//!
//! Each `Op` that carries Lorentz semantics maps 1-to-1 to a kernel here, named for
//! the op (snake-cased): the runtime [`apply`](super::run) dispatch is just
//! `kernel::<op>(children)`. Ops that share an implementation get a thin per-op wrapper
//! over a shared private helper (`gamma_iout`/`gamma_oout` → [`off_shell_fermion_current`],
//! `proj_m`/`proj_p` → [`chiral_project`], `proj_m_amp`/`proj_p_amp`/`identity_amp` →
//! [`scalar_bilinear_current`], `ffv_iout`/`ffv_oout` → [`fused_chiral_fermion_current`]),
//! so the op↔kernel mapping stays total and explicit. The kernels (and the typed
//! random-slot generators) are also re-exported through the feature-gated
//! `bench_internals` facade for the microbenches in `benches/`.
//!
//! Structural/const/algebraic ops (`External`, `Mul`, `Add`, `Coupling`, `Mass`, `Width`,
//! `Coeff`) are not kernels — they live in [`apply`](super::run) itself. Each kernel takes
//! the already-evaluated `children` in operand order; the cross-check tests call them
//! directly.

use num_complex::ComplexFloat;
use num_traits::Zero;

use super::waveform_slot::{MultivectorWf, WaveformSlot};
use crate::helas::repr::lorentz::{
    epsilon4, epsilon_vector, AsymRank2Tensor, Bispinor, Bra, ComplexVector, DiracAdjoint, Ket,
    LorentzVector, Multivector, SpinorRepr, VectorRepr,
};
use crate::helas::repr::numbers::Chirality;
use crate::helas::repr::{ri, Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};

/// Extract a bare real constant from a [`WaveformSlot::Real`] child.
pub fn expect_real<F: Real>(slot: &WaveformSlot<F>) -> F {
    match slot {
        WaveformSlot::Real(r) => *r,
        other => panic!("expected a real-constant slot, got {other:?}"),
    }
}

/// Extract a complex scalar value from a [`WaveformSlot::Scalar`] child (the fused
/// kernels' effective-coupling operands; their momentum is exactly zero, being sums
/// of momentum-free `Coupling`/`Coeff` products).
fn expect_scalar<F: Real>(slot: &WaveformSlot<F>) -> C<F> {
    match slot {
        WaveformSlot::Scalar(s) => s.value,
        other => panic!("expected a scalar slot, got {other:?}"),
    }
}

// ──────────────────────────── bare-current kernels ────────────────────────────
//
// The typed result arenas store momentum-stripped currents: a fermion is a bare
// `Bispinor`, a vector a bare contravariant `ComplexVector`, a scalar a bare `C<F>`.
// These kernels operate directly on that storage. The momentum a propagator or a `P`
// read-off needs is passed in explicitly — it is resolved once per phase-space point
// from the momentum table rather than carried on every current. The wavefunction-typed
// kernels below wrap these, re-attaching the routed momentum for the generic slot path
// and the unit cross-checks.

/// Dirac propagator on a bare ket spinor with routed momentum `q`.
#[inline]
pub fn propagate_fin_bare<F: Real>(
    spinor: &Bispinor<F, Ket>,
    q: &LorentzVector<F>,
    mass: F,
    width: F,
) -> Bispinor<F, Ket> {
    let num = spinor.slash(&(*q).into()) + *spinor * mass;
    let scale = ri(-F::one()) * C::new(q.m2() - mass * mass, mass * width).recip();
    num * scale
}

/// Dirac propagator on a bare bra spinor with routed momentum `q`.
#[inline]
pub fn propagate_fout_bare<F: Real>(
    spinor: &Bispinor<F, Bra>,
    q: &LorentzVector<F>,
    mass: F,
    width: F,
) -> Bispinor<F, Bra> {
    let num = spinor.slash(&(*q).into()) + *spinor * mass;
    let scale = ri(-F::one()) * C::new(q.m2() - mass * mass, mass * width).recip();
    num * scale
}

/// Vector propagator on a bare contravariant polarisation with routed momentum `q`.
#[inline]
pub fn propagate_vector_bare<F: Real>(
    eps: &ComplexVector<F>,
    q: &LorentzVector<F>,
    mass: F,
    width: F,
) -> ComplexVector<F> {
    if mass == F::zero() {
        *eps * ri(-q.m2().recip())
    } else {
        let vm2 = mass * mass;
        let denom = C::new(q.m2() - vm2, mass * width);
        let cs = eps.dot_lorentz(q) / vm2;
        (*eps - ComplexVector::from(*q) * cs) * ri(-F::one()) / denom
    }
}

/// Scalar propagator on a bare scalar value with routed momentum `q`.
pub fn propagate_scalar_bare<F: Real>(
    value: C<F>,
    q: &LorentzVector<F>,
    mass: F,
    width: F,
) -> C<F> {
    let denom = C::new(q.m2() - mass * mass, mass * width);
    value * ri(-F::one()) / denom
}

/// A structure momentum promoted to a bare contravariant vector current.
pub fn pmom_bare<F: Real>(q: &LorentzVector<F>) -> ComplexVector<F> {
    ComplexVector::from(*q)
}

/// `GammaVout` on bare spinors: two fermions → off-shell vector `ψ̄ γ^μ ψ`; a line read
/// against the vertex's defined adjoint picks up the `C γ^{μT} C⁻¹ = −γ^μ` sign.
#[inline]
pub fn gamma_vout_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
    reversed: bool,
) -> ComplexVector<F> {
    let eps = fo.vector_bilinear(fi, Chirality::Both);
    if reversed {
        -eps
    } else {
        eps
    }
}

/// `FfvVout` on bare spinors and effective couplings (see [`ffv_vout_c`]).
#[inline]
pub fn ffv_vout_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
    gl: C<F>,
    gr: C<F>,
    reversed: bool,
) -> ComplexVector<F> {
    let jl = fo.left_current(fi);
    let jr = fo.right_current(fi);
    if reversed {
        -(jr * gl + jl * gr)
    } else {
        jl * gl + jr * gr
    }
}

/// Continue a bare ket line by slashing with the vector polarisation, `ε̸ψ`.
pub fn off_shell_fin_bare<F: Real>(
    eps: &ComplexVector<F>,
    fi: &Bispinor<F, Ket>,
) -> Bispinor<F, Ket> {
    fi.slash(eps)
}

/// Continue a bare bra line by slashing with the vector polarisation, `ψ̄ε̸`.
pub fn off_shell_fout_bare<F: Real>(
    eps: &ComplexVector<F>,
    fo: &Bispinor<F, Bra>,
) -> Bispinor<F, Bra> {
    fo.slash(eps)
}

/// Fused chiral off-shell current on a bare ket line (see [`ffv_fin`]).
#[inline]
pub fn ffv_fin_bare<F: Real>(
    eps: &ComplexVector<F>,
    fi: &Bispinor<F, Ket>,
    gl: C<F>,
    gr: C<F>,
) -> Bispinor<F, Ket> {
    chiral_weighted(fi, gl, gr).slash(eps)
}

/// Fused chiral off-shell current on a bare bra line (see [`ffv_fout`]).
#[inline]
pub fn ffv_fout_bare<F: Real>(
    eps: &ComplexVector<F>,
    fo: &Bispinor<F, Bra>,
    gl: C<F>,
    gr: C<F>,
) -> Bispinor<F, Bra> {
    chiral_weighted(fo, gl, gr).slash(eps)
}

/// Chiral projection of a bare ket line.
pub fn proj_fin_bare<F: Real>(fi: &Bispinor<F, Ket>, chirality: Chirality) -> Bispinor<F, Ket> {
    project_spinor(fi, chirality)
}

/// Chiral projection of a bare bra line.
pub fn proj_fout_bare<F: Real>(fo: &Bispinor<F, Bra>, chirality: Chirality) -> Bispinor<F, Bra> {
    project_spinor(fo, chirality)
}

/// `Gamma5` on a bare ket line, `γ⁵ψ`.
pub fn gamma5_fin_bare<F: Real>(fi: &Bispinor<F, Ket>) -> Bispinor<F, Ket> {
    gamma5_spinor(fi)
}

/// `Gamma5` on a bare bra line, `ψ̄γ⁵`.
pub fn gamma5_fout_bare<F: Real>(fo: &Bispinor<F, Bra>) -> Bispinor<F, Bra> {
    gamma5_spinor(fo)
}

/// Scalar bilinear `ψ̄ Γ ψ` on bare spinors.
pub fn scalar_bilinear_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
    chirality: Chirality,
) -> C<F> {
    Bispinor::scalar_bilinear(fo, fi, chirality)
}

/// Pseudoscalar bilinear `ψ̄ γ⁵ ψ` on bare spinors.
pub fn pseudoscalar_bilinear_bare<F: Real>(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>) -> C<F> {
    Bispinor::pseudoscalar_bilinear(fo, fi, Chirality::Both)
}

/// `Metric`: contract two bare contravariant vectors → scalar.
pub fn metric_bare<F: Real>(v1: &ComplexVector<F>, v2: &ComplexVector<F>) -> C<F> {
    v1.dot(&v2.lower())
}

/// `MetricVout`: the contravariant current `g^{μν}V_ν = V^μ` — identity on bare storage.
pub fn metric_vout_bare<F: Real>(vin: &ComplexVector<F>) -> ComplexVector<F> {
    *vin
}

/// `EpsilonVout` on bare contravariant vectors: `E^σ = ε^{μνρσ} a_μ b_ν c_ρ`, the
/// three-vectors-in current characterised by `E·d = epsilon_amp_bare(a, b, c, d)`
/// under the Minkowski contraction. Output is contravariant, like every other
/// vector current here.
pub fn epsilon_vout_bare<F: Real>(
    a: &ComplexVector<F>,
    b: &ComplexVector<F>,
    c: &ComplexVector<F>,
) -> ComplexVector<F> {
    epsilon_vector(a, b, c)
}

/// `EpsilonAmp` on bare contravariant vectors: the fully contracted
/// `ε^{μνρσ} a_μ b_ν c_ρ d_σ` (ALOHA's `ε^{0123} = −1`; see [`epsilon4`]).
pub fn epsilon_amp_bare<F: Real>(
    a: &ComplexVector<F>,
    b: &ComplexVector<F>,
    c: &ComplexVector<F>,
    d: &ComplexVector<F>,
) -> C<F> {
    epsilon4(a, b, c, d)
}

/// `FierzOut` on bare spinors: the cut fermion line's `γ^α γ^β` chain as a Clifford
/// element, ready to contract into the other line's two gammas.
///
/// `γ^αγ^β = g^{αβ} − i σ^{αβ}` puts the chain in grades 0 and 2, so the line is fixed
/// by its scalar and tensor bilinears alone: `V^{αβ} = g^{αβ} s − i t^{αβ}`. Contracting
/// that against the other line's `γ_α γ_β` gives `4 s − σ_{αβ} t^{αβ}`, which in
/// [`Multivector`]'s normalisation (grade 2 is `½ T^{μν} σ_{μν}`) is `4s` on grade 0 and
/// `−2t` on grade 2. `reversed_order` is for two lines traversing the shared indices in
/// opposite orders (`γ^αγ^β` against `γ_βγ_α`), where the grade-2 term enters with the
/// other sign — the whole content of the index order.
///
/// The pair arrives as (bra, ket) by the ends' actual adjoints. A line the vertex reads
/// against its own arrow needs `C Γᵀ C⁻¹`, which for `X γ^αγ^β Y` is `Y γ^βγ^α X`: the
/// two gammas transpose and nothing else changes, so that case is one more flip of
/// `reversed_order` and is decided at rooting time rather than here.
#[inline]
pub fn fierz_out_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
    reversed_order: bool,
) -> Multivector<F> {
    let coeffs = fo.fierz_coefficients(fi);
    let two = F::one() + F::one();
    let scalar = Multivector::from_scalar(coeffs.scalar() * (two + two));
    let bivector = Multivector::from_bivector(&(coeffs.bivector() * two));
    if reversed_order {
        scalar + bivector
    } else {
        scalar - bivector
    }
}

/// `MultivectorIout` on a bare ket line: `M ψ`.
#[inline]
pub fn multivector_fin_bare<F: Real>(
    m: &Multivector<F>,
    fi: &Bispinor<F, Ket>,
) -> Bispinor<F, Ket> {
    fi.apply(m)
}

/// `MultivectorOout` on a bare bra line: `ψ̄ M`.
#[inline]
pub fn multivector_fout_bare<F: Real>(
    m: &Multivector<F>,
    fo: &Bispinor<F, Bra>,
) -> Bispinor<F, Bra> {
    fo.apply(m)
}

/// The `Sigma` half: ALOHA's `Sigma` is `½ σ^{μν}`, not the textbook
/// `σ^{μν} = (i/2)[γ^μ, γ^ν]`.
///
/// `aloha/aloha_object.py`'s `L_Sigma.sigma` table carries ±½ and ±½i where the
/// textbook matrix carries ±1 and ±1i, and the banked `ll_to_qqx_toy_tensor` row
/// measures it as a process-level ratio: the same four-fermion operator written with
/// two literal `Sigma`s and with its γγ expansion gives
/// `AMP(FFFFG)/AMP(FFFFT) = 4 × ggam/gtens` to 4.7e-14 over every helicity of every
/// banked point. The factor enters once per `Sigma`, so a kernel at the textbook
/// normalisation is 2× too large on a dipole and 4× on a tensor⊗tensor contact.
#[inline]
fn sigma_half<F: Real>() -> F {
    F::one() / (F::one() + F::one())
}

/// `SigmaVout`/`SigmaVoutRev` on bare operands: two fermions and a vector → the
/// off-shell vector current `J^μ = (ψ̄ Σ^{μν} ψ) v_ν`, the free index on `Sigma`'s
/// *first* Lorentz slot.
///
/// `negate` carries both minus signs the rooting resolves, which are the same sign:
/// a line read against the vertex's own adjoint conjugates the structure as
/// `C σ^{μνT} C⁻¹ = −σ^{μν}` (as `C γ^{μT} C⁻¹ = −γ^μ` does for [`gamma_vout_bare`]),
/// and putting the free index on the *second* slot instead is a transposition of an
/// antisymmetric tensor.
#[inline]
pub fn sigma_vout_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
    v: &ComplexVector<F>,
    negate: bool,
) -> ComplexVector<F> {
    let t = fo.tensor_bilinear(fi, Chirality::Both) * sigma_half::<F>();
    let j = t.contract_vector(v);
    if negate {
        -j
    } else {
        j
    }
}

/// `SigmaMv` on bare vectors: `Σ^{μν} a_μ b_ν` as a Clifford element, the operator a
/// `Sigma` becomes once both of its Lorentz indices are contracted and one of its
/// spinor indices continues a fermion line.
///
/// Lowering both arguments and antisymmetrising is exactly the wedge of the two
/// contravariant vectors, so in [`Multivector`]'s grade-2 normalisation
/// (`½ T^{μν} σ_{μν}`) the coefficient is `½ (a ∧ b)` — the ½ being [`sigma_half`].
#[inline]
pub fn sigma_mv_bare<F: Real>(a: &ComplexVector<F>, b: &ComplexVector<F>) -> Multivector<F> {
    Multivector::from_bivector(&(AsymRank2Tensor::wedge(a, b) * sigma_half::<F>()))
}

/// `SigmaOut`/`SigmaOutRev` on bare spinors: the cut fermion line of a
/// `Sigma ⊗ Sigma` contact as a Clifford element, ready to contract into the other
/// line's own `Sigma`.
///
/// A literal `Sigma` is the two gammas of [`fierz_out_bare`] already contracted, so the
/// cut line is pure grade 2 — there is no `g^{αβ}` term to leave a scalar behind. Its
/// bilinear is `½ t^{αβ}` and the surviving line reads it as `Σ^{αβ} · ½ t_{αβ}`, which
/// in the grade-2 normalisation is `T = ½ t`: one [`sigma_half`] per line, and the
/// factor two between `σ_{αβ} t^{αβ}` and `½ T^{μν} σ_{μν}` returns one of them.
/// `reversed_order` is the two lines' relative index order, and — because
/// `C σ^{αβT} C⁻¹ = −σ^{αβ}` and `σ^{βα} = −σ^{αβ}` are the same sign — also carries a
/// line read against the vertex's own adjoint.
#[inline]
pub fn sigma_out_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
    reversed_order: bool,
) -> Multivector<F> {
    let t = fo.tensor_bilinear(fi, Chirality::Both) * sigma_half::<F>();
    Multivector::from_bivector(&if reversed_order { -t } else { t })
}

/// `FierzPair` on bare spinors: `ψ̄ M ψ`, as the grade-diagonal pairing of the element
/// with the pair's own sixteen bilinears.
#[inline]
pub fn fierz_pair_bare<F: Real>(
    m: &Multivector<F>,
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
) -> C<F> {
    fo.fierz_coefficients(fi).fierz_pairing(m)
}

// ──────────────────────────── propagator ────────────────────────────

/// `Propagate`: apply a propagator (interned mass/width from the two real operands) to
/// the off-shell current. The propagator outputs a contravariant current.
pub fn propagate<F: Real>(
    current: &WaveformSlot<F>,
    mass: &WaveformSlot<F>,
    width: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    propagate_core(current, expect_real(mass), expect_real(width))
}

/// Apply a propagator with interned mass/width to an off-shell current. The current
/// already carries the conserved routed momentum (matching reference HELAS, where the
/// off-shell current routines output it: `fvixxx` q=fi−vc, `fvoxxx` q=fo+vc,
/// `jioxxx` jmom=fo−fi).
pub fn propagate_core<F: Real>(input: &WaveformSlot<F>, mass: F, width: F) -> WaveformSlot<F> {
    match input {
        WaveformSlot::FermionIn(wf) => WaveformSlot::FermionIn(propagate_fin(wf, mass, width)),
        WaveformSlot::FermionOut(wf) => WaveformSlot::FermionOut(propagate_fout(wf, mass, width)),
        WaveformSlot::Vector(wf) => WaveformSlot::Vector(propagate_vector(wf, mass, width)),
        WaveformSlot::Scalar(wf) => WaveformSlot::Scalar(propagate_scalar(wf, mass, width)),
        WaveformSlot::Real(_) => panic!("propagate step read a real-constant slot"),
        WaveformSlot::Multivector(_) => {
            panic!("propagate step read a Clifford element: it never leaves its vertex")
        }
        WaveformSlot::Empty => panic!("propagate step read an empty slot"),
    }
}

// The Dirac propagator -i (q̸ + m) / (q² - m² + i m Γ) puts the fermion chain in phase
// with the vector chain (bit-validated against MadGraph's W-arrays), so every off-shell
// chain type carries the same phase relative to MadGraph and diagram classes with
// different chain contents interfere correctly; pinned by the uux 2→6 per-diagram oracle
// (validation/madgraph/compare_amps.py), where continuum diagrams (two fermion
// propagators) meet H diagrams (one scalar propagator).

/// Dirac propagator on a flow-in (ket) off-shell current.
pub fn propagate_fin<F: Real>(wf: &InDiracWf<F>, mass: F, width: F) -> InDiracWf<F> {
    InDiracWf::from_spinor(
        propagate_fin_bare(&wf.spinor, &wf.momentum, mass, width),
        wf.momentum,
    )
}

/// Dirac propagator on a flow-out (bra) off-shell current.
pub fn propagate_fout<F: Real>(wf: &OutDiracWf<F>, mass: F, width: F) -> OutDiracWf<F> {
    OutDiracWf::from_spinor(
        propagate_fout_bare(&wf.spinor, &wf.momentum, mass, width),
        wf.momentum,
    )
}

/// Vector propagator on an off-shell vector current. The numerator is
/// `-i (g - q q / m²)` (massive) or `-i g / q²` (massless); see [`propagate_vector_bare`].
pub fn propagate_vector<F: Real>(wf: &VectorWf<F>, mass: F, width: F) -> VectorWf<F> {
    VectorWf {
        eps: propagate_vector_bare(&wf.eps, &wf.momentum, mass, width),
        momentum: wf.momentum,
    }
}

/// Scalar propagator on an off-shell scalar current: -i / (q² - m² + i m Γ) — the same
/// -i/D phase as the vector and Dirac propagators, so every chain type propagates
/// uniformly. The compensating signs live in the scalar-sink vertex roots (see
/// `build_at_leg`'s scalar-root arms); the combination is pinned per-diagram by the
/// internal-H chains (ee→μμττ, uux 2→6, and the b b̄ 2→6 spine-Yukawa diagrams) and the
/// external-H chains (e+e-→τ+τ-H) against MadGraph AMP().
pub fn propagate_scalar<F: Real>(wf: &ScalarWf<F>, mass: F, width: F) -> ScalarWf<F> {
    ScalarWf {
        value: propagate_scalar_bare(wf.value, &wf.momentum, mass, width),
        momentum: wf.momentum,
    }
}

// ──────────────────────────── momentum read-off ────────────────────────────
//
// The P nodes read structure momenta off the stored (HELAS-convention) current momenta:
// an input leg's directly (ALOHA `Pi = dble(Vi(1:2))`), the output leg's as the negated
// sum over all inputs (ALOHA `VVV1P0_1`: `P1 = −(V2+V3)`). Their slots carry *zero*
// routing momentum: only wavefunctions route momentum to the propagator, and each leg's
// wavefunction already appears exactly once per term — a P duplicating a leg's momentum
// would double-count it in the `Mul`/`Metric` bookkeeping.

/// `PMom`: the 4-momentum of the single input, as a vector current.
pub fn pmom<F: Real>(input: &WaveformSlot<F>) -> WaveformSlot<F> {
    let momentum = input.momentum().expect("PMom: empty slot");
    WaveformSlot::Vector(pmom_from_mom(momentum))
}

/// A structure momentum promoted to a vector current with zero routing momentum (see the
/// `PMom`/`PMomOut` note above): the `P` slots carry no routing momentum of their own.
pub fn pmom_from_mom<F: Real>(momentum: LorentzVector<F>) -> VectorWf<F> {
    VectorWf {
        eps: ComplexVector::from(momentum),
        momentum: LorentzVector::zero(),
    }
}

/// `PMomOut`: the 4-momentum of the vertex's *output* leg — minus the sum of the
/// input legs' momenta in the all-incoming convention. The only variadic kernel
/// (all vertex inputs), so it takes the operands as an iterator rather than fixed
/// arity.
///
/// A boson current stores the momentum flowing *into* the vertex, so it enters that
/// sum as it is. A fermion current stores the momentum flowing *along its line*:
/// the bra half of a pair carries the line's momentum into the vertex and the ket
/// half carries it out, so the pair contributes `p_bra − p_ket` — the same
/// combination the vector current those two fermions produce is routed with
/// ([`gamma_vout_c`]). Summing a fermion pair with two plus signs instead reads the
/// wrong momentum into every `P` that names the output leg of an `FFV` vertex,
/// which is invisible until a structure puts one there (SMEFTsim's dipoles do).
pub fn pmom_out<'a, F: Real + 'a>(
    children: impl IntoIterator<Item = &'a WaveformSlot<F>>,
) -> WaveformSlot<F> {
    let momentum = -children.into_iter().fold(LorentzVector::zero(), |acc, c| {
        let p = c.momentum().expect("PMomOut: empty slot");
        match c {
            WaveformSlot::FermionIn(_) => acc - p,
            _ => acc + p,
        }
    });
    WaveformSlot::Vector(pmom_from_mom(momentum))
}

// ──────────────────────────── fermion currents ────────────────────────────

/// Resolve the two fermion legs of a bilinear into `(bra = bra, ket = ket,
/// reversed)` by their *actual* runtime adjoint, not the UFO `Gamma` i/j position. A
/// fermion line carries one adjoint throughout, so with physically-typed externals
/// (see `build_external_core`) and adjoint-preserving currents, the two fermions
/// meeting at any vertex always have opposite adjoint.
///
/// `reversed` is `true` when the slots arrive in `(ket, bra)` order, i.e.
/// the line runs against the vertex's defined adjoint; callers use it to apply the
/// adjoint-reversal sign η_Γ of their Lorentz structure.
fn resolve_bra_ket<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
) -> (OutDiracWf<F>, InDiracWf<F>, bool) {
    match (a, b) {
        (WaveformSlot::FermionOut(fo), WaveformSlot::FermionIn(fi)) => (*fo, *fi, false),
        (WaveformSlot::FermionIn(fi), WaveformSlot::FermionOut(fo)) => (*fo, *fi, true),
        _ => panic!("a fermion bilinear needs exactly one ket and one bra leg"),
    }
}

/// `GammaIout`: continue a flow-in (ket) fermion line by slashing it with the vector
/// current, `ε̸ψ`, q = f.p − v.p (Fortran `fvixxx`). Same kernel as [`gamma_oout`]
/// because [`off_shell_fermion_current`] follows the input fermion's adjoint.
pub fn gamma_iout<F: Real>(v: &WaveformSlot<F>, f: &WaveformSlot<F>) -> WaveformSlot<F> {
    off_shell_fermion_current(v, f)
}

/// `GammaOout`: continue a flow-out (bra) fermion line by slashing it with the vector
/// current, `ψ̄ε̸`, q = f.p + v.p (Fortran `fvoxxx`). See [`gamma_iout`].
pub fn gamma_oout<F: Real>(v: &WaveformSlot<F>, f: &WaveformSlot<F>) -> WaveformSlot<F> {
    off_shell_fermion_current(v, f)
}

/// Off-shell fermion current from an FFV `Gamma` vertex (one vector leg `mu` +
/// one continuing fermion leg `f`). The current **follows the input fermion's
/// adjoint**, so no mid-line Dirac adjoint is ever needed:
///   - ket: `ε̸ψ`, q = f.p − v.p   (Fortran `fvixxx`)
///   - bra: `ψ̄ε̸`, q = f.p + v.p   (Fortran `fvoxxx`)
///
/// `Bispinor::slash` is adjoint-dependent, so the left/right action is automatic.
/// The propagator `(q̸+m)/D` is applied in a separate `Propagate` step.
pub fn off_shell_fermion_current<F: Real>(
    v: &WaveformSlot<F>,
    fermion: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v) = v else {
        panic!("off-shell fermion current: expected vector input");
    };
    match fermion {
        WaveformSlot::FermionIn(fi) => WaveformSlot::FermionIn(off_shell_fin(v, fi)),
        WaveformSlot::FermionOut(fo) => WaveformSlot::FermionOut(off_shell_fout(v, fo)),
        _ => panic!("off-shell fermion current: expected fermion input"),
    }
}

/// Continue a flow-in (ket) fermion line by slashing it with the vector current, `ε̸ψ`,
/// q = f.p − v.p (Fortran `fvixxx`).
pub fn off_shell_fin<F: Real>(v: &VectorWf<F>, fi: &InDiracWf<F>) -> InDiracWf<F> {
    InDiracWf::from_spinor(
        off_shell_fin_bare(&v.eps, &fi.spinor),
        fi.momentum - v.momentum,
    )
}

/// Continue a flow-out (bra) fermion line by slashing it with the vector current, `ψ̄ε̸`,
/// q = f.p + v.p (Fortran `fvoxxx`).
pub fn off_shell_fout<F: Real>(v: &VectorWf<F>, fo: &OutDiracWf<F>) -> OutDiracWf<F> {
    OutDiracWf::from_spinor(
        off_shell_fout_bare(&v.eps, &fo.spinor),
        fo.momentum + v.momentum,
    )
}

/// `ProjM`: left chiral projection of a continuing fermion current. See [`chiral_project`].
pub fn proj_m<F: Real>(f: &WaveformSlot<F>) -> WaveformSlot<F> {
    chiral_project(f, Chirality::Left)
}

/// `ProjP`: right chiral projection of a continuing fermion current. See [`chiral_project`].
pub fn proj_p<F: Real>(f: &WaveformSlot<F>) -> WaveformSlot<F> {
    chiral_project(f, Chirality::Right)
}

/// `ProjM`/`ProjP`: chiral projection on a continuing fermion current, preserving the
/// input adjoint. `project_left`/`project_right` are adjoint-dependent (a bra projects
/// different components than a ket), so the same call is correct for both flows.
pub fn chiral_project<F: Real>(child: &WaveformSlot<F>, chirality: Chirality) -> WaveformSlot<F> {
    match child {
        WaveformSlot::FermionIn(f) => WaveformSlot::FermionIn(proj_fin(f, chirality)),
        WaveformSlot::FermionOut(f) => WaveformSlot::FermionOut(proj_fout(f, chirality)),
        _ => panic!("chiral projection: expected fermion input"),
    }
}

/// Chiral projection of a stored spinor block (`project_left`/`project_right` are
/// adjoint-dependent, so the same call is correct for both flows).
fn project_spinor<F: Real, Fl: DiracAdjoint>(
    s: &Bispinor<F, Fl>,
    chirality: Chirality,
) -> Bispinor<F, Fl> {
    match chirality {
        Chirality::Left => s.project_left(),
        Chirality::Right => s.project_right(),
        Chirality::Both => *s,
    }
}

/// `Gamma5`: γ⁵ on a continuing fermion current, preserving the input adjoint.
///
/// `γ⁵ = P_R − P_L` is diagonal in the Weyl basis, so the left action on a ket and
/// the right action on a bra are the same weighting of the stored blocks — which is
/// what lets one kernel serve both flows, as [`chiral_project`] does.
pub fn gamma5<F: Real>(child: &WaveformSlot<F>) -> WaveformSlot<F> {
    match child {
        WaveformSlot::FermionIn(f) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            gamma5_fin_bare(&f.spinor),
            f.momentum,
        )),
        WaveformSlot::FermionOut(f) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            gamma5_fout_bare(&f.spinor),
            f.momentum,
        )),
        _ => panic!("gamma5: expected fermion input"),
    }
}

/// γ⁵ on a stored spinor block, `P_R − P_L`.
fn gamma5_spinor<F: Real, Fl: DiracAdjoint>(s: &Bispinor<F, Fl>) -> Bispinor<F, Fl> {
    s.project_right() - s.project_left()
}

/// Chiral projection of a flow-in fermion current, preserving the flow.
pub fn proj_fin<F: Real>(f: &InDiracWf<F>, chirality: Chirality) -> InDiracWf<F> {
    InDiracWf::from_spinor(proj_fin_bare(&f.spinor, chirality), f.momentum)
}

/// Chiral projection of a flow-out fermion current, preserving the flow.
pub fn proj_fout<F: Real>(f: &OutDiracWf<F>, chirality: Chirality) -> OutDiracWf<F> {
    OutDiracWf::from_spinor(proj_fout_bare(&f.spinor, chirality), f.momentum)
}

/// `GammaVout`: two fermions → off-shell vector current `ψ̄ γ^μ ψ`.
pub fn gamma_vout<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    let (fo, fi, reversed) = resolve_bra_ket(a, b);
    WaveformSlot::Vector(gamma_vout_c(&fo, &fi, reversed))
}

/// `GammaVout` on resolved bra/ket currents: two fermions → off-shell vector current
/// `ψ̄ γ^μ ψ`. Reading the fermion line against the vertex's defined adjoint conjugates
/// the structure as C γ^{μT} C⁻¹ = −γ^μ, so a `reversed` line picks up a relative −1.
/// (Scalar/pseudoscalar structures have +1 and need no flip.)
pub fn gamma_vout_c<F: Real>(fo: &OutDiracWf<F>, fi: &InDiracWf<F>, reversed: bool) -> VectorWf<F> {
    VectorWf {
        eps: gamma_vout_bare(&fo.spinor, &fi.spinor, reversed),
        momentum: fo.momentum - fi.momentum,
    }
}

// ──────────────────────────── fused chiral FFV kernels ────────────────────────────
//
// A chiral-pair FFV vertex (`Gamma·ProjM` and `Gamma·ProjP` structure variants of one
// contraction shape) evaluates as a single fused node: the per-chirality effective
// couplings arrive as scalar operands and the kernel forms `g_L·(left term) +
// g_R·(right term)` directly. Relative to the generic composition this reorders
// floating-point operations (couplings scale per-chirality before the sum), so
// agreement is approximate (≲1e-15 per kernel), certified by the `fused_*` tests.

/// `FfvVout`: fused chiral [`gamma_vout`] — `g_L·GammaVout(a, ProjM(b)) +
/// g_R·GammaVout(a, ProjP(b))` in one step.
///
/// The projector tags refer to the `b` operand's *storage* blocks: its left block
/// feeds `J_L` when `b` is the ket, but `J_R` when the pair arrives reversed
/// (`b` the bra), so the couplings swap currents in the reversed case — and the
/// reversal itself contributes the `C γ^{μT} C⁻¹ = −γ^μ` sign, as in [`gamma_vout`].
pub fn ffv_vout<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    gl: &WaveformSlot<F>,
    gr: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let gl = expect_scalar(gl);
    let gr = expect_scalar(gr);
    let (fo, fi, reversed) = resolve_bra_ket(a, b);
    WaveformSlot::Vector(ffv_vout_c(&fo, &fi, gl, gr, reversed))
}

/// `FfvVout` on resolved bra/ket currents and effective couplings.
pub fn ffv_vout_c<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    gl: C<F>,
    gr: C<F>,
    reversed: bool,
) -> VectorWf<F> {
    VectorWf {
        eps: ffv_vout_bare(&fo.spinor, &fi.spinor, gl, gr, reversed),
        momentum: fo.momentum - fi.momentum,
    }
}

/// `FfvIout`: fused chiral [`gamma_iout`] — continue a flow-in fermion line through a
/// chiral-pair FFV vertex. See [`fused_chiral_fermion_current`].
pub fn ffv_iout<F: Real>(
    v: &WaveformSlot<F>,
    f: &WaveformSlot<F>,
    gl: &WaveformSlot<F>,
    gr: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    fused_chiral_fermion_current(v, f, gl, gr)
}

/// `FfvOout`: fused chiral [`gamma_oout`] — continue a flow-out fermion line through a
/// chiral-pair FFV vertex. See [`fused_chiral_fermion_current`].
pub fn ffv_oout<F: Real>(
    v: &WaveformSlot<F>,
    f: &WaveformSlot<F>,
    gl: &WaveformSlot<F>,
    gr: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    fused_chiral_fermion_current(v, f, gl, gr)
}

/// Fused chiral off-shell fermion current: `g_L·(ε̸ P_L ψ) + g_R·(ε̸ P_R ψ)` =
/// `ε̸·(g_L ψ_L ⊕ g_R ψ_R)` — the slash is linear, so the chiral weights combine
/// into the input spinor's storage blocks before a single slash. The same `[gl, gl,
/// gr, gr]` weighting is correct for both adjoints: in the Weyl basis the slash maps
/// storage blocks crosswise, so each weighted input block lands on the output block
/// the corresponding projected term populates. Momentum routing follows
/// [`off_shell_fermion_current`] (ket: `f − v`, bra: `f + v`).
fn fused_chiral_fermion_current<F: Real>(
    v: &WaveformSlot<F>,
    f: &WaveformSlot<F>,
    gl: &WaveformSlot<F>,
    gr: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v) = v else {
        panic!("fused chiral fermion current: expected vector input");
    };
    let gl = expect_scalar(gl);
    let gr = expect_scalar(gr);
    match f {
        WaveformSlot::FermionIn(fi) => WaveformSlot::FermionIn(ffv_fin(v, fi, gl, gr)),
        WaveformSlot::FermionOut(fo) => WaveformSlot::FermionOut(ffv_fout(v, fo, gl, gr)),
        _ => panic!("fused chiral fermion current: expected fermion input"),
    }
}

/// The `[gl, gl, gr, gr]` chiral weighting of a spinor's storage blocks before a single
/// slash. The same weighting is correct for both adjoints: in the Weyl basis the slash
/// maps storage blocks crosswise, so each weighted input block lands on the output block
/// the corresponding projected term populates.
fn chiral_weighted<F: Real, Adj: DiracAdjoint>(
    s: &Bispinor<F, Adj>,
    gl: C<F>,
    gr: C<F>,
) -> Bispinor<F, Adj> {
    Bispinor::from_components([
        s.component(0) * gl,
        s.component(1) * gl,
        s.component(2) * gr,
        s.component(3) * gr,
    ])
}

/// Fused chiral off-shell current for a flow-in fermion (ket routing `f − v`).
pub fn ffv_fin<F: Real>(v: &VectorWf<F>, fi: &InDiracWf<F>, gl: C<F>, gr: C<F>) -> InDiracWf<F> {
    InDiracWf::from_spinor(
        ffv_fin_bare(&v.eps, &fi.spinor, gl, gr),
        fi.momentum - v.momentum,
    )
}

/// Fused chiral off-shell current for a flow-out fermion (bra routing `f + v`).
pub fn ffv_fout<F: Real>(v: &VectorWf<F>, fo: &OutDiracWf<F>, gl: C<F>, gr: C<F>) -> OutDiracWf<F> {
    OutDiracWf::from_spinor(
        ffv_fout_bare(&v.eps, &fo.spinor, gl, gr),
        fo.momentum + v.momentum,
    )
}

/// `ProjMAmp`: left chiral scalar bilinear `ψ̄ P_L ψ`. See [`scalar_bilinear_current`].
pub fn proj_m_amp<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    scalar_bilinear_current(a, b, Chirality::Left)
}

/// `ProjPAmp`: right chiral scalar bilinear `ψ̄ P_R ψ`. See [`scalar_bilinear_current`].
pub fn proj_p_amp<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    scalar_bilinear_current(a, b, Chirality::Right)
}

/// `IdentityAmp`: full scalar bilinear `ψ̄ δ ψ`. See [`scalar_bilinear_current`].
pub fn identity_amp<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    scalar_bilinear_current(a, b, Chirality::Both)
}

/// `Gamma5Amp`: pseudoscalar bilinear `ψ̄ γ⁵ ψ`; the bra/ket are picked by the legs'
/// actual adjoint. Like the scalar bilinears it takes no reversal sign — `C γ⁵ᵀ C⁻¹
/// = γ⁵`, so reading the pair against the vertex's defined adjoint leaves the
/// structure unchanged (the −1 a crossed pair needs is a rooting sign, applied in
/// [`super::root_lorentz`] alongside the `ProjM`/`Identity` case).
pub fn gamma5_amp<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    let (fo, fi, _) = resolve_bra_ket(a, b);
    WaveformSlot::Scalar(ScalarWf {
        value: pseudoscalar_bilinear_bare(&fo.spinor, &fi.spinor),
        momentum: fo.momentum - fi.momentum,
    })
}

/// `ProjMAmp`/`ProjPAmp`/`IdentityAmp`: scalar bilinear `ψ̄ Γ ψ` (`Γ = P_L`, `P_R`, or
/// `1`); the bra/ket are picked by the legs' actual adjoint.
pub fn scalar_bilinear_current<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    chirality: Chirality,
) -> WaveformSlot<F> {
    let (fo, fi_col, _) = resolve_bra_ket(a, b);
    WaveformSlot::Scalar(scalar_bilinear_c(&fo, &fi_col, chirality))
}

/// Scalar bilinear `ψ̄ Γ ψ` on resolved bra/ket currents (`Γ = P_L`, `P_R`, or `1`).
pub fn scalar_bilinear_c<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    chirality: Chirality,
) -> ScalarWf<F> {
    ScalarWf {
        value: scalar_bilinear_bare(&fo.spinor, &fi.spinor, chirality),
        momentum: fo.momentum - fi.momentum,
    }
}

// ──────────────────────────── metric / vector currents ────────────────────────────

/// `Metric`: contract two vectors → scalar.
pub fn metric<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v1) = a else {
        panic!("Metric: expected vector input");
    };
    let WaveformSlot::Vector(v2) = b else {
        panic!("Metric: expected vector input");
    };
    WaveformSlot::Scalar(metric_c(v1, v2))
}

/// `Metric`: contract two vectors → scalar.
pub fn metric_c<F: Real>(v1: &VectorWf<F>, v2: &VectorWf<F>) -> ScalarWf<F> {
    ScalarWf {
        value: metric_bare(&v1.eps, &v2.eps),
        momentum: v1.momentum + v2.momentum,
    }
}

/// `MetricVout`: off-shell vector current of a `Metric(out, v)` structure — the metric
/// contracts the output index against the partner vector `v`, `g^{μν}V_ν = V^μ`: the
/// physical contravariant current, an identity on contravariant storage. The UFO
/// coupling carries the vertex `i` and the propagator its `−i` (see `propagate_core`),
/// so no phase lives here (ALOHA's `VVS1P1N_1` = `−i·g·V` folds the propagator's −i
/// into the vertex routine instead). A trailing scalar leg (the Higgs) multiplies in
/// at the enclosing `Mul`.
pub fn metric_vout<F: Real>(v: &WaveformSlot<F>) -> WaveformSlot<F> {
    let WaveformSlot::Vector(vin) = v else {
        panic!("MetricVout: expected vector input");
    };
    WaveformSlot::Vector(metric_vout_c(vin))
}

/// `MetricVout`: the contravariant current `g^{μν}V_ν = V^μ` — an identity on
/// contravariant storage.
pub fn metric_vout_c<F: Real>(vin: &VectorWf<F>) -> VectorWf<F> {
    *vin
}

/// `EpsilonVout`: three vector currents → the off-shell vector `ε^{μνρσ} a_μ b_ν c_ρ`.
pub fn epsilon_vout<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    c: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let [va, vb, vc] = expect_vectors([a, b, c]);
    WaveformSlot::Vector(VectorWf {
        eps: epsilon_vout_bare(&va.eps, &vb.eps, &vc.eps),
        momentum: va.momentum + vb.momentum + vc.momentum,
    })
}

/// `EpsilonAmp`: four vector currents → the scalar `ε^{μνρσ} a_μ b_ν c_ρ d_σ`.
pub fn epsilon_amp<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    c: &WaveformSlot<F>,
    d: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let [va, vb, vc, vd] = expect_vectors([a, b, c, d]);
    WaveformSlot::Scalar(ScalarWf {
        value: epsilon_amp_bare(&va.eps, &vb.eps, &vc.eps, &vd.eps),
        momentum: va.momentum + vb.momentum + vc.momentum + vd.momentum,
    })
}

// ──────────────────── tensor-tensor (cyclic four-fermion) kernels ────────────────────

/// `FierzOut`: the cut fermion line as a Clifford element (see [`fierz_out_bare`]).
pub fn fierz_out<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    fierz_out_current(a, b, false)
}

/// `FierzOutRev`: [`fierz_out`] with the two lines' shared indices in opposite orders.
pub fn fierz_out_rev<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    fierz_out_current(a, b, true)
}

/// The cut line's Clifford element, carrying the momentum a fermion pair routes
/// (`p_bra − p_ket`, as for [`gamma_vout`]).
pub fn fierz_out_current<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    reversed_order: bool,
) -> WaveformSlot<F> {
    let (fo, fi, _) = resolve_bra_ket(a, b);
    WaveformSlot::Multivector(MultivectorWf {
        m: fierz_out_bare(&fo.spinor, &fi.spinor, reversed_order),
        momentum: fo.momentum - fi.momentum,
    })
}

/// `MultivectorIout`: continue a flow-in (ket) fermion line by applying the Clifford
/// element the cut line handed over, `M ψ`. Same kernel as [`multivector_oout`] because
/// [`multivector_current`] follows the input fermion's adjoint.
pub fn multivector_iout<F: Real>(m: &WaveformSlot<F>, f: &WaveformSlot<F>) -> WaveformSlot<F> {
    multivector_current(m, f)
}

/// `MultivectorOout`: continue a flow-out (bra) fermion line, `ψ̄ M`. See
/// [`multivector_iout`].
pub fn multivector_oout<F: Real>(m: &WaveformSlot<F>, f: &WaveformSlot<F>) -> WaveformSlot<F> {
    multivector_current(m, f)
}

/// Off-shell fermion current from a tensor-tensor contact: the Clifford element of the
/// cut line applied to the continuing fermion. The current follows the input's adjoint
/// (`M ψ` on a ket, `ψ̄ M` on a bra) and routes the element's momentum with the sign that
/// adjoint dictates, exactly as [`off_shell_fermion_current`] does for a vector leg.
pub fn multivector_current<F: Real>(
    m: &WaveformSlot<F>,
    fermion: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let WaveformSlot::Multivector(m) = m else {
        panic!("multivector current: expected a Clifford-element input");
    };
    match fermion {
        WaveformSlot::FermionIn(fi) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            multivector_fin_bare(&m.m, &fi.spinor),
            fi.momentum - m.momentum,
        )),
        WaveformSlot::FermionOut(fo) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            multivector_fout_bare(&m.m, &fo.spinor),
            fo.momentum + m.momentum,
        )),
        _ => panic!("multivector current: expected fermion input"),
    }
}

/// `FierzPair`: close the surviving fermion line into the amplitude against the Clifford
/// element the cut line produced, `ψ̄ M ψ` (see [`fierz_pair_bare`]).
pub fn fierz_pair<F: Real>(
    m: &WaveformSlot<F>,
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    let WaveformSlot::Multivector(m) = m else {
        panic!("FierzPair: expected a Clifford-element input");
    };
    let (fo, fi, _) = resolve_bra_ket(a, b);
    WaveformSlot::Scalar(ScalarWf {
        value: fierz_pair_bare(&m.m, &fo.spinor, &fi.spinor),
        momentum: m.momentum + fo.momentum - fi.momentum,
    })
}

/// `SigmaVout`: two fermions and a vector → the off-shell vector current
/// `(ψ̄ Σ^{μν} ψ) v_ν` (see [`sigma_vout_bare`]).
pub fn sigma_vout<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    v: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    sigma_vout_current(a, b, v, false)
}

/// `SigmaVoutRev`: [`sigma_vout`] with the free index on `Sigma`'s second Lorentz
/// slot, `(ψ̄ Σ^{νμ} ψ) v_ν` — the negative of it.
pub fn sigma_vout_rev<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    v: &WaveformSlot<F>,
) -> WaveformSlot<F> {
    sigma_vout_current(a, b, v, true)
}

/// The `Sigma` vector current on slots. The pair's momentum enters as a fermion
/// bilinear's (`p_bra − p_ket`) and the contracted vector adds its own, as it does at
/// any other vertex that reads a vector input.
pub fn sigma_vout_current<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    v: &WaveformSlot<F>,
    swapped: bool,
) -> WaveformSlot<F> {
    let (fo, fi, reversed) = resolve_bra_ket(a, b);
    let WaveformSlot::Vector(v) = v else {
        panic!("Sigma vector current: expected a vector input");
    };
    WaveformSlot::Vector(VectorWf {
        eps: sigma_vout_bare(&fo.spinor, &fi.spinor, &v.eps, reversed != swapped),
        momentum: fo.momentum - fi.momentum + v.momentum,
    })
}

/// `SigmaMv`: two vectors → the Clifford element `Σ^{μν} a_μ b_ν`
/// (see [`sigma_mv_bare`]).
pub fn sigma_mv<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    let [a, b] = expect_vectors([a, b]);
    WaveformSlot::Multivector(MultivectorWf {
        m: sigma_mv_bare(&a.eps, &b.eps),
        momentum: a.momentum + b.momentum,
    })
}

/// `SigmaOut`: the cut line of a `Sigma ⊗ Sigma` contact as a Clifford element
/// (see [`sigma_out_bare`]), carrying the momentum a fermion pair routes.
pub fn sigma_out<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    sigma_out_current(a, b, false)
}

/// `SigmaOutRev`: [`sigma_out`] with the two lines' shared indices in opposite orders.
pub fn sigma_out_rev<F: Real>(a: &WaveformSlot<F>, b: &WaveformSlot<F>) -> WaveformSlot<F> {
    sigma_out_current(a, b, true)
}

/// The cut `Sigma` line's Clifford element on slots.
pub fn sigma_out_current<F: Real>(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    reversed_order: bool,
) -> WaveformSlot<F> {
    let (fo, fi, _) = resolve_bra_ket(a, b);
    WaveformSlot::Multivector(MultivectorWf {
        m: sigma_out_bare(&fo.spinor, &fi.spinor, reversed_order),
        momentum: fo.momentum - fi.momentum,
    })
}

/// The vector currents behind `N` slots, panicking on any non-vector operand.
fn expect_vectors<F: Real, const N: usize>(slots: [&WaveformSlot<F>; N]) -> [VectorWf<F>; N] {
    slots.map(|s| match s {
        WaveformSlot::Vector(v) => *v,
        other => panic!("Epsilon: expected vector input, got {other:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::eval::prop_harness::{check_agree, rand_bra, rand_c, rand_ket, rand_vector};
    use crate::helas::repr::lorentz::{LorentzVector, Multivector};

    /// The fused-kernel oracle tolerance: far tighter than the whole-amplitude MG
    /// gate (1e-12), since a single kernel has few compounding roundings — fusion
    /// only reorders the per-chirality coupling multiplications.
    const FUSED_TOL: f64 = 1e-14;

    fn scalar_slot(g: C<f64>) -> WaveformSlot<f64> {
        WaveformSlot::Scalar(ScalarWf {
            value: g,
            momentum: LorentzVector::zero(),
        })
    }

    /// Tolerance for the Clifford-algebra oracles below: both sides evaluate the
    /// same bilinears through different routes (stored-component kernels against
    /// the graded Dirac basis, which goes through a 4×4 matrix product), so the
    /// gap is a handful of roundings on inputs of order one.
    const CLIFFORD_TOL: f64 = 1e-13;

    fn as_vector(slot: &WaveformSlot<f64>) -> ComplexVector<f64> {
        match slot {
            WaveformSlot::Vector(v) => v.eps,
            other => panic!("expected a vector slot, got {other:?}"),
        }
    }

    fn as_scalar(slot: &WaveformSlot<f64>) -> C<f64> {
        match slot {
            WaveformSlot::Scalar(s) => s.value,
            other => panic!("expected a scalar slot, got {other:?}"),
        }
    }

    /// The line's sixteen bilinears, the basis every oracle below is written in.
    fn line(bra: &WaveformSlot<f64>, ket: &WaveformSlot<f64>) -> Multivector<f64> {
        let (fo, fi, reversed) = resolve_bra_ket(bra, ket);
        assert!(!reversed, "the oracles pass (bra, ket) in that order");
        fo.spinor.fierz_coefficients(&fi.spinor)
    }

    /// A γ-chain composed by the evaluator is the Clifford product of its factors.
    ///
    /// `GammaVout(ψ̄, GammaIout(p, ψ))` is `ψ̄ γ^μ p̸ ψ` and
    /// `GammaVout(GammaOout(p, ψ̄), ψ)` is `ψ̄ p̸ γ^μ ψ`: the same two gammas in
    /// opposite orders, which is the whole content of a dipole structure
    /// (`γ^μ p̸ − p̸ γ^μ = −2i σ^{μν} p_ν`) and the one thing a bilinear that
    /// discarded the ordering would get wrong. Contracting the free index with an
    /// arbitrary `q` turns each into a scalar the graded basis states directly:
    /// `ψ̄ q̸ p̸ ψ = ⟨fierz(ψ̄, ψ), q̸ p̸⟩`.
    #[test]
    fn gamma_chain_order_is_the_clifford_product() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x6A11A_01);
        for _ in 0..256 {
            let (bra, ket) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let (p, q) = (rand_vector(&mut rng), rand_vector(&mut rng));
            let coeffs = line(&bra, &ket);
            let (pv, qv) = (as_vector(&p), as_vector(&q));

            let ket_side = metric(&gamma_vout(&bra, &gamma_iout(&p, &ket)), &q);
            let expected = coeffs.fierz_pairing(&Multivector::from_gamma_pair(&qv, &pv));
            assert!(
                (as_scalar(&ket_side) - expected).norm() < CLIFFORD_TOL,
                "psi-bar q-slash p-slash psi: kernel {:?} vs Clifford {expected:?}",
                as_scalar(&ket_side)
            );

            let bra_side = metric(&gamma_vout(&gamma_oout(&p, &bra), &ket), &q);
            let expected = coeffs.fierz_pairing(&Multivector::from_gamma_pair(&pv, &qv));
            assert!(
                (as_scalar(&bra_side) - expected).norm() < CLIFFORD_TOL,
                "psi-bar p-slash q-slash psi: kernel {:?} vs Clifford {expected:?}",
                as_scalar(&bra_side)
            );
        }
    }

    /// `Gamma5Amp` is the pseudoscalar bilinear, and `Gamma5` on a continuing
    /// current is `γ⁵` acting from the side the current's adjoint dictates: pinned
    /// on both flows against the graded basis, where `γ⁵ = P_R − P_L`.
    #[test]
    fn gamma5_acts_as_the_chirality_matrix_on_either_flow() {
        let g5: Multivector<f64> = Multivector::from_projector(Chirality::Right)
            - Multivector::from_projector(Chirality::Left);
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x6A11A_02);
        for _ in 0..256 {
            let (bra, ket) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let coeffs = line(&bra, &ket);
            let expected = coeffs.fierz_pairing(&g5);

            assert!(
                (as_scalar(&gamma5_amp(&bra, &ket)) - expected).norm() < CLIFFORD_TOL,
                "Gamma5Amp is not the pseudoscalar bilinear"
            );
            // γ⁵ on the ket, then the plain bilinear — the same number.
            assert!(
                (as_scalar(&identity_amp(&bra, &gamma5(&ket))) - expected).norm() < CLIFFORD_TOL,
                "Gamma5 on the ket does not reproduce psi-bar gamma5 psi"
            );
            // and on the bra, which is the case a γ⁵ mid-chain on a bra line hits.
            assert!(
                (as_scalar(&identity_amp(&gamma5(&bra), &ket)) - expected).norm() < CLIFFORD_TOL,
                "Gamma5 on the bra does not reproduce psi-bar gamma5 psi"
            );
        }
    }

    /// A γ⁵ inside a chain composes as the Clifford product, on either side of the
    /// slash — the structure SMEFTsim's CP-odd dipole (`FFV2`) is built from.
    #[test]
    fn gamma5_inside_a_chain_is_the_clifford_product() {
        let g5: Multivector<f64> = Multivector::from_projector(Chirality::Right)
            - Multivector::from_projector(Chirality::Left);
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x6A11A_03);
        for _ in 0..256 {
            let (bra, ket) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let (p, q) = (rand_vector(&mut rng), rand_vector(&mut rng));
            let coeffs = line(&bra, &ket);
            let (pv, qv) = (as_vector(&p), as_vector(&q));

            // ψ̄ q̸ p̸ γ⁵ ψ
            let ours = metric(&gamma_vout(&bra, &gamma_iout(&p, &gamma5(&ket))), &q);
            let expected =
                coeffs.fierz_pairing(&Multivector::from_gamma_pair(&qv, &pv).clifford_product(&g5));
            assert!(
                (as_scalar(&ours) - expected).norm() < CLIFFORD_TOL,
                "gamma5 at the end of a two-gamma chain"
            );
        }
    }

    /// `EpsilonVout` is `EpsilonAmp` with one index left free: contracting the
    /// current with a fourth vector reproduces the fully contracted symbol, which
    /// is what makes the two kernels one object rooted two ways.
    #[test]
    fn epsilon_current_contracts_to_the_epsilon_scalar() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x6A11A_04);
        for _ in 0..256 {
            let (a, b, c, d) = (
                rand_vector(&mut rng),
                rand_vector(&mut rng),
                rand_vector(&mut rng),
                rand_vector(&mut rng),
            );
            let contracted = as_scalar(&metric(&epsilon_vout(&a, &b, &c), &d));
            let full = as_scalar(&epsilon_amp(&a, &b, &c, &d));
            assert!(
                (contracted - full).norm() < CLIFFORD_TOL,
                "EpsilonVout . d = {contracted:?} vs EpsilonAmp = {full:?}"
            );
            // Antisymmetry: one transposition flips the sign, and a repeated
            // argument annihilates it.
            let swapped = as_scalar(&epsilon_amp(&b, &a, &c, &d));
            assert!(
                (swapped + full).norm() < CLIFFORD_TOL,
                "epsilon antisymmetry"
            );
            assert!(
                as_scalar(&epsilon_amp(&a, &a, &c, &d)).norm() < CLIFFORD_TOL,
                "epsilon with a repeated argument"
            );
        }
    }

    /// The Levi-Civita convention, stated where the evaluator uses it.
    ///
    /// `epsilon_amp` takes contravariant arguments and returns the *all-lower*
    /// symbol, `ε_{μνρσ} a^μ b^ν c^ρ d^σ`, which is `+1` on the ordered basis
    /// `(e₀, e₁, e₂, e₃)`. ALOHA stores the upper-index component
    /// (`aloha_object.py::L_Epsilon.give_parity`, `ε^{0123} = −1`) and applies the
    /// metric at contraction time, so the two differ by the determinant of the
    /// metric — the trap this test exists to keep visible. A flipped convention
    /// fails here before it reaches a process.
    #[test]
    fn epsilon_amp_returns_the_all_lower_symbol() {
        let e = |i: usize| {
            let mut c = [C::new(0.0, 0.0); 4];
            c[i] = C::new(1.0, 0.0);
            WaveformSlot::Vector(VectorWf {
                eps: ComplexVector::new(c),
                momentum: LorentzVector::zero(),
            })
        };
        let value = as_scalar(&epsilon_amp(&e(0), &e(1), &e(2), &e(3)));
        assert_eq!(value, C::new(1.0, 0.0), "epsilon_{{0123}} = +1");
        // Which is minus ALOHA's stored upper-index component.
        assert_eq!(-value, C::new(-1.0, 0.0), "ALOHA's epsilon^{{0123}} = -1");
    }

    /// `FfvVout` equals the generic chiral pair `g_L·GammaVout(a, ProjM(b)) +
    /// g_R·GammaVout(a, ProjP(b))` — in both operand orders, so the reversed-line
    /// coupling swap is certified too.
    #[test]
    fn ffv_vout_matches_generic_chiral_pair() {
        for (seed, swap) in [(0x11FA01u64, false), (0x11FA02, true)] {
            check_agree(
                512,
                seed,
                FUSED_TOL,
                move |rng| {
                    let (a, b) = if swap {
                        (rand_ket(rng), rand_bra(rng))
                    } else {
                        (rand_bra(rng), rand_ket(rng))
                    };
                    vec![a, b, scalar_slot(rand_c(rng)), scalar_slot(rand_c(rng))]
                },
                |c| ffv_vout(&c[0], &c[1], &c[2], &c[3]),
                |c| {
                    let (gl, gr) = (expect_scalar(&c[2]), expect_scalar(&c[3]));
                    gl * gamma_vout(&c[0], &proj_m(&c[1])) + gr * gamma_vout(&c[0], &proj_p(&c[1]))
                },
            );
        }
    }

    /// `FfvIout`/`FfvOout` equal the generic chiral pair
    /// `g_L·GammaXout(v, ProjM(f)) + g_R·GammaXout(v, ProjP(f))`.
    #[test]
    fn ffv_fermion_out_matches_generic_chiral_pair() {
        type Fused = fn(
            &WaveformSlot<f64>,
            &WaveformSlot<f64>,
            &WaveformSlot<f64>,
            &WaveformSlot<f64>,
        ) -> WaveformSlot<f64>;
        type Kernel = fn(&WaveformSlot<f64>, &WaveformSlot<f64>) -> WaveformSlot<f64>;
        type Gen = fn(&mut rand::rngs::StdRng) -> WaveformSlot<f64>;
        let cases: [(Fused, Kernel, Gen, u64); 2] = [
            (ffv_iout, gamma_iout, rand_ket, 0x11FA03),
            (ffv_oout, gamma_oout, rand_bra, 0x11FA04),
        ];
        for (fused, generic, gen_f, seed) in cases {
            check_agree(
                512,
                seed,
                FUSED_TOL,
                move |rng| {
                    vec![
                        rand_vector(rng),
                        gen_f(rng),
                        scalar_slot(rand_c(rng)),
                        scalar_slot(rand_c(rng)),
                    ]
                },
                |c| fused(&c[0], &c[1], &c[2], &c[3]),
                |c| {
                    let (gl, gr) = (expect_scalar(&c[2]), expect_scalar(&c[3]));
                    gl * generic(&c[0], &proj_m(&c[1])) + gr * generic(&c[0], &proj_p(&c[1]))
                },
            );
        }
    }

    /// The outer↔inner projector normalization used by the fusion rewrite:
    /// `ProjX(GammaXout(v, f)) = GammaXout(v, ProjX̄(f))` **bit-exactly** — the Weyl
    /// slash maps chiral storage blocks crosswise, so projecting after the slash
    /// selects exactly the values produced from the opposite projection before it.
    #[test]
    fn outer_projector_equals_flipped_inner_bit_exactly() {
        type Kernel = fn(&WaveformSlot<f64>, &WaveformSlot<f64>) -> WaveformSlot<f64>;
        type Gen = fn(&mut rand::rngs::StdRng) -> WaveformSlot<f64>;
        let cases: [(Kernel, Gen, u64); 2] = [
            (gamma_iout, rand_ket, 0x11FA05),
            (gamma_oout, rand_bra, 0x11FA06),
        ];
        for (gamma, gen_f, seed) in cases {
            check_agree(
                512,
                seed,
                0.0,
                move |rng| vec![rand_vector(rng), gen_f(rng)],
                |c| proj_m(&gamma(&c[0], &c[1])),
                |c| gamma(&c[0], &proj_p(&c[1])),
            );
            check_agree(
                512,
                seed ^ 1,
                0.0,
                move |rng| vec![rand_vector(rng), gen_f(rng)],
                |c| proj_p(&gamma(&c[0], &c[1])),
                |c| gamma(&c[0], &proj_m(&c[1])),
            );
        }
    }

    // `IdentityAmp` has no MG-validated process coverage (see
    // `mg_validated_suite_exercises_every_op`), so its kernel is pinned here
    // against the ops the MG net does exercise.

    /// `IdentityAmp` (ψ̄ 1 ψ) equals the sum of its chiral halves
    /// [`proj_m_amp`] + [`proj_p_amp`]: P_L + P_R = 1 and the bilinear is linear in Γ.
    #[test]
    fn identity_amp_is_chiral_projector_sum() {
        check_agree(
            256,
            0x11AA02,
            1e-12,
            |rng| vec![rand_bra(rng), rand_ket(rng)],
            |c| identity_amp(&c[0], &c[1]),
            |c| {
                let (WaveformSlot::Scalar(m), WaveformSlot::Scalar(p)) =
                    (proj_m_amp(&c[0], &c[1]), proj_p_amp(&c[0], &c[1]))
                else {
                    panic!("chiral bilinears produced non-scalars");
                };
                WaveformSlot::Scalar(ScalarWf {
                    value: m.value + p.value,
                    momentum: m.momentum,
                })
            },
        );
    }
    // ───────────── tensor-tensor (cyclic four-fermion) contact ─────────────

    /// The Weyl-basis matrix of `γ_μ` for each `μ`, from the graded basis's own faithful
    /// representation (pinned against hand-built gamma matrices in `repr::lorentz`).
    fn gamma_lower(mu: usize) -> [[C<f64>; 4]; 4] {
        let mut e = [C::zero(); 4];
        e[mu] = C::new(1.0, 0.0);
        Multivector::from_gamma(&ComplexVector::new(e)).to_weyl_matrix()
    }

    /// `ψ̄ A B ψ` from explicit 4×4 matrices, index by index.
    fn chain_bilinear(
        bra: &WaveformSlot<f64>,
        ket: &WaveformSlot<f64>,
        a: &[[C<f64>; 4]; 4],
        b: &[[C<f64>; 4]; 4],
    ) -> C<f64> {
        let (fo, fi, reversed) = resolve_bra_ket(bra, ket);
        assert!(!reversed, "the oracle passes (bra, ket) in that order");
        let mut total: C<f64> = C::zero();
        for (i, arow) in a.iter().enumerate() {
            for (k, &aik) in arow.iter().enumerate() {
                for (j, &bkj) in b[k].iter().enumerate() {
                    total += fo.spinor.component(i) * aik * bkj * fi.spinor.component(j);
                }
            }
        }
        total
    }

    /// The tensor-tensor contact against a direct 4×4 evaluation of the same two chains.
    ///
    /// `Σ_{αβ} [ψ̄₁ γ^α γ^β ψ₁][ψ̄₂ γ_α γ_β ψ₂]` written out over sixteen index pairs with
    /// explicit gamma matrices is the whole structure with nothing factored out — it
    /// knows nothing of the graded basis, of Fierz orthogonality, or of the `4s ∓ 2t`
    /// reconstruction, so it sees the normalisation, the relative weight of the two
    /// grades *and* the index order. Both orders are checked, and the two differ, which
    /// is what makes the order a measurement rather than a coincidence.
    ///
    /// Blind spots: it evaluates the pair in the vertex's own orientation, so it says
    /// nothing about the crossed-line and against-the-arrow readings (those are the
    /// rooting's, pinned by `rooting_soundness` and by the MadGraph row), and it fixes
    /// no overall phase convention beyond the one `to_weyl_matrix` already carries.
    #[test]
    fn tensor_contact_matches_the_direct_gamma_matrix_chains() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x7E1150_01);
        // g^{αα} per index: γ^α = g^{αα} γ_α, no sum.
        let raise = [1.0f64, -1.0, -1.0, -1.0];
        for _ in 0..64 {
            let (bra1, ket1) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let (bra2, ket2) = (rand_bra(&mut rng), rand_ket(&mut rng));

            let mut aligned: C<f64> = C::zero();
            let mut reversed: C<f64> = C::zero();
            for alpha in 0..4 {
                for beta in 0..4 {
                    let (ga, gb) = (gamma_lower(alpha), gamma_lower(beta));
                    let weight = C::new(raise[alpha] * raise[beta], 0.0);
                    let one = chain_bilinear(&bra1, &ket1, &ga, &gb);
                    aligned += weight * one * chain_bilinear(&bra2, &ket2, &ga, &gb);
                    reversed += weight * one * chain_bilinear(&bra2, &ket2, &gb, &ga);
                }
            }

            let ours_aligned = as_scalar(&fierz_pair(&fierz_out(&bra2, &ket2), &bra1, &ket1));
            let ours_reversed = as_scalar(&fierz_pair(&fierz_out_rev(&bra2, &ket2), &bra1, &ket1));
            let scale = aligned.norm().max(reversed.norm()).max(1.0);
            assert!(
                (ours_aligned - aligned).norm() < CLIFFORD_TOL * scale,
                "aligned tensor contact: {ours_aligned:?} vs direct {aligned:?}"
            );
            assert!(
                (ours_reversed - reversed).norm() < CLIFFORD_TOL * scale,
                "reversed tensor contact: {ours_reversed:?} vs direct {reversed:?}"
            );
            assert!(
                (aligned - reversed).norm() > 1e-6 * scale,
                "the two index orders coincided on this draw, so the check is vacuous"
            );

            // Which line is cut is a choice the rooting makes (at the amplitude sink
            // neither line carries the output leg), so the contraction must not care:
            // `4 s_A s_B ∓ t_A·t_B` is symmetric under exchanging the lines.
            let swapped_aligned = as_scalar(&fierz_pair(&fierz_out(&bra1, &ket1), &bra2, &ket2));
            let swapped_reversed =
                as_scalar(&fierz_pair(&fierz_out_rev(&bra1, &ket1), &bra2, &ket2));
            assert!(
                (swapped_aligned - ours_aligned).norm() < CLIFFORD_TOL * scale
                    && (swapped_reversed - ours_reversed).norm() < CLIFFORD_TOL * scale,
                "cutting the other line changed the contact"
            );
        }
    }

    /// The cut line's element is grades 0 and 2 alone, and the two index orders differ
    /// only in the sign of the grade-2 part.
    ///
    /// This is the structural half of `γ^αγ^β = g^{αβ} − i σ^{αβ}`: a chain that leaked
    /// weight into the vector, axial or pseudoscalar grades would still pair correctly
    /// against another `γγ` chain (those grades meet zeros) and only show up once a
    /// literal `Sigma` or a longer chain reaches the same slot.
    #[test]
    fn the_cut_line_element_is_grades_zero_and_two() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x7E1150_02);
        for _ in 0..64 {
            let (bra, ket) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let WaveformSlot::Multivector(a) = fierz_out(&bra, &ket) else {
                panic!("FierzOut must produce a Clifford element");
            };
            let WaveformSlot::Multivector(r) = fierz_out_rev(&bra, &ket) else {
                panic!("FierzOutRev must produce a Clifford element");
            };
            for mu in 0..4 {
                assert!(a.m.vector().component(mu).norm() == 0.0);
                assert!(a.m.axial().component(mu).norm() == 0.0);
            }
            assert!(a.m.pseudoscalar().norm() == 0.0);
            assert_eq!(a.m.scalar(), r.m.scalar());
            for slot in 0..6 {
                assert!(
                    (a.m.bivector().component(slot) + r.m.bivector().component(slot)).norm()
                        < CLIFFORD_TOL,
                    "the two index orders must differ by the grade-2 sign alone"
                );
            }
            assert!(
                a.m.bivector().component(0).norm() > 0.0,
                "a vanishing grade-2 part would make the sign check vacuous"
            );
        }
    }

    /// Applying the element to either end of the surviving line gives the same number as
    /// pairing it with the line, on both flows: `ψ̄ (M ψ) = (ψ̄ M) ψ = ⟨fierz(ψ̄, ψ), M⟩`.
    ///
    /// This is what `MultivectorIout`/`MultivectorOout` rest on — a four-fermion contact
    /// on an internal line roots at a fermion leg and produces a current instead of a
    /// number, and no gated row does that. The element is drawn with all sixteen grades
    /// independent, so the identity is pinned for an element a literal `Sigma` (grade 2
    /// only) or a longer chain would produce, not just for a `γγ` one.
    #[test]
    fn the_clifford_element_applies_to_either_end_of_the_line() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x7E1150_03);
        for _ in 0..256 {
            let (bra, ket) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let m = crate::helas::eval::prop_harness::rand_multivector(&mut rng);
            let paired = as_scalar(&fierz_pair(&m, &bra, &ket));
            let on_ket = as_scalar(&identity_amp(&bra, &multivector_iout(&m, &ket)));
            let on_bra = as_scalar(&identity_amp(&multivector_oout(&m, &bra), &ket));
            let scale = paired.norm().max(1.0);
            assert!(
                (on_ket - paired).norm() < CLIFFORD_TOL * scale,
                "M applied to the ket: {on_ket:?} vs the pairing {paired:?}"
            );
            assert!(
                (on_bra - paired).norm() < CLIFFORD_TOL * scale,
                "M applied to the bra: {on_bra:?} vs the pairing {paired:?}"
            );
        }
    }

    /// Every grade's worst deviation between two Clifford elements.
    fn mv_max_diff(a: &Multivector<f64>, b: &Multivector<f64>) -> f64 {
        let d = *a - *b;
        let mut m = d.scalar().norm().max(d.pseudoscalar().norm());
        for k in 0..4 {
            m = m
                .max(d.vector().component(k).norm())
                .max(d.axial().component(k).norm());
        }
        for k in 0..6 {
            m = m.max(d.bivector().component(k).norm());
        }
        m
    }

    fn as_multivector(slot: &WaveformSlot<f64>) -> Multivector<f64> {
        match slot {
            WaveformSlot::Multivector(m) => m.m,
            other => panic!("expected a Clifford-element slot, got {other:?}"),
        }
    }

    /// `Σ^{μν} a_μ b_ν` against the two γ-chains it is built from:
    /// `a̸ b̸ = a·b − i σ^{μν} a_μ b_ν` makes `σ^{μν} a_μ b_ν = (i/2)(a̸b̸ − b̸a̸)`, and
    /// ALOHA's `Sigma` is half of that.
    ///
    /// [`Multivector::from_gamma_pair`] is the independent route: it is a closed
    /// coefficient form checked against explicit 4×4 Weyl matrices in the repr layer
    /// and knows nothing of [`sigma_mv_bare`]'s wedge. Both the normalisation (the
    /// quarter, not a half) and the index order are pinned, the latter because
    /// swapping `a` and `b` negates one side and not the other.
    #[test]
    fn the_sigma_element_is_half_the_commutator_of_its_two_gammas() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x516_4A_01);
        for _ in 0..64 {
            let (a, b) = (rand_vector(&mut rng), rand_vector(&mut rng));
            let (av, bv) = (as_vector(&a), as_vector(&b));
            let quarter_i = C::new(0.0, 0.25);
            let expect = (Multivector::from_gamma_pair(&av, &bv)
                - Multivector::from_gamma_pair(&bv, &av))
                * quarter_i;
            let ours = as_multivector(&sigma_mv(&a, &b));
            let scale = mv_max_diff(&expect, &Multivector::zero()).max(1.0);
            assert!(
                mv_max_diff(&ours, &expect) < CLIFFORD_TOL * scale,
                "SigmaMv against (i/4)(a̸b̸ − b̸a̸)"
            );
            // Grade 2 alone, and antisymmetric in its two arguments.
            assert!(ours.scalar().norm() == 0.0 && ours.pseudoscalar().norm() == 0.0);
            let swapped = as_multivector(&sigma_mv(&b, &a));
            assert!(mv_max_diff(&swapped, &(-ours)) < CLIFFORD_TOL * scale);
            assert!(
                mv_max_diff(&ours, &Multivector::zero()) > 1e-6,
                "a vanishing element would make the antisymmetry check vacuous"
            );
        }
    }

    /// The `Sigma` vector current is the same object read at one free index:
    /// `[(ψ̄ Σ^{μν} ψ) v_ν] w_μ = ψ̄ (Σ^{μν} w_μ v_ν) ψ`.
    ///
    /// It ties [`sigma_vout_bare`] to [`sigma_mv_bare`] — one goes through the pair's
    /// tensor bilinear and a one-index contraction, the other through a wedge of two
    /// vectors and the graded pairing — so the ½, the index order and the variance
    /// have to agree on both routes. The `Rev` spelling (free index on the second
    /// Lorentz slot) is the negative, which is `σ`'s antisymmetry, and reading the pair
    /// against the vertex's adjoint is the `C σ^{μνT} C⁻¹ = −σ^{μν}` sign.
    #[test]
    fn the_sigma_vector_current_is_the_element_at_one_free_index() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x516_4A_02);
        for _ in 0..64 {
            let (bra, ket) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let (v, w) = (rand_vector(&mut rng), rand_vector(&mut rng));
            let current = sigma_vout(&bra, &ket, &v);
            let contracted = as_scalar(&metric(&current, &w));
            let paired = as_scalar(&fierz_pair(&sigma_mv(&w, &v), &bra, &ket));
            let scale = paired.norm().max(1.0);
            assert!(
                (contracted - paired).norm() < CLIFFORD_TOL * scale,
                "SigmaVout contracted with w: {contracted:?} vs the paired element {paired:?}"
            );
            assert!(paired.norm() > 1e-9, "a vanishing draw would be vacuous");

            let rev = as_vector(&sigma_vout_rev(&bra, &ket, &v));
            let fwd = as_vector(&current);
            let reversed_pair = as_vector(&sigma_vout(&ket, &bra, &v));
            for mu in 0..4 {
                assert!((rev.component(mu) + fwd.component(mu)).norm() < CLIFFORD_TOL * scale);
                assert!(
                    (reversed_pair.component(mu) + fwd.component(mu)).norm() < CLIFFORD_TOL * scale
                );
            }
        }
    }

    /// The two spellings of one four-fermion tensor operator agree inside this crate:
    /// `Σ ⊗ Σ` is exactly a quarter of the γγ expansion the toy model's `FFFFG` writes.
    ///
    /// `σ^{μν} ⊗ σ_{μν} = −½[(γ^μγ^ν)⊗(γ_μγ_ν) − (γ^μγ^ν)⊗(γ_νγ_μ)]` is the expansion,
    /// and ALOHA's `Sigma` carries a ½ per line, so the literal spelling is a quarter of
    /// it. The right-hand side runs entirely through the already-MadGraph-gated
    /// [`fierz_out`]/[`fierz_out_rev`] path, so this pins the new `Sigma` cut's
    /// normalisation *and* its sign against a route that has an external oracle —
    /// before the toy row is consulted. The `−⅛` is the product of the two: a kernel at
    /// the textbook normalisation would be four times too large here.
    #[test]
    fn the_sigma_contact_is_a_quarter_of_its_gamma_gamma_expansion() {
        let mut rng = crate::helas::eval::prop_harness::seeded_rng(0x516_4A_03);
        for _ in 0..64 {
            let (bra1, ket1) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let (bra2, ket2) = (rand_bra(&mut rng), rand_ket(&mut rng));
            let aligned = as_scalar(&fierz_pair(&fierz_out(&bra2, &ket2), &bra1, &ket1));
            let reversed = as_scalar(&fierz_pair(&fierz_out_rev(&bra2, &ket2), &bra1, &ket1));
            let expect = (aligned - reversed) * -0.125;
            let ours = as_scalar(&fierz_pair(&sigma_out(&bra2, &ket2), &bra1, &ket1));
            let scale = aligned.norm().max(reversed.norm()).max(1.0);
            assert!(
                (ours - expect).norm() < CLIFFORD_TOL * scale,
                "Sigma-spelled contact {ours:?} vs a quarter of the γγ expansion {expect:?}"
            );
            assert!(
                expect.norm() > 1e-9 * scale,
                "the expansion vanished on this draw, so the check is vacuous"
            );

            // The cut is pure grade 2 — a literal `Sigma` has no `g^{αβ}` term to leave
            // a scalar behind — and the two index orders differ by its sign alone.
            let cut = as_multivector(&sigma_out(&bra2, &ket2));
            let cut_rev = as_multivector(&sigma_out_rev(&bra2, &ket2));
            assert!(cut.scalar().norm() == 0.0 && cut.pseudoscalar().norm() == 0.0);
            for mu in 0..4 {
                assert!(cut.vector().component(mu).norm() == 0.0);
                assert!(cut.axial().component(mu).norm() == 0.0);
            }
            assert!(mv_max_diff(&cut_rev, &(-cut)) < CLIFFORD_TOL);
            assert!(mv_max_diff(&cut, &Multivector::zero()) > 1e-9);
        }
    }
}
