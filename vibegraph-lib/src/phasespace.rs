//! Phase-space utilities for LO cross-section calculations.
//!
//! Implements the Lorentz-invariant phase space (LIPS) measure for 2-body
//! final states in the CM frame, and provides the unit-hypercube mapping used
//! by the VEGAS integrator.
//!
//! # 2-body phase space
//!
//! For a 2→2 process `a b → 1 2` with massless final-state particles the
//! only kinematic degree of freedom is the CM scattering angle θ.  The
//! differential phase-space weight is
//!
//! ```text
//! dΦ₂ / d(cosθ) = |p_cm| / (8π √s)
//! ```
//!
//! where `|p_cm| = √s / 2` in the massless limit, giving
//!
//! ```text
//! dΦ₂ / d(cosθ) = 1 / (16π)
//! ```
//!
//! # VEGAS mapping
//!
//! VEGAS operates on the unit interval `u ∈ [0, 1]`.  For the 2→2 case we
//! use the linear map
//!
//! ```text
//! cosθ = 2u − 1,   d(cosθ)/du = 2
//! ```
//!
//! so the combined Jacobian that the integrand must carry is
//!
//! ```text
//! J(u) = dΦ₂/d(cosθ) × d(cosθ)/du = 2 / (16π) = 1 / (8π)
//! ```
//!
//! # Cross-section formula (2→2, massless, initial-state spin sum)
//!
//! ```text
//! σ = 1/(2s) × (1/4) × ∫₋₁¹ Σ|M|² dΦ₂
//!   = 1/(64π s) × ∫₀¹ Σ|M|²(2u−1) du
//! ```
//!
//! where the `1/4` averages over the 4 initial-state helicity combinations
//! and `Σ|M|²` is the full spin sum (all 16 helicity combinations for
//! fermion–antifermion scattering).
//!
//! # Unit conversion
//!
//! Results in natural units (ℏ = c = 1) are in GeV⁻².
//! Multiply by [`GEV2_TO_PB`] to obtain picobarns.

use std::f64::consts::PI;

use rand::Rng;

use crate::helas::repr::lorentz::LorentzVector;

/// Conversion factor: 1 GeV⁻² = 3.893793721×10⁸ pb.
///
/// Derived from (ℏc)² = 0.3893793721 GeV²·mb and 1 mb = 10⁹ pb.
pub const GEV2_TO_PB: f64 = 3.893_793_721e8;

/// Massless RAMBO: `n` isotropic massless four-momenta with total momentum
/// `(√s, 0, 0, 0)` (Kleiss–Stirling–Ellis flat phase space, no mass rescale).
///
/// The phase-space weight is uniform, so the points serve as unbiased kinematics
/// for evaluator tests and benchmarks; pair with beam momenta `(√s/2, 0, 0, ±√s/2)`
/// for a full 2 → n external set.
pub fn rambo_massless(sqrt_s: f64, n: usize, rng: &mut impl Rng) -> Vec<LorentzVector<f64>> {
    assert!(n >= 2, "RAMBO needs at least two final-state momenta");
    // Isotropic null vectors q_i with q⁰ ~ Gamma(2): q⁰ = −ln(r₁·r₂).
    let q: Vec<[f64; 4]> = (0..n)
        .map(|_| {
            let c = 2.0 * rng.random::<f64>() - 1.0;
            let phi = 2.0 * PI * rng.random::<f64>();
            let e = -(rng.random::<f64>() * rng.random::<f64>()).ln();
            let st = (1.0 - c * c).sqrt();
            [e, e * st * phi.cos(), e * st * phi.sin(), e * c]
        })
        .collect();
    // Boost + scale the ensemble into the CM frame of total energy √s.
    let tot = q.iter().fold([0.0f64; 4], |acc, qi| {
        [
            acc[0] + qi[0],
            acc[1] + qi[1],
            acc[2] + qi[2],
            acc[3] + qi[3],
        ]
    });
    let m = (tot[0] * tot[0] - tot[1] * tot[1] - tot[2] * tot[2] - tot[3] * tot[3]).sqrt();
    let b = [-tot[1] / m, -tot[2] / m, -tot[3] / m];
    let gamma = tot[0] / m;
    let a = 1.0 / (1.0 + gamma);
    let x = sqrt_s / m;
    q.into_iter()
        .map(|qi| {
            let bq = b[0] * qi[1] + b[1] * qi[2] + b[2] * qi[3];
            let e = x * (gamma * qi[0] + bq);
            let f = x * (qi[0] + a * bq);
            LorentzVector::new(
                e,
                x * qi[1] + f * b[0],
                x * qi[2] + f * b[1],
                x * qi[3] + f * b[2],
            )
        })
        .collect()
}

/// Returns the differential 2-body LIPS weight `dΦ₂/d(cosθ)` in the CM frame.
///
/// For massless final-state particles this is `|p_cm| / (8π √s)`, which
/// simplifies to `1 / (16π)` when `|p_cm| = √s / 2`.
#[inline]
pub fn lips2_dcostheta(sqrt_s: f64) -> f64 {
    let p_cm = sqrt_s / 2.0; // massless: |p_cm| = E_cm/2
    p_cm / (8.0 * PI * sqrt_s)
}

/// Returns the combined Jacobian for integrating over `u ∈ [0, 1]` where
/// `cosθ = 2u − 1` (the mapping used by the VEGAS driver).
///
/// `J = dΦ₂/d(cosθ) × d(cosθ)/du = 2 × lips2_dcostheta(sqrt_s)`.
#[inline]
pub fn lips2_jacobian_u(sqrt_s: f64) -> f64 {
    2.0 * lips2_dcostheta(sqrt_s)
}

/// Map a unit-interval sample `u ∈ [0, 1)` to `cosθ ∈ (−1, 1)`.
#[inline]
pub fn u_to_costheta(u: f64) -> f64 {
    2.0 * u - 1.0
}

/// Overall prefactor for the 2→2 cross section in natural units (GeV⁻²).
///
/// `σ = prefactor2(sqrt_s) × ∫₀¹ Σ|M|²(cosθ(u)) du`
///
/// where `Σ|M|²` is the full helicity sum (sum over *all* helicity
/// combinations, not averaged) and the prefactor encodes flux, spin-averaging
/// (`1/4` over 4 initial-state helicities), and the LIPS Jacobian:
///
/// ```text
/// prefactor2 = 1/(2s) × 1/4 × J(u) = 1/(2s) × 1/4 × 1/(8π) = 1/(64π s)
/// ```
#[inline]
pub fn prefactor2(sqrt_s: f64) -> f64 {
    let s = sqrt_s * sqrt_s;
    1.0 / (64.0 * PI * s)
}
