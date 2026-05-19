use crate::helas::repr::{C, Real, SpinorRepr};

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// A Dirac spinor wavefunction together with its (signed) 4-momentum.
///
/// `momentum` stores `p * nsf`:  positive for particles, negative for
/// antiparticles.  This matches the HELAS convention used when building
/// currents and computing the s-channel propagator momentum.
#[derive(Clone, Copy, Debug)]
pub struct DiracWf<F: Real, B: SpinorRepr<F>> {
    pub spinor: B::Spinor,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: [F; 4],
}

impl<F: Real, B: SpinorRepr<F>> DiracWf<F, B> {
    /// Construct a flowing-IN wavefunction.
    pub fn ixxxxx(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> Self {
        let spinor = B::ixxxxx(p, mass, nhel, nsf);
        let sf = F::from(nsf).unwrap();
        DiracWf {
            spinor,
            momentum: [p[0] * sf, p[1] * sf, p[2] * sf, p[3] * sf],
        }
    }

    /// Construct a flowing-OUT wavefunction.
    pub fn oxxxxx(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> Self {
        let spinor = B::oxxxxx(p, mass, nhel, nsf);
        let sf = F::from(nsf).unwrap();
        DiracWf {
            spinor,
            momentum: [p[0] * sf, p[1] * sf, p[2] * sf, p[3] * sf],
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
