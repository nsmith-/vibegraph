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
//! fibers changes.  The [`Intertwiner`] trait encodes one such orientation.
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

use super::{C, Real, lorentz::LorentzVector};
use crate::helas::repr::lorentz::{ComplexVector, Scalar, SpinorRepr};
use std::marker::PhantomData;

// ─────────────────────────────────────────────────────────────────────────────
// Intertwiner — base trait
// ─────────────────────────────────────────────────────────────────────────────

/// A Spin(1,3)-equivariant linear map between two representation fibers.
///
/// The `momentum` argument carries the 4-momentum flowing through the vertex.
/// For bilinear fermion currents (with no momentum insertion) pass
/// `FourMomentum::zero()`.
pub trait Intertwiner<F: Real, In: Copy, Out: Copy> {
    /// Apply the intertwiner to the input fiber `input` with momentum `momentum`.
    fn apply(input: &In, momentum: LorentzVector<F>) -> Out;
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

impl<F: Real, B: SpinorRepr<F>> Intertwiner<F, (B, B), ComplexVector<F>> for GammaL<B> {
    fn apply(input: &(B, B), _momentum: LorentzVector<F>) -> ComplexVector<F> {
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

impl<F: Real, B: SpinorRepr<F>> Intertwiner<F, (B, B), ComplexVector<F>> for GammaR<B> {
    fn apply(input: &(B, B), _momentum: LorentzVector<F>) -> ComplexVector<F> {
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

impl<F: Real, B: SpinorRepr<F>> Intertwiner<F, (ComplexVector<F>, B), B> for GammaV<B> {
    fn apply(_input: &(ComplexVector<F>, B), _momentum: LorentzVector<F>) -> B {
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
/// Output: an antisymmetric rank-2 tensor `[[C<F>; 4]; 4]`.
///
/// # TODO
/// Implement for [`SpinorRepr`].
/// Implement an antisymmetric rank-2 tensor type and return that instead of a raw `[[C<F>; 4]; 4]`.
pub struct SigmaTensor<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner<F, (B, B), [[C<F>; 4]; 4]> for SigmaTensor<B> {
    fn apply(_input: &(B, B), _momentum: LorentzVector<F>) -> [[C<F>; 4]; 4] {
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
/// Implement for [`crate::helas::repr::lorentz::WeylBasis`] using
/// `ε_{αβ} = [[0,1],[-1,0]]`.
pub struct Epsilon<B> {
    _marker: PhantomData<B>,
}

impl<F: Real, B: SpinorRepr<F>> Intertwiner<F, (B, B), Scalar<F>> for Epsilon<B> {
    fn apply(_input: &(B, B), _momentum: LorentzVector<F>) -> Scalar<F> {
        todo!("Epsilon: Lorentz scalar bilinear ε_{{αβ}} ψ^α χ^β — implementation pending")
    }
}
