use crate::helas::repr::{
    C, Real, SpinorRepr,
    lorentz::{Charge, FourMomentum, SpinorHelicity},
};

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

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
pub struct DiracWf<F: Real, B: SpinorRepr<F>> {
    pub spinor: B::Fiber,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: FourMomentum<F>,
}

impl<F: Real, B: SpinorRepr<F>> DiracWf<F, B> {
    /// Construct a flowing-IN wavefunction.
    pub fn ixxxxx(p: FourMomentum<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = B::ixxxxx(p, mass, nhel, nsf);
        DiracWf {
            spinor,
            momentum: p.scaled(nsf.sign()),
        }
    }

    /// Construct a flowing-OUT wavefunction.
    pub fn oxxxxx(p: FourMomentum<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = B::oxxxxx(p, mass, nhel, nsf);
        DiracWf {
            spinor,
            momentum: p.scaled(nsf.sign()),
        }
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
    /// Polarisation / Lorentz components (covariant, metric signs already
    /// absorbed for `iovxxx` contraction).
    pub eps: [C<F>; 4],
    pub momentum: [F; 4],
}
