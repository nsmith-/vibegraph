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

    fn propagate(&self, _q: [F; 4], _wf: [C<F>; 4]) -> [C<F>; 4] {
        todo!(
            "DiracPropagator: apply (q̸ + m) / (q² − m² + imΓ) to spinor fiber \
             in the Weyl basis"
        )
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

    fn propagate(&self, _q: [F; 4], _wf: [C<F>; 4]) -> [C<F>; 4] {
        todo!("MasslessVectorPropagator: scale each component by −1/q² (Feynman gauge)")
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

    fn propagate(&self, _q: [F; 4], _wf: [C<F>; 4]) -> [C<F>; 4] {
        todo!(
            "MassiveVectorPropagator: (−g^μν + q^μq^ν/m²)/(q²−m²+imΓ) · ε_ν \
             using Fabio fixed-width prescription"
        )
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
