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

use super::waveform_slot::WaveformSlot;
use crate::helas::repr::lorentz::{
    epsilon4, epsilon_vector, Bispinor, Bra, ComplexVector, DiracAdjoint, Ket, LorentzVector,
    SpinorRepr, VectorRepr,
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
pub fn pseudoscalar_bilinear_bare<F: Real>(
    fo: &Bispinor<F, Bra>,
    fi: &Bispinor<F, Ket>,
) -> C<F> {
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

/// `PMomOut`: the 4-momentum of the vertex's *output* leg, `−Σ (input momenta)`, as a
/// vector current. The only variadic kernel (all vertex inputs), so it takes the
/// operands as an iterator rather than fixed arity.
pub fn pmom_out<'a, F: Real + 'a>(
    children: impl IntoIterator<Item = &'a WaveformSlot<F>>,
) -> WaveformSlot<F> {
    let momentum = -children.into_iter().fold(LorentzVector::zero(), |acc, c| {
        acc + c.momentum().expect("PMomOut: empty slot")
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
    use crate::helas::repr::lorentz::LorentzVector;

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
}
