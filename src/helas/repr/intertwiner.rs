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
use crate::helas::repr::lorentz::{ComplexVector, LorentzRepr, Rank2Tensor, Scalar, SpinorRepr};
use std::marker::PhantomData;

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

// ─────────────────────────────────────────────────────────────────────────────
// GammaL — left-chiral current
// ─────────────────────────────────────────────────────────────────────────────

/// Left-chiral vector current: `J_L^μ = ψ̄ γ^μ P_L ψ`.
///
/// In the Weyl basis, only the right-chiral components of `fo` (indices 2–3)
/// and left-chiral components of `fi` (indices 0–1) contribute.
/// See [`SpinorRepr::left_current`] for the explicit `σ̄^μ` formula.
///
/// Used in QCD and weak-interaction amplitudes; delegates to `B::left_current`.
pub struct GammaL<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner2Leg<F> for GammaL<B> {
    type In1 = B;
    type In2 = B;
    type Out = ComplexVector<F>;

    fn apply(input: &(Self::In1, Self::In2)) -> Self::Out {
        B::left_current(&input.0, &input.1)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GammaR — right-chiral current
// ─────────────────────────────────────────────────────────────────────────────

/// Right-chiral vector current: `J_R^μ = ψ̄ γ^μ P_R ψ`.
///
/// In the Weyl basis, only the left-chiral components of `fo` (indices 0–1)
/// and right-chiral components of `fi` (indices 2–3) contribute.
/// See [`SpinorRepr::right_current`] for the explicit `σ^μ` formula.
///
/// Delegates to `B::right_current`.
pub struct GammaR<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner2Leg<F> for GammaR<B> {
    type In1 = B;
    type In2 = B;
    type Out = ComplexVector<F>;

    fn apply(input: &(Self::In1, Self::In2)) -> Self::Out {
        B::right_current(&input.0, &input.1)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GammaV — off-shell spinor propagation
// ─────────────────────────────────────────────────────────────────────────────

/// Off-shell spinor intertwiner: `Ψ^μ = γ^μ · ε_μ` acting on a spinor current.
///
/// Maps a polarisation 4-vector together with an off-shell spinor to a new
/// off-shell spinor.  Used in the construction of off-shell fermion currents
/// when a vector boson is attached to a fermion line.
///
/// # TODO
/// Implement for [`SpinorRepr`] using new methods on the SpinorRepr trait.
pub struct GammaV<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner2Leg<F> for GammaV<B> {
    type In1 = ComplexVector<F>;
    type In2 = B;
    type Out = B;

    fn apply(_input: &(Self::In1, Self::In2)) -> Self::Out {
        todo!("GammaV: γ^μ acting on off-shell spinor current — Weyl implementation pending")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SigmaTensor — σ^μν bilinear
// ─────────────────────────────────────────────────────────────────────────────

/// Tensor bilinear intertwiner: `ψ̄ σ^μν ψ` where `σ^μν = i/2 [γ^μ, γ^ν]`.
///
/// Used in magnetic-moment operators, weak dipole moments, and loop-induced
/// tensor couplings (FCNC operators, anomalous gauge couplings).
///
/// Input: `(fo, fi)` pair of Dirac spinors.
/// Output: an antisymmetric rank-2 tensor [`Rank2Tensor<F>`].
///
/// # TODO
/// Implement for [`SpinorRepr`].
pub struct SigmaTensor<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner2Leg<F> for SigmaTensor<B> {
    type In1 = B;
    type In2 = B;
    type Out = Rank2Tensor<F>;

    fn apply(_input: &(Self::In1, Self::In2)) -> Self::Out {
        todo!("SigmaTensor: σ^μν bilinear — implementation pending")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Epsilon — Lorentz scalar (Majorana/spinor metric)
// ─────────────────────────────────────────────────────────────────────────────

/// Lorentz scalar bilinear: `ψ^T C ψ` where `C = iγ^2 γ^0` is the charge
/// conjugation matrix.
///
/// This is the Majorana mass term and the SL(2,ℂ) spinor metric
/// `ε_{αβ} ψ^α χ^β`. It maps two Weyl (or Dirac) spinors to a Lorentz scalar
/// `C<F>`.
///
/// # TODO
/// Implement for [`crate::helas::repr::lorentz::Bispinor`] using
/// `ε_{αβ} = [[0,1],[-1,0]]`.
pub struct Epsilon<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner2Leg<F> for Epsilon<B> {
    type In1 = B;
    type In2 = B;
    type Out = Scalar<F>;

    fn apply(_input: &(Self::In1, Self::In2)) -> Self::Out {
        todo!("Epsilon: Lorentz scalar bilinear ε_{{αβ}} ψ^α χ^β — implementation pending")
    }
}
