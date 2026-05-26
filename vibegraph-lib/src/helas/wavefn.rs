use crate::helas::repr::lorentz::{Bispinor, Charge, ComplexVector, LorentzVector, SpinorHelicity};
use crate::helas::repr::Real;
use std::marker::PhantomData;

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// Marker for flowing-IN spinors (`u`/`v` columns).
#[derive(Clone, Copy, Debug)]
pub struct FlowIn;

/// Marker for flowing-OUT spinors (`ū`/`v̄` rows).
#[derive(Clone, Copy, Debug)]
pub struct FlowOut;

/// A Dirac spinor wavefunction together with its (signed) 4-momentum.
///
/// `momentum` stores `p * nsf.sign()`: positive for particles, negative for
/// antiparticles.  This matches the HELAS convention used when building
/// currents and computing the s-channel propagator momentum.
///
/// `spinor` has type `B::Fiber` (= `[C<F>; 4]` for any `B: SpinorRepr<F>`),
/// since [`SpinorRepr<F>`] is a subtrait of [`crate::helas::repr::LorentzRepr<F>`]
/// with `Fiber = [C<F>; 4]`.
#[derive(Clone, Copy, Debug)]
pub struct DiracWf<F: Real, Flow = FlowIn> {
    pub spinor: Bispinor<F>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F>,
    _flow: PhantomData<Flow>,
}

/// Flowing-IN typed spinor wavefunction.
pub type InDiracWf<F> = DiracWf<F, FlowIn>;

/// Flowing-OUT typed spinor wavefunction.
pub type OutDiracWf<F> = DiracWf<F, FlowOut>;

impl<F: Real, Flow> DiracWf<F, Flow> {
    #[inline(always)]
    fn from_parts(spinor: Bispinor<F>, momentum: LorentzVector<F>) -> Self {
        Self {
            spinor,
            momentum,
            _flow: PhantomData,
        }
    }
}

impl<F: Real> InDiracWf<F> {
    /// Construct a flowing-IN wavefunction.
    pub fn new(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = Bispinor::ixxxxx(p, mass, nhel, nsf);
        Self::from_parts(spinor, p.scaled(nsf.sign()))
    }
}

impl<F: Real> OutDiracWf<F> {
    /// Construct a flowing-OUT wavefunction.
    pub fn new(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = Bispinor::oxxxxx(p, mass, nhel, nsf);
        Self::from_parts(spinor, p.scaled(nsf.sign()))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Vector (gauge boson) wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// An off-shell vector wavefunction: 4 complex polarisation components plus
/// the associated 4-momentum.
///
/// Used as both the result of `j3xxxx` and the input to `iovxxx`.
#[derive(Clone, Copy, Debug)]
pub struct VectorWf<F: Real> {
    /// Polarisation / Lorentz components in HELAS convention.
    ///
    /// `iovxxx` contracts these components with bilinear currents using an
    /// explicit Minkowski (+,−,−,−) contraction (`mink_dot`).
    pub eps: ComplexVector<F>,
    pub momentum: LorentzVector<F>,
}
