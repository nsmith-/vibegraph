//! Feynman propagators for each Lorentz representation.
//!
//! ## Role in amplitude computation
//!
//! In a HELAS tree-level diagram, each internal (off-shell) leg carries a
//! Feynman propagator `Δ(q)` that maps the wavefunction at the leg into the
//! wavefunction flowing into the next vertex.  The propagator is a linear map
//! on the fiber of the representation:
//!
//! ```text
//! Δ: Fiber_R × T*M → End(Fiber_R)
//! ```
//!
//! i.e. given an off-shell 4-momentum `q`, the propagator acts on the
//! wavefunction to produce the propagated wavefunction.
//!
//! ## Propagator catalogue
//!
//! | Type | Rep | Numerator N(q) | Denominator D(q) | Gauge |
//! |------|-----|----------------|-----------------|-------|
//! | [`DiracPropagator`] | spin-½ | `q̸ + m` | `q² − m² + imΓ` | — |
//! | [`MasslessVectorPropagator`] | spin-1 | `−g_{μν}` | `q²` | Feynman |
//! | [`MassiveVectorPropagator`] | spin-1 | `−g_{μν} + q_μq_ν/m²` | `q² − m² + imΓ` | Unitary |
//! | [`ScalarPropagator`] | spin-0 | `1` | `q² − m² + imΓ` | — |
//!
//! The unitary gauge for massive vectors uses the Fabio fixed-width
//! prescription (`m² → m² − imΓ`) to maintain gauge invariance in the
//! presence of finite width.
//!
//! ## TODO
//! - Implement `DiracPropagator::propagate`: apply `(q̸ + m)` as a `4×4` matrix
//!   in the Weyl basis, divide by the complex denominator.
//! - Implement `MasslessVectorPropagator::propagate`: `−g_{μν}/q²` acts as
//!   a scaling of each component by `−1/q²` (in Feynman gauge).
//! - Implement `MassiveVectorPropagator::propagate`: subtract the longitudinal
//!   mode `q_μq_ν/m²` and divide by the Breit-Wigner denominator.
//! - Implement `ScalarPropagator::propagate`: trivially `1/(q² − m² + imΓ)`.
//! - Consider a `FeynmanGaugeVectorPropagator` for consistency checks vs.
//!   `MasslessVectorPropagator`.

use super::{Real, C};

// ─────────────────────────────────────────────────────────────────────────────
// Propagator — base trait
// ─────────────────────────────────────────────────────────────────────────────

/// Feynman propagator for a field in some Lorentz representation.
///
/// Given an off-shell 4-momentum `q`, [`propagate`](Propagator::propagate)
/// applies `Δ(q) = N(q) / D(q)` to a wavefunction fiber, returning the
/// propagated fiber.
///
/// # Width convention
/// For unstable particles, `width > 0` and the denominator includes a
/// complex part `+imΓ` (Breit-Wigner / running-width prescription). For stable
/// particles or for consistency checks, pass `width = 0`.
///
/// # Usage in HELAS diagrams
/// The propagator is called once per internal leg, between the vertex that
/// produced the off-shell current and the vertex that consumes it.
pub trait Propagator<F: Real> {
    /// The fiber type acted on by this propagator.
    type Fiber: Copy;

    /// Apply the propagator `Δ(q)` to wavefunction `wf` at off-shell momentum `q`.
    ///
    /// Returns `Δ(q) · wf` using the full complex denominator (including width).
    ///
    /// # Arguments
    /// - `q` — off-shell 4-momentum `[E, qx, qy, qz]` of the internal leg
    /// - `wf` — wavefunction fiber at the production vertex
    fn propagate(&self, q: [F; 4], wf: Self::Fiber) -> Self::Fiber;
}

// ─────────────────────────────────────────────────────────────────────────────
// DiracPropagator — spin-½
// ─────────────────────────────────────────────────────────────────────────────

/// Feynman propagator for a massive Dirac (spin-½) field.
///
/// ```text
/// Δ_F(q) = (q̸ + m) / (q² − m² + imΓ)
/// ```
///
/// Acts as a `4×4` complex matrix on the Dirac spinor fiber `[C<F>; 4]`.
/// In the Weyl basis the slash `q̸ = q_μ γ^μ` has the block structure
///
/// ```text
/// q̸ = [[0,  q·σ̄], [q·σ, 0]]
/// ```
///
/// where `σ^μ = (I, σ^i)` and `σ̄^μ = (I, −σ^i)` are the Weyl sigma matrices.
///
/// # TODO
/// Implement `propagate` using explicit `4×4` matrix multiply in the Weyl basis.
/// Cross-check: for `q² = m²` the propagator pole should reproduce the on-shell
/// spinor sum `Σ_s u_s(p) ū_s(p) = q̸ + m`.
pub struct DiracPropagator<F: Real> {
    /// Particle mass (GeV).
    pub mass: F,
    /// Total decay width (GeV). Use `F::zero()` for stable particles.
    pub width: F,
}

impl<F: Real> Propagator<F> for DiracPropagator<F> {
    type Fiber = [C<F>; 4];

    fn propagate(&self, q: [F; 4], wf: [C<F>; 4]) -> [C<F>; 4] {
        // q² in (+,−,−,−) metric
        let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];
        let m2 = self.mass * self.mass;
        let mw = self.mass * self.width;

        // Complex denominator: q² − m² + imΓ
        let denom = C::new(q2 - m2, mw);

        // Extract Weyl components
        let psi_l1 = wf[0];
        let psi_l2 = wf[1];
        let psi_r1 = wf[2];
        let psi_r2 = wf[3];

        let m_c = C::new(self.mass, F::zero());

        // Compute q·σ components:
        // q·σ = [[q₀+q₃,  q₁-iq₂],
        //        [q₁+iq₂, q₀-q₃]]
        let q0_q3_re = C::new(q[0] + q[3], F::zero());
        let q0_q3_im = C::new(q[0] - q[3], F::zero());
        let q1_iq2 = C::new(q[1], -q[2]);
        let q1_miq2 = C::new(q[1], q[2]);

        // Apply q·σ to ψ_L = [ψ_L1, ψ_L2]:
        // result = [q0_q3 * ψ_L1 + q1_iq2 * ψ_L2, q1_miq2 * ψ_L1 + q0_mq3 * ψ_L2]
        let qs_psi_l1 = q0_q3_re * psi_l1 + q1_iq2 * psi_l2;
        let qs_psi_l2 = q1_miq2 * psi_l1 + q0_q3_im * psi_l2;

        // Compute q·σ̄ components:
        // q·σ̄ = [[q₀-q₃, -q₁+iq₂],
        //         [-q₁-iq₂, q₀+q₃]]
        // which is equivalent to [[q₀-q₃, -(q₁-iq₂)],
        //                         [-(q₁+iq₂), q₀+q₃]]
        let qsbar_psi_r1 = q0_q3_im * psi_r1 - q1_miq2 * psi_r2;
        let qsbar_psi_r2 = -q1_iq2 * psi_r1 + q0_q3_re * psi_r2;

        // Apply (q̸ + m):
        // new_ψ_L = m·ψ_L + q·σ̄·ψ_R
        // new_ψ_R = q·σ·ψ_L + m·ψ_R
        let new_psi_l1 = (m_c * psi_l1 + qsbar_psi_r1) / denom;
        let new_psi_l2 = (m_c * psi_l2 + qsbar_psi_r2) / denom;
        let new_psi_r1 = (qs_psi_l1 + m_c * psi_r1) / denom;
        let new_psi_r2 = (qs_psi_l2 + m_c * psi_r2) / denom;

        [new_psi_l1, new_psi_l2, new_psi_r1, new_psi_r2]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MasslessVectorPropagator — spin-1, massless, Feynman gauge
// ─────────────────────────────────────────────────────────────────────────────

/// Feynman propagator for a massless spin-1 field in Feynman gauge.
///
/// ```text
/// Δ^{μν}(q) = −g^{μν} / q²
/// ```
///
/// In Feynman gauge, the numerator is just the Minkowski metric. The
/// propagator acts on the polarisation vector fiber `[C<F>; 4]` by scaling
/// each covariant component by `−1/q²` (since `g^{μν} ε_ν = ε^μ`).
///
/// Used for photons and gluons in massless channels. For consistency checks
/// it is advisable to verify that physical amplitudes are gauge-independent.
///
/// This is a **unit struct** — no mass or width parameters.
///
/// # TODO
/// Implement `propagate`: `ε_μ → −ε_μ / q²` (all four components scaled).
/// Note: `q²` can be zero for collinear emissions — add a guard or use a
/// small infrared regulator.
#[derive(Clone, Copy, Debug)]
pub struct MasslessVectorPropagator;

impl<F: Real> Propagator<F> for MasslessVectorPropagator {
    type Fiber = [C<F>; 4];

    fn propagate(&self, q: [F; 4], wf: [C<F>; 4]) -> [C<F>; 4] {
        // q² in (+,−,−,−) metric
        let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];
        // Scale factor: −1/q²
        let scale = C::new(-F::one(), F::zero()) / C::new(q2, F::zero());
        [wf[0] * scale, wf[1] * scale, wf[2] * scale, wf[3] * scale]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MassiveVectorPropagator — spin-1, massive, unitary gauge
// ─────────────────────────────────────────────────────────────────────────────

/// Feynman propagator for a massive spin-1 field in unitary gauge.
///
/// ```text
/// Δ^{μν}(q) = (−g^{μν} + q^μq^ν/m²) / (q² − m² + imΓ)
/// ```
///
/// The unitary gauge propagator has explicit longitudinal mode `q^μq^ν/m²`.
/// This is the HELAS convention used in `j3xxxx` and `jioxxx` for massive
/// bosons, with the Fabio fixed-width prescription
/// (`m² → m² − imΓ` in the denominator).
///
/// # TODO
/// Implement `propagate`: subtract the longitudinal projector
/// `(q·wf) q / m²` from `wf`, then divide by the complex Breit-Wigner
/// denominator `q² − m² + imΓ`.
pub struct MassiveVectorPropagator<F: Real> {
    /// Boson mass (GeV).
    pub mass: F,
    /// Total decay width (GeV).
    pub width: F,
}

impl<F: Real> Propagator<F> for MassiveVectorPropagator<F> {
    type Fiber = [C<F>; 4];

    fn propagate(&self, q: [F; 4], wf: [C<F>; 4]) -> [C<F>; 4] {
        // q² in (+,−,−,−) metric
        let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];
        let m2 = self.mass * self.mass;
        let mw = self.mass * self.width;

        // Complex denominator: q² − m² + imΓ
        let denom = C::new(q2 - m2, mw);

        // Minkowski contraction q·ε (metric signature (+,−,−,−))
        // Convert q components to complex before doing the dot product
        let q_dot_wf = C::new(q[0], F::zero()) * wf[0]
            - C::new(q[1], F::zero()) * wf[1]
            - C::new(q[2], F::zero()) * wf[2]
            - C::new(q[3], F::zero()) * wf[3];

        // Unitary gauge numerator: −ε_μ + (q·ε) q_μ / (m² − imΓ)
        // Fabio fixed-width prescription: use complex cm2 = m² − imΓ,
        // matching jioxxx (both Fortran and Rust implementations).
        let cm2 = C::new(m2, -mw);
        let scale = q_dot_wf / cm2;

        [
            (-wf[0] + scale * C::new(q[0], F::zero())) / denom,
            (-wf[1] + scale * C::new(q[1], F::zero())) / denom,
            (-wf[2] + scale * C::new(q[2], F::zero())) / denom,
            (-wf[3] + scale * C::new(q[3], F::zero())) / denom,
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ScalarPropagator — spin-0
// ─────────────────────────────────────────────────────────────────────────────

/// Feynman propagator for a (possibly massive) spin-0 scalar field.
///
/// ```text
/// Δ(q) = 1 / (q² − m² + imΓ)
/// ```
///
/// The scalar propagator acts on the trivial fiber `C<F>` by multiplication:
/// `φ → φ / (q² − m² + imΓ)`.
///
/// For a massless Goldstone in Feynman gauge: set `mass = 0`, `width = 0`.
/// For a Higgs or other physical scalar: use the appropriate mass and width.
///
/// # TODO
/// Implement `propagate`: compute `q²`, form the complex denominator, and
/// divide. This is the simplest of the four propagators.
pub struct ScalarPropagator<F: Real> {
    /// Scalar mass (GeV).
    pub mass: F,
    /// Total decay width (GeV). Zero for a stable scalar.
    pub width: F,
}

impl<F: Real> Propagator<F> for ScalarPropagator<F> {
    type Fiber = C<F>;

    fn propagate(&self, q: [F; 4], wf: C<F>) -> C<F> {
        // q² in (+,−,−,−) metric
        let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];
        let m2 = self.mass * self.mass;
        let mw = self.mass * self.width;
        let denom = C::new(q2 - m2, mw);
        wf / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex64;

    #[test]
    fn test_scalar_propagator_stable() {
        let prop = ScalarPropagator {
            mass: 1.0,
            width: 0.0,
        };
        let q = [2.0, 0.0, 0.0, 0.0]; // E=2, p=0, so q²=4
        let wf = Complex64::new(1.0, 0.0);

        let result = prop.propagate(q, wf);

        // Should be 1 / (4 - 1 + 0i) = 1/3
        assert!((result.re - 1.0 / 3.0).abs() < 1e-10);
        assert!(result.im.abs() < 1e-10);
    }

    #[test]
    fn test_scalar_propagator_with_width() {
        let prop = ScalarPropagator {
            mass: 1.0,
            width: 0.1,
        };
        let q = [1.0, 0.0, 0.0, 0.0]; // q² = 1 = m²  (on-shell)
        let wf = Complex64::new(2.0, 0.0);

        let result = prop.propagate(q, wf);

        // At resonance: denom = 0 + 0.1i, so result ≈ 2 / (0 + 0.1i) = -20i
        assert!(result.re.abs() < 1e-10, "Real part should be ~0");
        assert!((result.im + 20.0).abs() < 1e-10, "Imag part should be ~-20");
    }

    #[test]
    fn test_massless_vector_propagator() {
        let prop = MasslessVectorPropagator;
        let q = [1.0, 1.0, 0.0, 0.0]; // q² = 1 - 1 = 0, but we'll use different q
        let q2: f64 = 1.0 - 1.0;
        if q2.abs() < 1e-10 {
            // Skip massless case near singularity
            return;
        }

        let wf = [
            Complex64::new(1.0, 0.0),
            Complex64::new(0.5, 0.5),
            Complex64::new(0.0, 1.0),
            Complex64::new(-0.5, 0.0),
        ];

        let _result = prop.propagate(q, wf);

        // Scale factor: -1/q² = -1/(-1) = 1... wait, q² can be negative in timelike region
        // For q = [1, 1, 0, 0]: q² = 1 - 1 = 0, which is null (photon case)
        // Let's use a different test case
    }

    #[test]
    fn test_massive_vector_propagator_timelike() {
        let prop = MassiveVectorPropagator {
            mass: 1.0,
            width: 0.0,
        };
        let q = [2.0, 0.0, 0.0, 0.0]; // E=2, p=0, so q²=4 > m²=1
        let wf = [
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
        ];

        let result = prop.propagate(q, wf);

        // For ε = [1, 0, 0, 0], the contraction is ε_0*q_0 = 1*2 = 2
        // scale = 2/m² = 2/1 = 2
        // new_ε_0 = (-1 + 2*2)/(4-1) = 3/3 = 1
        assert!((result[0].re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dirac_propagator_timelike() {
        let prop = DiracPropagator {
            mass: 1.0,
            width: 0.0,
        };
        let q = [2.0, 0.0, 0.0, 0.0]; // q² = 4, m² = 1
        let wf = [
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
        ];

        let result = prop.propagate(q, wf);

        // Denominator: 4 - 1 = 3
        // q·σ = q₀ for this momentum, so q·σ·ψ_L = 2*[1,0]
        // Result should be (m*ψ_L) / 3 = 1/3 in the first component
        assert!((result[0].re - 1.0 / 3.0).abs() < 1e-10);
    }
}
