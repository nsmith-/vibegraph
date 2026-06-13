//! Spin(1,3)-equivariant maps between representation fibers (intertwiners).
//!
//! ## What is an intertwiner?
//!
//! A Feynman rule for a vertex factor is not just a matrix — it is an
//! *intertwiner*: a Spin(1,3)-equivariant linear map between the fibers of the
//! bundles attached to each leg of the vertex.  Equivariance means the map
//! commutes with the group action, i.e. it preserves Lorentz covariance.
//!
//! For example, the QED vertex `ieγ^μ` intertwines:
//!
//! \`\`\`text
//! γ^μ : S* ⊗ S → T*M        (fermion current → photon leg)
//! \`\`\`
//!
//! where `S` is the Dirac spinor bundle and `T*M` is the cotangent bundle
//! (spin-(½,½)).
//!
//! ## Orientations
//!
//! Each intertwiner has multiple *orientations* depending on which legs are
//! incoming vs. outgoing and which particle species are flowing through them:
//!
//! - **Both fermions incoming** (e.g. two on-shell fermion lines in a current):
//!   produces an off-shell vector.
//! - **One fermion in, one fermion out** (e.g. off-shell fermion propagation):
//!   produces an off-shell spinor.
//! - **Both fermions outgoing** (e.g. building a fermion-loop contribution):
//!   produces an off-shell vector with different normalization.
//!
//! The same coupling constant appears in all orientations; only the map between
//! fibers changes.  The [`Intertwiner2Leg`], [`Intertwiner3Leg`], and
//! [`Intertwiner4Leg`] traits encode each orientation by leg count.
//!
//! ## Implemented intertwiners
//!
//! | Type | Map | `(j_L,j_R)` chain | Description |
//! |------|-----|-------------------|-------------|
//! | [`Gamma`]  | S* ⊗ S → T\*M | `(½,0)×(0,½)→(½,½)` | Full vector current `ψ̄ γ^μ ψ` |
//! | [`GammaL`] | S* ⊗ S → T\*M | `(½,0)×(½,0)→(½,½)` | Left bilinear current `ψ̄ γ^μ P_L ψ` |
//! | [`GammaR`] | S* ⊗ S → T\*M | `(0,½)×(0,½)→(½,½)` | Right bilinear current `ψ̄ γ^μ P_R ψ` |
//!
//! ## Status
//!
//! - [`GammaL`] and [`GammaR`] are **fully implemented** — they
//!   delegate to [`SpinorRepr::left_current`] and [`SpinorRepr::right_current`].
//!   These are used directly in [`crate::helas::vertex`].
//! - [`GammaV`], [`SigmaTensor`], and [`Epsilon`] are **stubs** pending
//!   implementation.

use super::Real;
use crate::helas::repr::lorentz::LorentzRepr;

// ─────────────────────────────────────────────────────────────────────────────
// Intertwiner traits — leg-count-specific
// ─────────────────────────────────────────────────────────────────────────────

/// A 2-leg intertwiner: 2 inputs → 1 output.
///
/// Used for bilinear currents like `ψ̄ γ^μ ψ` (two fermions → one vector).
/// Input momenta are derived from the wavefunction fiber data by routing
/// convention; no explicit momentum argument is needed for local vertices.
pub trait Intertwiner2Leg<F: Real> {
    type In1: LorentzRepr<F>;
    type In2: LorentzRepr<F>;
    type Out: LorentzRepr<F>;

    /// Apply the intertwiner to two input fibers.
    fn apply(input: &(Self::In1, Self::In2)) -> Self::Out;
}

/// A 3-leg intertwiner: 3 inputs → 1 output.
///
/// Used for off-shell current reductions like `jgggxx` (3 vectors → 1 vector).
/// Corresponds to HELAS routines that build off-shell currents from three
/// on-shell wavefunctions.
pub trait Intertwiner3Leg<F: Real> {
    type In1: LorentzRepr<F>;
    type In2: LorentzRepr<F>;
    type In3: LorentzRepr<F>;
    type Out: LorentzRepr<F>;

    /// Apply the intertwiner to three input fibers.
    fn apply(input: &(Self::In1, Self::In2, Self::In3)) -> Self::Out;
}

/// A 4-leg intertwiner: 4 inputs → 1 output.
///
/// Used for direct quartic contact maps like `ggggxx` (4 vectors → scalar).
/// Corresponds to direct quartic-vector contact amplitude routines that do not
/// reduce to chains of lower-arity routines.
pub trait Intertwiner4Leg<F: Real> {
    type In1: LorentzRepr<F>;
    type In2: LorentzRepr<F>;
    type In3: LorentzRepr<F>;
    type In4: LorentzRepr<F>;
    type Out: LorentzRepr<F>;

    /// Apply the intertwiner to four input fibers.
    fn apply(input: &(Self::In1, Self::In2, Self::In3, Self::In4)) -> Self::Out;
}
