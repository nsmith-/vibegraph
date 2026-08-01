//! Phase-space utilities for LO cross-section calculations.
//!
//! Implements the Lorentz-invariant phase space (LIPS) measure for 2-body
//! final states in the CM frame, and provides the unit-hypercube mapping used
//! by the VEGAS integrator. The [`rambo`] submodule generalizes to `n`-body
//! flat sampling over an arbitrary scalar field, and [`rng`] supplies the
//! counter-based uniform substreams that feed it. The [`channel`] submodule is
//! the abstraction seam — [`PhaseSpaceMap`]/[`Channel`]/[`Combiner`] — that lets
//! the sampler, channel map, and integrator be swapped independently; flat RAMBO
//! and the 2-body LIPS map sit behind it as [`RamboChannel`] and [`Lips2Channel`],
//! and [`MultiChannel`] combines per-diagram channels into one variance-minimising
//! [`Combiner`].
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

pub mod channel;
pub mod diagram_channel;
pub mod rambo;
pub mod rng;

pub use channel::{
    kleiss_pittau_step, select_channel, AlphaAdaptation, Channel, Combiner, Lips2Channel,
    MultiChannel, PhaseSpaceMap, PhaseSpacePoint, RamboChannel, ScaledChannel, ScaledMultiChannel,
};
pub use diagram_channel::{DiagramChannel, Resonance, TChannel};
pub use rambo::{rambo, rambo_massive, rambo_massless, RamboPoint};

/// Conversion factor: 1 GeV⁻² = 3.893793721×10⁸ pb.
///
/// Derived from (ℏc)² = 0.3893793721 GeV²·mb and 1 mb = 10⁹ pb.
pub const GEV2_TO_PB: f64 = 3.893_793_721e8;

/// The identical-particle symmetry factor `1 / Π_s n_s!` of an outgoing multiset.
///
/// Every map here covers the whole of `dΦ_n`, whose legs are *labelled*: each
/// permutation of a set of identical outgoing particles is a distinct
/// configuration under the measure, while the cross section counts each such
/// configuration once. `n_s` is the multiplicity of species `s` among `outgoing`;
/// the entries need only compare equal for the same species, which PDG codes and
/// model particle ids both do, both separating a particle from its antiparticle.
/// `g g → g g` is the case that makes the factor visible: without its `1/2!` the
/// cross section comes out exactly twice MadGraph's, with the `|M|²`, the flux and
/// the initial-state average all already agreeing.
///
/// The factor belongs to one subprocess's final state rather than to the weight a
/// map returns. A hadronic run pools the channels of subprocesses whose outgoing
/// *masses* agree while their species do not — `g g → g g` and `u ū → d d̄` are
/// both `[0, 0]` — so a map weight carrying the factor would stop being a density
/// on the one labelled `dΦ_n` those subprocesses share. Each subprocess instead
/// multiplies its own term of the summed matrix element by its own factor.
///
/// Permutations of identical legs are not enumerated as extra sampling channels:
/// the per-diagram channel set is already closed under them, since the image of a
/// diagram under a swap of two identical outgoing legs is another diagram of the
/// same process.
pub fn identical_particle_factor<S: PartialEq>(outgoing: &[S]) -> f64 {
    // Each leg contributes the next factor of its own species' factorial, counted
    // by how many earlier legs it matches, so one pass builds `Π_s n_s!`.
    let permutations: f64 = outgoing
        .iter()
        .enumerate()
        .map(|(i, s)| (outgoing[..i].iter().filter(|&other| other == s).count() + 1) as f64)
        .product();
    1.0 / permutations
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

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;

    /// The factor counts species, not legs and not masses: `[g, g]` and `[q, q̄]`
    /// carry the same mass list and different factors, which is the whole reason a
    /// mass list cannot own it.
    #[test]
    fn identical_particle_factor_counts_species_not_masses() {
        assert_eq!(identical_particle_factor(&[21, 21]), 0.5);
        assert_eq!(identical_particle_factor(&[2, -2]), 1.0);
        assert_eq!(identical_particle_factor(&[21, 21, 21]), 1.0 / 6.0);
        assert_eq!(identical_particle_factor(&[21, 21, 21, 21]), 1.0 / 24.0);
        // Two species at once: `1/(2! · 3!)`.
        assert_eq!(identical_particle_factor(&[21, 2, 21, 2, 2]), 1.0 / 12.0);
        // A particle is not its own antiparticle.
        assert_eq!(identical_particle_factor(&[2, 2, -2]), 0.5);
        assert_eq!(identical_particle_factor::<i32>(&[]), 1.0);
    }

    /// Massive RAMBO points are on-shell to the Newton tolerance and conserve the
    /// total four-momentum `(√s, 0, 0, 0)`.
    #[test]
    fn rambo_massive_on_shell_and_conserving() {
        let mut rng = StdRng::seed_from_u64(0xC0FFEE);
        let sqrt_s = 500.0;
        for masses in [
            vec![0.0, 0.0, 91.19],
            vec![173.0, 173.0],
            vec![1.777, 1.777, 0.0, 0.0, 125.0],
        ] {
            let p = rambo_massive(sqrt_s, &masses, &mut rng);
            let mut tot = [0.0f64; 4];
            for (q, m) in p.iter().zip(&masses) {
                let m2 = q.e() * q.e() - q.px() * q.px() - q.py() * q.py() - q.pz() * q.pz();
                assert!(
                    (m2 - m * m).abs() < 1e-9 * sqrt_s * sqrt_s,
                    "off-shell: m² = {m2}, expected {}",
                    m * m
                );
                tot[0] += q.e();
                tot[1] += q.px();
                tot[2] += q.py();
                tot[3] += q.pz();
            }
            assert!((tot[0] - sqrt_s).abs() < 1e-9 * sqrt_s);
            for c in &tot[1..] {
                assert!(c.abs() < 1e-9 * sqrt_s, "momentum not conserved: {tot:?}");
            }
        }
    }
}
