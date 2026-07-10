//! Lorentz-primitive evaluation kernels.
//!
//! Each `Op` that carries Lorentz semantics maps 1-to-1 to a `pub(crate)` kernel here,
//! named for the op (snake-cased): the runtime [`apply`](super::run) dispatch is just
//! `kernel::<op>(children)`. Ops that share an implementation get a thin per-op wrapper
//! over a shared private helper (`gamma_iout`/`gamma_oout` → [`off_shell_fermion_current`],
//! `proj_m`/`proj_p` → [`chiral_project`], `proj_m_amp`/`proj_p_amp`/`identity_amp` →
//! [`scalar_bilinear_current`]), so the op↔kernel mapping stays total and explicit.
//!
//! Structural/const/algebraic ops (`External`, `Mul`, `Add`, `Coupling`, `Mass`, `Width`,
//! `Coeff`) are not kernels — they live in [`apply`](super::run) itself. Each kernel takes
//! the already-evaluated `children` in operand order; the cross-check tests call them
//! directly.

use num_complex::ComplexFloat;
use num_traits::Zero;

use super::waveform_slot::WaveformSlot;
use crate::helas::repr::lorentz::{
    Bispinor, ComplexVector, DiracAdjoint, LorentzVector, SpinorRepr, VectorRepr,
};
use crate::helas::repr::numbers::Chirality;
use crate::helas::repr::{ri, Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};

/// Extract a bare real constant from a [`WaveformSlot::Real`] child.
pub(crate) fn expect_real<F: Real>(slot: WaveformSlot<F>) -> F {
    match slot {
        WaveformSlot::Real(r) => r,
        other => panic!("expected a real-constant slot, got {other:?}"),
    }
}

// ──────────────────────────── propagator ────────────────────────────

/// `Propagate`: apply a propagator (interned mass/width from the two real children) to the
/// off-shell current child. The propagator outputs a contravariant current.
pub(crate) fn propagate<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let mass = expect_real(children[1]);
    let width = expect_real(children[2]);
    propagate_core(&children[0], mass, width)
}

/// Apply a propagator with interned mass/width to an off-shell current. The current
/// already carries the conserved routed momentum (matching reference HELAS, where the
/// off-shell current routines output it: `fvixxx` q=fi−vc, `fvoxxx` q=fo+vc,
/// `jioxxx` jmom=fo−fi).
pub(crate) fn propagate_core<F: Real>(
    input: &WaveformSlot<F>,
    mass: F,
    width: F,
) -> WaveformSlot<F> {
    match input {
        // Dirac propagator: -i (q̸ + m) / (q² - m² + i m Γ). The -i puts the fermion
        // chain in phase with the vector chain (which is bit-validated against
        // MadGraph's W-arrays), so every off-shell chain type carries the same
        // phase relative to MadGraph and diagram classes with different chain
        // contents interfere correctly; pinned by the uux 2→6 per-diagram oracle
        // (validation/madgraph/compare_amps.py), where continuum diagrams
        // (two fermion propagators) meet H diagrams (one scalar propagator).
        WaveformSlot::FermionIn(wf) => {
            let num = wf.spinor.slash(&wf.momentum.into()) + wf.spinor * mass;
            let scale =
                ri(-F::one()) * C::new(wf.momentum.m2() - mass * mass, mass * width).recip();
            WaveformSlot::FermionIn(InDiracWf::from_spinor(num * scale, wf.momentum))
        }
        WaveformSlot::FermionOut(wf) => {
            let num = wf.spinor.slash(&wf.momentum.into()) + wf.spinor * mass;
            let scale =
                ri(-F::one()) * C::new(wf.momentum.m2() - mass * mass, mass * width).recip();
            WaveformSlot::FermionOut(OutDiracWf::from_spinor(num * scale, wf.momentum))
        }
        WaveformSlot::Vector(wf) => {
            let q = wf.momentum;
            if mass == F::zero() {
                WaveformSlot::Vector(VectorWf {
                    // -i / q^2
                    eps: wf.eps * ri(-q.m2().recip()),
                    momentum: q,
                })
            } else {
                let vm2 = mass * mass;
                let denom = C::new(q.m2() - vm2, mass * width);
                // -i (g - q q / m²) / (q² - m² + i m Γ). Real m² in the subtraction,
                // like ALOHA's OM3 = 1/M3².
                let cs = wf.eps.dot_lorentz(&q) / vm2;
                WaveformSlot::Vector(VectorWf {
                    eps: (wf.eps - ComplexVector::from(q) * cs) * ri(-F::one()) / denom,
                    momentum: q,
                })
            }
        }
        WaveformSlot::Scalar(wf) => {
            // Scalar propagator: -i / (q² - m² + i m Γ) — the same -i/D phase as
            // the vector and Dirac propagators, so every chain type propagates
            // uniformly. The compensating signs live in the scalar-sink vertex
            // roots (see `build_at_leg`'s scalar-root arms); the combination is
            // pinned per-diagram by the internal-H chains (ee→μμττ, uux 2→6, and
            // the b b̄ 2→6 spine-Yukawa diagrams) and the external-H chains
            // (e+e-→τ+τ-H) against MadGraph AMP().
            let denom = C::new(wf.momentum.m2() - mass * mass, mass * width);
            WaveformSlot::Scalar(ScalarWf {
                value: wf.value * ri(-F::one()) / denom,
                momentum: wf.momentum,
            })
        }
        WaveformSlot::Real(_) => panic!("propagate step read a real-constant slot"),
        WaveformSlot::Empty => panic!("propagate step read an empty slot"),
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

/// `PMom`: the 4-momentum of the single child input, as a vector current.
pub(crate) fn pmom<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let momentum = children[0].momentum().expect("PMom: empty slot");
    WaveformSlot::Vector(VectorWf {
        eps: ComplexVector::from(momentum),
        momentum: LorentzVector::zero(),
    })
}

/// `PMomOut`: the 4-momentum of the vertex's *output* leg, `−Σ (input momenta)`, as a
/// vector current.
pub(crate) fn pmom_out<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let momentum = -children.iter().fold(LorentzVector::zero(), |acc, c| {
        acc + c.momentum().expect("PMomOut: empty slot")
    });
    WaveformSlot::Vector(VectorWf {
        eps: ComplexVector::from(momentum),
        momentum: LorentzVector::zero(),
    })
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
    a: WaveformSlot<F>,
    b: WaveformSlot<F>,
) -> (OutDiracWf<F>, InDiracWf<F>, bool) {
    match (a, b) {
        (WaveformSlot::FermionOut(fo), WaveformSlot::FermionIn(fi)) => (fo, fi, false),
        (WaveformSlot::FermionIn(fi), WaveformSlot::FermionOut(fo)) => (fo, fi, true),
        _ => panic!("a fermion bilinear needs exactly one ket and one bra leg"),
    }
}

/// `GammaIout`: continue a flow-in (ket) fermion line by slashing it with the vector
/// current, `ε̸ψ`, q = f.p − v.p (Fortran `fvixxx`). Same kernel as [`gamma_oout`]
/// because [`off_shell_fermion_current`] follows the input fermion's adjoint.
pub(crate) fn gamma_iout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    off_shell_fermion_current(children[0], children[1])
}

/// `GammaOout`: continue a flow-out (bra) fermion line by slashing it with the vector
/// current, `ψ̄ε̸`, q = f.p + v.p (Fortran `fvoxxx`). See [`gamma_iout`].
pub(crate) fn gamma_oout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    off_shell_fermion_current(children[0], children[1])
}

/// Off-shell fermion current from an FFV `Gamma` vertex (one vector leg `mu` +
/// one continuing fermion leg `f`). The current **follows the input fermion's
/// adjoint**, so no mid-line Dirac adjoint is ever needed:
///   - ket: `ε̸ψ`, q = f.p − v.p   (Fortran `fvixxx`)
///   - bra: `ψ̄ε̸`, q = f.p + v.p   (Fortran `fvoxxx`)
///
/// `Bispinor::slash` is adjoint-dependent, so the left/right action is automatic.
/// The propagator `(q̸+m)/D` is applied in a separate `Propagate` step.
pub(crate) fn off_shell_fermion_current<F: Real>(
    v: WaveformSlot<F>,
    fermion: WaveformSlot<F>,
) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v) = v else {
        panic!("off-shell fermion current: expected vector input");
    };
    match fermion {
        WaveformSlot::FermionIn(fi) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            fi.spinor.slash(&v.eps),
            fi.momentum - v.momentum,
        )),
        WaveformSlot::FermionOut(fo) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            fo.spinor.slash(&v.eps),
            fo.momentum + v.momentum,
        )),
        _ => panic!("off-shell fermion current: expected fermion input"),
    }
}

/// `ProjM`: left chiral projection of a continuing fermion current. See [`chiral_project`].
pub(crate) fn proj_m<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    chiral_project(children[0], Chirality::Left)
}

/// `ProjP`: right chiral projection of a continuing fermion current. See [`chiral_project`].
pub(crate) fn proj_p<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    chiral_project(children[0], Chirality::Right)
}

/// `ProjM`/`ProjP`: chiral projection on a continuing fermion current, preserving the
/// input adjoint. `project_left`/`project_right` are adjoint-dependent (a bra projects
/// different components than a ket), so the same call is correct for both flows.
pub(crate) fn chiral_project<F: Real>(
    child: WaveformSlot<F>,
    chirality: Chirality,
) -> WaveformSlot<F> {
    fn project<F: Real, Fl: DiracAdjoint>(
        s: Bispinor<F, Fl>,
        chirality: Chirality,
    ) -> Bispinor<F, Fl> {
        match chirality {
            Chirality::Left => s.project_left(),
            Chirality::Right => s.project_right(),
            Chirality::Both => s,
        }
    }
    match child {
        WaveformSlot::FermionIn(f) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            project(f.spinor, chirality),
            f.momentum,
        )),
        WaveformSlot::FermionOut(f) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            project(f.spinor, chirality),
            f.momentum,
        )),
        _ => panic!("chiral projection: expected fermion input"),
    }
}

/// `GammaVout`: two fermions → off-shell vector current `ψ̄ γ^μ ψ`.
pub(crate) fn gamma_vout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let (fo, fi, reversed) = resolve_bra_ket(children[0], children[1]);
    let eps = fo.vector_bilinear(&fi, Chirality::Both);
    // Reading the fermion line against the vertex's defined adjoint conjugates the
    // structure as C γ^{μT} C⁻¹ = −γ^μ, so the vector current picks up a relative −1.
    // (Scalar/pseudoscalar structures have +1 and need no flip.)
    WaveformSlot::Vector(VectorWf {
        eps: if reversed { -eps } else { eps },
        momentum: fo.momentum - fi.momentum,
    })
}

/// `ProjMAmp`: left chiral scalar bilinear `ψ̄ P_L ψ`. See [`scalar_bilinear_current`].
pub(crate) fn proj_m_amp<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    scalar_bilinear_current(children, Chirality::Left)
}

/// `ProjPAmp`: right chiral scalar bilinear `ψ̄ P_R ψ`. See [`scalar_bilinear_current`].
pub(crate) fn proj_p_amp<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    scalar_bilinear_current(children, Chirality::Right)
}

/// `IdentityAmp`: full scalar bilinear `ψ̄ δ ψ`. See [`scalar_bilinear_current`].
pub(crate) fn identity_amp<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    scalar_bilinear_current(children, Chirality::Both)
}

/// `ProjMAmp`/`ProjPAmp`/`IdentityAmp`: scalar bilinear `ψ̄ Γ ψ` (`Γ = P_L`, `P_R`, or
/// `1`); the bra/ket are picked by the legs' actual adjoint.
pub(crate) fn scalar_bilinear_current<F: Real>(
    children: &[WaveformSlot<F>],
    chirality: Chirality,
) -> WaveformSlot<F> {
    let (fo, fi_col, _) = resolve_bra_ket(children[0], children[1]);
    let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, chirality);
    WaveformSlot::Scalar(ScalarWf {
        value,
        momentum: fo.momentum - fi_col.momentum,
    })
}

// ──────────────────────────── metric / vector currents ────────────────────────────

/// `Metric`: contract two vectors → scalar.
pub(crate) fn metric<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v1) = children[0] else {
        panic!("Metric: expected vector input");
    };
    let WaveformSlot::Vector(v2) = children[1] else {
        panic!("Metric: expected vector input");
    };
    WaveformSlot::Scalar(ScalarWf {
        value: v1.eps.dot(&v2.eps.lower()),
        momentum: v1.momentum + v2.momentum,
    })
}

/// `MetricNegI`: [`metric`] times the vertex's −i (a pure-metric structure rooted as an
/// amplitude).
pub(crate) fn metric_neg_i<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    match metric(children) {
        WaveformSlot::Scalar(s) => WaveformSlot::Scalar(ScalarWf {
            value: s.value * ri(-F::one()),
            momentum: s.momentum,
        }),
        other => panic!("MetricNegI produced a non-scalar: {other:?}"),
    }
}

/// `MetricVout`: off-shell vector current of a `Metric(out, v)` structure — the metric
/// contracts the output index against the partner vector `v`, `g^{μν}V_ν = V^μ`: the
/// physical contravariant current, an identity on contravariant storage. The UFO
/// coupling carries the vertex `i` and the propagator its `−i` (see `propagate_core`),
/// so no phase lives here (ALOHA's `VVS1P1N_1` = `−i·g·V` folds the propagator's −i
/// into the vertex routine instead). A trailing scalar leg (the Higgs) multiplies in
/// at the enclosing `Mul`.
pub(crate) fn metric_vout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let WaveformSlot::Vector(vin) = children[0] else {
        panic!("MetricVout: expected vector input");
    };
    WaveformSlot::Vector(vin)
}

/// `LowerVout`: [`metric_vout`] times the momentum-odd structure's −1 — the physical
/// contravariant current `−g^{μν}V_ν = −V^μ` of each P-carrying (VVV) structure term.
/// P-less structures (VVS) carry +1 and P-carrying ones −1 relative to the naive
/// rooted-term sum: the momentum-grade parity of rooting the UFO structure at the
/// off-shell leg. Pinned per-diagram against MadGraph's e+e-→W+W- AMP()
/// (validation/madgraph/compare_amps.py).
pub(crate) fn lower_vout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let WaveformSlot::Vector(vin) = children[0] else {
        panic!("LowerVout: expected vector input");
    };
    WaveformSlot::Vector(VectorWf {
        eps: -vin.eps,
        momentum: vin.momentum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::eval::prop_harness::{check_agree, rand_bra, rand_ket, rand_vector};

    // `MetricNegI` and `IdentityAmp` have no MG-validated process coverage (see
    // `mg_validated_suite_exercises_every_op`), so their kernels are pinned here
    // against the ops the MG net does exercise.

    /// `MetricNegI` is exactly −i × [`metric`] — the vertex −i of an amplitude-rooted
    /// pure-metric (VVS) contraction, and nothing else. Multiplying by −i is a
    /// component sign-shuffle, so the agreement is bit-exact.
    #[test]
    fn metric_neg_i_is_neg_i_times_metric() {
        check_agree(
            256,
            0x11AA01,
            0.0,
            |rng| vec![rand_vector(rng), rand_vector(rng)],
            |c| metric_neg_i(c),
            |c| match metric(c) {
                WaveformSlot::Scalar(s) => WaveformSlot::Scalar(ScalarWf {
                    value: s.value * ri(-1.0),
                    momentum: s.momentum,
                }),
                other => panic!("metric produced a non-scalar: {other:?}"),
            },
        );
    }

    /// `IdentityAmp` (ψ̄ 1 ψ) equals the sum of its chiral halves
    /// [`proj_m_amp`] + [`proj_p_amp`]: P_L + P_R = 1 and the bilinear is linear in Γ.
    #[test]
    fn identity_amp_is_chiral_projector_sum() {
        check_agree(
            256,
            0x11AA02,
            1e-12,
            |rng| vec![rand_bra(rng), rand_ket(rng)],
            |c| identity_amp(c),
            |c| {
                let (WaveformSlot::Scalar(m), WaveformSlot::Scalar(p)) =
                    (proj_m_amp(c), proj_p_amp(c))
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
