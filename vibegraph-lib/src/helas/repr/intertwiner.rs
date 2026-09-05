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
//! ## Where the vertex factors live
//!
//! This module holds the leg-count traits. The vertex factors themselves are
//! methods on the representation types in
//! [`lorentz`](crate::helas::repr::lorentz), which is where each has a concrete
//! basis to be written in:
//!
//! | Vertex factor | Map | `(j_L,j_R)` chain | Where |
//! |---------------|-----|-------------------|-------|
//! | `ψ̄ γ^μ P_L ψ` | S\* ⊗ S → T\*M | `(½,0)×(½,0)→(½,½)` | [`SpinorRepr::left_current`](crate::helas::repr::lorentz::SpinorRepr::left_current) |
//! | `ψ̄ γ^μ P_R ψ` | S\* ⊗ S → T\*M | `(0,½)×(0,½)→(½,½)` | [`SpinorRepr::right_current`](crate::helas::repr::lorentz::SpinorRepr::right_current) |
//! | `ψ̄ γ^μ Γ ψ` | S\* ⊗ S → T\*M | `(½,0)×(0,½)→(½,½)` | [`SpinorRepr::vector_bilinear`](crate::helas::repr::lorentz::SpinorRepr::vector_bilinear) |
//! | `ψ̄ σ^{μν} Γ ψ` | S\* ⊗ S → Λ²T\*M | `(½,0)×(0,½)→(1,0)⊕(0,1)` | [`SpinorRepr::tensor_bilinear`](crate::helas::repr::lorentz::SpinorRepr::tensor_bilinear) |
//! | all sixteen `ψ̄ Γ_A ψ` | S\* ⊗ S → Cl(1,3)⊗ℂ | — | [`SpinorRepr::fierz_coefficients`](crate::helas::repr::lorentz::SpinorRepr::fierz_coefficients) |
//! | `ε^{μνρσ}` | (T\*M)³ → T\*M | `(½,½)³→(½,½)` | [`epsilon_vector`](crate::helas::repr::lorentz::epsilon_vector), [`epsilon4`](crate::helas::repr::lorentz::epsilon4) |
//!
//! An arbitrary Clifford element acts on a spinor through [`SpinorRepr::apply`](crate::helas::repr::lorentz::SpinorRepr::apply),
//! and two of them compose through [`Multivector::clifford_product`](crate::helas::repr::lorentz::Multivector::clifford_product), so a
//! γ-chain of any length needs no intertwiner type of its own.
//!
//! The traits below are the interface a dispatch layer would use to name one
//! orientation per implementor; nothing implements them yet.

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
